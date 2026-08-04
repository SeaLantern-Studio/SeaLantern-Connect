mod client;
mod openfrp;
mod sakurafrp;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use crypto_box::aead::{Aead, AeadCore, OsRng};
use crypto_box::{SalsaBox, SecretKey};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;
use tempfile::NamedTempFile;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};

const STATUS_EVENT: &str = "frp-client-status";
const PROGRESS_EVENT: &str = "frp-download-progress";
const OPENFRP_PREMIUM_URL: &str = "https://console.openfrp.net/premium";
const SAKURA_KEYS_URL: &str = "https://www.natfrp.com/user/";
const SAKURA_PURCHASE_URL: &str = "https://www.natfrp.com/purchase/buy";
const MAX_OUTPUT_LINES: usize = 120;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CREDENTIAL_FILE: &str = "credential.dat";
const CREDENTIAL_VERSION: u8 = 1;
// This keeps tokens out of app-local files without depending on an OS credential prompt.
const CREDENTIAL_KEY: [u8; 32] = [
    0x56, 0xa4, 0x12, 0xd9, 0x7c, 0x3e, 0xf1, 0x08, 0x65, 0xb0, 0x2d, 0x94, 0xce, 0x41, 0x7a, 0x1f,
    0xe8, 0x33, 0x6b, 0x05, 0xaf, 0x72, 0x19, 0xc4, 0x90, 0x2e, 0xbd, 0x58, 0xf6, 0x0a, 0x47, 0x81,
];

type OutputMap = Arc<Mutex<HashMap<FrpProvider, VecDeque<String>>>>;

#[cfg(target_os = "windows")]
struct FrpProcessJob {
    handle: usize,
}

#[cfg(target_os = "windows")]
impl FrpProcessJob {
    fn new() -> Result<Self, String> {
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }

        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&limits).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if configured == 0 {
            unsafe {
                CloseHandle(handle);
            }
            return Err(std::io::Error::last_os_error().to_string());
        }

        Ok(Self {
            handle: handle as usize,
        })
    }

    fn assign(&self, process: &Child) -> Result<(), String> {
        let assigned = unsafe {
            AssignProcessToJobObject(self.handle as HANDLE, process.as_raw_handle() as HANDLE)
        };
        if assigned == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Drop for FrpProcessJob {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.handle as HANDLE);
        }
    }
}

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

    fn log_target(self) -> &'static str {
        match self {
            Self::OpenFrp => crate::logging::OPENFRP_LOG_TARGET,
            Self::SakuraFrp => crate::logging::SAKURAFRP_LOG_TARGET,
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
    tunnel_ids: Mutex<HashMap<FrpProvider, String>>,
    outputs: OutputMap,
    #[cfg(target_os = "windows")]
    process_job: FrpProcessJob,
}

impl FrpState {
    pub(crate) fn new() -> Self {
        Self {
            downloading: Mutex::new(None),
            credentials: Mutex::new(HashMap::new()),
            accounts: Mutex::new(HashMap::new()),
            processes: Mutex::new(HashMap::new()),
            tunnel_ids: Mutex::new(HashMap::new()),
            outputs: Arc::new(Mutex::new(HashMap::new())),
            #[cfg(target_os = "windows")]
            process_job: FrpProcessJob::new().expect("failed to create FRP process job object"),
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

    pub(crate) fn stop_all(&self) {
        let Ok(mut processes) = self.processes.lock() else {
            log::warn!("FRP process state is unavailable during shutdown");
            return;
        };
        for (provider, process) in processes.iter_mut() {
            let _ = process.kill();
            let _ = process.wait();
            log::info!(
                "stopped {} tunnel process during shutdown",
                provider.display_name()
            );
        }
        processes.clear();
    }
}

impl Drop for FrpState {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FrpSessionStatus {
    provider: FrpProvider,
    authenticated: bool,
    account_name: Option<String>,
    running: bool,
    tunnel_id: Option<String>,
    output: Vec<String>,
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
    allow_port: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateFrpTunnel {
    node_id: String,
    name: String,
    local_port: u16,
    remote_port: String,
}

#[derive(Deserialize, Serialize)]
struct EncryptedCredential {
    version: u8,
    nonce: String,
    ciphertext: String,
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
    app: AppHandle,
    state: State<'_, FrpState>,
    provider: FrpProvider,
) -> Result<FrpSessionStatus, String> {
    restore_session(&app, &state, provider).await;
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) async fn login_sakurafrp(
    app: AppHandle,
    state: State<'_, FrpState>,
    credential: String,
) -> Result<FrpSessionStatus, String> {
    let provider = FrpProvider::SakuraFrp;
    let credential = clean_token(&credential);
    if credential.is_empty() {
        return Err("provider credential is required".to_owned());
    }
    let account = sakurafrp::account(&credential).await?;
    remember_session(&app, &state, provider, credential, account)?;
    session_status(&state, provider)
}

#[tauri::command]
pub(crate) async fn login_openfrp(
    app: AppHandle,
    state: State<'_, FrpState>,
) -> Result<FrpSessionStatus, String> {
    let credential = openfrp::browser(&app).await?;
    let account = openfrp::account(&credential).await?;
    remember_session(&app, &state, FrpProvider::OpenFrp, credential, account)?;
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
    app: AppHandle,
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
    state
        .tunnel_ids
        .lock()
        .map_err(|_| "FRP tunnel state is unavailable".to_owned())?
        .remove(&provider);
    state
        .outputs
        .lock()
        .map_err(|_| "FRP output state is unavailable".to_owned())?
        .remove(&provider);
    if let Err(error) = remove_saved(&app, provider) {
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
    validate_tunnel(provider, &request)?;
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
    {
        let mut tunnel_ids = state
            .tunnel_ids
            .lock()
            .map_err(|_| "FRP tunnel state is unavailable".to_owned())?;
        if tunnel_ids
            .get(&provider)
            .is_some_and(|active| active == tunnel_id)
        {
            tunnel_ids.remove(&provider);
        }
    }
    tunnels(provider, &credential).await
}

#[tauri::command]
pub(crate) async fn start_frp_tunnel(
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
    let credential = credential(&state, provider)?;
    let token = match provider {
        FrpProvider::OpenFrp => openfrp::token(&credential).await?,
        FrpProvider::SakuraFrp => credential,
    };
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
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    let mut process = command
        .current_dir(client::directory(&app, provider)?)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    #[cfg(target_os = "windows")]
    if let Err(error) = state.process_job.assign(&process) {
        let _ = process.kill();
        let _ = process.wait();
        return Err(format!("failed to supervise FRP process: {error}"));
    }
    state
        .outputs
        .lock()
        .map_err(|_| "FRP output state is unavailable".to_owned())?
        .insert(provider, VecDeque::new());
    if let Some(stdout) = process.stdout.take() {
        capture_output(stdout, provider, state.outputs.clone());
    }
    if let Some(stderr) = process.stderr.take() {
        capture_output(stderr, provider, state.outputs.clone());
    }
    log::info!("started {} tunnel process", provider.display_name());
    processes.insert(provider, process);
    state
        .tunnel_ids
        .lock()
        .map_err(|_| "FRP tunnel state is unavailable".to_owned())?
        .insert(provider, tunnel_id.trim().to_owned());
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

async fn restore_session(app: &AppHandle, state: &FrpState, provider: FrpProvider) {
    if state
        .accounts
        .lock()
        .map(|accounts| accounts.contains_key(&provider))
        .unwrap_or(false)
    {
        return;
    }
    let Some(credential) = load_saved(app, provider) else {
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
    app: &AppHandle,
    state: &FrpState,
    provider: FrpProvider,
    credential: String,
    account: String,
) -> Result<(), String> {
    if let Err(error) = save_credential(app, provider, &credential) {
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

fn credential_path(app: &AppHandle, provider: FrpProvider) -> Result<PathBuf, String> {
    Ok(client::directory(app, provider)?.join(CREDENTIAL_FILE))
}

fn save_credential(app: &AppHandle, provider: FrpProvider, credential: &str) -> Result<(), String> {
    let saved = encrypt_credential(credential)?;
    write_saved(&credential_path(app, provider)?, &saved)
}

fn encrypt_credential(credential: &str) -> Result<EncryptedCredential, String> {
    let secret = SecretKey::from(CREDENTIAL_KEY);
    let cipher = SalsaBox::new(&secret.public_key(), &secret);
    let nonce = SalsaBox::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, credential.as_bytes())
        .map_err(|_| "failed to encrypt credential".to_owned())?;
    Ok(EncryptedCredential {
        version: CREDENTIAL_VERSION,
        nonce: URL_SAFE_NO_PAD.encode(nonce.as_slice()),
        ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
    })
}

fn load_saved(app: &AppHandle, provider: FrpProvider) -> Option<String> {
    let path = match credential_path(app, provider) {
        Ok(path) => path,
        Err(error) => {
            log::warn!(
                "could not resolve saved {} credential: {error}",
                provider.display_name()
            );
            return None;
        }
    };
    match read_saved(&path) {
        Ok(Some(credential)) => Some(credential),
        Ok(None) => None,
        Err(error) => {
            log::warn!(
                "could not read saved {} credential: {error}",
                provider.display_name()
            );
            None
        }
    }
}

fn remove_saved(app: &AppHandle, provider: FrpProvider) -> Result<(), String> {
    let path = credential_path(app, provider)?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn write_saved(path: &Path, saved: &EncryptedCredential) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "credential directory is unavailable".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let content = serde_json::to_vec(saved).map_err(|error| error.to_string())?;
    let mut temporary = NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    temporary
        .write_all(&content)
        .map_err(|error| error.to_string())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        temporary
            .as_file()
            .set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|error| error.to_string())?;
    }
    let persisted = temporary
        .persist(path)
        .map_err(|error| error.error.to_string())?;
    persisted.sync_all().map_err(|error| error.to_string())?;

    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn read_saved(path: &Path) -> Result<Option<String>, String> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.to_string()),
    };
    let saved: EncryptedCredential =
        serde_json::from_slice(&content).map_err(|error| error.to_string())?;
    if saved.version != CREDENTIAL_VERSION {
        return Err("unsupported credential format".to_owned());
    }
    decrypt_credential(saved)
}

fn decrypt_credential(saved: EncryptedCredential) -> Result<Option<String>, String> {
    let nonce: [u8; 24] = URL_SAFE_NO_PAD
        .decode(saved.nonce)
        .map_err(|_| "credential nonce is invalid".to_owned())?
        .try_into()
        .map_err(|_| "credential nonce has an invalid length".to_owned())?;
    let ciphertext = URL_SAFE_NO_PAD
        .decode(saved.ciphertext)
        .map_err(|_| "credential ciphertext is invalid".to_owned())?;
    let secret = SecretKey::from(CREDENTIAL_KEY);
    let cipher = SalsaBox::new(&secret.public_key(), &secret);
    let credential = cipher
        .decrypt(crypto_box::Nonce::from_slice(&nonce), ciphertext.as_ref())
        .map_err(|_| "credential authentication failed".to_owned())?;
    String::from_utf8(credential)
        .map(Some)
        .map_err(|_| "credential is invalid".to_owned())
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
    let (running, exit_status) = match processes.get_mut(&provider) {
        Some(process) => match process.try_wait().map_err(|error| error.to_string())? {
            Some(status) => (false, Some(status)),
            None => (true, None),
        },
        None => (false, None),
    };
    if !running {
        processes.remove(&provider);
    }
    drop(processes);
    if let Some(status) = exit_status {
        push_output(
            &state.outputs,
            provider,
            format!("frpc exited with status {status}"),
        );
    }
    let tunnel_id = state
        .tunnel_ids
        .lock()
        .map_err(|_| "FRP tunnel state is unavailable".to_owned())?
        .get(&provider)
        .cloned();
    let output = state
        .outputs
        .lock()
        .map_err(|_| "FRP output state is unavailable".to_owned())?
        .get(&provider)
        .map(|lines| lines.iter().cloned().collect())
        .unwrap_or_default();
    Ok(FrpSessionStatus {
        provider,
        authenticated: account_name.is_some(),
        account_name,
        running,
        tunnel_id,
        output,
    })
}

fn capture_output<R>(reader: R, provider: FrpProvider, outputs: OutputMap)
where
    R: Read + Send + 'static,
{
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(line) => {
                    let line = strip_ansi(&line);
                    if line.trim().is_empty() {
                        continue;
                    }
                    log::info!(target: provider.log_target(), "{line}");
                    push_output(&outputs, provider, line);
                }
                Err(error) => {
                    log::warn!("failed to read {} output: {error}", provider.display_name());
                    break;
                }
            }
        }
    });
}

fn strip_ansi(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\u{1b}' || chars.next_if_eq(&'[').is_none() {
            result.push(character);
            continue;
        }
        for character in chars.by_ref() {
            if ('@'..='~').contains(&character) {
                break;
            }
        }
    }
    result
}

fn push_output(outputs: &OutputMap, provider: FrpProvider, line: String) {
    let Ok(mut outputs) = outputs.lock() else {
        return;
    };
    let lines = outputs.entry(provider).or_default();
    if lines.len() >= MAX_OUTPUT_LINES {
        lines.pop_front();
    }
    lines.push_back(line);
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

fn validate_tunnel(provider: FrpProvider, request: &CreateFrpTunnel) -> Result<(), String> {
    if request.node_id.trim().is_empty() {
        return Err("an FRP node must be selected".to_owned());
    }
    if !valid_tunnel_name(request.name.trim()) {
        return Err(
            "the tunnel name must be 2 to 32 characters, start with a letter, and contain only letters, numbers, underscores, or hyphens"
                .to_owned(),
        );
    }
    let remote_port = request.remote_port.trim();
    if remote_port.is_empty() && provider == FrpProvider::OpenFrp {
        return Err("OpenFRP TCP tunnels require a remote port".to_owned());
    }
    if !remote_port.is_empty() && !remote_port.parse::<u16>().is_ok_and(|port| port > 0) {
        return Err("the remote port must be between 1 and 65535".to_owned());
    }
    Ok(())
}

fn valid_tunnel_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    (2..=32).contains(&bytes.len())
        && bytes[0].is_ascii_alphabetic()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn clean_token(value: &str) -> String {
    let mut value = value.trim();
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
    use super::{
        clean_token, decrypt_credential, encrypt_credential, read_saved, strip_ansi,
        valid_tunnel_name, write_saved,
    };

    #[test]
    fn keeps_sakura_token() {
        assert_eq!(clean_token("key:value"), "key:value");
    }

    #[test]
    fn checks_tunnel_name() {
        assert!(valid_tunnel_name("SeaLantern_1"));
        assert!(!valid_tunnel_name("1server"));
        assert!(!valid_tunnel_name("a"));
        assert!(!valid_tunnel_name("中文"));
    }

    #[test]
    fn strips_ansi() {
        assert_eq!(strip_ansi("\u{1b}[0mready \u{1b}[1;34mnow"), "ready now");
    }

    #[test]
    fn saves_encrypted_credential() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credential.dat");
        let credential = "sakura-secret-token";
        let saved = encrypt_credential(credential).unwrap();

        write_saved(&path, &saved).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains(credential));
        assert_eq!(read_saved(&path).unwrap(), Some(credential.to_owned()));
        assert_eq!(
            decrypt_credential(saved).unwrap(),
            Some(credential.to_owned())
        );
    }

    #[test]
    fn ignores_missing_credential() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(
            read_saved(&directory.path().join("credential.dat")).unwrap(),
            None
        );
    }
}
