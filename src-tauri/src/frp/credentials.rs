use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use crypto_box::aead::{Aead, AeadCore, OsRng};
use crypto_box::{SalsaBox, SecretKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tempfile::NamedTempFile;

use super::{FrpProvider, client};

const CREDENTIAL_FILE: &str = "credential.dat";
const CREDENTIAL_VERSION: u8 = 1;
// This keeps tokens out of app-local files without depending on an OS credential prompt.
const CREDENTIAL_KEY: [u8; 32] = [
    0x56, 0xa4, 0x12, 0xd9, 0x7c, 0x3e, 0xf1, 0x08, 0x65, 0xb0, 0x2d, 0x94, 0xce, 0x41, 0x7a, 0x1f,
    0xe8, 0x33, 0x6b, 0x05, 0xaf, 0x72, 0x19, 0xc4, 0x90, 0x2e, 0xbd, 0x58, 0xf6, 0x0a, 0x47, 0x81,
];

#[derive(Deserialize, Serialize)]
struct EncryptedCredential {
    version: u8,
    nonce: String,
    ciphertext: String,
}

pub(super) fn save(app: &AppHandle, provider: FrpProvider, credential: &str) -> Result<(), String> {
    let saved = encrypt(credential)?;
    write(&path(app, provider)?, &saved)
}

pub(super) fn load(app: &AppHandle, provider: FrpProvider) -> Option<String> {
    let credential_path = match path(app, provider) {
        Ok(path) => path,
        Err(error) => {
            log::warn!(
                "could not resolve saved {} credential: {error}",
                provider.display_name()
            );
            return None;
        }
    };
    match read(&credential_path) {
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

pub(super) fn remove(app: &AppHandle, provider: FrpProvider) -> Result<(), String> {
    match fs::remove_file(path(app, provider)?) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn path(app: &AppHandle, provider: FrpProvider) -> Result<PathBuf, String> {
    Ok(client::directory(app, provider)?.join(CREDENTIAL_FILE))
}

fn encrypt(credential: &str) -> Result<EncryptedCredential, String> {
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

fn decrypt(saved: EncryptedCredential) -> Result<Option<String>, String> {
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

fn write(path: &Path, saved: &EncryptedCredential) -> Result<(), String> {
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

fn read(path: &Path) -> Result<Option<String>, String> {
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
    decrypt(saved)
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt, read, write};

    #[test]
    fn saves_encrypted_credential() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credential.dat");
        let credential = "sakura-secret-token";
        let saved = encrypt(credential).unwrap();

        write(&path, &saved).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(!content.contains(credential));
        assert_eq!(read(&path).unwrap(), Some(credential.to_owned()));
        assert_eq!(decrypt(saved).unwrap(), Some(credential.to_owned()));
    }

    #[test]
    fn ignores_missing_credential() {
        let directory = tempfile::tempdir().unwrap();
        assert_eq!(
            read(&directory.path().join("credential.dat")).unwrap(),
            None
        );
    }
}
