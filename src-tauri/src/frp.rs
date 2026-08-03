mod client;
mod openfrp;
mod sakurafrp;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

const STATUS_EVENT: &str = "frp-client-status";
const PROGRESS_EVENT: &str = "frp-download-progress";
const CREDENTIAL_SERVICE: &str = "SeaLantern Connect FRP";
const OPENFRP_PREMIUM_URL: &str = "https://console.openfrp.net/premium";
const SAKURA_KEYS_URL: &str = "https://www.natfrp.com/user/";
const SAKURA_PURCHASE_URL: &str = "https://www.natfrp.com/purchase/buy";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FrpProvider {
    OpenFrp,
    SakuraFrp,
}

impl FrpProvider {
    fn directory(self) -> &'static str {
        match self {
            Self::OpenFrp => "openfrp",
            Self::SakuraFrp => "sakurafrp",
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::OpenFrp => "OpenFRP",
            Self::SakuraFrp => "SakuraFRP",
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrpClientStatus {
    provider: FrpProvider,
    installed: bool,
    downloading: bool,
    path: String,
    error: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FrpDownloadProgress {
    provider: FrpProvider,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: u8,
}

pub(crate) struct FrpState {
    downloading: Mutex<Option<FrpProvider>>,
    credentials: Mutex<HashMap<FrpProvider, String>>,
    accounts: Mutex<HashMap<FrpProvider, String>>,
    processes: Mutex<HashMap<FrpProvider, Child>>,
}

impl FrpState {
    pub(crate) fn new() -> Self {
        Self {
            downloading: Mutex::new(None),
            credentials: Mutex::new(HashMap::new()),
            accounts: Mutex::new(HashMap::new()),
            processes: Mutex::new(HashMap::new()),
        }
    }

    fn begin_download(&self, provider: FrpProvider) -> Result<(), String> {
        let mut downloading = self
            .downloading
            .lock()
            .map_err(|_| "FRP download state is unavailable".to_owned())?;
        if downloading.is_some() {
            return Err("another FRP client download is already running".to_owned());
        }
        *downloading = Some(provider);
        Ok(())
    }

    fn end_download(&self, provider: FrpProvider) {
        if let Ok(mut downloading) = self.downloading.lock()
            && *downloading == Some(provider)
        {
            *downloading = None;
        }
    }

    fn is_downloading(&self, provider: FrpProvider) -> bool {
        self.downloading
            .lock()
            .map(|value| *value == Some(provider))
            .unwrap_or(false)
    }
}

impl Drop for FrpState {
    fn drop(&mut self) {
        if let Ok(processes) = self.processes.get_mut() {
            for process in processes.values_mut() {
                let _ = process.kill();
                let _ = process.wait();
            }
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrpSessionStatus {
    provider: FrpProvider,
    authenticated: bool,
    account_name: Option<String>,
    running: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrpTunnel {
    id: String,
    name: String,
    node: Option<String>,
    local_port: Option<u16>,
    remote_endpoint: Option<String>,
    online: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrpNode {
    id: String,
    name: String,
    description: Option<String>,
    vip: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateFrpTunnel {
    node_id: String,
    name: String,
    local_port: u16,
    remote_port: String,
}

fn status(
    app: &AppHandle,
    state: &FrpState,
    provider: FrpProvider,
    error: Option<String>,
) -> Result<FrpClientStatus, String> {
    let path = client::path(app, provider)?;
    Ok(FrpClientStatus {
        provider,
        installed: path.is_file(),
        downloading: state.is_downloading(provider),
        path: path.to_string_lossy().into_owned(),
        error,
    })
}

fn emit_status(app: &AppHandle, status: &FrpClientStatus) {
    if let Err(error) = app.emit(STATUS_EVENT, status) {
        log::warn!("failed to emit FRP client status: {error}");
    }
}

#[tauri::command]
pub(crate) fn get_frp_client_status(
    app: AppHandle,
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpClientStatus, String> {
    status(&app, &state, provider, None)
}

#[tauri::command]
pub(crate) async fn download_frp_client(
    app: AppHandle,
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpClientStatus, String> {
    state.begin_download(provider)?;
    emit_status(&app, &status(&app, &state, provider, None)?);

    let result = client::install(&app, provider).await;
    if let Err(error) = &result {
        log::error!(
            "failed to install {} client: {error}",
            provider.display_name()
        );
    }
    state.end_download(provider);
    let final_status = status(&app, &state, provider, result.as_ref().err().cloned())?;
    emit_status(&app, &final_status);
    result.map(|_| final_status)
}

#[tauri::command]
pub(crate) async fn get_frp_session_status(
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpSessionStatus, String> {
    restore_session(&state, provider).await;
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) async fn login_frp(
    state: State<'_, FrpState>,
    provider: FrpProvider,
    credential: String,
) -> Result<FrpSessionStatus, String> {
    let credential = clean_credential(provider, &credential);
    if credential.is_empty() {
        return Err("provider credential is required".to_owned());
    }
    let account = match provider {
        FrpProvider::OpenFrp => openfrp::account(&credential).await?,
        FrpProvider::SakuraFrp => sakurafrp::account(&credential).await?,
    };
    remember_session(&state, provider, credential, account)?;
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) async fn login_openfrp(
    app: AppHandle,
    state: State<'_, FrpState>,
) -> Result<FrpSessionStatus, String> {
    let credential = openfrp::browser(&app).await?;
    let account = openfrp::account(&credential).await?;
    remember_session(&state, FrpProvider::OpenFrp, credential, account)?;
    session_status(&state, FrpProvider::OpenFrp)
}

#[tauri::command]
pub(crate) fn open_sakura_keys(app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(SAKURA_KEYS_URL, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn open_sakura_purchase(app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(SAKURA_PURCHASE_URL, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn open_premium(app: AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(OPENFRP_PREMIUM_URL, None::<&str>)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn logout_frp(
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpSessionStatus, String> {
    stop_process(&state, provider)?;
    state
        .credentials
        .lock()
        .map_err(|_| "FRP credential state is unavailable".to_owned())?
        .remove(&provider);
    state
        .accounts
        .lock()
        .map_err(|_| "FRP account state is unavailable".to_owned())?
        .remove(&provider);
    if let Err(error) = remove_saved(provider) {
        log::warn!(
            "failed to remove saved {} credential: {error}",
            provider.display_name()
        );
    }
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) async fn list_frp_tunnels(
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<Vec<FrpTunnel>, String> {
    let credential = credential(&state, provider)?;
    tunnels(provider, &credential).await
}

#[tauri::command]
pub(crate) async fn list_frp_nodes(
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<Vec<FrpNode>, String> {
    let credential = credential(&state, provider)?;
    match provider {
        FrpProvider::OpenFrp => openfrp::nodes(&credential).await,
        FrpProvider::SakuraFrp => sakurafrp::nodes(&credential).await,
    }
}

#[tauri::command]
pub(crate) async fn create_frp_tunnel(
    state: State<'_, FrpState>,
    provider: FrpProvider,
    request: CreateFrpTunnel,
) -> Result<Vec<FrpTunnel>, String> {
    validate_tunnel(&request)?;
    let credential = credential(&state, provider)?;
    match provider {
        FrpProvider::OpenFrp => openfrp::create(&credential, &request).await?,
        FrpProvider::SakuraFrp => sakurafrp::create(&credential, &request).await?,
    }
    tunnels(provider, &credential).await
}

#[tauri::command]
pub(crate) async fn delete_frp_tunnel(
    state: State<'_, FrpState>,
    provider: FrpProvider,
    tunnel_id: String,
) -> Result<Vec<FrpTunnel>, String> {
    let tunnel_id = tunnel_id.trim();
    if tunnel_id.is_empty() {
        return Err("a tunnel must be selected".to_owned());
    }
    if session_status(&state, provider)?.running {
        return Err("stop the mapping before deleting its tunnel".to_owned());
    }
    let credential = credential(&state, provider)?;
    match provider {
        FrpProvider::OpenFrp => openfrp::remove(&credential, tunnel_id).await?,
        FrpProvider::SakuraFrp => sakurafrp::remove(&credential, tunnel_id).await?,
    }
    tunnels(provider, &credential).await
}

#[tauri::command]
pub(crate) fn start_frp_tunnel(
    app: AppHandle,
    state: State<'_, FrpState>,
    provider: FrpProvider,
    tunnel_id: String,
) -> Result<FrpSessionStatus, String> {
    if tunnel_id.trim().is_empty() {
        return Err("a tunnel must be selected".to_owned());
    }
    let executable = client::path(&app, provider)?;
    if !executable.is_file() {
        return Err("the provider client is not installed".to_owned());
    }
    let token = credential(&state, provider)?;
    let mut processes = state
        .processes
        .lock()
        .map_err(|_| "FRP process state is unavailable".to_owned())?;
    if let Some(process) = processes.get_mut(&provider)
        && process
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_none()
    {
        return Err("this FRP provider is already running".to_owned());
    }
    processes.remove(&provider);

    let mut command = Command::new(&executable);
    match provider {
        FrpProvider::OpenFrp => {
            command.args(["-u", &token, "-p", tunnel_id.trim()]);
        }
        FrpProvider::SakuraFrp => {
            command.args(["-f", &format!("{token}:{}", tunnel_id.trim())]);
        }
    }
    let process = command
        .current_dir(client::directory(&app, provider)?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;
    log::info!("started {} tunnel process", provider.display_name());
    processes.insert(provider, process);
    drop(processes);
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) fn stop_frp_tunnel(
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpSessionStatus, String> {
    stop_process(&state, provider)?;
    session_status(&state, provider)
}

async fn tunnels(provider: FrpProvider, credential: &str) -> Result<Vec<FrpTunnel>, String> {
    match provider {
        FrpProvider::OpenFrp => openfrp::tunnels(credential).await,
        FrpProvider::SakuraFrp => sakurafrp::tunnels(credential).await,
    }
}

async fn restore_session(state: &FrpState, provider: FrpProvider) {
    if state
        .accounts
        .lock()
        .map(|accounts| accounts.contains_key(&provider))
        .unwrap_or(false)
    {
        return;
    }
    let Some(credential) = load_saved(provider) else {
        return;
    };
    let account = match provider {
        FrpProvider::OpenFrp => openfrp::account(&credential).await,
        FrpProvider::SakuraFrp => sakurafrp::account(&credential).await,
    };
    match account {
        Ok(account) => {
            if let Err(error) = cache_session(state, provider, credential, account) {
                log::warn!(
                    "failed to restore {} session: {error}",
                    provider.display_name()
                );
            }
        }
        Err(error) => {
            log::warn!(
                "could not restore saved {} credential: {error}",
                provider.display_name()
            );
        }
    }
}

fn remember_session(
    state: &FrpState,
    provider: FrpProvider,
    credential: String,
    account: String,
) -> Result<(), String> {
    if let Err(error) = save_credential(provider, &credential) {
        log::warn!(
            "failed to persist {} credential: {error}",
            provider.display_name()
        );
    }
    cache_session(state, provider, credential, account)
}

fn cache_session(
    state: &FrpState,
    provider: FrpProvider,
    credential: String,
    account: String,
) -> Result<(), String> {
    state
        .credentials
        .lock()
        .map_err(|_| "FRP credential state is unavailable".to_owned())?
        .insert(provider, credential);
    state
        .accounts
        .lock()
        .map_err(|_| "FRP account state is unavailable".to_owned())?
        .insert(provider, account);
    Ok(())
}

fn credential_entry(provider: FrpProvider) -> Result<keyring::Entry, String> {
    keyring::Entry::new(CREDENTIAL_SERVICE, provider.directory()).map_err(|error| error.to_string())
}

fn save_credential(provider: FrpProvider, credential: &str) -> Result<(), String> {
    credential_entry(provider)?
        .set_password(credential)
        .map_err(|error| error.to_string())
}

fn load_saved(provider: FrpProvider) -> Option<String> {
    match credential_entry(provider)
        .and_then(|entry| entry.get_password().map_err(|error| error.to_string()))
    {
        Ok(credential) => Some(credential),
        Err(error) => {
            log::debug!("no saved {} credential: {error}", provider.display_name());
            None
        }
    }
}

fn remove_saved(provider: FrpProvider) -> Result<(), String> {
    credential_entry(provider)?
        .delete_credential()
        .map_err(|error| error.to_string())
}

fn credential(state: &FrpState, provider: FrpProvider) -> Result<String, String> {
    state
        .credentials
        .lock()
        .map_err(|_| "FRP credential state is unavailable".to_owned())?
        .get(&provider)
        .cloned()
        .ok_or_else(|| "the provider is not authorized".to_owned())
}

fn session_status(state: &FrpState, provider: FrpProvider) -> Result<FrpSessionStatus, String> {
    let account_name = state
        .accounts
        .lock()
        .map_err(|_| "FRP account state is unavailable".to_owned())?
        .get(&provider)
        .cloned();
    let mut processes = state
        .processes
        .lock()
        .map_err(|_| "FRP process state is unavailable".to_owned())?;
    let running = match processes.get_mut(&provider) {
        Some(process) => process
            .try_wait()
            .map_err(|error| error.to_string())?
            .is_none(),
        None => false,
    };
    if !running {
        processes.remove(&provider);
    }
    Ok(FrpSessionStatus {
        provider,
        authenticated: account_name.is_some(),
        account_name,
        running,
    })
}

fn stop_process(state: &FrpState, provider: FrpProvider) -> Result<(), String> {
    if let Some(mut process) = state
        .processes
        .lock()
        .map_err(|_| "FRP process state is unavailable".to_owned())?
        .remove(&provider)
    {
        process.kill().map_err(|error| error.to_string())?;
        process.wait().map_err(|error| error.to_string())?;
        log::info!("stopped {} tunnel process", provider.display_name());
    }
    Ok(())
}

fn validate_tunnel(request: &CreateFrpTunnel) -> Result<(), String> {
    if request.node_id.trim().is_empty() {
        return Err("an FRP node must be selected".to_owned());
    }
    if request.name.trim().is_empty() || request.name.chars().count() > 32 {
        return Err("the tunnel name must contain 1 to 32 characters".to_owned());
    }
    if !request.remote_port.is_empty() && request.remote_port.parse::<u16>().is_err() {
        return Err("the remote port must be empty or between 1 and 65535".to_owned());
    }
    Ok(())
}

fn clean_credential(provider: FrpProvider, value: &str) -> String {
    let mut value = value.trim();
    if provider == FrpProvider::OpenFrp
        && let Some((name, content)) = value.split_once(':')
        && name.trim().eq_ignore_ascii_case("authorization")
    {
        value = content.trim();
    }
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        value = &value[1..value.len() - 1];
    }
    value.trim().to_owned()
}

fn value_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(value) => value.to_string(),
        None => String::new(),
    }
}

fn value_u16(value: Option<&Value>) -> Option<u16> {
    value.and_then(|value| {
        value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
    })
}

fn api_message(value: &Value, fallback: &str) -> String {
    value
        .get("msg")
        .and_then(Value::as_str)
        .unwrap_or(fallback)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{FrpProvider, clean_credential};

    #[test]
    fn cleans_openfrp_auth() {
        assert_eq!(
            clean_credential(FrpProvider::OpenFrp, " Authorization: 'Bearer token' "),
            "Bearer token"
        );
    }

    #[test]
    fn keeps_sakura_token() {
        assert_eq!(
            clean_credential(FrpProvider::SakuraFrp, "key:value"),
            "key:value"
        );
    }
}
