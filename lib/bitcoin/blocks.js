
const {colors, humanSecs, humanMinAgo, humanBytes} = require('../utils')
    , {blockchaininfo, lastblocks} = require('./rpc');

const _ =require('lodash');

function Blocks(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  const maxBlocks = 4;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    const lasts = await lastblocks(bi.blockchaininfo.blocks, maxBlocks);
    self.updateData(lasts.lastblocks)
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Blocks.prototype.updateData = function(blocks) {

  const titles = []
      , data = [];

  blocks.forEach(b => {
    const height = `${b.height}`
        , time = humanMinAgo(b.time, true) //minutes ago
        , value = b.total_size

    const label = [
      humanBytes(value),
      ' ',
      `${time}`,
      //`${b.txs}tx`,  //transactions counts
    ].join('\n');

    titles.push(height);
    data.push({
      label,
      value
    });
  })

  this.bar.setLabel(`Blocks (last ${data.length} confirmed)`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Blocks;
