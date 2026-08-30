//! Being asked to update over and over is what teaches people to dismiss update
//! prompts without reading them — which is exactly the wrong habit for a signing
//! tool, where an update may carry a security fix.
//!
//! So "Ahora no" is an answer that sticks: that version stops being announced,
//! a newer one still is, and the whole background check can be turned off.

use nitum_pdf::application::UpdatePreferences;
use nitum_pdf::infrastructure::NativeUpdatePreferences;

fn preferences() -> (NativeUpdatePreferences, tempfile::TempDir) {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("updates.conf");
    (NativeUpdatePreferences::at(path), directory)
}

#[test]
fn checking_is_on_before_anyone_has_chosen() {
    let (preferences, _directory) = preferences();
    assert!(
        preferences.automatic(),
        "a signing tool that silently falls behind on security fixes is worse \
         than one that mentions an update once per version"
    );
    assert!(!preferences.is_dismissed("0.9.0"));
}

#[test]
fn a_dismissed_version_stays_dismissed_but_a_newer_one_does_not() {
    let (preferences, _directory) = preferences();

    preferences.dismiss("0.9.0").expect("recording the answer");

    assert!(
        preferences.is_dismissed("0.9.0"),
        "the version that was turned down is not announced again"
    );
    assert!(
        !preferences.is_dismissed("0.9.1"),
        "a newer version is still worth mentioning; dismissing one release must \
         not silence every release after it"
    );

    // Turning down a newer one replaces the answer rather than accumulating.
    preferences.dismiss("0.9.1").expect("recording the answer");
    assert!(preferences.is_dismissed("0.9.1"));
    assert!(
        !preferences.is_dismissed("0.9.0"),
        "only the most recent answer matters; older versions are behind the \
         dismissed one anyway"
    );
}

#[test]
fn the_two_settings_do_not_overwrite_each_other() {
    let (preferences, _directory) = preferences();

    preferences.dismiss("1.0.0").expect("recording the answer");
    preferences
        .set_automatic(false)
        .expect("turning checks off");

    // Both live in one file, so writing either has to preserve the other.
    assert!(!preferences.automatic());
    assert!(preferences.is_dismissed("1.0.0"));

    preferences.set_automatic(true).expect("turning checks on");
    assert!(preferences.automatic());
    assert!(
        preferences.is_dismissed("1.0.0"),
        "turning checks back on does not un-answer a version"
    );
}

#[test]
fn preferences_survive_a_restart() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("updates.conf");

    let first = NativeUpdatePreferences::at(path.clone());
    first.set_automatic(false).expect("turning checks off");
    first.dismiss("2.0.0").expect("recording the answer");

    // A fresh instance reads the same file, the way the next launch would.
    let second = NativeUpdatePreferences::at(path);
    assert!(!second.automatic());
    assert!(second.is_dismissed("2.0.0"));
}
