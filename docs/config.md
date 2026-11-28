# Configuration

`satop` reads its settings from four sources. Each one overrides the previous:

1. built-in defaults
2. the configuration file (`~/.satoprc`)
3. environment variables
4. command-line options

Run `satop --help` for the short version of this page.

## Options

CLI parameters, environment variables and configuration file keys are listed in the table below.

| option | environment | file key | default | description |
|---|---|---|---|---|
| `--conf <FILE>` | `CONF` | — | `.satoprc` | configuration file; a relative path is resolved against `$HOME` |
| `--rpcconnect <IP>` | `RPCCONNECT` | `rpcconnect` | `127.0.0.1` | address of the Bitcoin node |
| `--rpcport <PORT>` | `RPCPORT` | `rpcport` | `8332` | JSON-RPC port of the node (1–65535) |
| `--rpcclienttimeout <MS>` | `RPCCLIENTTIMEOUT` | `rpcclienttimeout` | `5000` | HTTP timeout in milliseconds; `0` disables it |
| `--rpccookiefile <FILE>` | `RPCCOOKIEFILE` | `rpccookiefile` | — | auth cookie written by the node; absolute path |
| `--rpcuser <USER>` | `RPCUSER` | `rpcuser` | — | JSON-RPC username |
| `--rpcpassword <PASSWORD>` | `RPCPASSWORD` | `rpcpassword` | — | JSON-RPC password |
| `--intervalsys <MS>` | `INTERVALSYS` | `intervalsys` | `3000` | refresh interval of the system panels, in milliseconds (at least 1) |
| `--intervalrpc <MS>` | `INTERVALRPC` | `intervalrpc` | `5000` | refresh interval of the Bitcoin panels, in milliseconds (at least 1) |
| `-v`, `--verbose[=BOOL]` | `VERBOSE` | `verbose` | `false` | print the final configuration as JSON on stderr |
| `--confgen` | — | — | — | write an example configuration file and exit |
| `-?`, `--help` | — | — | — | print help and exit |
| `-V`, `--version` | — | — | — | print version and exit |

Values are given as `--option value` or `--option=value`. `--verbose` takes no
value or an explicit one with `=`: `--verbose`, `--verbose=true`, `--verbose=0`.

Invalid values (a port out of range, a non-numeric interval, an interval of 0)
stop the program with an error that says where the value came from, and exit
code 2. Unknown options are rejected the same way.

## Authentication

The cookie file is the preferred method: the node writes a fresh one at every
start, usually in `~/.bitcoin/.cookie` (or `~/.bitcoin/<network>/.cookie`). See
[cookie files](https://github.com/Kixunil/security_writings/blob/master/cookie_files.md).

When `rpccookiefile` is set, it takes precedence over `rpcuser` and
`rpcpassword`:

- if the cookie file exists, it must contain `user:password` (the password may
  contain `:`); otherwise `satop` reports `Cookie file … in unexpected format`
- if it does not exist and there is nothing to fall back to, `satop` reports
  `Cookie file … not found`. The node may simply not be running yet: the cookie
  is read again at every connection attempt, so a node restart (which writes a
  new cookie) does not require restarting `satop`.

Without a cookie file, both `rpcuser` and `rpcpassword` are required in the configuration file.
There is no default cookie path: set `rpccookiefile`
explicitly. Credentials for `rpcuser`/`rpcpassword` can be generated with
Bitcoin Core utility [rpcauth.py](https://github.com/bitcoin/bitcoin/blob/master/share/rpcauth/README.md).

## Configuration file .satoprc

The file uses the INI format: one `key = value` per line, with the keys listed in
the table above. Lines starting with `#` or `;` are comments.

```ini
# Bitcoin node address and port
rpcconnect = 127.0.0.1
rpcport = 8332
rpcclienttimeout = 5000

# Preferred: the cookie written by the node, absolute path
rpccookiefile = /home/bitcoin/.bitcoin/.cookie

# Alternatively, user and password
#rpcuser = <username>
#rpcpassword = <password>

intervalsys = 3000
intervalrpc = 5000

# all time values is in milliseconds
```

### Location

By default the file is `$HOME/.satoprc`. Use `--conf` or `CONF` to pick another
one; a relative path is resolved against `$HOME`:

```sh
satop --conf .satoprc-testnet           # $HOME/.satoprc-testnet
CONF=/etc/satoprc satop                 # absolute path
```

### Generating it

```sh
satop --confgen
```

writes a commented example to the configuration file path (`$HOME/.satoprc`, or
the path given by `--conf`/`CONF`) with permissions `0600`, since it may hold a
password. If the file already exists, `satop` asks before overwriting it; only
the answer `y` overwrites.

## Examples

```sh
# Node on another host, testnet port
satop --rpcconnect 192.168.1.10 --rpcport 18332

# Same, through environment variables
RPCCONNECT=192.168.1.10 RPCPORT=18332 satop

# Cookie authentication with a faster refresh
satop --rpccookiefile /home/bitcoin/.bitcoin/.cookie --intervalsys 1000

# Check which values are in effect
satop --verbose
```

## Bitcoin Core RPC Configuration

`satop` talks to the node over its JSON-RPC interface. Settings go in the
node's `bitcoin.conf` (in its data directory, usually `~/.bitcoin/`); restart
the node after changing it.

### Enable the RPC server

`bitcoind` serves RPC by default. Bitcoin Core with the graphical interface
(`bitcoin-qt`) needs:

```ini
server=1
```

The RPC port depends on the network: 8332 (mainnet), 18332 (testnet3), 48332
(testnet4), 38332 (signet), 18443 (regtest). Pass it to `satop` with
`--rpcport` when it is not 8332.

### Cookie: let satop read it

With no `rpcuser`/`rpcpassword`/`rpcauth` in `bitcoin.conf`, the node writes
the cookie file at every start, readable only by the user running the node. If
`satop` runs as another user, let that user's group read it (Bitcoin Core 28 or
later):

```ini
rpccookieperms=group
```

then add the `satop` user to the node's group, for example
`sudo usermod -aG bitcoin "$USER"` (log in again), and point `rpccookiefile` at
the cookie, such as `/home/bitcoin/.bitcoin/.cookie`.

### A dedicated user and password

Alternatively, create RPC credentials with `rpcauth.py` from the Bitcoin Core
sources: it prints the `rpcauth=` line for `bitcoin.conf` and the password to
give `satop`:

```sh
python3 share/rpcauth/rpcauth.py satop
```

```ini
rpcauth=satop:<salt>$<hash>
```

```ini
# ~/.satoprc
rpcuser = satop
rpcpassword = <password printed by rpcauth.py>
```

`rpcauth` keeps only a hash in `bitcoin.conf`; avoid the plain
`rpcuser`/`rpcpassword` settings on the node side.

### Read-only access

`satop` only reads. These are all the methods it calls, so a dedicated user can
be limited to them:

```ini
rpcwhitelist=satop:getblockchaininfo,getnetworkinfo,uptime,getpeerinfo,getmempoolinfo,estimatesmartfee,getblockstats
rpcwhitelistdefault=0
```

Once any `rpcwhitelist` is set, users without one (the cookie included) are
allowed nothing; `rpcwhitelistdefault=0` keeps their full access.

### Node on another machine

By default the node accepts RPC only from `127.0.0.1`, which is also the
`satop` default. The simplest setup is to run `satop` on the node itself. To
reach it from another machine of your local network, allow that address and
listen on an interface it can reach:

```ini
rpcbind=127.0.0.1
rpcbind=192.168.1.10
rpcallowip=192.168.1.0/24
```

```sh
satop --rpcconnect=192.168.1.10
```

RPC traffic is unencrypted, credentials included: never expose the port to the
internet, and prefer a read-only user as above.
