
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
  rpccookiefile: {
    type: 'string',
    env: 'RPCCOOKIEFILE',
    description: "Location of the auth cookie. Absolute path"
  },
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
  /**
  * TODO
  * timeout connection
  * delay interval polling data
  * sysInterval = 3000;
  * rpcInterval = 5000;
  */
  verbose: {
    short: 'v',
    type: 'boolean',
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