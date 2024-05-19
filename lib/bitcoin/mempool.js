
const {colors, humanBytes} = require('../utils')
    , {mempoolinfo} = require('./rpc');

function Mempool(line, opts) {
  this.opts = opts;
  this.line = line;

  var self = this;
  var updater = async function() {
    const mem = await mempoolinfo();
    this.memData = {
        style: {
          line: 'red'//colors[i % colors.length],
        },
        x: Array(61)
          .fill()
          .map((_, i) => 60 - i),
        y: Array(61).fill(0),
      }
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc/2);
}

Mempool.prototype.updateData = function(mempool) {
  const memSize = humanBytes(mempool.bytes)
  const totSize = humanBytes(mempool.maxmempool)
  this.line.setLabel(`Mempool ( ${memSize} / ${totSize} ) ${mempool.size}txs`);
  //this.line.setContent(nodeText.join('\n'));
  this.line.setData(this.memData);
  this.line.screen.render();
};

module.exports = Mempool;
