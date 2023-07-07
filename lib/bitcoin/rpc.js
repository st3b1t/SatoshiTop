
const {rpcClient} = require('../utils');

const {getrpcFake} = require('../../test/rpc-fake');

const getrpc = async (cmd, params = {}) => {
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

        return await rpcClient.connect({
            host: rpcconnect,
            port: Number(rpcport),
            username: rpcuser,
            password: rpcpassword,
            timeout: 0
        });
    },
    status: async blkinfo => {
        return {
            //TODO from blockchaininfo
               /* +summaryItem("Status")
        if (blkinfo.initialblockdownload || blkinfo.headers > blkinfo.blocks)
            span Initial block download progress #{(100 * blkinfo.verificationprogress).toLocaleString()}%
        else
            span.text-success Synchronized*/
            status: 'Synchronized'
        }
    },
    uptime: async opts => ({uptime: await getrpc('uptime') }),
    peerinfo: async opts => ({peerinfo: await getrpc('getpeerinfo') }),
    networkinfo: async opts => ({networkinfo: await getrpc('getnetworkinfo') }),
    blockchaininfo: async opts => ({blockchaininfo: await getrpc('getblockchaininfo') }),
}