

const {loadAuth, rpcClient} = require('../utils');

const {getFake} = require('../../test/rpc-fake');

const connect = async opts => {

    if (process.env.FAKEMODE) return;

    const optsAuth = await loadAuth(opts);

    //TODO reconnect on close

    return await rpcClient.connect(optsAuth);
};

const get = async (cmd, params = {}) => {
    //TODO caching

    if (process.env.FAKEMODE) {
        return getFake(cmd, params);
    }

    return await rpcClient.request(cmd, params);
};

module.exports = {
    get,
    connect,
    status: async blkinfo => {
        let status = '';

        if (blkinfo) {
            const {headers, blocks, initialblockdownload, verificationprogress} = blkinfo;

            if (initialblockdownload || headers > blocks) {
                status = `{red-fg}Syncing ${Math.floor(verificationprogress*100)}%{/red-fg}`
            }
            else {
                status = '{green-fg}Synchronized{/green-fg}'
            }
        }
        return {status}
    },
    uptime: async opts => ({uptime: await get('uptime') }),
    peerinfo: async opts => ({peerinfo: await get('getpeerinfo') }),
    networkinfo: async opts => ({networkinfo: await get('getnetworkinfo') }),
    //http://127.0.0.1:8332/rest/chaininfo.json
    blockchaininfo: async opts => ({blockchaininfo: await get('getblockchaininfo') }),
    //http://127.0.0.1:8332/rest/mempool/info.json
    //mempool

}