
const {colors, humanSecs, humanBytes} = require('../utils')
    , {blockchaininfo, lastblocks} = require('./rpc');

const _ =require('lodash');

function Blocks(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    const lasts = await lastblocks(bi.blockchaininfo.blocks, 4);
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
        , size = b.total_size

    titles.push(height);
    data.push(size);
  })

  this.bar.setLabel(`Blocks (last ${data.length} confirmed)`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Blocks;
