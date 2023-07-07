
const {colors, humanSecs, truncate} = require('../utils')
    , {getpeerinfo, getrpc} = require('./rpc');

function Peers(table, opts) {
  this.table = table;

  var self = this;
  var updater = async function() {
    const pi = await getpeerinfo();
    //https://developer.bitcoin.org/reference/rpc/getpeerinfo.html
    self.updateData({...pi})
  };
  updater();
  this.interval = setInterval(updater, opts.rpcinterval);
}

Peers.prototype.updateData = function(peerData) {

  const headers = [
      'subver',
      'network',
      'addr'
  ];
  const par = 'conntime';

//console.log('getpeerinfo............',data..length)


//  var data = data.peers
//    .sort(function(a, b) {
//      return b[par] - a[par];
//    })

  var data = [];

  if (peerData.peerinfo) {
    data.push([`${peerData.peerinfo.length} peers connected`])
    peerData.peerinfo.forEach(p => {
      data.push([
        p.subver,
        p.network,
        truncate(p.addr, 20)
      ]);
    });
  }

  this.table.setData({
    headers,
    data
  });

  this.table.screen.render();
};

module.exports = Peers;
