# SatoshiTop

Bitcoin full node monitoring dashboard for terminal.

*(Some parts is WORK IN PROGRESS... 🚧)*

Proposals and reporting problems: [New issue](https://github.com/st3b1t/SatoshiTop/issues/new) or contribute by voting with a 👍 in [proposals issues](https://github.com/st3b1t/SatoshiTop/labels/Proposal)

Follow me for last updates: [@st3b1t](https://twitter.com/st3b1t) or full list of [Versions](https://github.com/st3b1t/SatoshiTop/releases)

![system](docs/satoshitop.gif)

Sober and useful evolution of the senseless [SatoshiBanner](https://github.com/st3b1t/SatoshiBanner)

## Use case
You can define it an *htop for Bitcoin*... having a modern dashboard that displays a lot of informations similar
to a modern bitcoin explorer, but keeping very poor requirements, it doesn't need a graphical environment (output is ncurses)
and the data exchange is minimal suitable for slow connections like `Tor` and monitors/lcds with low resolution and low fps.

## Features
- general system resources: cpu, mem, disk, network, processes, temperature
- bitcoin core specific: status, blocks, uptime, tor, peers... (WORK IN PROGRESS)

**Other possibilities:**

- http rest interface: this would allow viewing through a browser, while maintaining a tiny, text-only data exchange.
- send output via tty to mini LCD with low resolution and poor in colors.

Many other features are work in progress, many more [Blessed widgets](https://github.com/chjj/blessed#widgets) and [contrib](https://github.com/yaronn/blessed-contrib#widgets) will be able to be integrated.
This project has been organized in small readable and easy to edit files to encourage contributions from the active community of developers and bitcoiners, every Pull Request and suggestion is welcome.

### Requirements

* Linux / OSX / Windows (maybe partial support)
* Node.js >= v18 and NPM

## Quick Start

Install last stable version in global from [official npm package](https://npmjs.com/package/satop)
*(Installing packages from NPM repositories is always a quick but unsafe way if you want to keep your system free of malicious packages or dependencies)*

```sh
$ npm install satop -g
$ satop --rpccookiefile=/home/<user>/.bitcoin/.cookie
```
The path `/home/<user>/.bitcoin/.cookie` depends from Bitcoin [datadir config](https://github.com/bitcoin/bitcoin/blob/master/doc/init.md#configuration)

Or use username and password
```sh
$  satop --rpcuser=<username> --rpcpassword=<password>
```
(*All commands entered are stored in the bash history file. But we don't want the password to be stored where anyone can find it. For this, put a space in front of the command shown above.*)

If `rpccookiefile` has valid value(file exists and readble) auth method bypass any values of `rpcuser/rpcpassword` and it use only the cookie file.

## Source Code Installation

Get source code:
```sh
$ git clone https://github.com/st3b1t/SatoshiTop.git
$ cd ./SatoshiTop
```
Now you have the development version, `master` branch.

Choice a stable release from https://github.com/st3b1t/SatoshiTop/releases.
Or select latest automagically:
```sh
$ git checkout $(git describe --tags `git rev-list --tags --max-count=1`)
```

Copy the example config file [dot.satoprc.example](./dot.satoprc.example) in your home path:
```sh
$ cp ./dot.satoprc.example ~/.satoprc
$ chmod 0600 ~/.satoprc
```
Now uncomment and set `rpccookiefile` with path od Bitcoin cookie file or set `rpcuser` and `rpcpassword`.
(*Don't forget to set restrictive read permissions for this file in case of storing rpcuser and rpcpassword*)

Install dependencies and `satop` command in the global way and try it:
```sh
$ npm install
$ npm install -g .
$ satop --version
$ satop --help
```

[Docker](./docs/docker.md) and [Snap](./docs/snap.md) setup is work in progress...

## Usage

Start SatoshiTop with the `satop` command in system terminal(the same system whose resources you want to monitor).

```sh
$ satop
```

If Bitcoin daemon listening in different address(`127.0.0.1`) or a different port(`8333`), you can use command line parameters:

```sh
$ satop --rpcconnect=192.168.0.3 --rpcport=18332
```

To stop satop use `q`, or `ctrl+c` in most shell environments.

Learn more about command line [parameters](docs/cli.md)...


### Development

```sh
$ npm run dev
```

This script Set environment var `FAKEMODE=true` to allow simulate a RPC connection(doesn't need Bitcoin Core).

Additional parameter `--intervalrpc=1000` allows to speed up data polling and `nodemon` to reload the code at each edits.

### Contributing

Pull Requests are welcome, please make sure that your changes are tested

In order of priority how you can help out:

1. share this project in your social channels and click stars!
2. testing and report bugs in [new issue](https://github.com/st3b1t/SatoshiTop/issues/new)
3. try to resolve [easy issues](https://github.com/st3b1t/SatoshiTop/labels/good%20first%20issue))
4. search `//TODO` in the source code
5. add new features


## License

Copyright (c) 2023 [st3b1t](https://github.com/st3b1t)

Released under [MIT license](LICENSE).
