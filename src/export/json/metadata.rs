use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct Metadata {
    hyperfine_version: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    pub platform: Platform,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            hyperfine_version: env!("CARGO_PKG_VERSION"),
            start_time: utc_now(),
            platform: Platform::detect(),
        }
    }
}

fn utc_now() -> Option<String> {
    use std::{fmt::Write, time::SystemTime};

    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    let mut timestamp = String::new();
    write!(&mut timestamp, "{}", humantime::format_rfc3339_seconds(now)).ok()?;
    Some(timestamp)
}

#[derive(Debug, Default, Serialize)]
pub(super) struct Platform {
    #[serde(skip_serializing_if = "Option::is_none")]
    os: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    architecture: Option<String>,
}

impl Platform {
    fn detect() -> Self {
        use platform_info::{PlatformInfo, PlatformInfoAPI, UNameAPI};

        let Ok(info) = PlatformInfo::new() else {
            return Self::default();
        };
        Self {
            os: Some(info.sysname().to_string_lossy().into_owned()),
            architecture: Some(info.machine().to_string_lossy().into_owned()),
        }
    }
}
