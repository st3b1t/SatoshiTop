
const jayson = require('jayson/promise');

//TODO https://github.com/ruimarinho/bitcoin-core

//https://github.com/janoside/btc-rpc-explorer/blob/master/app.js#L878

let client;

module.exports = {
  connect: async conf => {
    const usernamePassword = `${conf.username}:${conf.password}`
    const rpcOpts = {
      host: conf.host,
      port: conf.port,
      timeout: conf.timeout,
      headers: {
        "Authorization": `Basic ${btoa(usernamePassword)}`
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