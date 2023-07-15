

const {loadAuth, rpcClient} = require('../utils');

const {getrpcFake} = require('../../test/rpc-fake');

const connect = async opts => {

    if (process.env.FAKEMODE) return;

    const optsAuth = await loadAuth(opts);

    //TODO reconnect on close

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
                status = `{red-fg}Syncing ${Math.floor(verificationprogress*100)}%{/red-fg}`
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

    //http://127.0.0.1:8332/rest/chaininfo.json
    blockchaininfo: async opts => ({blockchaininfo: await getrpc('getblockchaininfo') }),

    //http://127.0.0.1:8332/rest/mempool/info.json
    //mempool

}