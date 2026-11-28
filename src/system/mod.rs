//! System metrics, ported from `lib/system/*.js`.
//!
//! A [`SysSource`] reads one metric at a time, so that the app can sample
//! each one at its own cadence (cpu, network and ram every `intervalsys/3`,
//! processes and temperature every `intervalsys`, disks every
//! `intervalsys*5`). [`real::RealSys`] reads the machine with `sysinfo`;
//! [`crate::tests::sys_fake::FakeSys`] simulates it (`FAKESYS`).

pub mod history;
pub mod real;

use crate::tests::sys_fake::FakeSys;

/// Load of every core, in percent (0–100), in core order.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuSample {
    pub per_core: Vec<f64>,
}

/// Cumulative traffic of the default network interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetSample {
    /// Interface name; `None` when there is no usable interface.
    pub iface: Option<String>,
    /// Bytes received since boot.
    pub rx_total: u64,
    /// Bytes transmitted since boot.
    pub tx_total: u64,
}

/// Memory, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RamSample {
    pub total: u64,
    pub available: u64,
}

impl RamSample {
    /// Memory in use: total minus available, as in the original.
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }

    /// Memory in use, in percent (0–100).
    pub fn used_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.used() as f64 / self.total as f64
        }
    }
}

/// One mounted filesystem.
#[derive(Debug, Clone, PartialEq)]
pub struct DiskSample {
    /// Device, e.g. `/dev/nvme0n1p3`.
    pub fs: String,
    /// Mount point, e.g. `/`.
    pub mount: String,
    /// Size in bytes.
    pub total: u64,
    /// Space in use, in bytes.
    pub used: u64,
}

impl DiskSample {
    /// Space in use, in percent (0–100).
    pub fn use_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            100.0 * self.used as f64 / self.total as f64
        }
    }
}

/// One running process.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcSample {
    pub pid: u32,
    /// Share of the whole machine's CPU, in percent (0–100).
    pub cpu: f64,
    /// Share of the total memory, in percent (0–100).
    pub mem: f64,
    pub user: String,
    pub command: String,
}

/// CPU temperature in °C; `None` when there are no sensors (e.g. in a VM).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TempSample {
    /// Package temperature, or the average of the CPU sensors.
    pub main: Option<f64>,
    /// Hottest CPU sensor.
    pub max: Option<f64>,
}

/// Something that reads the system metrics.
pub trait SysSource {
    fn cpu(&mut self) -> CpuSample;
    fn net(&mut self) -> NetSample;
    fn ram(&mut self) -> RamSample;
    fn disks(&mut self) -> Vec<DiskSample>;
    fn processes(&mut self) -> Vec<ProcSample>;
    fn temperature(&mut self) -> TempSample;
}

/// The source chosen at runtime.
pub enum AnySys {
    /// Boxed: the sysinfo handles are much larger than the fake.
    Real(Box<real::RealSys>),
    Fake(FakeSys),
}

impl AnySys {
    /// [`FakeSys`] when `FAKESYS` is set (see [`crate::config::env_flag`]),
    /// otherwise the real machine.
    pub fn from_env() -> Self {
        if crate::config::env_flag_var("FAKESYS") {
            Self::Fake(FakeSys::new(crate::tests::seed_from_env()))
        } else {
            Self::Real(Box::default())
        }
    }

    fn source(&mut self) -> &mut dyn SysSource {
        match self {
            Self::Real(sys) => sys.as_mut(),
            Self::Fake(sys) => sys,
        }
    }
}

impl SysSource for AnySys {
    fn cpu(&mut self) -> CpuSample {
        self.source().cpu()
    }
    fn net(&mut self) -> NetSample {
        self.source().net()
    }
    fn ram(&mut self) -> RamSample {
        self.source().ram()
    }
    fn disks(&mut self) -> Vec<DiskSample> {
        self.source().disks()
    }
    fn processes(&mut self) -> Vec<ProcSample> {
        self.source().processes()
    }
    fn temperature(&mut self) -> TempSample {
        self.source().temperature()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_used_and_percent() {
        let ram = RamSample {
            total: 16,
            available: 4,
        };
        assert_eq!(ram.used(), 12);
        assert_eq!(ram.used_percent(), 75.0);
        let empty = RamSample {
            total: 0,
            available: 0,
        };
        assert_eq!(empty.used_percent(), 0.0);
    }

    #[test]
    fn disk_use_percent() {
        let disk = DiskSample {
            fs: "/dev/sda1".into(),
            mount: "/".into(),
            total: 200,
            used: 50,
        };
        assert_eq!(disk.use_percent(), 25.0);
    }
}
