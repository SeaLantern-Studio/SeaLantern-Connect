use sculk::ErrorCategory;
use sculk::minecraft::lan::LanBroadcaster;
use sculk::tunnel::{
    JoinConfig, JoinOptions, JoinUri, TunnelEvent, TunnelPhase, TunnelService, TunnelStatus,
    TunnelUpdate,
};
use serde::Serialize;
use std::num::NonZeroU16;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Manager, State};

const STATUS_EVENT: &str = "connect-status";
const LAN_NAME: &str = "SeaLantern Connect";

pub struct ConnectState {
    service: TunnelService,
    broadcaster: Mutex<Option<LanBroadcaster>>,
    last_message: Mutex<Option<String>>,
}

impl ConnectState {
    pub fn new() -> Self {
        Self {
            service: TunnelService::new(),
            broadcaster: Mutex::new(None),
            last_message: Mutex::new(None),
        }
    }

    fn snapshot(&self) -> ConnectSnapshot {
        snapshot_from_status(
            self.service.status(),
            self.last_message.lock().ok().and_then(|value| value.clone()),
        )
    }

    fn set_message(&self, message: Option<String>) {
        if let Ok(mut current) = self.last_message.lock() {
            *current = message;
        }
    }

    fn sync_broadcast(&self, status: &TunnelStatus) -> Result<(), String> {
        let mut current = self
            .broadcaster
            .lock()
            .map_err(|_| "LAN broadcast state is unavailable".to_owned())?;
        let desired_port = if status.state.phase == TunnelPhase::Active {
            status.state.local_addr.and_then(|addr| NonZeroU16::new(addr.port()))
        } else {
            None
        };

        if desired_port.is_none() {
            if let Some(broadcaster) = current.take() {
                broadcaster.stop().map_err(|error| error.to_string())?;
            }
            return Ok(());
        }

        if current.as_ref().is_some_and(|item| !item.is_finished()) {
            return Ok(());
        }
        if let Some(broadcaster) = current.take() {
            let _ = broadcaster.stop();
        }
        *current = Some(
            LanBroadcaster::start(LAN_NAME, desired_port.expect("checked above"))
                .map_err(|error| error.to_string())?,
        );
        Ok(())
    }

    fn stop_broadcast(&self) {
        if let Ok(mut current) = self.broadcaster.lock()
            && let Some(broadcaster) = current.take()
        {
            let _ = broadcaster.stop();
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitePreview {
    valid: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectSnapshot {
    phase: &'static str,
    local_address: Option<String>,
    route: Option<&'static str>,
    rtt_ms: Option<u64>,
    tx_bytes: u64,
    rx_bytes: u64,
    error: Option<&'static str>,
    message: Option<String>,
}

#[tauri::command]
pub fn validate_invite(uri: String) -> Result<InvitePreview, String> {
    uri.trim()
        .parse::<JoinUri>()
        .map(|_| InvitePreview { valid: true })
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_status(state: State<'_, ConnectState>) -> ConnectSnapshot {
    state.snapshot()
}

#[tauri::command]
pub async fn start_join(uri: String, state: State<'_, ConnectState>) -> Result<(), String> {
    let join_uri = uri
        .trim()
        .parse::<JoinUri>()
        .map_err(|error| error.to_string())?;
    state.set_message(None);
    let config = JoinConfig::new()
        .event_delay(Duration::from_secs(1))
        .reconnect_timeout(None);
    state
        .service
        .start_join(JoinOptions::new(join_uri).config(config))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn stop_join(state: State<'_, ConnectState>) -> Result<(), String> {
    state.stop_broadcast();
    state.set_message(None);
    state
        .service
        .shutdown()
        .await
        .map_err(|error| error.to_string())
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle().clone();
    let service = app.state::<ConnectState>().service.clone();
    tauri::async_runtime::spawn(async move {
        let mut updates = service.subscribe();
        while let Some(update) = updates.recv().await {
            apply_update(&handle, update);
        }
    });
    Ok(())
}

fn apply_update(app: &AppHandle, update: TunnelUpdate) {
    let state = app.state::<ConnectState>();
    match update {
        TunnelUpdate::Status(status) => {
            if let Err(error) = state.sync_broadcast(&status) {
                state.set_message(Some(format!("LAN broadcast failed: {error}")));
            }
        }
        TunnelUpdate::Event(event) => state.set_message(event_message(event)),
        _ => {}
    }
    let _ = app.emit(STATUS_EVENT, state.snapshot());
}

fn event_message(event: TunnelEvent) -> Option<String> {
    match event {
        TunnelEvent::Connected => Some("已连接到房主".to_owned()),
        TunnelEvent::Disconnected { reason } => Some(format!("连接已断开：{reason}")),
        TunnelEvent::Reconnecting { attempt } => Some(format!("正在进行第 {attempt} 次重连")),
        TunnelEvent::Reconnected => Some("已恢复连接".to_owned()),
        TunnelEvent::PathChanged { is_relay, rtt_ms, .. } => Some(format!(
            "当前使用{}，延迟 {rtt_ms} ms",
            if is_relay { "中继连接" } else { "直连" }
        )),
        TunnelEvent::Error { message, .. } => Some(message),
        _ => None,
    }
}

fn snapshot_from_status(status: TunnelStatus, message: Option<String>) -> ConnectSnapshot {
    let connection = status.connections.iter().find(|connection| connection.alive);
    ConnectSnapshot {
        phase: match status.state.phase {
            TunnelPhase::Idle => "idle",
            TunnelPhase::Starting => "starting",
            TunnelPhase::Active => "active",
            TunnelPhase::Stopping => "stopping",
        },
        local_address: status.state.local_addr.map(|addr| addr.to_string()),
        route: connection.map(|value| if value.is_relay { "relay" } else { "direct" }),
        rtt_ms: connection.map(|value| value.rtt_ms),
        tx_bytes: connection.map_or(0, |value| value.tx_bytes),
        rx_bytes: connection.map_or(0, |value| value.rx_bytes),
        error: status.last_error.map(category_name),
        message,
    }
}

fn category_name(category: ErrorCategory) -> &'static str {
    match category {
        ErrorCategory::InvalidJoinUri => "invalid_join_uri",
        ErrorCategory::InvalidEndpoint => "invalid_endpoint",
        ErrorCategory::AuthorizationDenied => "authorization_denied",
        ErrorCategory::HostUnreachable => "host_unreachable",
        ErrorCategory::TargetUnavailable => "target_unavailable",
        ErrorCategory::LocalPortUnavailable => "local_port_unavailable",
        ErrorCategory::IdentityUnavailable => "identity_unavailable",
        ErrorCategory::OperationConflict => "operation_conflict",
        ErrorCategory::ResourceLimit => "resource_limit",
        ErrorCategory::InvalidConfiguration => "invalid_configuration",
        ErrorCategory::Internal => "internal",
        _ => "unknown",
    }
}
