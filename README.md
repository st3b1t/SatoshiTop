# SatoshiTop

Bitcoin full node monitoring dashboard for terminal.

*Update november 2026: Since version 3.x a complete porting in Rust previously it was a NodeJs project, still available in the [v2.5.x-nodejs](https://github.com/st3b1t/SatoshiTop/tree/v2.5.x-nodejs) branch*

SatoshiTop is an *htop for Bitcoin*: a dashboard that shows much of what a modern Bitcoin
explorer shows, with very low requirements. It needs no graphical environment
and exchanges very little data with the node, so it suits slow connections such
as Tor and low-resolution, low-refresh monitors and LCDs.

## Features

- **System resources:** cpu, memory, disks, network, processes, temperature
- **Bitcoin Core:** node status, sync progress, uptime, user agent, fee
  estimates, mempool usage, last blocks, connected peers, Tor address
- If the node cannot be reached, the Bitcoin Node panel shows the error and
  `satop` keeps retrying; system panels keep working

## Requirements

- **Zero dependencies:** a single binary that needs nothing installed at
  runtime, no Node.js, no OpenSSL, no ncurses; only the system C library that
  every Linux has (the static musl build not even that)
- Linux is the only supported platform
- A Bitcoin Core node with the RPC interface reachable (see
  [Bitcoin Core RPC Configuration](docs/config.md#bitcoin-core-rpc-configuration))
- A terminal of at least 40x12 characters; 120x40 or larger shows every panel
  comfortably
- If after running you see strange characters or question marks (`?`) set (for example in
`~/.profile` the line: `export LANG=en_US.utf8 TERM=xterm-256color`

## Installing

Precompiled binaries or building from source: see [docs/install.md](docs/install.md).

## Usage

Start `satop` on the machine whose resources you want to monitor, with the
cookie file of Bitcoin Core (its path depends on the node's [datadir](https://github.com/bitcoin/bitcoin/blob/master/doc/init.md#configuration)):

```sh
satop --rpccookiefile=/home/<user>/.bitcoin/.cookie
```

or with user name and password:

```sh
 satop --rpcuser=<username> --rpcpassword=<password>
```

The leading space keeps the command, and the password, out of the shell
history in most setups; a configuration file avoids the problem altogether:

```sh
satop --confgen      # create a new commented ~/.satoprc, readable only by you
```

Press `q`, `Esc` or `Ctrl-C` to quit.

## Documentation

- [docs/install.md](docs/install.md): precompiled binaries, building from
  source, static build for older systems
- [docs/config.md](docs/config.md): options, environment variables,
  `~/.satoprc`, authentication
- [docs/console.md](docs/console.md): starting `satop` from a remote shell on
  the machine's screen, or at boot; small displays
- [docs/development.md](docs/development.md): code layout, adding features,
  tests, fake mode

## Credits

SatoshiTop is written by [st3b1t](https://github.com/st3b1t).
Proposals and bug reports go to its [issue tracker](https://github.com/st3b1t/SatoshiTop/issues).

### Donate

Donate via Lightning Network Bolt12 ⚡ to incentivize future development and bug fixes.

![donate](https://github.com/st3b1t/st3b1t/raw/main/donate_bolt12.png)

lno1pgqppmsrse80qf0aara4slvcjxrvu6j2rp5ftmjy4yntlsmsutpkvkt6878s98hg39x6ct93flx6jjkpcygfnqzmap67azxk5m792zhs0mg50z8vqgpqc4ecv0ff9klzlrru7uz95jylq9nfhzc5fpk0ezgu6490tr6rrwcqx06a2clgplqdqzfgd3x4h960zypk9md35uxh4suz3frxcz9dckyfm0gs5hl5lgxt8mgawyt63nsu4lzz0pvs95j825qfsvyxcuy9ntytqa7ljz70pcqletnnuklslxgx7zrj8ylrqqertseym9scshtsgcqe0tlyfhwynug9qve279l8j6txr72qr7qrnjrkm3lh7v23vpwlk298jgf34jlrl46s

## License

MIT, like SatoshiTop: see [LICENSE](LICENSE).
