

const {loadAuth, rpcClient} = require('../utils');

const {getFake} = require('../../test/rpc-fake');

const connect = async opts => {

    if (process.env.FAKERPC) return;

    const optsAuth = await loadAuth(opts);

    //TODO reconnect on close

    await rpcClient.connect(optsAuth);

    try{
        await rpcClient.request('ping',[])
    }
    catch(e) {
        let message;
        if(e.code===401) {
            message = 'Incorrect rpcuser or rpcpassword'
        }
        else {
            message = e.code
        }
        return {error:{message}}
    }
    return {connected: true};
};

const get = async (cmd, params = []) => {
    //TODO caching

    if (process.env.FAKERPC) {
        return getFake(cmd, params);
    }

    return await rpcClient.request(cmd, params);
};

module.exports = {
    get,
    connect,
    uptime: async () => ({uptime: await get('uptime') }),
    peerinfo: async () => ({peerinfo: await get('getpeerinfo') }),
    networkinfo: async () => ({networkinfo: await get('getnetworkinfo') }),
    //http://127.0.0.1:8332/rest/chaininfo.json
    blockchaininfo: async () => ({blockchaininfo: await get('getblockchaininfo') }),
    //http://127.0.0.1:8332/rest/mempool/info.json
    //mempool
    mempoolinfo: async () => ({mempoolinfo: await get('getmempoolinfo') }),
    estimatefees: async () => ([
        //TODO parallelize
        await get('estimatesmartfee', [1]),
        await get('estimatesmartfee', [3]),
        await get('estimatesmartfee', [6])
    ]),
    status: async blkinfo => {
        let status = '';

        if (blkinfo) {
            const {headers, blocks, initialblockdownload, verificationprogress} = blkinfo;

            if (initialblockdownload || headers > blocks) {
                status = `{red-fg}Syncing ${Math.floor(verificationprogress*100)}%{/red-fg}`
            }
            else {
                status = '{green-fg}Synched{/green-fg}'
            }
        }
        return {status}
    },
    lastblocks: async (height, nth = 4) => {
        const lastblocks = [];
        for (let h = height; h > (height-nth); h--) {
            lastblocks.push( await get('getblockstats', [h]) );
        }
        return {lastblocks};
    }
}