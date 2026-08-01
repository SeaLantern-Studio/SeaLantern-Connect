use sculk::ErrorCategory;
use sculk::minecraft::probe_server;
use sculk::persist::{self, HostState};
use sculk::tunnel::{
    HostConfig, HostedServiceHandle, HostedServiceOptions, HostedServiceStatus, NodeOptions,
    SculkNode, SecretKey, ServiceId, TokenRefreshPolicy, TunnelEvent,
};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

const HOST_START_TIMEOUT: Duration = Duration::from_secs(15);
const MC_PROBE_TIMEOUT: Duration = Duration::from_secs(1);
const MC_HEALTH_INTERVAL: Duration = Duration::from_secs(5);
const MC_HEALTH_FAILURES_MAX: u8 = 3;

pub struct HostStart {
    pub mc_port: u16,
    pub max_players: Option<u32>,
    pub secret_key: SecretKey,
    pub relay_url: Option<sculk::tunnel::RelayUrl>,
    pub token_refresh: TokenRefreshPolicy,
    pub state_path: PathBuf,
}

pub enum HostUpdate {
    Started {
        uri: String,
        status: HostedServiceStatus,
    },
    Status(HostedServiceStatus),
    UriChanged {
        uri: String,
        status: HostedServiceStatus,
    },
    Event(TunnelEvent),
    Error(String),
    MinecraftUnavailable,
    Failed(String),
    Stopped(Result<(), String>),
}

enum HostCommand {
    Stop,
}

pub struct HostTask {
    commands: mpsc::UnboundedSender<HostCommand>,
}

impl HostTask {
    pub fn spawn(start: HostStart, updates: mpsc::UnboundedSender<HostUpdate>) -> Self {
        let (commands, command_rx) = mpsc::unbounded_channel();
        tauri::async_runtime::spawn(run_host(start, command_rx, updates));
        Self { commands }
    }

    pub fn stop(&self) -> bool {
        self.commands.send(HostCommand::Stop).is_ok()
    }
}

pub fn token_refresh_policy(value: &str) -> Option<TokenRefreshPolicy> {
    match value {
        "always" => Some(TokenRefreshPolicy::Always),
        "never" => Some(TokenRefreshPolicy::Never),
        "1h" => Some(TokenRefreshPolicy::After(Duration::from_secs(60 * 60))),
        "3h" => Some(TokenRefreshPolicy::After(Duration::from_secs(3 * 60 * 60))),
        "6h" => Some(TokenRefreshPolicy::After(Duration::from_secs(6 * 60 * 60))),
        "12h" => Some(TokenRefreshPolicy::After(Duration::from_secs(12 * 60 * 60))),
        "24h" => Some(TokenRefreshPolicy::After(Duration::from_secs(24 * 60 * 60))),
        _ => None,
    }
}

async fn run_host(
    start: HostStart,
    mut commands: mpsc::UnboundedReceiver<HostCommand>,
    updates: mpsc::UnboundedSender<HostUpdate>,
) {
    let target_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, start.mc_port));
    let saved = match persist::load_host_state(&start.state_path) {
        Ok(saved) => saved,
        Err(error) => return send(&updates, HostUpdate::Failed(error.to_string())),
    };
    let service_id = saved
        .as_ref()
        .map_or_else(ServiceId::generate, |state| state.service_id);
    let token_state = saved.map(|state| state.token_state);
    let node = match tokio::time::timeout(
        HOST_START_TIMEOUT,
        SculkNode::bind(NodeOptions {
            secret_key: Some(start.secret_key),
            relay_url: start.relay_url,
            ..NodeOptions::default()
        }),
    )
    .await
    {
        Ok(Ok(node)) => node,
        Ok(Err(error)) => return send(&updates, HostUpdate::Failed(error.to_string())),
        Err(_) => {
            return send(
                &updates,
                HostUpdate::Failed("node startup timed out; check relay settings".to_owned()),
            );
        }
    };
    let host = match node
        .start_service(HostedServiceOptions {
            service_id,
            target_addr,
            token_state,
            token_refresh: start.token_refresh,
            config: HostConfig::new().max_players(start.max_players),
        })
        .await
    {
        Ok(host) => host,
        Err(error) => {
            node.close().await;
            return send(&updates, HostUpdate::Failed(error.to_string()));
        }
    };
    let mut events = match host.subscribe().await {
        Ok(events) => events,
        Err(error) => {
            node.close().await;
            return send(&updates, HostUpdate::Failed(error.to_string()));
        }
    };
    let mut statuses = match host.subscribe_status().await {
        Ok(statuses) => statuses,
        Err(error) => {
            node.close().await;
            return send(&updates, HostUpdate::Failed(error.to_string()));
        }
    };
    if let Err(error) = persist_host_state(&start.state_path, &host).await {
        node.close().await;
        return send(&updates, HostUpdate::Failed(error));
    }
    let status = match host.status().await {
        Ok(status) => status,
        Err(error) => {
            node.close().await;
            return send(&updates, HostUpdate::Failed(error.to_string()));
        }
    };
    let uri = match expose_host_uri(&host).await {
        Ok(uri) => uri,
        Err(error) => {
            node.close().await;
            return send(&updates, HostUpdate::Failed(error));
        }
    };
    let mut uri_generation = status.uri_generation;
    if updates.send(HostUpdate::Started { uri, status }).is_err() {
        node.close().await;
        return;
    }

    let first_health_check = tokio::time::Instant::now() + MC_HEALTH_INTERVAL;
    let mut health_checks = tokio::time::interval_at(first_health_check, MC_HEALTH_INTERVAL);
    health_checks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut health_failures = 0_u8;
    let mut pending_target_error = None;
    loop {
        tokio::select! {
            command = commands.recv() => {
                if matches!(command, Some(HostCommand::Stop)) {
                    let result = host.stop().await.map_err(|error| error.to_string());
                    node.close().await;
                    send(&updates, HostUpdate::Stopped(result));
                } else {
                    node.close().await;
                }
                return;
            }
            event = events.recv() => {
                match event {
                    Ok(event) if is_target_unavailable_event(&event) => {
                        pending_target_error.get_or_insert(event);
                    }
                    Ok(event) => send(&updates, HostUpdate::Event(event)),
                    Err(broadcast::error::RecvError::Lagged(count)) => send(
                        &updates,
                        HostUpdate::Error(format!("missed {count} host events")),
                    ),
                    Err(broadcast::error::RecvError::Closed) => {
                        node.close().await;
                        send(&updates, HostUpdate::Failed(
                            "host event channel closed unexpectedly".to_owned(),
                        ));
                        return;
                    }
                }
            }
            status = statuses.recv() => {
                let Some(status) = status else {
                    node.close().await;
                    send(&updates, HostUpdate::Failed(
                        "host status channel closed unexpectedly".to_owned(),
                    ));
                    return;
                };
                if status.uri_generation > uri_generation {
                    uri_generation = status.uri_generation;
                    if let Err(error) = persist_host_state(&start.state_path, &host).await {
                        node.close().await;
                        send(&updates, HostUpdate::Failed(error));
                        return;
                    }
                    match expose_host_uri(&host).await {
                        Ok(uri) => send(&updates, HostUpdate::UriChanged { uri, status }),
                        Err(error) => {
                            node.close().await;
                            send(&updates, HostUpdate::Failed(error));
                            return;
                        }
                    }
                } else {
                    send(&updates, HostUpdate::Status(status));
                }
            }
            _ = health_checks.tick() => {
                let available = minecraft_available(target_addr).await;
                if available && let Some(event) = pending_target_error.take() {
                    send(&updates, HostUpdate::Event(event));
                }
                if !record_health_check(&mut health_failures, available) {
                    continue;
                }
                let result = host.stop().await.map_err(|error| error.to_string());
                node.close().await;
                send(&updates, match result {
                    Ok(()) => HostUpdate::MinecraftUnavailable,
                    Err(error) => HostUpdate::Failed(error),
                });
                return;
            }
        }
    }
}

fn send(updates: &mpsc::UnboundedSender<HostUpdate>, update: HostUpdate) {
    let _ = updates.send(update);
}

async fn minecraft_available(addr: SocketAddr) -> bool {
    tokio::task::spawn_blocking(move || probe_server(addr, MC_PROBE_TIMEOUT))
        .await
        .is_ok_and(|result| result.is_ok())
}

fn record_health_check(failures: &mut u8, available: bool) -> bool {
    if available {
        *failures = 0;
        return false;
    }
    *failures = failures.saturating_add(1);
    *failures >= MC_HEALTH_FAILURES_MAX
}

fn is_target_unavailable_event(event: &TunnelEvent) -> bool {
    matches!(
        event,
        TunnelEvent::Error {
            category: ErrorCategory::TargetUnavailable,
            ..
        }
    )
}

async fn persist_host_state(path: &Path, host: &HostedServiceHandle) -> Result<(), String> {
    let token_state = host
        .token_state()
        .await
        .map_err(|error| error.to_string())?;
    persist::save_host_state(
        path,
        &HostState {
            service_id: host.service_id(),
            token_state,
        },
    )
    .map_err(|error| error.to_string())
}

async fn expose_host_uri(host: &HostedServiceHandle) -> Result<String, String> {
    host.join_uri()
        .await
        .map_err(|error| error.to_string())?
        .expose_secret_uri()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_timed_refresh_policy() {
        assert_eq!(
            token_refresh_policy("3h"),
            Some(TokenRefreshPolicy::After(Duration::from_secs(3 * 60 * 60)))
        );
        assert_eq!(token_refresh_policy("invalid"), None);
    }

    #[test]
    fn maps_every_supported_refresh_policy() {
        assert_eq!(
            token_refresh_policy("always"),
            Some(TokenRefreshPolicy::Always)
        );
        assert_eq!(
            token_refresh_policy("never"),
            Some(TokenRefreshPolicy::Never)
        );
        for (value, hours) in [("1h", 1), ("3h", 3), ("6h", 6), ("12h", 12), ("24h", 24)] {
            assert_eq!(
                token_refresh_policy(value),
                Some(TokenRefreshPolicy::After(Duration::from_secs(
                    hours * 60 * 60
                )))
            );
        }
    }

    #[test]
    fn health_check_requires_consecutive_failures() {
        let mut failures = 0;
        assert!(!record_health_check(&mut failures, false));
        assert!(!record_health_check(&mut failures, false));
        assert!(record_health_check(&mut failures, false));

        assert!(!record_health_check(&mut failures, true));
        assert_eq!(failures, 0);
    }
}
