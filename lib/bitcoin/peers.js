
const {colors, humanSecs} = require('../utils')
    , {connect, getpeerinfo, getrpc} = require('./rpc');

function Peers(table, opts) {
  this.table = table;
  this.rpc = connect(opts);

  var self = this;
  var updater = async function() {
    //const pi = await getpeerinfo();
    const pi = await getrpc('getpeerinfo');
    //https://developer.bitcoin.org/reference/rpc/getpeerinfo.html
console.log(pi)
    self.updateData({...pi})
  };
  updater();
  this.interval = setInterval(updater, opts.rpcinterval);
}

Peers.prototype.updateData = function(data) {

  const headers = ['subver','network','addr'];
  const par = 'conntime';

//  var data = data.peers
//    .sort(function(a, b) {
//      return b[par] - a[par];
//    })

    var data = data.peerinfo?.map(p => {
      return [
        p.subver,
        p.network,
        p.addr
      ];
    });

  this.table.setData({
    headers,
    data
  });

  this.table.screen.render();
};

module.exports = Peers;
