use serde::Serialize;

use crate::hardware::{Cpu, Disk, Hardware, Memory};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    cpu: Option<Cpu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    memory: Option<Memory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk: Option<Disk>,
}

impl Platform {
    fn detect() -> Self {
        let hardware = Hardware::detect();
        let (os, architecture) = uname().unzip();

        Self {
            os,
            architecture,
            cpu: hardware.cpu,
            memory: hardware.memory,
            disk: hardware.disk,
        }
    }
}

fn uname() -> Option<(String, String)> {
    use platform_info::{PlatformInfo, PlatformInfoAPI, UNameAPI};

    let info = PlatformInfo::new().ok()?;
    Some((
        info.sysname().to_string_lossy().into_owned(),
        info.machine().to_string_lossy().into_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{Cpu, Disk, Memory, Platform};

    fn platform() -> Platform {
        Platform {
            os: Some("Linux".into()),
            architecture: Some("x86_64".into()),
            cpu: None,
            memory: None,
            disk: None,
        }
    }

    #[test]
    fn hardware_properties_are_nested_in_the_platform_block() {
        let platform = Platform {
            cpu: Some(Cpu {
                model: Some("Example CPU".into()),
                physical_cores: Some(4),
                logical_cores: Some(8),
            }),
            memory: Some(Memory {
                total_bytes: 16_000_000_000,
            }),
            disk: Some(Disk {
                path: "/example".into(),
                total_bytes: 1000,
                available_bytes: 250,
            }),
            ..platform()
        };

        assert_eq!(
            serde_json::to_value(platform).unwrap(),
            serde_json::json!({
                "os": "Linux",
                "architecture": "x86_64",
                "cpu": {
                    "model": "Example CPU",
                    "physical_cores": 4,
                    "logical_cores": 8
                },
                "memory": {"total_bytes": 16_000_000_000u64},
                "disk": {
                    "path": "/example",
                    "total_bytes": 1000,
                    "available_bytes": 250
                }
            })
        );
    }

    #[test]
    fn unavailable_hardware_properties_are_omitted() {
        assert_eq!(
            serde_json::to_value(platform()).unwrap(),
            serde_json::json!({
                "os": "Linux",
                "architecture": "x86_64"
            })
        );
    }
}
