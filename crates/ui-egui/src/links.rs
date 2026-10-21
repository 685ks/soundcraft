//! Where SoundCraft points people: this fork's repository and issues, and the launcher notice.
//!
//! SoundCraft here is Arthavan Projects' fork of storytold/soundcraft. It is not affiliated with
//! or endorsed by ArtCraft, so no ArtCraft names, logos or community links appear in the app.

/// This fork's source repository.
pub const GITHUB: &str = "https://github.com/685ks/soundcraft";
/// Where to report a problem.
pub const ISSUES: &str = "https://github.com/685ks/soundcraft/issues";
/// The project this fork comes from.
pub const UPSTREAM: &str = "https://github.com/storytold/soundcraft";
/// The fork note shown in About.
pub const FORK_NOTE: &str = "A fork of storytold/soundcraft, not affiliated with or endorsed by ArtCraft.";
/// Shown in About when the Arthavan launcher started the app.
pub const LAUNCHER_UPDATES_NOTE: &str = "Updates are managed by Arthavan Projects";

/// Whether the Arthavan launcher started this app (`ARTHAVAN_LAUNCHER=1`). The launcher then
/// installs updates, so the app must not check for or offer any of its own.
pub fn launcher_managed() -> bool {
    launcher_managed_from(std::env::var("ARTHAVAN_LAUNCHER").ok().as_deref())
}

/// [`launcher_managed`] for a given value of `ARTHAVAN_LAUNCHER`.
pub fn launcher_managed_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim() == "1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_point_at_the_fork_and_never_at_artcraft_services() {
        for url in [GITHUB, ISSUES] {
            assert!(url.starts_with("https://github.com/685ks/soundcraft"), "{url}");
        }
        for text in [GITHUB, ISSUES, UPSTREAM, FORK_NOTE, LAUNCHER_UPDATES_NOTE] {
            assert!(!text.contains("getartcraft") && !text.contains("discord"), "{text}");
        }
        assert!(FORK_NOTE.contains("not affiliated with or endorsed by ArtCraft"));
    }

    #[test]
    fn the_launcher_flag_is_read_from_arthavan_launcher() {
        assert!(launcher_managed_from(Some("1")));
        assert!(launcher_managed_from(Some(" 1\n")));
        for v in [None, Some(""), Some("0"), Some("yes"), Some("true")] {
            assert!(!launcher_managed_from(v), "{v:?}");
        }
    }
}
