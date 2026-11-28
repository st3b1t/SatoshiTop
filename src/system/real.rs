//! System metrics of the real machine, read with `sysinfo` (Linux only).

use std::collections::HashSet;

use sysinfo::{
    Components, Disks, Networks, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users,
};

use super::{CpuSample, DiskSample, NetSample, ProcSample, RamSample, SysSource, TempSample};

/// File systems that are not real storage: snap images, memory, containers.
const VIRTUAL_FILESYSTEMS: [&str; 6] = [
    "squashfs", "tmpfs", "devtmpfs", "overlay", "ramfs", "efivarfs",
];

/// Substrings of sensor labels that belong to the CPU, lowercase.
const CPU_SENSORS: [&str; 5] = ["package id", "tctl", "tdie", "coretemp", "k10temp"];

/// Owns the sysinfo handles. CPU percentages need two reads to be meaningful:
/// the first one after creation is always 0.
pub struct RealSys {
    system: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    users: Users,
}

impl RealSys {
    pub fn new() -> Self {
        Self {
            system: System::new(),
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            users: Users::new_with_refreshed_list(),
        }
    }
}

impl Default for RealSys {
    fn default() -> Self {
        Self::new()
    }
}

impl SysSource for RealSys {
    fn cpu(&mut self) -> CpuSample {
        self.system.refresh_cpu_usage();
        CpuSample {
            per_core: self
                .system
                .cpus()
                .iter()
                .map(|cpu| f64::from(cpu.cpu_usage()))
                .collect(),
        }
    }

    fn net(&mut self) -> NetSample {
        self.networks.refresh(true);
        // Read the routes at every sample, to follow network changes.
        let routes = std::fs::read_to_string("/proc/net/route").unwrap_or_default();
        let iface = default_interface(&routes).or_else(|| {
            // No default route: the busiest non-loopback interface.
            self.networks
                .iter()
                .filter(|(name, _)| name.as_str() != "lo")
                .max_by_key(|(_, data)| data.total_received())
                .map(|(name, _)| name.clone())
        });
        let data = iface.as_ref().and_then(|name| self.networks.get(name));
        NetSample {
            rx_total: data.map_or(0, |d| d.total_received()),
            tx_total: data.map_or(0, |d| d.total_transmitted()),
            iface: data.and(iface),
        }
    }

    fn ram(&mut self) -> RamSample {
        self.system.refresh_memory();
        RamSample {
            total: self.system.total_memory(),
            available: self.system.available_memory(),
        }
    }

    fn disks(&mut self) -> Vec<DiskSample> {
        // Refresh the list too: devices may be mounted or unmounted.
        self.disks.refresh(true);
        let disks = self.disks.iter().map(|disk| RawDisk {
            fs: disk.name().to_string_lossy().into_owned(),
            mount: disk.mount_point().to_string_lossy().into_owned(),
            file_system: disk.file_system().to_string_lossy().into_owned(),
            total: disk.total_space(),
            available: disk.available_space(),
        });
        real_disks(disks)
    }

    fn processes(&mut self) -> Vec<ProcSample> {
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_user(UpdateKind::OnlyIfNotSet),
        );
        let cores = self.system.cpus().len().max(1) as f64;
        let total_memory = self.system.total_memory().max(1) as f64;
        let users = &self.users;
        self.system
            .processes()
            .values()
            // sysinfo lists threads as processes on Linux.
            .filter(|p| p.thread_kind().is_none())
            .map(|p| ProcSample {
                pid: p.pid().as_u32(),
                // sysinfo gives percent of one core; the original shows
                // percent of the whole machine.
                cpu: f64::from(p.cpu_usage()) / cores,
                mem: 100.0 * p.memory() as f64 / total_memory,
                user: p
                    .user_id()
                    .and_then(|uid| users.get_user_by_id(uid))
                    .map(|u| u.name().to_owned())
                    .unwrap_or_default(),
                command: p.name().to_string_lossy().into_owned(),
            })
            .collect()
    }

    fn temperature(&mut self) -> TempSample {
        self.components.refresh(true);
        let sensors = self
            .components
            .iter()
            .filter_map(|c| Some((c.label().to_owned(), f64::from(c.temperature()?))));
        cpu_temperature(sensors)
    }
}

/// The interface of the default route in `/proc/net/route`: destination
/// `00000000`, flag `RTF_UP`, lowest metric.
pub fn default_interface(routes: &str) -> Option<String> {
    const RTF_UP: u32 = 0x1;
    routes
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let (iface, destination, flags, metric) = (
                fields.first()?,
                fields.get(1)?,
                fields.get(3)?,
                fields.get(6)?,
            );
            let flags = u32::from_str_radix(flags, 16).ok()?;
            let metric: u32 = metric.parse().ok()?;
            (*destination == "00000000" && flags & RTF_UP != 0).then(|| (metric, iface.to_string()))
        })
        .min()
        .map(|(_, iface)| iface)
}

/// A disk as read from sysinfo, before filtering.
#[derive(Debug, Clone)]
pub struct RawDisk {
    pub fs: String,
    pub mount: String,
    pub file_system: String,
    pub total: u64,
    pub available: u64,
}

/// Keeps real storage only: no virtual file systems, no empty ones, and each
/// device once (at its first mount point, e.g. not again for bind mounts).
pub fn real_disks(disks: impl IntoIterator<Item = RawDisk>) -> Vec<DiskSample> {
    let mut seen = HashSet::new();
    disks
        .into_iter()
        .filter(|d| d.total > 0 && !VIRTUAL_FILESYSTEMS.contains(&d.file_system.as_str()))
        .filter(|d| seen.insert(d.fs.clone()))
        .map(|d| DiskSample {
            used: d.total.saturating_sub(d.available),
            fs: d.fs,
            mount: d.mount,
            total: d.total,
        })
        .collect()
}

/// CPU temperature from `(label, °C)` sensor readings: `main` is the package
/// sensor if any, otherwise the average of the CPU sensors; `max` is the
/// hottest CPU sensor.
pub fn cpu_temperature(sensors: impl IntoIterator<Item = (String, f64)>) -> TempSample {
    let cpu: Vec<(String, f64)> = sensors
        .into_iter()
        .map(|(label, t)| (label.to_lowercase(), t))
        .filter(|(label, _)| CPU_SENSORS.iter().any(|s| label.contains(s)) || label.contains("cpu"))
        .collect();
    if cpu.is_empty() {
        return TempSample::default();
    }
    let package = cpu
        .iter()
        .find(|(label, _)| {
            ["package id", "tctl", "tdie"]
                .iter()
                .any(|s| label.contains(s))
        })
        .map(|(_, t)| *t);
    let average = cpu.iter().map(|(_, t)| t).sum::<f64>() / cpu.len() as f64;
    let max = cpu
        .iter()
        .map(|(_, t)| *t)
        .fold(f64::NEG_INFINITY, f64::max);
    TempSample {
        main: Some(package.unwrap_or(average)),
        max: Some(max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROUTES: &str = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT
wlan0\t00000000\t0101A8C0\t0003\t0\t0\t600\t00000000\t0\t0\t0
enp2s0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0
docker0\t000011AC\t00000000\t0001\t0\t0\t0\t0000FFFF\t0\t0\t0
";

    #[test]
    fn default_interface_has_the_lowest_metric() {
        assert_eq!(default_interface(ROUTES).as_deref(), Some("enp2s0"));
    }

    #[test]
    fn no_default_route() {
        let routes = "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\n\
                      docker0\t000011AC\t00000000\t0001\t0\t0\t0\n";
        assert_eq!(default_interface(routes), None);
        assert_eq!(default_interface(""), None);
    }

    #[test]
    fn a_default_route_that_is_down_is_ignored() {
        let routes = "Iface\tDestination\tGateway\tFlags\tRefCnt\tUse\tMetric\n\
                      eth0\t00000000\t0101A8C0\t0002\t0\t0\t100\n";
        assert_eq!(default_interface(routes), None);
    }

    fn raw(fs: &str, mount: &str, file_system: &str, total: u64) -> RawDisk {
        RawDisk {
            fs: fs.into(),
            mount: mount.into(),
            file_system: file_system.into(),
            total,
            available: total / 4,
        }
    }

    #[test]
    fn real_disks_skips_virtual_empty_and_duplicate_ones() {
        let disks = real_disks([
            raw("/dev/nvme0n1p3", "/", "ext4", 400),
            raw("/dev/loop3", "/snap/core/1", "squashfs", 100),
            raw("tmpfs", "/run", "tmpfs", 100),
            raw("/dev/nvme0n1p3", "/var/lib/docker", "ext4", 400),
            raw("/dev/sdb1", "/media/empty", "ext4", 0),
            raw("/dev/nvme0n1p2", "/boot/efi", "vfat", 8),
        ]);
        let mounts: Vec<&str> = disks.iter().map(|d| d.mount.as_str()).collect();
        // `/boot` is hidden by the panel, not here (as in the original).
        assert_eq!(mounts, ["/", "/boot/efi"]);
        assert_eq!(disks[0].used, 300);
        assert_eq!(disks[0].use_percent(), 75.0);
    }

    #[test]
    fn cpu_temperature_prefers_the_package_sensor() {
        let temp = cpu_temperature([
            ("Core 0".to_owned(), 50.0),
            ("coretemp Package id 0".to_owned(), 55.0),
            ("coretemp Core 1".to_owned(), 60.0),
            ("nvme Composite".to_owned(), 80.0),
        ]);
        assert_eq!(temp.main, Some(55.0));
        assert_eq!(temp.max, Some(60.0));
    }

    #[test]
    fn cpu_temperature_averages_without_package() {
        let temp = cpu_temperature([("cpu_thermal".to_owned(), 40.0), ("CPU 2".to_owned(), 50.0)]);
        assert_eq!(temp.main, Some(45.0));
        assert_eq!(temp.max, Some(50.0));
    }

    #[test]
    fn cpu_temperature_without_sensors() {
        assert_eq!(cpu_temperature([]), TempSample::default());
        assert_eq!(
            cpu_temperature([("nvme Composite".to_owned(), 40.0)]),
            TempSample::default()
        );
    }

    /// Smoke test on the machine running the tests: plausible values and no
    /// panic, even without sensors (VMs, containers).
    #[test]
    fn the_real_machine_can_be_sampled() {
        let mut sys = RealSys::new();
        let cpu = sys.cpu();
        assert!(!cpu.per_core.is_empty());
        assert!(cpu.per_core.iter().all(|&c| (0.0..=100.0).contains(&c)));
        let ram = sys.ram();
        assert!(ram.total > 0 && ram.available <= ram.total);
        let net = sys.net();
        if net.iface.is_some() {
            assert!(net.rx_total > 0, "counters of {:?} are read", net.iface);
        }
        let _ = sys.disks();
        let _ = sys.temperature();
        let processes = sys.processes();
        let me = std::process::id();
        assert!(processes.iter().any(|p| p.pid == me), "own process listed");
        assert!(processes.iter().all(|p| p.cpu >= 0.0 && p.mem >= 0.0));
    }
}
