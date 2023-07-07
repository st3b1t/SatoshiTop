
const jayson = require('jayson/promise');

//TODO https://github.com/ruimarinho/bitcoin-core

//https://github.com/janoside/btc-rpc-explorer/blob/master/app.js#L878

let client;

module.exports = {
  connected: async () => (Bool(client)),
  connect: async opts => {

    const {host, port, username, password} = opts;

    //don't reconnect
    if (client) return client;

    const userpass = `${username}:${password}`
    const rpcOpts = {
      host,
      port,
      timeout: opts.timeout,
      headers: {
        "Authorization": `Basic ${btoa(userpass)}`
      }
    };

    client = await jayson.client.http(rpcOpts);

    return client;
  },
  request: async (cmd, params) => {

    if (!client) return;

    const resp = await client.request(cmd, params)
        , {result, error} = resp;

    if(error) {
      throw error
    }
    return result;
  }
}