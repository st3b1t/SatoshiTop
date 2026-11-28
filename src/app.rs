//! The application: state, scheduler and event loop, ported from
//! `lib/satop.js` and the `setInterval` calls of every panel.
//!
//! Everything runs on one tokio `current_thread` runtime, in a single
//! `tokio::select!` over keys, the system timers, the RPC timer and the RPC
//! results. System metrics are sampled directly in the loop (sysinfo is
//! synchronous and fast); node snapshots come from a separate task, so a slow
//! node never blocks keys or redraws.

use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;
use tokio::time::{Interval, MissedTickBehavior};

use crate::bitcoin::rpc::{AnyRpc, RpcError};
use crate::bitcoin::snapshot::{NodeSnapshot, SnapshotFetcher};
use crate::config::Config;
use crate::system::history::SysHistory;
use crate::system::{AnySys, SysSource};
use crate::ui::dashboard;

/// How often each group of metrics is refreshed, as in the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cadence {
    /// `intervalsys/3`: cpu, network, ram.
    pub fast: Duration,
    /// `intervalsys`: processes, temperature.
    pub sys: Duration,
    /// `intervalsys*5`: disks.
    pub disks: Duration,
    /// `intervalrpc`: every Bitcoin panel.
    pub rpc: Duration,
}

impl Cadence {
    pub fn from_config(config: &Config) -> Self {
        // Timers need a non-zero period; the config already forbids 0.
        let ms = |value: u64| Duration::from_millis(value.max(1));
        Self {
            fast: ms(config.intervalsys / 3),
            sys: ms(config.intervalsys),
            disks: ms(config.intervalsys.saturating_mul(5)),
            rpc: ms(config.intervalrpc),
        }
    }
}

/// Everything the dashboard shows.
pub struct AppState {
    pub sys: SysHistory,
    /// Latest node snapshot; `None` until the first one arrives.
    pub node: Option<NodeSnapshot>,
    pub cadence: Cadence,
    /// `rpcconnect:rpcport`, shown in the connection error.
    pub node_address: String,
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        Self {
            sys: SysHistory::new(),
            node: None,
            cadence: Cadence::from_config(config),
            node_address: format!("{}:{}", config.rpcconnect, config.rpcport),
        }
    }

    /// A state with the default cadences, for view tests.
    #[cfg(test)]
    pub fn for_tests() -> Self {
        let config = Config::merge(&crate::config::Cli::default(), None, "/x".into());
        Self::new(&config)
    }
}

/// What a key does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Nothing,
}

/// `q`, `Esc` and `Ctrl-C` quit, as in the original.
pub fn action_for(key: &KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::Nothing;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        _ => Action::Nothing,
    }
}

/// Runs the dashboard until the user quits. The terminal is restored on
/// return and on panic.
pub fn run(config: Config) -> io::Result<()> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        // Without a terminal (e.g. output redirected) report an error
        // instead of panicking.
        let mut terminal = ratatui::try_init()
            .map_err(|e| io::Error::new(e.kind(), format!("cannot open the terminal: {e}")))?;
        let result = event_loop(&mut terminal, &config).await;
        ratatui::restore();
        result
    })
}

fn timer(period: Duration) -> Interval {
    let mut timer = tokio::time::interval(period);
    timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    timer
}

/// Starts the task that fetches node snapshots on request. Returns the request
/// and result channels, or the error if no backend can be built.
fn spawn_rpc(
    config: &Config,
) -> Result<(mpsc::Sender<()>, mpsc::Receiver<NodeSnapshot>), RpcError> {
    let backend = AnyRpc::from_env(config)?;
    let (request_tx, mut request_rx) = mpsc::channel::<()>(1);
    let (snapshot_tx, snapshot_rx) = mpsc::channel(1);
    tokio::spawn(async move {
        let mut fetcher = SnapshotFetcher::new(backend);
        while request_rx.recv().await.is_some() {
            if snapshot_tx.send(fetcher.fetch().await).await.is_err() {
                break;
            }
        }
    });
    Ok((request_tx, snapshot_rx))
}

async fn event_loop(terminal: &mut DefaultTerminal, config: &Config) -> io::Result<()> {
    let cadence = Cadence::from_config(config);
    let mut state = AppState::new(config);
    let mut sys = AnySys::from_env();
    let mut events = EventStream::new();

    let (mut fast, mut slow, mut disks, mut rpc) = (
        timer(cadence.fast),
        timer(cadence.sys),
        timer(cadence.disks),
        timer(cadence.rpc),
    );
    let (rpc_requests, mut snapshots) = match spawn_rpc(config) {
        Ok((requests, snapshots)) => (Some(requests), Some(snapshots)),
        Err(e) => {
            state.node = Some(NodeSnapshot::failed(e));
            (None, None)
        }
    };
    // A tick does not start while the previous snapshot is still on its way.
    let mut rpc_in_flight = false;
    let mut last_net = Instant::now();

    loop {
        terminal.draw(|frame| dashboard::draw(frame, &state))?;

        tokio::select! {
            event = events.next() => match event {
                Some(Ok(Event::Key(key))) => {
                    if action_for(&key) == Action::Quit {
                        return Ok(());
                    }
                }
                // Resize and other events: just redraw.
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(e),
                None => return Ok(()),
            },
            _ = fast.tick() => {
                state.sys.push_cpu(&sys.cpu());
                let now = Instant::now();
                state.sys.push_net(sys.net(), now.duration_since(last_net).as_secs_f64());
                last_net = now;
                state.sys.push_ram(sys.ram());
            }
            _ = slow.tick() => {
                state.sys.push_processes(sys.processes());
                state.sys.push_temperature(sys.temperature());
            }
            _ = disks.tick() => {
                state.sys.push_disks(sys.disks());
            }
            _ = rpc.tick(), if snapshots.is_some() && !rpc_in_flight => {
                if let Some(requests) = &rpc_requests {
                    rpc_in_flight = requests.try_send(()).is_ok();
                }
            }
            snapshot = async { snapshots.as_mut()?.recv().await }, if snapshots.is_some() => {
                match snapshot {
                    Some(snapshot) => state.node = Some(snapshot),
                    // The RPC task is gone (it only stops on errors of its
                    // channels): stop asking.
                    None => snapshots = None,
                }
                rpc_in_flight = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn quit_keys_match_the_original() {
        assert_eq!(
            action_for(&key(KeyCode::Char('q'), KeyModifiers::NONE)),
            Action::Quit
        );
        assert_eq!(
            action_for(&key(KeyCode::Esc, KeyModifiers::NONE)),
            Action::Quit
        );
        assert_eq!(
            action_for(&key(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Action::Quit
        );
        assert_eq!(
            action_for(&key(KeyCode::Char('c'), KeyModifiers::NONE)),
            Action::Nothing
        );
        assert_eq!(
            action_for(&key(KeyCode::Char('x'), KeyModifiers::NONE)),
            Action::Nothing
        );
    }

    #[test]
    fn key_releases_are_ignored() {
        let mut release = key(KeyCode::Char('q'), KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(action_for(&release), Action::Nothing);
    }

    #[test]
    fn cadences_follow_the_intervals() {
        let mut config = Config::merge(&crate::config::Cli::default(), None, "/x".into());
        let cadence = Cadence::from_config(&config);
        assert_eq!(cadence.fast, Duration::from_millis(1000));
        assert_eq!(cadence.sys, Duration::from_millis(3000));
        assert_eq!(cadence.disks, Duration::from_millis(15000));
        assert_eq!(cadence.rpc, Duration::from_millis(5000));

        config.intervalsys = 1000;
        config.intervalrpc = 1;
        let cadence = Cadence::from_config(&config);
        assert_eq!(cadence.fast, Duration::from_millis(333));
        assert_eq!(cadence.rpc, Duration::from_millis(1));
        config.intervalsys = 2;
        assert_eq!(Cadence::from_config(&config).fast, Duration::from_millis(1));
    }
}
