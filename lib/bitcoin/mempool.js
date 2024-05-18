
const {colors, humanBytes} = require('../utils')
    , {mempoolinfo} = require('./rpc');

function Mempool(mempoolBox, opts) {
  this.opts = opts;
  this.mempoolBox = mempoolBox;

  var self = this;
  var updater = async function() {
    const mem = await mempoolinfo();
    self.updateData(mem.mempoolinfo)
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Mempool.prototype.updateData = function(mempool) {

  //console.log(mempool)

  const labels = [
      ]
      , vals = [
      ];

  const nodeText = labels.map((label,i) => {
    let val = vals[i] ? `{red-fg}${vals[i]}{/red-fg}`: `{white-fg}unknown{/white-fg}`;
    return `${label}: ${val}`;
  });

  nodeText.push('\n{red-fg}[work in progress]{/red-fg}\n')


  const memSize = humanBytes(mempool.bytes)
  const totSize = humanBytes(mempool.maxmempool)
  this.mempoolBox.setLabel(`Mempool ( ${memSize} / ${totSize} )`);
  this.mempoolBox.setContent(nodeText.join('\n'));
  this.mempoolBox.screen.render();
};

module.exports = Mempool;
