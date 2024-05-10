
const {colors, virtualBytes, humanBlocks} = require('../utils')
    , {mempoolinfo, estimatefees} = require('./rpc');

const _ = require('lodash');

function Fees(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  const maxFees = 4;

  var self = this;
  var updater = async function() {

    const {mempoolinfo: {mempoolminfee}} = await mempoolinfo()
        , fees = await estimatefees()
        , [uno, tre, sei] = fees
        , minFee = Math.min(2 * mempoolminfee, sei.feerate);

    self.updateData([
      {block: -1, fee: virtualBytes(minFee)},
      {block: sei.blocks, fee: virtualBytes(sei.feerate)},
      {block: tre.blocks, fee: virtualBytes(tre.feerate)},
      {block: uno.blocks, fee: virtualBytes(uno.feerate)},
    ])
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Fees.prototype.updateData = function(fees) {

  const titles = []
      , data = [];

  fees.forEach(({block, fee}) => {

    const title = block > -1 ? humanBlocks(block, true): 'Slow';

    titles.push(title);
    data.push({
      label: ` ${fee}`,
      value: 50
    });
  })

  this.bar.setLabel(`Transaction Fees (sat/vB)`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Fees;
