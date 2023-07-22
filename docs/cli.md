
## CommandLine Options

Some of the options can be set by the related environment variable `env: <NAMEVAR>`

  --conf
    Specify configuration file. Relative path will be prefixed by $HOME location (default: .satoprc)
    (env: CONF)

  --rpcconnect
    Send commands to node running on <ip> (default: 127.0.0.1)
    (env: RPCCONNECT)

  --rpcclienttimeout
    Timeout in seconds during HTTP requests, or 0 for no timeout (default: 5000)
    (env: RPCCLIENTTIMEOUT)

  --rpccookiefile
    Location of the auth cookie. Absolute path
    (env: RPCCOOKIEFILE)

  --rpcuser
    Username for JSON-RPC connections
    (env: RPCUSER)

  --rpcpassword
    Password for JSON-RPC connections
    (env: RPCPASSWORD)

  --rpcport
    Connect to JSON-RPC on <port> (default: 8332)
    (env: RPCPORT)

  --intervalsys
    Delay interval for system pick data, value in ms (default: 3000)
    (env: INTERVALSYS)

  --intervalrpc
    Delay interval for rpc pick data, value in ms (default: 5000)
    (env: INTERVALRPC)

  --confgen
    Generate new configuration file(.satoprc) in your own $HOME directory

  -v, --verbose
    verbose mode on
    (env: VERBOSE)

  -?, --help
    print help

  -V, --version
    print version


