use super::*;
use crate::usage::providers::{claude, claudework};

#[test]
fn claude_work_has_an_independent_registered_identity() {
    let work = provider("claudework");
    assert_ne!(work, provider("claude"));
    assert_eq!(work.name(), "Claude Work");
    assert_eq!(work.icon(), provider("claude").icon());
    assert_eq!(work.service().meta().bar_label, Some("Work"));
    assert_eq!(provider("claude").service().meta().bar_label, None);
    let body = r#"{"five_hour":{"utilization":17},"seven_day":{"utilization":2}}"#;
    let sign_in = claude::SignIn {
        email: Some("team@example.com".into()),
        plan: Some("team".into()),
        tier: None,
    };
    let report = claudework::parse(body, sign_in.clone()).unwrap();
    assert_eq!(report.provider, work);
    assert_eq!(report.account.email.as_deref(), Some("team@example.com"));
    assert_eq!(report.account.plan.as_deref(), Some("Team"));
    assert_eq!(report.windows[0].percent(), 17);
    assert_eq!(report.windows[1].percent(), 2);
    let personal = claude::parse(body, sign_in).unwrap();
    assert_eq!(personal.provider, provider("claude"));
    assert_eq!(report.windows, personal.windows);
}

#[test]
fn claude_work_keychain_matches_claude_code_path_hashing() {
    assert_eq!(
        claudework::keychain_service("/Users/alice/.claude-work"),
        "Claude Code-credentials-be865d75"
    );
    assert_eq!(
        claudework::keychain_service("/Users/alice/caf\u{e9}"),
        claudework::keychain_service("/Users/alice/cafe\u{301}")
    );
    assert_ne!(
        claudework::keychain_service("/Users/alice/.claude-work"),
        claudework::keychain_service("/Users/alice/.claude-work/")
    );
}

#[gpui::test]
fn claude_work_bar_label_is_visible_alongside_personal_and_codex(cx: &mut gpui::TestAppContext) {
    use crate::{
        config::status_bar::Detail,
        sidebar::layout_tests::{fixture_window, full_draw},
    };
    use gpui::{px, size};
    let (view, cx) = cx.add_window_view(|window, cx| {
        let mut view = fixture_window(window, cx);
        view.config.usage.show = true;
        view.live.status = crate::state::ConnectionStatus::Connected;
        view.usage.host = Some(super::super::Host::Local);
        view.usage.choose(provider("claudework"));
        let readings = ["codex", "claude", "claudework"].map(|id| Reading {
            provider: provider(id),
            report: Some(report(provider(id), 17.)),
            error: None,
            access: None,
        });
        view.usage.entries.insert(
            super::super::Host::Local,
            super::super::Entry {
                readings: readings.to_vec(),
                ..Default::default()
            },
        );
        view
    });
    for detail in [Detail::Detailed, Detail::Compact] {
        cx.simulate_resize(size(px(1800.), px(600.)));
        view.update(cx, |view, cx| {
            view.config.status_bar.usage = detail;
            cx.notify();
        });
        cx.update(|window, cx| full_draw(window, cx).clear(cx));
        let row = cx.debug_bounds("usage").unwrap();
        for selector in ["usage-claude", "usage-codex", "usage-claudework-label"] {
            let element = cx.debug_bounds(selector).unwrap();
            assert!(
                element.left() >= row.left() && element.right() <= row.right(),
                "{selector}"
            );
            assert!(
                element.top() >= row.top() && element.bottom() <= row.bottom(),
                "{selector}"
            );
        }
    }
    cx.simulate_resize(size(px(1000.), px(600.)));
    cx.update(|window, cx| full_draw(window, cx).clear(cx));
    let row = cx.debug_bounds("usage").unwrap();
    let work = cx.debug_bounds("usage-claudework").unwrap();
    let label = cx.debug_bounds("usage-claudework-label").unwrap();
    assert_eq!(work.left(), row.left());
    assert!(label.right() <= work.right());
    assert!(work.right() <= row.right() && work.bottom() <= row.bottom());
}

#[cfg(unix)]
mod host {
    use super::*;
    use crate::usage::probe::Shell;
    use std::path::PathBuf;

    const WORK: &str =
        r#"{"claudeAiOauth":{"accessToken":"work-fixture","subscriptionType":"team"}}"#;
    const PERSONAL: &str =
        r#"{"claudeAiOauth":{"accessToken":"personal-fixture","subscriptionType":"pro"}}"#;

    struct Host {
        root: tempfile::TempDir,
    }

    impl Host {
        fn new(macos: bool, work_keychain: bool, work_file: bool) -> Self {
            let root = tempfile::tempdir().unwrap();
            let host = Self { root };
            let home = host.home();
            let bin = home.join(".local/bin");
            let directory = home.join("team config");
            std::fs::create_dir_all(home.join(".pi/agent")).unwrap();
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::create_dir_all(&bin).unwrap();
            host.accounts(directory.to_str().unwrap());
            std::fs::write(
                directory.join(".claude.json"),
                r#"{"oauthAccount":{"emailAddress":"team@example.com"}}"#,
            )
            .unwrap();
            std::fs::write(
                home.join(".claude.json"),
                r#"{"oauthAccount":{"emailAddress":"personal@example.com"}}"#,
            )
            .unwrap();
            if work_file {
                std::fs::write(directory.join(".credentials.json"), WORK).unwrap();
            }
            crate::test_executable::write(
                bin.join("uname"),
                format!(
                    "#!/bin/sh\necho {}\n",
                    if macos { "Darwin" } else { "Linux" }
                ),
                0o755,
            )
            .unwrap();
            crate::test_executable::write(
                bin.join("security"),
                format!(
                    "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$HOME/keychain.log\"\ncase \"$*\" in\n  *'-s {service} -w'*) {work} ;;\n  *'-s Claude Code-credentials -w'*) printf '%s' '{PERSONAL}' ;;\n  *) exit 44 ;;\nesac\n",
                    service = claudework::keychain_service(directory.to_str().unwrap()),
                    work = if work_keychain { format!("printf '%s' '{WORK}'") } else { "exit 44".into() },
                ),
                0o755,
            ).unwrap();
            host.response(200);
            host
        }

        fn home(&self) -> PathBuf {
            self.root.path().join("home")
        }

        fn accounts(&self, directory: &str) {
            std::fs::write(
                self.home().join(".pi/agent/claude-accounts.json"),
                serde_json::json!({"work": {"configDir": directory}}).to_string(),
            )
            .unwrap();
        }

        fn response(&self, status: u16) {
            crate::test_executable::write(
                self.home().join(".local/bin/curl"),
                format!(
                    "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$HOME/http.log\"\ncat <&3 >> \"$HOME/http.log\"\nprintf '%s' '{{\"five_hour\":{{\"utilization\":17}},\"seven_day\":{{\"utilization\":2}}}}'\nprintf '\\n@@herdr-status {status}'\n"
                ),
                0o755,
            ).unwrap();
        }

        fn fetch(&self, id: &str) -> Option<crate::Result<Report>> {
            let mut command = std::process::Command::new("/bin/sh");
            command
                .arg("-s")
                .env_clear()
                .env("HOME", self.home())
                .env("USER", "alice")
                .env("PATH", "/usr/bin:/bin")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null());
            let mut exec = Exec::Remote(Shell::start(command).unwrap());
            let mut jar = CookieJar::default();
            let provider = provider(id);
            let mut probe = Probe::new(&mut exec, provider, None, &mut jar, Consent::Quiet);
            provider.service().fetch(&mut probe)
        }

        fn log(&self, name: &str) -> String {
            std::fs::read_to_string(self.home().join(name)).unwrap_or_default()
        }
    }

    #[test]
    fn claude_work_uses_only_its_own_keychain_and_metadata() {
        let host = Host::new(true, true, false);
        let work = host.fetch("claudework").unwrap().unwrap();
        assert_eq!(work.provider, provider("claudework"));
        assert_eq!(work.account.email.as_deref(), Some("team@example.com"));
        assert_eq!(work.account.plan.as_deref(), Some("Team"));
        assert!(host.log("http.log").contains("Bearer work-fixture"));
        assert!(!host.log("http.log").contains("personal-fixture"));
        assert!(
            !host
                .log("keychain.log")
                .contains("-s Claude Code-credentials -w")
        );
        assert!(host.log("keychain.log").contains("-a alice"));
        let personal = host.fetch("claude").unwrap().unwrap();
        assert_eq!(personal.provider, provider("claude"));
        assert_eq!(personal.account.plan.as_deref(), Some("Pro"));
        assert_eq!(
            personal.account.email.as_deref(),
            Some("personal@example.com")
        );
        assert!(host.log("http.log").contains("Bearer personal-fixture"));
    }

    #[test]
    fn claude_work_without_global_config_does_not_probe_credentials() {
        let host = Host::new(true, true, false);
        std::fs::remove_file(host.home().join(".pi/agent/claude-accounts.json")).unwrap();
        assert!(host.fetch("claudework").is_none());
        assert!(host.log("keychain.log").is_empty());
        assert!(host.log("http.log").is_empty());
    }

    #[test]
    fn claude_work_rejects_relative_config_directory() {
        let host = Host::new(true, true, false);
        host.accounts(".claudework");
        assert!(host.fetch("claudework").is_none());
        assert!(host.log("keychain.log").is_empty());
        assert!(host.log("http.log").is_empty());
    }

    #[test]
    fn claude_work_never_falls_back_to_personal_credentials_or_email() {
        let host = Host::new(true, false, false);
        assert!(host.fetch("claudework").is_none());
        assert!(
            !host
                .log("keychain.log")
                .contains("-s Claude Code-credentials -w")
        );
        assert!(host.log("http.log").is_empty());
        let host = Host::new(true, true, false);
        std::fs::remove_file(host.home().join("team config/.claude.json")).unwrap();
        let work = host.fetch("claudework").unwrap().unwrap();
        assert_eq!(work.account.email, None);
    }

    #[test]
    fn claude_work_uses_config_scoped_file_credentials_without_keychain() {
        for macos in [false, true] {
            let host = Host::new(macos, false, true);
            let work = host.fetch("claudework").unwrap().unwrap();
            assert_eq!(work.account.plan.as_deref(), Some("Team"));
            assert!(host.log("http.log").contains("Bearer work-fixture"));
            if !macos {
                assert!(host.log("keychain.log").is_empty());
            }
        }
    }

    #[test]
    fn claude_work_auth_failure_does_not_switch_refresh_or_retry() {
        let host = Host::new(true, true, false);
        host.response(401);
        assert!(matches!(
            host.fetch("claudework").unwrap(),
            Err(Error::UsageRejected)
        ));
        let http = host.log("http.log");
        assert_eq!(http.matches("Authorization: Bearer").count(), 1);
        assert!(!http.contains("personal-fixture"));
        // GET is curl's default; only a non-GET method is written explicitly.
        assert!(!http.contains("request = \"POST\""));
        assert!(!http.contains("data-raw"));
        assert!(http.contains(claude::URL));
    }
}
