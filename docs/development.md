# Development

How to set up a development environment, find your way in the code, add a
feature and test it, with or without a Bitcoin node.

## Requirements

| tool | version | needed for |
|---|---|---|
| Rust | 1.88 or later, edition 2024 | building ([rustup](https://rustup.rs)) |
| `rustfmt`, `clippy` | from the same toolchain | formatting and lints (`rustup component add rustfmt clippy`) |
| tmux | any | optional: trying the dashboard at fixed sizes |
| Node.js | 18 or later | optional: running the original for comparison |
| Bitcoin Core | any recent release | optional: testing against a real node (regtest is enough) |

No C libraries are needed. Cargo downloads every dependency, including the
charting library [malevich](https://crates.io/crates/malevich), then check that
everything is green:

```sh
cargo build && cargo test && cargo clippy --all-targets
```

The reference for the behaviour is the original Node.js version, in the
[v2.5.x-nodejs](https://github.com/st3b1t/SatoshiTop/tree/v2.5.x-nodejs) branch.
If malevich lacks something, work around it in our code.

## Code layout

Data flows one way: sources produce samples, the app keeps them, pure
functions turn them into widgets.

```
SysSource / RpcBackend  →  AppState  →  ui::panels::*::render  →  terminal
 (real or fake)            (history,     (pure: state in,
                            snapshot)      widgets out)
```

| path | contents |
|---|---|
| `src/main.rs` | the `satop` binary: options, `--confgen`, `--verbose`, then `app::run` |
| `src/app.rs` | `AppState`, the refresh cadences and the event loop (one tokio `select!` on keys, timers and RPC results) |
| `src/config/` | command line (clap), `~/.satoprc`, merge and validation, RPC credentials |
| `src/system/` | `SysSource` trait, the real source (sysinfo), `SysHistory` (the last 61 samples) |
| `src/bitcoin/` | `RpcBackend` trait and HTTP client, RPC types, `NodeSnapshot`; one file per panel with its pure logic (`fees.rs`, `blocks.rs`, …) |
| `src/ui/` | `dashboard.rs` (grid and dispatch), `panels/` (one file per panel), `widgets/lcd.rs`, `panel_block` and `BORDER_COLOR` in `mod.rs` |
| `src/util/` | formatting (`human.rs`), `COLORS`, `truncate` |
| `src/tests/` | the fake sources used by `FAKESYS`/`FAKERPC`, with JSON fixtures; production code, not only tests |
| `src/test_util.rs` | helpers shared by the unit tests (`TempDir`) |

Charts are drawn with malevich (`Plot` rendered through its ratatui widget);
tables and text with ratatui. Before drawing a chart by hand, look for a
mark or preset in malevich that already does it.

## Adding a feature

### Conventions

- Keep logic pure and testable: computing values, labels and titles goes in
  plain functions (in `src/bitcoin/<panel>.rs` or the panel file), and
  `render` only lays them out.
- Keep the observable behaviour of SatoshiTop: same options, environment
  variables, `~/.satoprc` and panels. A deliberate difference is fine, but
  record why.
- Comments, doc comments and test messages are in English.
- Panels use `ui::panel_block(title)` for their border, so all borders share
  `ui::BORDER_COLOR`.
- A feature is done when `cargo build` and `cargo clippy --all-targets` give no
  warnings, `cargo test` passes and `FAKESYS=1 FAKERPC=1 cargo run` shows it.
  Run `cargo fmt` before committing.
- `.github/workflows/` holds the GitHub Actions workflows:
  - `ci.yml`, on every push and pull request to `master`, runs the same checks
    with Rust 1.88 (`cargo fmt --check`, clippy with `-D warnings`, tests);
  - `build.yml` and `release.yml` run on every pushed `v*` tag (`v3.0.1`,
    `v3.1.0`, `v3.1.0-alpha.1`; a tag with a `-` suffix gives a GitHub
    pre-release). They create the GitHub release of the tag if it does not
    exist and attach the archives, each with its `.sha256`: `build.yml` the five
    `satop-debian-13-x86-64`, `satop-debian-12-x86-64`,
    `satop-ubuntu-24.04-x86-64`, `satop-ubuntu-22.04-x86-64` (built inside a
    container of each distribution) and `satop-raspios-64-arm64` (built with
    `cross`); `release.yml` the static `satop-<tag>-x86_64-linux-musl`.

  To publish a release: `git tag v3.1.0 && git push origin v3.1.0`.

### A new value from the node

1. Add the RPC result type in `src/bitcoin/types.rs` (serde, only the fields
   you need).
2. Add a field to `NodeSnapshot` in `src/bitcoin/snapshot.rs` and fetch it in
   `SnapshotFetcher::fetch`, in the `futures::join!` with the other calls. Each
   field is a `Result`, so one failing call does not hide the others.
3. Answer the same method in `src/tests/rpc_fake.rs`, with a fixture in
   `src/tests/fixtures/` if the response is large, so fake mode shows it.

### A new system metric

1. Add a method to the `SysSource` trait in `src/system/mod.rs`, and implement
   it in `src/system/real.rs` (sysinfo or `/proc`) and in
   `src/tests/sys_fake.rs`.
2. Store it in `SysHistory` (`src/system/history.rs`) and sample it in
   `src/app.rs` at the right cadence: `intervalsys/3` for fast values,
   `intervalsys` for processes and temperature, `intervalsys*5` for disks.

### A new panel

1. Create `src/ui/panels/<name>.rs` with a `TITLE` constant and a
   `render(frame, area, …)` function; declare it in `src/ui/panels/mod.rs`.
2. Give it a cell in `PANELS` in `src/ui/dashboard.rs` (row, column, row span,
   column span on the 11x12 grid) and a branch in the `match` of `draw`.
   Other cells may need to shrink: the dashboard tests check that the panels
   cover the screen without gaps or overlaps.
3. Show an empty `panel_block(TITLE)` until the data arrives, and the error in
   red if its field of the snapshot failed, like the other Bitcoin panels.

## Test environment

There are five levels, from the fastest to the most realistic.

### Unit tests

```sh
cargo test                     # all tests
cargo test fees                # only tests whose name contains "fees"
cargo test -- --nocapture      # show what the tests print
```

What is tested and how:

- **Pure logic** (formatting, fees, block columns, config precedence,
  credentials): plain inputs and expected values. Expected formats were
  generated from the JavaScript with `node -e`, so they match the original.
- **Charts:** a malevich `Plot` is rendered to a string with
  `malevich::Frame::plain(width, height)` and checked for its labels.
- **Panels and layout:** drawn on ratatui's `TestBackend` at fixed sizes;
  the tests read the buffer back (text, colors, exact `Rect`s).
- **RPC client:** a local tokio listener plays the node and answers with
  200, an RPC error, 401, a delay or a closed connection.
- **Files:** `test_util::TempDir` creates a temporary directory for
  configuration and cookie files.

Tests never change the process environment (`set_var` is unsafe in edition
2024 and tests run in parallel): code that reads a variable takes it as a
parameter.

### Fake mode

Environment variables replace the real data sources with simulated data:

| variable | effect |
|---|---|
| `FAKERPC=1` | simulates the Bitcoin node's RPC responses |
| `FAKESYS=1` | simulates all system metrics (cpu, ram, network, temperature, disks, processes) |
| `FAKESEED=<n>` | fixes the random seed of the simulations, for reproducible runs |

They are not options and do not appear in `--help`. An unset or empty
variable, `0`, `false`, `no` and `off` leave simulation off; any other value
turns it on. With `FAKERPC` no credentials are needed.

```sh
FAKESYS=1 FAKERPC=1 cargo run                          # everything simulated
FAKESYS=1 FAKERPC=1 cargo run -- --intervalsys=500 --intervalrpc=1000   # faster refresh
FAKESYS=1 cargo run                                    # real node, simulated system
FAKESYS=1 FAKERPC=1 FAKESEED=42 cargo run              # same data at every run
```

The fake node sits at height 840004 and is syncing; its peers, blocks and
mempool come from the fixtures. The fake system has 4 cores, 16 GiB of memory,
`eth0`, three disks and seventeen processes.

### Fixed terminal sizes

tmux runs the dashboard at an exact size and captures the screen as text, which
is the quickest way to check layout changes:

```sh
tmux new-session -d -s satop -x 80 -y 24 'FAKESYS=1 FAKERPC=1 cargo run'
tmux capture-pane -p -t satop          # print the screen
tmux resize-window -t satop -x 40 -y 12
tmux send-keys -t satop q              # quit
```

Sizes worth checking: 40x12 (the minimum), 80x24, 120x40.

### A real node on regtest

regtest is a private chain on your machine: blocks are mined on demand and
nothing is downloaded.

```sh
bitcoind -regtest -daemon -fallbackfee=0.0001
bitcoin-cli -regtest createwallet dev
bitcoin-cli -regtest -generate 101          # mature coinbase, so there are funds
cargo run -- --rpcport=18443 --rpccookiefile="$HOME/.bitcoin/regtest/.cookie"
```

In another shell, make the panels move:

```sh
bitcoin-cli -regtest -generate 1                                         # a new block
bitcoin-cli -regtest sendtoaddress "$(bitcoin-cli -regtest getnewaddress)" 0.1   # a mempool transaction
```

Stop the node with `bitcoin-cli -regtest stop`. Fee estimates stay
empty (`-- SvB`) on regtest until enough blocks with transactions are mined.

### A remote node through an SSH tunnel

For development only: to run `satop` from your working copy against a node on
another machine, without exposing its RPC port on the network.
`scripts/satop-tunnel-rpc.sh` asks for the node's host name, SSH user and SSH
port, then opens a background tunnel from `127.0.0.1:8332` to port 8332 on the
node, the same as:

```sh
ssh -fN -l <user> -p 22 -L 127.0.0.1:8332:localhost:8332 <host>
```

Then run `cargo run -- --rpcuser=<username> --rpcpassword=<password>`.

- Port 8332 on your machine must be free; for testnet, signet or regtest
  change the second port of `-L` (18332, 38332, 18443).
- The cookie file is on the node: use `rpcuser`/`rpcpassword`, or copy the
  cookie and pass it with `--rpccookiefile`.
- The system panels show your machine, not the node.
- Close the tunnel with `pkill -f 'ssh -fN .*-L 127.0.0.1:8332:localhost:8332'`.

## Comparing with the original

The Node.js version lives in the
[v2.5.x-nodejs](https://github.com/st3b1t/SatoshiTop/tree/v2.5.x-nodejs) branch;
clone it outside this repository:

```sh
git clone -b v2.5.x-nodejs https://github.com/st3b1t/SatoshiTop satoshitop-nodejs
cd satoshitop-nodejs
npm install
FAKERPC=1 npm run dev
```
