
const {colors, humanSecs} = require('../utils')
    , {getblockchaininfo} = require('./rpc');

function Mempool(mempoolBox, opts) {
  this.mempoolBox = mempoolBox;

  var self = this;
  var updater = async function() {
    const bi = await getblockchaininfo();
    self.updateData({...bi})
  };
  updater();
  this.interval = setInterval(updater, opts.rpcinterval);
}

Mempool.prototype.updateData = function(data) {
  const labels = [
        'Blocks',
      ]
      , vals = [
        data.blockchaininfo?.blocks,
      ];

  const nodeText = labels.map((label,i) => {
    return `${label}: {red-fg}${vals[i]}{/red-fg}`;
  });

  nodeText.push('\n[work in progress]\n')

  this.mempoolBox.setContent(nodeText.join('\n'));
  this.mempoolBox.screen.render();
};

module.exports = Mempool;
