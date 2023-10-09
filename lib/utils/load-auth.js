/**
 * load rpc auth parameters from private files
 */
const fs = require('fs');

module.exports = async opts => {
  let {
      rpcconnect,
      rpcclienttimeout,
      rpcport,
      rpccookiefile,
      rpcpassword,
      rpcuser,
  } = opts;

  if (fs.existsSync(rpccookiefile)) {

      try {

          [ rpcuser, rpcpassword ] = fs.readFileSync(rpccookiefile).toString().trim().split(':', 2);
      }
      catch(err) {
          console.error(err.message);
          process.exit(1)
      }

      //TODO
      /*fs.watchFile(rpccookiefile, (curr, prev) => {
          debugLog(`RPC auth cookie change detected; attempting reconnect...`);
          connect();
      });*/

      if (!rpcuser || !rpcpassword) {
          throw new Error(`Cookie file ${rpccookiefile} in unexpected format`);
      }
  }

  return {
    host: rpcconnect,
    port: Number(rpcport),
    username: `${rpcuser}`.trim(),
    password: `${rpcpassword}`.trim(),
    timeout: Number(rpcclienttimeout)
  };
}
