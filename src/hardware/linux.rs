//! Hardware detection on Linux, based on procfs.

use super::{Cpu, Disk, Memory};

use std::collections::HashSet;

pub(super) fn cpu() -> Option<Cpu> {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    Some(Cpu {
        model: parse_cpu_model(&cpuinfo).or_else(device_tree_model),
        physical_cores: parse_physical_cores(&cpuinfo),
        logical_cores: super::logical_cores(),
    })
}

/// Machine model from the device tree, used by ARM boards that do not name the
/// CPU in `/proc/cpuinfo`. Virtual machines usually have no device tree.
fn device_tree_model() -> Option<String> {
    [
        "/proc/device-tree/model",
        "/sys/firmware/devicetree/base/model",
    ]
    .iter()
    .find_map(|path| std::fs::read(path).ok())
    .and_then(|model| parse_device_tree_model(&model))
}

pub(super) fn parse_device_tree_model(model: &[u8]) -> Option<String> {
    // Device tree strings are NUL-terminated.
    let end = model
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(model.len());
    let model = String::from_utf8_lossy(&model[..end]);
    let model = model.trim();

    (!model.is_empty()).then(|| model.to_owned())
}

pub(super) fn memory() -> Option<Memory> {
    let meminfo = std::fs::read_to_string("/proc/meminfo").ok()?;

    Some(Memory {
        total_bytes: parse_memory_total(&meminfo)?,
    })
}

pub(super) fn disk() -> Option<Disk> {
    super::disk_of_current_dir()
}

/// Value of a `key: value` line in a procfs file.
fn proc_value<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name.trim() == key).then(|| value.trim())
    })
}

pub(super) fn parse_cpu_model(cpuinfo: &str) -> Option<String> {
    /// Keys that carry the CPU model name, in order of preference.
    ///
    /// x86 uses `model name`, while ARM systems use `Model` or `Hardware`. The
    /// lowercase x86 `model` key holds a number and is deliberately absent.
    const MODEL_KEYS: [&str; 3] = ["model name", "Model", "Hardware"];

    MODEL_KEYS
        .iter()
        .find_map(|key| proc_value(cpuinfo, key))
        .filter(|model| !model.is_empty())
        .map(str::to_owned)
}

pub(super) fn parse_physical_cores(cpuinfo: &str) -> Option<usize> {
    let mut cores = HashSet::new();
    let mut processors = 0;

    // Processor entries are separated by a blank line.
    for block in cpuinfo.split("\n\n") {
        let mut is_processor = false;
        let mut physical_id = None;
        let mut core_id = None;

        for line in block.lines() {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };

            match key.trim() {
                "processor" => is_processor = value.trim().parse::<u32>().is_ok(),
                "physical id" => physical_id = value.trim().parse::<u32>().ok(),
                "core id" => core_id = value.trim().parse::<u32>().ok(),
                _ => {}
            }
        }

        if !is_processor {
            continue;
        }

        // Without both ids, the file does not describe the core topology.
        // Hyperthreads of the same core map to the same pair and count once.
        cores.insert((physical_id?, core_id?));
        processors += 1;
    }

    (processors > 0).then_some(cores.len())
}

pub(super) fn parse_memory_total(meminfo: &str) -> Option<u64> {
    let value = proc_value(meminfo, "MemTotal")?;
    let (number, unit) = value
        .split_once(char::is_whitespace)
        .map_or((value, ""), |(number, unit)| (number, unit.trim()));

    let number = number.parse::<u64>().ok()?;
    match unit {
        "" => Some(number),
        "kB" => number.checked_mul(1024),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_cpu_model, parse_device_tree_model, parse_memory_total, parse_physical_cores,
    };

    const X86_CPUINFO: &str = "\
processor\t: 0
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 158
model name\t: Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz
physical id\t: 0
siblings\t: 12
core id\t\t: 0
cpu cores\t: 6

processor\t: 1
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 158
model name\t: Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz
physical id\t: 0
siblings\t: 12
core id\t\t: 0
cpu cores\t: 6

processor\t: 2
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 158
model name\t: Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz
physical id\t: 0
siblings\t: 12
core id\t\t: 1
cpu cores\t: 6

processor\t: 12
vendor_id\t: GenuineIntel
cpu family\t: 6
model\t\t: 158
model name\t: Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz
physical id\t: 1
siblings\t: 12
core id\t\t: 0
cpu cores\t: 6
";

    #[test]
    fn cpu_model_is_read_from_cpuinfo() {
        assert_eq!(
            parse_cpu_model(X86_CPUINFO).as_deref(),
            Some("Intel(R) Core(TM) i7-8700K CPU @ 3.70GHz")
        );
    }

    #[test]
    fn cpu_model_falls_back_to_arm_keys() {
        let arm = "processor\t: 0\nBogoMIPS\t: 108.00\nModel\t\t: Raspberry Pi 4 Model B Rev 1.4\n";
        assert_eq!(
            parse_cpu_model(arm).as_deref(),
            Some("Raspberry Pi 4 Model B Rev 1.4")
        );

        let arm64 = "processor\t: 0\nBogoMIPS\t: 50.00\nmodel name\t: Neoverse-N1\n";
        assert_eq!(parse_cpu_model(arm64).as_deref(), Some("Neoverse-N1"));
    }

    #[test]
    fn cpu_model_is_none_without_a_known_key() {
        assert_eq!(parse_cpu_model("processor\t: 0\n"), None);
        // The numeric x86 `model` key must not be mistaken for a model name.
        assert_eq!(parse_cpu_model("model\t\t: 158\n"), None);
    }

    #[test]
    fn physical_cores_are_counted_from_socket_and_core_ids() {
        // Hyperthreads (processors 0 and 1) share one core, so three distinct
        // cores remain: (0, 0), (0, 1) and (1, 0).
        assert_eq!(parse_physical_cores(X86_CPUINFO), Some(3));
    }

    #[test]
    fn physical_cores_are_none_without_topology_keys() {
        let arm = "processor\t: 0\nBogoMIPS\t: 108.00\nmodel name\t: Neoverse-N1\n";
        assert_eq!(parse_physical_cores(arm), None);
        assert_eq!(parse_physical_cores(""), None);
    }

    #[test]
    fn device_tree_model_is_nul_terminated() {
        assert_eq!(
            parse_device_tree_model(b"Raspberry Pi 4 Model B Rev 1.4\0").as_deref(),
            Some("Raspberry Pi 4 Model B Rev 1.4")
        );
        assert_eq!(parse_device_tree_model(b"\0"), None);
        assert_eq!(parse_device_tree_model(b""), None);
    }

    #[test]
    fn memory_total_is_parsed_from_meminfo() {
        let meminfo = "\
MemTotal:       16384000 kB
MemFree:         1234567 kB
MemAvailable:    9876543 kB
";
        assert_eq!(parse_memory_total(meminfo), Some(16_777_216_000));
        assert_eq!(parse_memory_total("MemFree: 42 kB\n"), None);
        assert_eq!(parse_memory_total("MemTotal: not-a-number kB\n"), None);
    }
}
