# Satoshi Top

Bitcoin full node monitoring dashboard for terminal

![system](docs/satop.gif)

### Requirements

* Linux / OSX / Windows (partial support)
* Node.js >= v8

### Installation

```sh
$ npm install satop -g
```
Copy config file `dot.satoprc.example` in path `$HOME/.satoprc`

### Usage

Start Satoshi Top with the `satop` command

```sh
$ satop
```

To stop satop use `q`, or `ctrl+c` in most shell environments.

Learn more...

```sh
$ satop --help
```

### Troubleshooting

If you see question marks or other different characters, try to run it with these environment variables:

```sh
$ LANG=en_US.utf8 TERM=xterm-256color satop
```

## License

Copyright (c) 2023, st3b1t

Released under [the MIT license](LICENSE).
