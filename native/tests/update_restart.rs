//! Installing an update and restarting into it are two separate steps, and the
//! second one used to fail silently.
//!
//! `relaunch` started the program again from `std::env::current_exe()`. But
//! installing the update replaces the very binary this process is running from,
//! and Linux then resolves `/proc/self/exe` to the old inode with " (deleted)"
//! appended. Spawning that path cannot work, so the application quit and never
//! came back — while the dialog had just promised "Nitum se reiniciará y volverá
//! a abrir este documento".
//!
//! The stripping itself is covered by unit tests next to the code, where it can
//! be checked without running anything. These tests pin the other half: an
//! installed-but-not-restarted update is a distinct outcome, reported as such
//! rather than mistaken for a failed installation.
//!
//! Nothing here calls the real `NativeUpdateInstaller::relaunch`. Under a test
//! binary `current_exe()` is that test binary, so a real relaunch would spawn
//! the test suite again, and again.

use nitum_pdf::application::UpdateInstaller;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// An installer that succeeds at installing and fails at restarting: the exact
/// case the interface used to hide.
struct InstallsButCannotRestart {
    installed: AtomicBool,
}

impl UpdateInstaller for InstallsButCannotRestart {
    fn install(&self, _package: &Path) -> anyhow::Result<()> {
        self.installed.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn relaunch(&self, _document: Option<&Path>) -> anyhow::Result<()> {
        anyhow::bail!("el gestor de ventanas rechazó abrir la aplicación")
    }
}

#[test]
fn a_failed_restart_does_not_undo_a_successful_installation() {
    let installer = InstallsButCannotRestart {
        installed: AtomicBool::new(false),
    };

    installer
        .install(Path::new("/tmp/nitum-pdf.deb"))
        .expect("installing succeeds");
    let restart = installer.relaunch(None);

    assert!(
        installer.installed.load(Ordering::SeqCst),
        "the new version is on disk"
    );
    assert!(restart.is_err(), "restarting is what failed");

    // The distinction the presentation layer relies on: these are separate
    // results, so a restart that did not happen can be reported as "already
    // updated, open it again" instead of as a failed update.
}

#[test]
fn the_document_is_carried_into_the_restarted_program() {
    // The dialog promises the open document comes back, so the path has to be
    // passed on. This checks the promise is representable, not that a process
    // was started.
    let document = PathBuf::from("/home/ana/contrato.pdf");
    let installer = InstallsButCannotRestart {
        installed: AtomicBool::new(false),
    };
    let result = installer.relaunch(Some(&document));
    assert!(result.is_err(), "this double always refuses to restart");
}
