
const {rpcClient} = require('../utils');

const {fake_blockchaininfo, fake_networkinfo} = require('../../test/bitcoin_fakes');

const getrpc = async (cmd, params) => {
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

        rpcClient.connect({
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
    uptime: async opts => {
        return {
            uptime: await getrpc('uptime')
        }
    },
    getblockchaininfo: async opts => {
        return {
            //blockchaininfo: fake_blockchaininfo()
            blockchaininfo: await getrpc('getblockchaininfo')
        }
    },
    getnetworkinfo: async opts => {
       return {
         //networkinfo: fake_networkinfo()
         networkinfo: await getrpc('getnetworkinfo')
       }
    }
}