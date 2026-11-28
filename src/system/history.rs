//! What the system panels show: sliding windows for the charts (cpu,
//! network) and the latest readings for the rest.
//!
//! Pure state: samples come from a [`super::SysSource`], and the time between
//! network samples is passed in, so everything can be tested without a clock.

use malevich::stream::{Rate, Ring};

use super::{CpuSample, DiskSample, NetSample, ProcSample, RamSample, TempSample};

/// Samples kept per chart: 61, as in the original (x from 60 down to 0).
pub const HISTORY_LEN: usize = 61;

/// History and latest values of every system metric.
#[derive(Clone)]
pub struct SysHistory {
    /// One window per core, load in percent; created at the first sample.
    pub cores: Vec<Ring>,
    /// Received bytes per second.
    pub rx: Ring,
    /// Transmitted bytes per second.
    pub tx: Ring,
    /// Latest network reading (interface and totals).
    pub net: Option<NetSample>,
    pub ram: Option<RamSample>,
    pub disks: Vec<DiskSample>,
    pub processes: Vec<ProcSample>,
    /// Latest temperature reading; `None` until the first one.
    pub temperature: Option<TempSample>,
    /// Hottest CPU temperature seen since start.
    pub max_temperature: Option<f64>,
    rx_rate: Rate,
    tx_rate: Rate,
}

impl Default for SysHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl SysHistory {
    pub fn new() -> Self {
        Self {
            cores: Vec::new(),
            rx: Ring::new(HISTORY_LEN),
            tx: Ring::new(HISTORY_LEN),
            net: None,
            ram: None,
            disks: Vec::new(),
            processes: Vec::new(),
            temperature: None,
            max_temperature: None,
            rx_rate: Rate::new(),
            tx_rate: Rate::new(),
        }
    }

    pub fn push_cpu(&mut self, sample: &CpuSample) {
        if self.cores.len() != sample.per_core.len() {
            // First sample (or, unlikely, a change in the number of cores).
            self.cores = (0..sample.per_core.len())
                .map(|_| Ring::new(HISTORY_LEN))
                .collect();
        }
        for (ring, &load) in self.cores.iter().zip(&sample.per_core) {
            ring.push(load);
        }
    }

    /// Records a network reading taken `elapsed_secs` after the previous one.
    ///
    /// The first reading only sets the baseline. A counter that goes back
    /// (interface reset, or a switch to another interface) counts as 0, like
    /// `Math.max(0, rx_sec)` in the original.
    pub fn push_net(&mut self, sample: NetSample, elapsed_secs: f64) {
        if self.net.as_ref().map(|n| &n.iface) != Some(&sample.iface) {
            // New interface: its counters are unrelated to the previous ones.
            self.rx_rate = Rate::new();
            self.tx_rate = Rate::new();
        }
        let elapsed = elapsed_secs.max(1e-3);
        let rx = self.rx_rate.delta(sample.rx_total as f64);
        let tx = self.tx_rate.delta(sample.tx_total as f64);
        if rx.is_finite() && tx.is_finite() {
            self.rx.push(rx.max(0.0) / elapsed);
            self.tx.push(tx.max(0.0) / elapsed);
        }
        self.net = Some(sample);
    }

    pub fn push_ram(&mut self, sample: RamSample) {
        self.ram = Some(sample);
    }

    pub fn push_disks(&mut self, disks: Vec<DiskSample>) {
        self.disks = disks;
    }

    pub fn push_processes(&mut self, processes: Vec<ProcSample>) {
        self.processes = processes;
    }

    pub fn push_temperature(&mut self, sample: TempSample) {
        if let Some(max) = sample.max.or(sample.main) {
            self.max_temperature = Some(self.max_temperature.map_or(max, |m| m.max(max)));
        }
        self.temperature = Some(sample);
    }

    /// Latest received and transmitted rates, bytes per second.
    pub fn net_rates(&self) -> (f64, f64) {
        let last = |ring: &Ring| ring.snapshot().last().copied().unwrap_or(0.0);
        (last(&self.rx), last(&self.tx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(iface: &str, rx: u64, tx: u64) -> NetSample {
        NetSample {
            iface: Some(iface.into()),
            rx_total: rx,
            tx_total: tx,
        }
    }

    #[test]
    fn cpu_windows_slide_at_61_samples() {
        let mut history = SysHistory::new();
        for i in 0..70 {
            history.push_cpu(&CpuSample {
                per_core: vec![i as f64, 100.0 - i as f64],
            });
        }
        assert_eq!(history.cores.len(), 2);
        let core0 = history.cores[0].snapshot();
        assert_eq!(core0.len(), HISTORY_LEN);
        assert_eq!(core0.first(), Some(&9.0));
        assert_eq!(core0.last(), Some(&69.0));
        assert_eq!(history.cores[1].snapshot().last(), Some(&31.0));
    }

    #[test]
    fn network_rates_need_two_readings() {
        let mut history = SysHistory::new();
        history.push_net(net("eth0", 1000, 500), 1.0);
        assert!(history.rx.is_empty(), "first reading is only a baseline");
        history.push_net(net("eth0", 3000, 1500), 2.0);
        assert_eq!(history.rx.snapshot(), [1000.0]);
        assert_eq!(history.tx.snapshot(), [500.0]);
        assert_eq!(history.net_rates(), (1000.0, 500.0));
    }

    #[test]
    fn a_counter_going_back_counts_as_zero() {
        let mut history = SysHistory::new();
        history.push_net(net("eth0", 5000, 5000), 1.0);
        history.push_net(net("eth0", 100, 6000), 1.0);
        assert_eq!(history.net_rates(), (0.0, 1000.0));
    }

    #[test]
    fn a_new_interface_restarts_the_baseline() {
        let mut history = SysHistory::new();
        history.push_net(net("eth0", 1000, 1000), 1.0);
        history.push_net(net("eth0", 2000, 2000), 1.0);
        history.push_net(net("wlan0", 9_000_000, 9_000_000), 1.0);
        assert_eq!(history.rx.snapshot(), [1000.0], "no spike on switch");
        assert_eq!(history.net.unwrap().iface.as_deref(), Some("wlan0"));
    }

    #[test]
    fn temperature_keeps_the_session_maximum() {
        let mut history = SysHistory::new();
        history.push_temperature(TempSample {
            main: Some(50.0),
            max: Some(60.0),
        });
        history.push_temperature(TempSample {
            main: Some(45.0),
            max: Some(52.0),
        });
        assert_eq!(history.max_temperature, Some(60.0));
        assert_eq!(history.temperature.unwrap().main, Some(45.0));
        history.push_temperature(TempSample::default());
        assert_eq!(history.max_temperature, Some(60.0));
    }

    #[test]
    fn latest_values_replace_the_previous_ones() {
        let mut history = SysHistory::new();
        history.push_ram(RamSample {
            total: 10,
            available: 5,
        });
        history.push_ram(RamSample {
            total: 10,
            available: 2,
        });
        assert_eq!(history.ram.unwrap().used(), 8);
        history.push_disks(vec![]);
        assert!(history.disks.is_empty());
    }
}
