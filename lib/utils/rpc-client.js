
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

    client = await jayson.client.http(rpcOpts)

    //client = new jayson.Client.http(rpcOpts)
//TODO handle error connection
/*    client.catch(err=>{
console.log('CLIENT CATCH',err)
    })*/

//console.log('CLIENT',client)

    //TODO disconnect on process exit

    return client;
  },
  request: async (cmd, params) => {

    if (!client) return;

    const {error, result} = await client.request(cmd, params);

    if(error) {
      throw error
    }
    return result;
  }
}