# Installing

## Precompiled binaries

Download the archive for your system from the releases page.

Each archive holds `satop`, `LICENSE` and `README.md`. Extract it and install
the binary:

```sh
tar xzf SatoshiTop-debian-12-x86-64.tar.gz
sudo install -m 0755 SatoshiTop-debian-12-x86-64/satop /usr/local/bin/satop
satop --version
```

## Compile from source

Build with Rust 1.88 or later. Distribution packages (Debian's, for one) may be
older, so install the toolchain with rustup. On Debian:

install the build tools and rustup for building from source:
```sh
sudo apt install build-essential git curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```sh
cargo build --release
sudo install -m 0755 target/release/satop /usr/local/bin/satop
```

Or, only for the current user: `cargo install --path .` (into `~/.cargo/bin`).
To uninstall: `sudo rm /usr/local/bin/satop` or `cargo uninstall satoshitop`.

## Building on one machine, running on another

The binary needs the glibc it was built with or a newer one. Copied to an older
system it fails, for example from Ubuntu 24.04 to Debian 12:

```
./satop: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found
```

Build a static binary with musl instead; it runs on any x86_64 Linux:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
# binary in target/x86_64-unknown-linux-musl/release/satop
```
