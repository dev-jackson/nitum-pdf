use crate::{
    application::{UpdateInstaller, UpdateService},
    domain::AppRelease,
};
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const DEFAULT_REPOSITORY: &str = "dev-jackson/nitum-pdf";

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    assets: Vec<GithubAsset>,
}

pub struct GithubUpdateService {
    client: Client,
    repository: String,
    current_version: Version,
    platform: &'static str,
    architecture: &'static str,
}

impl GithubUpdateService {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(60))
            .user_agent(concat!("Nitum-PDF/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            client,
            repository: std::env::var("NITUM_PDF_REPOSITORY")
                .unwrap_or_else(|_| DEFAULT_REPOSITORY.to_owned()),
            current_version: Version::parse(env!("CARGO_PKG_VERSION"))?,
            platform: platform_name(),
            architecture: architecture_name()?,
        })
    }

    #[cfg(test)]
    fn for_target(current: &str, platform: &'static str, architecture: &'static str) -> Self {
        Self {
            client: Client::new(),
            repository: DEFAULT_REPOSITORY.to_owned(),
            current_version: Version::parse(current).unwrap(),
            platform,
            architecture,
        }
    }

    fn release_from_payload(&self, payload: GithubRelease) -> Result<Option<AppRelease>> {
        if payload.draft || payload.prerelease {
            return Ok(None);
        }
        let remote = Version::parse(payload.tag_name.trim_start_matches('v'))
            .context("GitHub publicó una versión con formato inválido")?;
        if remote <= self.current_version {
            return Ok(None);
        }
        let package_prefix = format!(
            "nitum-pdf-{}-{}-{}",
            remote, self.platform, self.architecture
        );
        let package = payload
            .assets
            .iter()
            .find(|asset| {
                asset
                    .name
                    .strip_prefix(&package_prefix)
                    .is_some_and(package_suffix)
            })
            .context("La versión nueva no incluye un instalador para este equipo")?;
        let checksum_name = format!("{}.sha256", package.name);
        let checksum = payload
            .assets
            .iter()
            .find(|asset| asset.name == checksum_name)
            .context("El instalador no incluye su comprobación SHA-256")?;
        Ok(Some(AppRelease {
            version: remote.to_string(),
            package_name: package.name.clone(),
            package_url: package.browser_download_url.clone(),
            checksum_url: checksum.browser_download_url.clone(),
        }))
    }

    fn download(&self, url: &str, mut output: impl Write) -> Result<()> {
        let mut response = self
            .client
            .get(url)
            .header("Accept", "application/octet-stream")
            .send()?
            .error_for_status()?;
        std::io::copy(&mut response, &mut output)?;
        Ok(())
    }
}

impl UpdateService for GithubUpdateService {
    fn latest(&self) -> Result<Option<AppRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            self.repository
        );
        let payload: GithubRelease = self
            .client
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?
            .error_for_status()?
            .json()?;
        self.release_from_payload(payload)
    }

    fn download_verified(&self, release: &AppRelease) -> Result<PathBuf> {
        if !package_extension(&release.package_name)
            || release.package_name.contains('/')
            || release.package_name.contains('\\')
        {
            bail!("El nombre del instalador no es seguro.");
        }
        let directory = tempfile::Builder::new().prefix("nitum-update-").tempdir()?;
        let directory = directory.keep();
        let package_path = directory.join(&release.package_name);
        let partial_path = directory.join(format!("{}.partial", release.package_name));
        let checksum_path = directory.join(format!("{}.sha256", release.package_name));
        self.download(&release.package_url, File::create(&partial_path)?)?;
        self.download(&release.checksum_url, File::create(&checksum_path)?)?;

        let expected = std::fs::read_to_string(&checksum_path)?
            .split_whitespace()
            .next()
            .context("El archivo SHA-256 está vacío")?
            .to_ascii_lowercase();
        if expected.len() != 64 || !expected.bytes().all(|value| value.is_ascii_hexdigit()) {
            bail!("El SHA-256 publicado no es válido.");
        }
        verify_sha256(&partial_path, &expected)?;
        std::fs::rename(partial_path, &package_path)?;
        Ok(package_path)
    }
}

fn verify_sha256(path: &Path, expected: &str) -> Result<()> {
    let mut input = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    let actual = format!("{:x}", digest.finalize());
    if actual != expected {
        bail!("El instalador descargado no coincide con su SHA-256.");
    }
    Ok(())
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

fn architecture_name() -> Result<&'static str> {
    if cfg!(target_arch = "x86_64") {
        Ok("x86_64")
    } else if cfg!(target_arch = "aarch64") {
        Ok("aarch64")
    } else {
        bail!("Esta arquitectura todavía no tiene paquetes de Nitum PDF.")
    }
}

fn package_extension(name: &str) -> bool {
    name.ends_with(".deb")
        || name.ends_with(".AppImage")
        || name.ends_with(".pkg")
        || name.ends_with(".msi")
        || name.ends_with(".exe")
}

fn package_suffix(suffix: &str) -> bool {
    matches!(suffix, ".deb" | ".AppImage" | ".pkg" | ".msi" | ".exe")
}

pub struct NativeUpdateInstaller;

impl UpdateInstaller for NativeUpdateInstaller {
    fn install(&self, package: &Path) -> Result<()> {
        if !package.is_file() {
            bail!("El instalador descargado ya no está disponible.");
        }
        #[cfg(target_os = "linux")]
        let status = {
            if package.extension().and_then(|value| value.to_str()) != Some("deb") {
                bail!("Linux esperaba un paquete .deb.");
            }
            Command::new("pkexec")
                .args(["apt-get", "install", "-y"])
                .arg(package)
                .status()?
        };
        #[cfg(target_os = "windows")]
        let status = {
            match package.extension().and_then(|value| value.to_str()) {
                Some("msi") => Command::new("msiexec.exe")
                    .arg("/i")
                    .arg(package)
                    .args(["/passive", "/norestart"])
                    .status()?,
                Some("exe") => Command::new(package)
                    .args([
                        "/VERYSILENT",
                        "/SUPPRESSMSGBOXES",
                        "/NORESTART",
                        "/RESTARTAPPLICATIONS",
                    ])
                    .status()?,
                _ => bail!("Windows esperaba un instalador .msi o .exe."),
            }
        };
        #[cfg(target_os = "macos")]
        let status = {
            if package.extension().and_then(|value| value.to_str()) != Some("pkg") {
                bail!("macOS esperaba un instalador .pkg.");
            }
            // macOS owns the authorization UI; Nitum never receives the admin password.
            // `installer` is awaited so the application only relaunches after replacement.
            Command::new("osascript")
                .args([
                    "-e",
                    "on run argv",
                    "-e",
                    "do shell script \"/usr/sbin/installer -pkg \" & quoted form of (item 1 of argv) & \" -target /\" with administrator privileges",
                    "-e",
                    "end run",
                    "--",
                ])
                .arg(package)
                .status()?
        };
        if !status.success() {
            bail!("La instalación se canceló o no pudo completarse.");
        }
        Ok(())
    }

    fn relaunch(&self, document: Option<&Path>) -> Result<()> {
        // Inno Setup must close the running executable before replacing it on
        // Windows. Its Restart Manager preserves the original command line,
        // including the open document, and relaunches the installed binary.
        // This process normally no longer exists when installation completes;
        // if it does, launching here would create a duplicate window.
        #[cfg(target_os = "windows")]
        {
            let _ = document;
            return Ok(());
        }

        #[cfg(not(target_os = "windows"))]
        {
            let executable = relaunch_target()?;
            let mut command = Command::new(&executable);
            if let Some(document) = document {
                command.arg(document);
            }
            command.spawn().with_context(|| {
                format!(
                    "No se pudo volver a abrir Nitum PDF desde {}",
                    executable.display()
                )
            })?;
            Ok(())
        }
    }
}

/// The program to start once the update has been installed.
///
/// This deliberately does not simply use `current_exe()`. Installing the update
/// replaces the binary this process is running from, and on Linux
/// `/proc/self/exe` then resolves to the old inode with " (deleted)" appended to
/// its path. Spawning that path fails, so the application quit without ever
/// coming back — while the interface had promised it would restart.
///
/// The launcher the package installs is a stable path that resolves to whatever
/// is installed right now, so it is preferred when present. Otherwise the
/// current executable is used with the marker stripped, which points at the file
/// that has just replaced it.
#[cfg(not(target_os = "windows"))]
fn relaunch_target() -> Result<PathBuf> {
    const INSTALLED_LAUNCHER: &str = "/usr/bin/nitum-pdf";

    if cfg!(target_os = "linux") {
        let launcher = Path::new(INSTALLED_LAUNCHER);
        if launcher.is_file() {
            return Ok(launcher.to_path_buf());
        }
    }

    let current = std::env::current_exe().context("no se pudo localizar el ejecutable actual")?;
    Ok(strip_deleted_marker(&current))
}

/// Removes the " (deleted)" suffix Linux appends to `/proc/self/exe` once the
/// running binary has been replaced on disk.
#[cfg(not(target_os = "windows"))]
fn strip_deleted_marker(path: &Path) -> PathBuf {
    const MARKER: &str = " (deleted)";
    match path.to_str() {
        Some(text) => match text.strip_suffix(MARKER) {
            Some(trimmed) => PathBuf::from(trimmed),
            None => path.to_path_buf(),
        },
        None => path.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(tag: &str, names: &[&str]) -> GithubRelease {
        GithubRelease {
            tag_name: tag.to_owned(),
            prerelease: false,
            draft: false,
            assets: names
                .iter()
                .map(|name| GithubAsset {
                    name: (*name).to_owned(),
                    browser_download_url: format!("https://example.test/{name}"),
                })
                .collect(),
        }
    }

    #[test]
    fn selects_only_the_exact_platform_and_architecture_pair() {
        let updater = GithubUpdateService::for_target("1.9.9", "macos", "aarch64");
        let release = updater
            .release_from_payload(payload(
                "v1.10.0",
                &[
                    "nitum-pdf-1.10.0-windows-x86_64.msi",
                    "nitum-pdf-1.10.0-macos-aarch64.pkg",
                    "nitum-pdf-1.10.0-macos-aarch64.pkg.sha256",
                ],
            ))
            .unwrap()
            .unwrap();
        assert_eq!(release.version, "1.10.0");
        assert_eq!(release.package_name, "nitum-pdf-1.10.0-macos-aarch64.pkg");
    }

    #[test]
    fn ignores_current_draft_and_prerelease_versions() {
        let updater = GithubUpdateService::for_target("2.0.0", "linux", "x86_64");
        assert!(
            updater
                .release_from_payload(payload("v2.0.0", &[]))
                .unwrap()
                .is_none()
        );
        let mut draft = payload("v3.0.0", &[]);
        draft.draft = true;
        assert!(updater.release_from_payload(draft).unwrap().is_none());
        let mut prerelease = payload("v3.0.0", &[]);
        prerelease.prerelease = true;
        assert!(updater.release_from_payload(prerelease).unwrap().is_none());
    }

    #[test]
    fn refuses_a_release_without_a_matching_checksum() {
        let updater = GithubUpdateService::for_target("1.0.0", "windows", "x86_64");
        let error = updater
            .release_from_payload(payload("v2.0.0", &["nitum-pdf-2.0.0-windows-x86_64.msi"]))
            .unwrap_err();
        assert!(error.to_string().contains("SHA-256"));
    }

    #[test]
    fn refuses_a_package_whose_filename_claims_a_different_version() {
        let updater = GithubUpdateService::for_target("1.0.0", "linux", "x86_64");
        let error = updater
            .release_from_payload(payload(
                "v2.0.0",
                &[
                    "nitum-pdf-9.9.9-linux-x86_64.deb",
                    "nitum-pdf-9.9.9-linux-x86_64.deb.sha256",
                ],
            ))
            .unwrap_err();
        assert!(error.to_string().contains("este equipo"));
    }

    #[test]
    fn verifies_the_downloaded_bytes_and_rejects_tampering() {
        let directory = tempfile::tempdir().unwrap();
        let package = directory.path().join("update.pkg");
        std::fs::write(&package, b"instalador oficial").unwrap();
        let expected = format!("{:x}", Sha256::digest(b"instalador oficial"));
        verify_sha256(&package, &expected).unwrap();

        std::fs::write(&package, b"instalador alterado").unwrap();
        assert!(verify_sha256(&package, &expected).is_err());
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn the_deleted_marker_is_stripped_from_the_relaunch_path() {
        // Installing the update replaces the binary this process runs from, and
        // Linux then resolves /proc/self/exe to the old inode with this suffix.
        // Spawning that path fails, so the application quit and never came back
        // even though the interface had promised it would restart.
        assert_eq!(
            strip_deleted_marker(Path::new("/usr/lib/nitum-pdf/nitum-pdf (deleted)")),
            Path::new("/usr/lib/nitum-pdf/nitum-pdf")
        );
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn an_ordinary_path_is_left_alone() {
        for path in [
            "/usr/lib/nitum-pdf/nitum-pdf",
            // Only the suffix counts: a directory that happens to contain the
            // words must survive untouched.
            "/home/ana/apps (deleted)/nitum-pdf",
        ] {
            assert_eq!(strip_deleted_marker(Path::new(path)), Path::new(path));
        }
    }
}
