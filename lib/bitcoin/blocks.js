
const {colors, humanSecs, humanMins, humanMinAgo, humanBytes} = require('../utils')
    , {blockchaininfo, lastblocks} = require('./rpc');

function Blocks(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  const maxBlocks = 5;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    const lasts = await lastblocks(bi.blockchaininfo.blocks, maxBlocks);
    self.updateData(lasts.lastblocks, bi.blockchaininfo)
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Blocks.prototype.updateData = function(blocks, binfo) {

  const titles = []
      , data = [];

  let prevB;

  blocks
  //.filter(b=>b)
  .reverse()
  .forEach(b => {
    const height = `${b.height}`
        , btime = prevB ? (b.time - prevB.time) : 0
        , value = b.total_size
        , htime = humanMins(btime);

    const label = [
      humanBytes(value),
      `${b.avgfeerate}SvB`,
      //b.time,
      htime,
      //`${b.txs}tx`,  //transactions counts
    ].join('\n');

    if(prevB) { //salta il primo
      titles.push(height);
      data.push({
        label,
        value
      });
    }
    prevB = b;
  });

  const avgTime = humanMins(binfo.mediantime);
  const blocksDisk = humanBytes(binfo.size_on_disk)
  this.bar.setLabel(`Blocks (tot ${blocksDisk})`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Blocks;
