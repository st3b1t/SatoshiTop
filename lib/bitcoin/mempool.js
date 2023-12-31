
const {colors, humanSecs} = require('../utils')
    , {blockchaininfo} = require('./rpc');

function Mempool(mempoolBox, opts) {
  this.opts = opts;
  this.mempoolBox = mempoolBox;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    self.updateData({...bi})
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Mempool.prototype.updateData = function(data) {
  const labels = [
      ]
      , vals = [
      ];

  const nodeText = labels.map((label,i) => {
    let val = vals[i] ? `{red-fg}${vals[i]}{/red-fg}`: `{white-fg}unknown{/white-fg}`;
    return `${label}: ${val}`;
  });

  nodeText.push('\n{red-fg}[work in progress]{/red-fg}\n')

  this.mempoolBox.setContent(nodeText.join('\n'));
  this.mempoolBox.screen.render();
};

module.exports = Mempool;
