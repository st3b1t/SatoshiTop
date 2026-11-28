//! Simulated machine for `FAKESYS` (`FAKEMODE` in the original), ported from `test/sys-fake.js`.
//!
//! The original simulates only disks and processes; here every metric is
//! simulated, so that the dashboard is reproducible in containers, CI and VMs.

use serde::Deserialize;

use super::Rng;
use crate::system::{
    CpuSample, DiskSample, NetSample, ProcSample, RamSample, SysSource, TempSample,
};

const PROCESSES: &str = include_str!("fixtures/sys-processes.json");

/// Simulated cores.
const CORES: usize = 4;
/// Simulated memory: 16 GiB.
const RAM_TOTAL: u64 = 16 * 1024 * 1024 * 1024;
/// Highest simulated process CPU load, as in the original (`rnd()*30`).
const PROCESS_CPU_MAX: f64 = 30.0;

/// The three file systems of the original fake, with its size and use
/// percent (`/boot/efi` included: the panel hides it).
const DISKS: [(&str, &str, u64, f64); 3] = [
    ("/dev/nvme0n1p3", "/", 398_394_933_248, 97.89),
    ("/dev/nvme0n1p2", "/boot/efi", 824_180_736, 17.15),
    ("/dev/dm-1", "/media/Blockchain", 97_826_279_424, 26.27),
];

#[derive(Deserialize)]
struct ProcessList {
    list: Vec<FixtureProcess>,
}

#[derive(Deserialize)]
struct FixtureProcess {
    pid: u32,
    mem: f64,
    user: String,
    command: String,
}

/// A machine that wanders around plausible values.
pub struct FakeSys {
    rng: Rng,
    cores: Vec<f64>,
    ram_used: f64,
    rx_total: u64,
    tx_total: u64,
    temperature: f64,
    processes: Vec<FixtureProcess>,
}

impl FakeSys {
    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let cores = (0..CORES).map(|_| rng.next_f64() * 100.0).collect();
        let processes = serde_json::from_str::<ProcessList>(PROCESSES)
            .expect("sys-processes fixture is valid")
            .list;
        Self {
            ram_used: RAM_TOTAL as f64 * (0.3 + 0.4 * rng.next_f64()),
            rx_total: rng.below(1 << 40),
            tx_total: rng.below(1 << 38),
            temperature: 40.0 + rng.next_f64() * 30.0,
            cores,
            processes,
            rng,
        }
    }

    /// Moves `value` by up to `step` either way, within `[min, max]`.
    fn wander(&mut self, value: f64, step: f64, min: f64, max: f64) -> f64 {
        (value + (self.rng.next_f64() * 2.0 - 1.0) * step).clamp(min, max)
    }
}

impl SysSource for FakeSys {
    fn cpu(&mut self) -> CpuSample {
        for i in 0..self.cores.len() {
            self.cores[i] = self.wander(self.cores[i], 15.0, 0.0, 100.0);
        }
        CpuSample {
            per_core: self.cores.clone(),
        }
    }

    fn net(&mut self) -> NetSample {
        // Up to ~2 MB in, ~500 kB out per sample.
        self.rx_total += self.rng.below(2_000_000);
        self.tx_total += self.rng.below(500_000);
        NetSample {
            iface: Some("eth0".to_owned()),
            rx_total: self.rx_total,
            tx_total: self.tx_total,
        }
    }

    fn ram(&mut self) -> RamSample {
        let total = RAM_TOTAL as f64;
        self.ram_used = self.wander(self.ram_used, total * 0.02, total * 0.1, total * 0.95);
        RamSample {
            total: RAM_TOTAL,
            available: RAM_TOTAL - self.ram_used as u64,
        }
    }

    fn disks(&mut self) -> Vec<DiskSample> {
        DISKS
            .iter()
            .map(|&(fs, mount, total, use_percent)| DiskSample {
                fs: fs.to_owned(),
                mount: mount.to_owned(),
                total,
                used: (total as f64 * use_percent / 100.0) as u64,
            })
            .collect()
    }

    fn processes(&mut self) -> Vec<ProcSample> {
        let rng = &mut self.rng;
        self.processes
            .iter()
            .map(|p| ProcSample {
                pid: p.pid,
                cpu: rng.next_f64() * PROCESS_CPU_MAX,
                mem: p.mem,
                user: p.user.clone(),
                command: p.command.clone(),
            })
            .collect()
    }

    fn temperature(&mut self) -> TempSample {
        self.temperature = self.wander(self.temperature, 3.0, 40.0, 70.0);
        let hottest = self.temperature + self.rng.next_f64() * 10.0;
        TempSample {
            main: Some(self.temperature.round()),
            max: Some(hottest.round()),
        }
    }
}

#[cfg(test)]
mod sys_fake_tests {
    use super::*;

    #[test]
    fn every_metric_is_simulated() {
        let mut sys = FakeSys::new(5);
        let cpu = sys.cpu();
        assert_eq!(cpu.per_core.len(), CORES);
        assert!(cpu.per_core.iter().all(|c| (0.0..=100.0).contains(c)));

        let ram = sys.ram();
        assert_eq!(ram.total, RAM_TOTAL);
        assert!((10.0..=95.0).contains(&ram.used_percent()));

        let first = sys.net();
        let second = sys.net();
        assert_eq!(second.iface.as_deref(), Some("eth0"));
        assert!(second.rx_total >= first.rx_total && second.tx_total >= first.tx_total);

        let disks = sys.disks();
        let mounts: Vec<&str> = disks.iter().map(|d| d.mount.as_str()).collect();
        assert_eq!(mounts, ["/", "/boot/efi", "/media/Blockchain"]);
        assert!((disks[0].use_percent() - 97.89).abs() < 0.01);
        assert!((disks[2].use_percent() - 26.27).abs() < 0.01);

        let processes = sys.processes();
        assert_eq!(processes.len(), 17);
        assert!(
            processes
                .iter()
                .all(|p| (0.0..PROCESS_CPU_MAX).contains(&p.cpu))
        );
        assert!(
            processes
                .iter()
                .any(|p| p.command == "bitcoin" && p.user == "bitcoin")
        );

        let temp = sys.temperature();
        let (main, max) = (temp.main.unwrap(), temp.max.unwrap());
        assert!((40.0..=70.0).contains(&main));
        assert!(max >= main);
    }

    #[test]
    fn same_seed_same_machine() {
        let (mut a, mut b) = (FakeSys::new(9), FakeSys::new(9));
        for _ in 0..10 {
            assert_eq!(a.cpu(), b.cpu());
            assert_eq!(a.processes(), b.processes());
            assert_eq!(a.temperature(), b.temperature());
        }
    }
}
