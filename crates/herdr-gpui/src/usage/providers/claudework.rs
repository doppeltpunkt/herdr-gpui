//! The independent Work sign-in used by Pi's account-aware Claude bridge.
//! Only the global account file selects its directory. Credentials stay in
//! that directory's Keychain namespace; Herdr never copies or refreshes them.

use super::claude;
use crate::{
    Result,
    usage::{
        model::{Provider, Report},
        probe::{HostPath, Probe, Request},
        service::{Meta, Service},
    },
};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub(crate) struct ClaudeWork;

static META: Meta = Meta::new("claudework", "Claude Work")
    .bar_label("Work")
    .icon("icons/agent-claude.svg")
    .dashboard("https://claude.ai/settings/usage")
    .status_page("https://status.anthropic.com");

impl Service for ClaudeWork {
    fn meta(&self) -> &'static Meta {
        &META
    }

    fn fetch(&self, probe: &mut Probe) -> Option<Result<Report>> {
        let directory = probe.file_text(
            &HostPath::env_or("PI_CODING_AGENT_DIR", ".pi/agent", "claude-accounts.json"),
            &["work", "configDir"],
        )?;
        // HostPath::absolute represents POSIX paths, including SSH hosts.
        if !directory.starts_with('/') {
            return None;
        }
        let file =
            |name: &str| HostPath::absolute(format!("{}/{name}", directory.trim_end_matches('/')));
        let credentials = claude::keychain_for(probe, &keychain_service(&directory))
            .or_else(|| probe.file(&file(".credentials.json")))?;
        let token = probe.field(&credentials, &["claudeAiOauth", "accessToken"])?;
        let sign_in = claude::SignIn {
            plan: probe.text(&credentials, &["claudeAiOauth", "subscriptionType"]),
            tier: probe.text(&credentials, &["claudeAiOauth", "rateLimitTier"]),
            // Never label Work using the Personal config in ~/.claude.json.
            email: probe.file_text(&file(".claude.json"), &["oauthAccount", "emailAddress"]),
        };
        let request = Request::get(claude::URL)
            .bearer(&token)
            .header("anthropic-beta", claude::BETA)
            .header("User-Agent", claude::AGENT);
        Some(probe.body(request).and_then(|body| parse(&body, sign_in)))
    }
}

/// Claude Code hashes the NFC-normalized config path, not its realpath.
pub(crate) fn keychain_service(directory: &str) -> String {
    let normalized: String = directory.nfc().collect();
    let digest = Sha256::digest(normalized.as_bytes());
    format!(
        "{}-{:02x}{:02x}{:02x}{:02x}",
        claude::KEYCHAIN,
        digest[0],
        digest[1],
        digest[2],
        digest[3]
    )
}

pub(crate) fn parse(body: &str, sign_in: claude::SignIn) -> Result<Report> {
    let mut report = claude::parse(body, sign_in)?;
    report.provider = Provider(&ClaudeWork);
    Ok(report)
}
