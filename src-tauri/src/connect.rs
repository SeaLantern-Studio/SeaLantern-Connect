use crate::settings::SettingsState;
use sculk::ErrorCategory;
use sculk::minecraft::lan::{LanBroadcaster, LanScanner};
use sculk::minecraft::probe_server;
use sculk::tunnel::{
    HostConfig, HostOptions, JoinConfig, JoinOptions, JoinUri, LocalPort, TunnelEvent, TunnelMode,
    TunnelPhase, TunnelService, TunnelStatus, TunnelUpdate,
};
use serde::Serialize;
use std::net::{Ipv4Addr, SocketAddr};
use std::num::NonZeroU16;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{App, AppHandle, Emitter, Manager, State};

const STATUS_EVENT: &str = "connect-status";
const LAN_NAME: &str = "SeaLantern Connect";
const MC_PROBE_TIMEOUT: Duration = Duration::from_secs(1);
const MC_HEALTH_INTERVAL: Duration = Duration::from_secs(5);
const MC_HEALTH_FAILURES_MAX: u8 = 3;

pub struct ConnectState {
    service: TunnelService,
    broadcaster: Mutex<Option<LanBroadcaster>>,
    scanner: Mutex<Option<LanScanner>>,
    detected_port: Mutex<Option<u16>>,
    host_port: Mutex<Option<u16>>,
    last_message: Mutex<Option<String>>,
    pending_join_uri: Mutex<Option<String>>,
    host_monitor_generation: AtomicU64,
}

impl ConnectState {
    pub fn new() -> Self {
        Self {
            service: TunnelService::new(),
            broadcaster: Mutex::new(None),
            scanner: Mutex::new(None),
            detected_port: Mutex::new(None),
            host_port: Mutex::new(None),
            last_message: Mutex::new(None),
            pending_join_uri: Mutex::new(None),
            host_monitor_generation: AtomicU64::new(0),
        }
    }

    fn snapshot(&self) -> ConnectSnapshot {
        snapshot_from_status(
            self.service.status(),
            self.last_message
                .lock()
                .ok()
                .and_then(|value| value.clone()),
            self.host_port.lock().ok().and_then(|value| *value),
        )
    }

    fn set_message(&self, message: Option<String>) {
        if let Ok(mut current) = self.last_message.lock() {
            *current = message;
        }
    }

    fn stop_lan_scanner(&self) -> Result<(), String> {
        let scanner = self
            .scanner
            .lock()
            .map_err(|_| "LAN scan state is unavailable".to_owned())?
            .take();
        if let Some(scanner) = scanner {
            scanner.stop().map_err(|error| error.to_string())?;
        }
        *self
            .detected_port
            .lock()
            .map_err(|_| "LAN scan state is unavailable".to_owned())? = None;
        Ok(())
    }

    fn set_pending_join_uri(&self, uri: Option<String>) {
        if let Ok(mut pending) = self.pending_join_uri.lock() {
            *pending = uri;
        }
    }

    fn take_pending_join_uri(&self) -> Option<String> {
        self.pending_join_uri
            .lock()
            .ok()
            .and_then(|mut pending| pending.take())
    }

    fn sync_broadcast(&self, status: &TunnelStatus) -> Result<(), String> {
        let mut current = self
            .broadcaster
            .lock()
            .map_err(|_| "LAN broadcast state is unavailable".to_owned())?;
        let desired_port = if status.state.phase == TunnelPhase::Active
            && status.state.mode == Some(TunnelMode::Join)
        {
            status
                .state
                .local_addr
                .and_then(|addr| NonZeroU16::new(addr.port()))
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
    mode: Option<&'static str>,
    local_address: Option<String>,
    share_uri: Option<String>,
    player_count: usize,
    host_port: Option<u16>,
    route: Option<&'static str>,
    rtt_ms: Option<u64>,
    tx_bytes: u64,
    rx_bytes: u64,
    error: Option<&'static str>,
    message: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanScanSnapshot {
    scanning: bool,
    port: Option<u16>,
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
pub fn start_lan_scan(state: State<'_, ConnectState>) -> Result<LanScanSnapshot, String> {
    let detected = *state
        .detected_port
        .lock()
        .map_err(|_| "LAN scan state is unavailable".to_owned())?;
    let mut scanner = state
        .scanner
        .lock()
        .map_err(|_| "LAN scan state is unavailable".to_owned())?;
    if detected.is_none() && scanner.is_none() {
        *scanner = Some(LanScanner::start().map_err(|error| error.to_string())?);
    }
    Ok(LanScanSnapshot {
        scanning: scanner.is_some(),
        port: detected,
    })
}

#[tauri::command]
pub fn get_lan_scan(state: State<'_, ConnectState>) -> Result<LanScanSnapshot, String> {
    let mut scanner = state
        .scanner
        .lock()
        .map_err(|_| "LAN scan state is unavailable".to_owned())?;
    let mut detected = state
        .detected_port
        .lock()
        .map_err(|_| "LAN scan state is unavailable".to_owned())?;
    if let Some(current) = scanner.as_ref() {
        while let Ok(port) = current.try_recv() {
            *detected = Some(port.get());
        }
        if current.is_finished() {
            scanner.take();
        }
    }
    Ok(LanScanSnapshot {
        scanning: scanner.is_some(),
        port: *detected,
    })
}

#[tauri::command]
pub fn restart_lan_scan(state: State<'_, ConnectState>) -> Result<LanScanSnapshot, String> {
    state.stop_lan_scanner()?;
    start_lan_scan(state)
}

#[tauri::command]
pub fn stop_lan_scan(state: State<'_, ConnectState>) -> Result<(), String> {
    state.stop_lan_scanner()
}

#[tauri::command]
pub async fn start_host(
    port: u16,
    max_players: Option<u32>,
    app: AppHandle,
    state: State<'_, ConnectState>,
) -> Result<(), String> {
    if port == 0 {
        return Err("Minecraft 端口必须在 1 到 65535 之间".to_owned());
    }
    if !minecraft_available(port).await {
        return Err(format!(
            "端口 {port} 没有可用的 Minecraft 世界，请确认已开放局域网联机"
        ));
    }
    let identity = app.state::<SettingsState>().host_identity()?;
    let relay_url = app.state::<SettingsState>().relay_url()?;
    state.stop_broadcast();
    state.set_message(None);
    if let Ok(mut current_port) = state.host_port.lock() {
        *current_port = Some(port);
    }
    state.stop_lan_scanner()?;
    let config = HostConfig::new()
        .event_delay(Duration::from_secs(1))
        .max_players(max_players);
    let options = HostOptions::new(port)
        .secret_key(Some(identity.secret_key))
        .relay_url(relay_url)
        .service_id(identity.service_id)
        .token(identity.token)
        .config(config);
    if let Err(error) = state.service.start_host(options).await {
        if let Ok(mut current_port) = state.host_port.lock() {
            *current_port = None;
        }
        return Err(error.to_string());
    }
    let generation = state
        .host_monitor_generation
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_add(1);
    tauri::async_runtime::spawn(monitor_host(app, generation, port));
    Ok(())
}

#[tauri::command]
pub async fn start_join(
    uri: String,
    local_port: Option<u16>,
    settings: State<'_, SettingsState>,
    state: State<'_, ConnectState>,
) -> Result<(), String> {
    let uri = uri.trim().to_owned();
    let join_uri = uri.parse::<JoinUri>().map_err(|error| error.to_string())?;
    state.set_message(None);
    state.set_pending_join_uri(Some(uri));
    state
        .host_monitor_generation
        .fetch_add(1, Ordering::Relaxed);
    if let Ok(mut current_port) = state.host_port.lock() {
        *current_port = None;
    }
    let config = JoinConfig::new()
        .event_delay(Duration::from_secs(1))
        .reconnect_timeout(settings.reconnect_timeout()?);
    let local_port = match local_port {
        Some(port) => LocalPort::Fixed(
            NonZeroU16::new(port).ok_or_else(|| "本地端口必须在 1 到 65535 之间".to_owned())?,
        ),
        None => LocalPort::Auto,
    };
    if let Err(error) = state
        .service
        .start_join(
            JoinOptions::new(join_uri)
                .local_port(local_port)
                .config(config),
        )
        .await
    {
        state.set_pending_join_uri(None);
        return Err(error.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_join(state: State<'_, ConnectState>) -> Result<(), String> {
    state.stop_broadcast();
    state.set_message(None);
    state.set_pending_join_uri(None);
    state
        .host_monitor_generation
        .fetch_add(1, Ordering::Relaxed);
    state
        .service
        .shutdown()
        .await
        .map_err(|error| error.to_string())
}

async fn minecraft_available(port: u16) -> bool {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    tokio::task::spawn_blocking(move || probe_server(addr, MC_PROBE_TIMEOUT))
        .await
        .is_ok_and(|result| result.is_ok())
}

async fn monitor_host(app: AppHandle, generation: u64, port: u16) {
    let mut failures = 0;
    let mut interval = tokio::time::interval(MC_HEALTH_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval.tick().await;

    loop {
        interval.tick().await;
        {
            let state = app.state::<ConnectState>();
            if state.host_monitor_generation.load(Ordering::Relaxed) != generation {
                return;
            }
            let status = state.service.status();
            if status.state.mode != Some(TunnelMode::Host) {
                return;
            }
        }
        if !record_health_check(&mut failures, minecraft_available(port).await) {
            continue;
        }

        let service = {
            let state = app.state::<ConnectState>();
            state.set_message(Some("Minecraft 世界已关闭，房间已自动停止。".to_owned()));
            state
                .host_monitor_generation
                .fetch_add(1, Ordering::Relaxed);
            state.service.clone()
        };
        let _ = service.shutdown().await;
        let state = app.state::<ConnectState>();
        let _ = app.emit(STATUS_EVENT, state.snapshot());
        return;
    }
}

fn record_health_check(failures: &mut u8, available: bool) -> bool {
    if available {
        *failures = 0;
        return false;
    }
    *failures = failures.saturating_add(1);
    *failures >= MC_HEALTH_FAILURES_MAX
}

#[tauri::command]
pub async fn stop_tunnel(state: State<'_, ConnectState>) -> Result<(), String> {
    stop_join(state).await
}

pub fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(SettingsState::load(app.handle())?);
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
            if status.state.phase == TunnelPhase::Idle
                && let Ok(mut current_port) = state.host_port.lock()
            {
                *current_port = None;
            }
            if let Err(error) = state.sync_broadcast(&status) {
                state.set_message(Some(format!("LAN broadcast failed: {error}")));
            }
            if status.state.phase == TunnelPhase::Active
                && let Some(uri) = state.take_pending_join_uri()
                && let Err(error) = app.state::<SettingsState>().remember_join_uri(uri)
            {
                state.set_message(Some(format!("保存偏好失败：{error}")));
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
        TunnelEvent::PlayerJoined { id } => Some(format!("玩家 {id} 已加入")),
        TunnelEvent::PlayerLeft { id, .. } => Some(format!("玩家 {id} 已离开")),
        TunnelEvent::Disconnected { reason } => Some(format!("连接已断开：{reason}")),
        TunnelEvent::Reconnecting { attempt } => Some(format!("正在进行第 {attempt} 次重连")),
        TunnelEvent::Reconnected => Some("已恢复连接".to_owned()),
        TunnelEvent::PathChanged {
            is_relay, rtt_ms, ..
        } => Some(format!(
            "当前使用{}，延迟 {rtt_ms} ms",
            if is_relay { "中继连接" } else { "直连" }
        )),
        TunnelEvent::Error { message, .. } => Some(message),
        _ => None,
    }
}

fn snapshot_from_status(
    status: TunnelStatus,
    message: Option<String>,
    host_port: Option<u16>,
) -> ConnectSnapshot {
    let connection = status
        .connections
        .iter()
        .find(|connection| connection.alive);
    ConnectSnapshot {
        phase: match status.state.phase {
            TunnelPhase::Idle => "idle",
            TunnelPhase::Starting => "starting",
            TunnelPhase::Active => "active",
            TunnelPhase::Stopping => "stopping",
        },
        mode: status.state.mode.map(|mode| match mode {
            TunnelMode::Host => "host",
            TunnelMode::Join => "join",
        }),
        local_address: status.state.local_addr.map(|addr| addr.to_string()),
        share_uri: status
            .state
            .join_uri
            .as_ref()
            .and_then(|uri| uri.expose_secret_uri().ok()),
        player_count: status
            .connections
            .iter()
            .filter(|connection| connection.alive)
            .count(),
        host_port,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_health_requires_three_consecutive_failures() {
        let mut failures = 0;
        assert!(!record_health_check(&mut failures, false));
        assert!(!record_health_check(&mut failures, false));
        assert!(record_health_check(&mut failures, false));
    }

    #[test]
    fn successful_host_health_check_resets_failures() {
        let mut failures = 2;
        assert!(!record_health_check(&mut failures, true));
        assert_eq!(failures, 0);
        assert!(!record_health_check(&mut failures, false));
    }
}
