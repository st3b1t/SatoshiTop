SatoshiTop
=====

> Bitcoin full node monitoring dashboard for terminal

## SYNOPSIS

`satop`


## DESCRIPTION

SatoshiTop is a system monitoring dashboard for Bitcoin full node in terminal much like htop, top, gtop etc...

## COMMAND-LINE OPTIONS

  --conf
        Specify configuration file. Relative paths will be prefixed by $HOME location. (default: .satoprc)
        (env: CONF)

  --rpcconnect
        Send commands to node running on <ip> (default: 127.0.0.1)
        (env: RPCCONNECT)

  --rpccookiefile
        Location of the auth cookie. Absolute path
        (env: RPCCOOKIEFILE)

  --rpcpassword
        Password for JSON-RPC connections
        (env: RPCPASSWORD)

  --rpcport
        Connect to JSON-RPC on <port> (default: 8332)
        (env: RPCPORT)

  --rpcuser
        Username for JSON-RPC connections
        (env: RPCUSER)

  -v, --verbose
        verbose mode on

  -?, --help
        print help

  -V, --version
        print version

## Website

https://github.com/st3b1t/SatoshiTop

## BUGS

Please report any bugs to https://github.com/st3b1t/SatoshiTop.


## LICENSE

Copyright (c) 2023, st3b1t (MIT License).


## SEE ALSO

node.js(1)

