# Satoshi Top

Bitcoin full node monitoring dashboard for terminal

Some parts is WORK IN PROGRESS...

![system](docs/satoshitop.gif)

## Features
- general system resources: cpu, mem, disk, network, processes, temperature
- bitcoin core specific: status, blocks, uptime, tor, peers... (WORK IN PROGRESS)

System base monitor is inspired by [gtop](https://github.com/aksakalli/gtop) and many other features Bitcoin specific are work in progress, many more [Blessed widgets](https://github.com/chjj/blessed#widgets) and [contrib](https://github.com/yaronn/blessed-contrib#widgets) will be able to be integrated.
This project has been organized in small readable and easy to edit files to encourage contributions from the active community of developers and bitcoiners, every Pull Request and suggestion is welcome.


### Requirements

* Linux / OSX / Windows (partial support)
* Node.js >= v18

### Installation

By official [NPM package](https://npmjs.com/package/satop)

```sh
$ npm install satop -g
```
Copy config file `dot.satoprc.example` in path `$HOME/.satoprc` and add bitcoin `rpcuser` `rpcpassword`.
Don't forget to set restrictive read permissions for this file, for example: `chmod 0600 ~/.satoprc`)

*Snap and Docker* setup is work in progress...

### Usage

Start Satoshi Top with the `satop` command in system terminal(the same system whose resources you want to monitor).

```sh
$ satop
```

If Bitcoin daemon listening in different address(`127.0.0.1`) or a different port(`8333`), you can use command line parameters:

```sh
$ satop --rpcconnect=192.168.0.3 --rpcport=18332
```

To stop satop use `q`, or `ctrl+c` in most shell environments.

Learn more about [command line parameters](docs/cli.md)...

```sh
$ satop --help
```

### Run in Docker

You need to assign host `net` and `pid` to access the metrics in the host machine.

```sh
$ docker build . -f ./docker/Dockerfile -t st3b1t/satoshitop
$ docker run --rm -it --name satoshitop --net="host" --pid="host" st3b1t/satoshitop
```

or Docker Compose
```sh
cd docker
docker compose up
```


## Run via Docker

1. `docker build . -f ./docker/Dockerfile -t fbbe`
2. `docker run -it -p 3000:3000 -e BITCOIND_ADDR=172.17.0.1:8332 `

### Development

```sh
$ npm run dev
```

This script Set environment var `FAKEMODE` to allow bypass the real RPC connection.
Additional parameter `--intervalrpc=1000` allows to speed up data polling and `nodemon` to reload the code at each edits.


## License

Copyright (c) 2023 [st3b1t](https://github.com/st3b1t) [@st3b1t](https://twitter.com/st3b1t)

Released under [the MIT license](LICENSE).
