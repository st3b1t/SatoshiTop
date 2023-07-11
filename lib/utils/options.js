
module.exports = {
  //like bitcoin-cli arguments
  conf: {
    type: 'string',
    env: 'CONF',
    default: '.satoprc',
    description: "Specify configuration file. Relative paths will be prefixed by $HOME location.",
  },
  rpcconnect: {
    type: 'string',
    env: 'RPCCONNECT',
    default: '127.0.0.1',
    description: "Send commands to node running on <ip>",
  },
/*  rpccookiefile: {
    type: 'string',
    env: 'RPCCOOKIEFILE',
    description: "Location of the auth cookie. Absolute path"
  },*/
  rpcpassword: {
    type: 'string',
    env: 'RPCPASSWORD',
    description: "Password for JSON-RPC connections"
  },
  rpcport: {
    type: 'string',
    env: 'RPCPORT',
    default: '8332',
    description: "Connect to JSON-RPC on <port>",
  },
  rpcuser: {
    type: 'string',
    env: 'RPCUSER',
    description: "Username for JSON-RPC connections"
  },
  intervalsys: {
    type: 'string',
    env : 'INTERVALSYS',
    default: 3000,
    description: "Delay interval for system pick data, value in ms"
  },
  intervalrpc: {
    type: 'string',
    env : 'INTERVALRPC',
    default: 5000,
    description: "Delay interval for rpc pick data, value in ms"
  },
  //TODO rpctimeout connection
  verbose: {
    short: 'v',
    type: 'boolean',
    env: 'VERBOSE',
    default: false,
    description: 'verbose mode on'
  },
  help: {
    short: '?',
    type: 'boolean',
    default: false,
    description: 'print help'
  },
  version: {
    short: 'V',
    type: 'boolean',
    default: false,
    description: 'print version'
  }
};