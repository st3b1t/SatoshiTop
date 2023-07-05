
const {rpcClient} = require('../utils');

//DEBUG const {fake_blockchaininfo, fake_networkinfo} = require('../test/bitcoin_fake');

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
    status: async opts => {
        return {
            //blockchaininfo: fake_blockchaininfo()
               /* +summaryItem("Status")
        if (getblockchaininfo.initialblockdownload || getblockchaininfo.headers > getblockchaininfo.blocks)
            span Initial block download progress #{(100 * getblockchaininfo.verificationprogress).toLocaleString()}%
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
        //getRpcData("getblockchaininfo")
        return {
            blockchaininfo: fake_blockchaininfo()
        }
    },
    getnetworkinfo: async opts => {
        //getRpcData("");
       return {
         networkinfo: await getrpc('getnetworkinfo')//fake_networkinfo()
       }
    }
}