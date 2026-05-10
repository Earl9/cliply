use crate::error::CliplyError;
use crate::logger;
use crate::services::settings_service;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::process::Command;
use std::time::Duration;
use tauri::AppHandle;

const GITHUB_OWNER: &str = "earl";
const GITHUB_REPO: &str = "cliply";
const GITHUB_LATEST_RELEASE_URL: &str = "https://api.github.com/repos/earl/cliply/releases/latest";
const GITHUB_RELEASES_URL: &str = "https://api.github.com/repos/earl/cliply/releases?per_page=20";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub release_name: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub release_url: Option<String>,
    pub installer_asset_name: Option<String>,
    pub installer_download_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    html_url: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Eq, PartialEq)]
struct SemverVersion {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Vec<SemverIdentifier>,
}

#[derive(Debug, Eq, PartialEq)]
enum SemverIdentifier {
    Numeric(u64),
    Text(String),
}

pub fn check_for_updates(app: &AppHandle) -> Result<UpdateCheckResult, CliplyError> {
    let current_version = current_version();
    let channel = update_channel(app);
    logger::info(
        app,
        "update_check_started",
        format!(
            "current_version={} channel={} owner={} repo={}",
            current_version, channel, GITHUB_OWNER, GITHUB_REPO
        ),
    );

    let release = fetch_release(&channel).map_err(|error| {
        logger::error(app, "update_check_failed", &error);
        error
    })?;
    let latest_version = normalize_version(&release.tag_name);
    let has_update = is_newer_version(&current_version, &latest_version);
    let installer_asset = find_installer_asset(&release.assets);
    let result = UpdateCheckResult {
        current_version,
        latest_version: Some(latest_version.clone()),
        has_update,
        release_name: release.name,
        release_notes: release.body,
        published_at: release.published_at,
        release_url: Some(release.html_url),
        installer_asset_name: installer_asset.map(|asset| asset.name.clone()),
        installer_download_url: installer_asset.map(|asset| asset.browser_download_url.clone()),
    };

    logger::info(
        app,
        "update_check_success",
        format!(
            "channel={} latest_version={} has_update={} installer_asset={}",
            channel,
            latest_version,
            result.has_update,
            result.installer_asset_name.as_deref().unwrap_or("none")
        ),
    );
    if result.has_update {
        logger::info(
            app,
            "update_available",
            format!("latest_version={latest_version}"),
        );
    }

    Ok(result)
}

pub fn open_release_page(url: String) -> Result<(), CliplyError> {
    let parsed = url::Url::parse(&url)
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;
    if parsed.scheme() != "https" {
        return Err(CliplyError::PlatformUnavailable(
            "Only HTTPS release URLs can be opened".to_string(),
        ));
    }
    if parsed.host_str() != Some("github.com") {
        return Err(CliplyError::PlatformUnavailable(
            "Only GitHub release URLs can be opened".to_string(),
        ));
    }

    open_url(parsed.as_str()).map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))
}

fn update_channel(app: &AppHandle) -> String {
    settings_service::get_settings(app)
        .map(|settings| settings.update.channel)
        .ok()
        .filter(|channel| channel == "stable" || channel == "beta")
        .unwrap_or_else(|| "beta".to_string())
}

fn fetch_release(channel: &str) -> Result<GithubRelease, CliplyError> {
    if channel == "stable" {
        return fetch_latest_release();
    }

    let releases = fetch_releases()?;
    select_latest_release(releases).ok_or_else(|| {
        CliplyError::PlatformUnavailable("No GitHub releases were found".to_string())
    })
}

fn fetch_latest_release() -> Result<GithubRelease, CliplyError> {
    let body = fetch_url(GITHUB_LATEST_RELEASE_URL)?;
    serde_json::from_str::<GithubRelease>(&body)
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))
}

fn fetch_releases() -> Result<Vec<GithubRelease>, CliplyError> {
    let body = fetch_url(GITHUB_RELEASES_URL)?;
    serde_json::from_str::<Vec<GithubRelease>>(&body)
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))
}

fn fetch_url(url: &str) -> Result<String, CliplyError> {
    let response = ureq::AgentBuilder::new()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "Cliply update checker")
        .call()
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    let body = response
        .into_string()
        .map_err(|error| CliplyError::PlatformUnavailable(error.to_string()))?;

    Ok(body)
}

fn current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn normalize_version(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
}

fn is_newer_version(current: &str, latest: &str) -> bool {
    compare_versions(latest, current).is_gt()
}

fn select_latest_release(releases: Vec<GithubRelease>) -> Option<GithubRelease> {
    releases
        .into_iter()
        .max_by(|left, right| compare_versions(&left.tag_name, &right.tag_name))
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    match (parse_semver(left), parse_semver(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => normalize_version(left).cmp(&normalize_version(right)),
    }
}

fn parse_semver(value: &str) -> Option<SemverVersion> {
    let normalized = normalize_version(value);
    let without_build = normalized.split('+').next()?;
    let (core, pre_release) = without_build
        .split_once('-')
        .map_or((without_build, ""), |(core, pre_release)| {
            (core, pre_release)
        });
    let mut core_parts = core.split('.');
    let major = core_parts.next()?.parse().ok()?;
    let minor = core_parts.next().unwrap_or("0").parse().ok()?;
    let patch = core_parts.next().unwrap_or("0").parse().ok()?;
    Some(SemverVersion {
        major,
        minor,
        patch,
        pre_release: parse_pre_release(pre_release),
    })
}

fn parse_pre_release(value: &str) -> Vec<SemverIdentifier> {
    value
        .split('.')
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse::<u64>()
                .map(SemverIdentifier::Numeric)
                .unwrap_or_else(|_| SemverIdentifier::Text(part.to_ascii_lowercase()))
        })
        .collect()
}

fn find_installer_asset(assets: &[GithubReleaseAsset]) -> Option<&GithubReleaseAsset> {
    assets
        .iter()
        .filter_map(|asset| {
            let score = installer_asset_score(&asset.name);
            (score > 0).then_some((score, asset))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, asset)| asset)
}

fn installer_asset_score(name: &str) -> i32 {
    let lower = name.to_ascii_lowercase();
    if !(lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".msix")) {
        return 0;
    }

    let has_installer_marker = lower.contains("setup") || lower.contains("installer");
    if !has_installer_marker && !lower.ends_with(".msi") && !lower.ends_with(".msix") {
        return 0;
    }

    let mut score = 0;
    if lower.contains("cliply") {
        score += 20;
    }
    if lower.contains("setup") {
        score += 40;
    }
    if lower.contains("installer") {
        score += 30;
    }
    if lower.ends_with(".exe") {
        score += 20;
    }
    if lower.ends_with(".msi") {
        score += 10;
    }
    if lower.ends_with(".msix") {
        score += 8;
    }
    if lower.contains("x64") || lower.contains("amd64") {
        score += 5;
    }
    score
}

fn open_url(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

impl Ord for SemverVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
            .then_with(|| compare_pre_release(&self.pre_release, &other.pre_release))
    }
}

impl PartialOrd for SemverVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn compare_pre_release(left: &[SemverIdentifier], right: &[SemverIdentifier]) -> Ordering {
    match (left.is_empty(), right.is_empty()) {
        (true, true) => return Ordering::Equal,
        (true, false) => return Ordering::Greater,
        (false, true) => return Ordering::Less,
        (false, false) => {}
    }

    for (left, right) in left.iter().zip(right.iter()) {
        let ordering = match (left, right) {
            (SemverIdentifier::Numeric(left), SemverIdentifier::Numeric(right)) => left.cmp(right),
            (SemverIdentifier::Numeric(_), SemverIdentifier::Text(_)) => Ordering::Less,
            (SemverIdentifier::Text(_), SemverIdentifier::Numeric(_)) => Ordering::Greater,
            (SemverIdentifier::Text(left), SemverIdentifier::Text(right)) => left.cmp(right),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }

    left.len().cmp(&right.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_beta_versions() {
        assert!(is_newer_version("0.4.0-beta.1", "v0.4.1"));
        assert!(is_newer_version("0.4.0-beta.1", "0.4.0"));
        assert!(is_newer_version("0.4.0-beta.1", "0.4.0-beta.2"));
        assert!(!is_newer_version("0.4.1", "0.4.0-beta.2"));
    }

    #[test]
    fn selects_installer_asset() {
        let assets = vec![
            GithubReleaseAsset {
                name: "Cliply-portable.zip".to_string(),
                browser_download_url: "https://github.com/example/zip".to_string(),
            },
            GithubReleaseAsset {
                name: "cliply.exe".to_string(),
                browser_download_url: "https://github.com/example/plain-exe".to_string(),
            },
            GithubReleaseAsset {
                name: "Cliply_0.4.1_x64-setup.exe".to_string(),
                browser_download_url: "https://github.com/example/exe".to_string(),
            },
        ];

        let asset = find_installer_asset(&assets).expect("installer asset");
        assert_eq!(asset.name, "Cliply_0.4.1_x64-setup.exe");
    }

    #[test]
    fn beta_channel_can_select_prerelease() {
        let releases = vec![
            test_release("v0.4.0"),
            test_release("v0.4.1-beta.2"),
            test_release("v0.4.1-beta.1"),
        ];

        let release = select_latest_release(releases).expect("latest release");
        assert_eq!(release.tag_name, "v0.4.1-beta.2");
    }

    fn test_release(tag_name: &str) -> GithubRelease {
        GithubRelease {
            tag_name: tag_name.to_string(),
            name: None,
            body: None,
            published_at: None,
            html_url: format!("https://github.com/earl/cliply/releases/tag/{tag_name}"),
            assets: Vec::new(),
        }
    }
}
