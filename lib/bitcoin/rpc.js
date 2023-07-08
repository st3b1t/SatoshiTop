
const {rpcClient} = require('../utils');

const {getrpcFake} = require('../../test/rpc-fake');

const getrpc = async (cmd, params = {}) => {
    //TODO caching

    if (process.env.FAKEMODE) {
        return getrpcFake(cmd, params);
    }

    return await rpcClient.request(cmd, params);
};

module.exports = {
    getrpc,
    connect: async opts => {
        let {
            rpcconnect,
            //TODO rpccookiefile
            rpcpassword,
            rpcuser,
            rpcport,
        } = opts;

        if (process.env.FAKEMODE) return;

        return await rpcClient.connect({
            host: rpcconnect,
            port: Number(rpcport),
            username: rpcuser,
            password: rpcpassword,
            timeout: 0
        });
    },
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