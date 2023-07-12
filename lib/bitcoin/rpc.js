
const fs = require('fs');

const {rpcClient} = require('../utils');

const {getrpcFake} = require('../../test/rpc-fake');

const loadAuth = async opts => {
    //TODO move in file
    let {
        rpcconnect,
        rpcport,
        rpccookiefile,
        rpcpassword,
        rpcuser,
    } = opts;

    let authType = 'usernamePassword';

    if (fs.existsSync(rpccookiefile)) {
        authType = 'cookie';
    }

    if (authType == 'cookie') {
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
      //authType,
      username: `${rpcuser}`.trim(),
      password: `${rpcpassword}`.trim(),
      timeout: 1000 //TODO Number(rpctimeout),
    };
};

const connect = async opts => {

    if (process.env.FAKEMODE) return;

    const optsAuth = await loadAuth(opts);

    return await rpcClient.connect(optsAuth);
};

const getrpc = async (cmd, params = {}) => {
    //TODO caching

    if (process.env.FAKEMODE) {
        return getrpcFake(cmd, params);
    }

    return await rpcClient.request(cmd, params);
};


module.exports = {
    connect,
    getrpc,
    status: async blkinfo => {
        let status = '';

        if (blkinfo) {
            const {headers, blocks, initialblockdownload, verificationprogress} = blkinfo;

            if (initialblockdownload || headers > blocks) {
                status = `{red-fg}Block download ${Math.floor(verificationprogress*100)}%{/red-fg}`
            }
            else {
                status = '{green-fg}Synchronized{/green-fg}'
            }
        }
        return {status}
    },
    uptime: async opts => ({uptime: await getrpc('uptime') }),
    peerinfo: async opts => ({peerinfo: await getrpc('getpeerinfo') }),
    networkinfo: async opts => ({networkinfo: await getrpc('getnetworkinfo') }),
    blockchaininfo: async opts => ({blockchaininfo: await getrpc('getblockchaininfo') }),
}