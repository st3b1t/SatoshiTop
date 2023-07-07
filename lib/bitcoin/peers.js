
const {colors, humanSecs, truncate} = require('../utils')
    , {peerinfo, getrpc} = require('./rpc');

function Peers(table, opts) {
  this.table = table;

  var self = this;
  var updater = async function() {
    const pi = await peerinfo();
    //https://developer.bitcoin.org/reference/rpc/getpeerinfo.html
    self.updateData({...pi})
  };
  updater();
  this.interval = setInterval(updater, opts.rpcinterval);
}

Peers.prototype.updateData = function(peerData) {

  const headers = [
      'Version',
      'Network',
      'Address'
  ];
  const par = 'conntime';

//console.log('getpeerinfo............',data..length)


//  var data = data.peers
//    .sort(function(a, b) {
//      return b[par] - a[par];
//    })

  var data = [];

  if (peerData.peerinfo) {

    this.table.setLabel(`Peers (${peerData.peerinfo.length} connected)`);

    //data.push([`${peerData.peerinfo.length} peers connected`])
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
