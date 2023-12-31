
const {colors, truncate, humanSecs, humanSubver} = require('../utils')
    , {peerinfo} = require('./rpc');

function Peers(table, opts) {
  this.opts = opts;
  this.table = table;

  var self = this;
  var updater = async function() {
    const pi = await peerinfo();
    //https://developer.bitcoin.org/reference/rpc/getpeerinfo.html
    self.updateData({...pi})
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Peers.prototype.updateData = function(peerData) {

  //TODO change with screen witdh
  const addrCol = this.table.options.columnWidth.at(-1);

  const headers = [
    'UserAgent',
    //'Network',
    'Address',
  ];
  const data = [];

  if (peerData.peerinfo) {

    this.table.setLabel(`Peers (${peerData.peerinfo.length} connected)`);

    peerData.peerinfo
    .sort(function(a, b) {
      //may be sort by last data exchanged
      let ma = Math.max(a.lastsend, a.lastrecv)
        , mb = Math.max(b.lastsend, b.lastrecv)
      return mb - ma;
    })
    .forEach(p => {
      data.push([
        humanSubver(p.subver),
        //p.network,
        truncate(p.addr, addrCol),
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
