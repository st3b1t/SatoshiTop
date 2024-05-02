
const {colors, virtualBytes} = require('../utils')
    , {mempoolinfo, estimatefees} = require('./rpc');

const _ =require('lodash');

function Fees(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  const maxFees = 4;

  var self = this;
  var updater = async function() {

    const {mempoolinfo: {mempoolminfee}} = await mempoolinfo();
    const [uno, tre, sei] = await estimatefees();

    const minFee = Math.min(2 * mempoolminfee, sei.feerate);

    self.updateData([
      {block: 0, fee: virtualBytes(minFee)},
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

  fees.forEach(f => {

    const label = `${f.fee}\n`+`sat/vB`;

    titles.push(`${f.block} block`);
    data.push({
      label,
      value: 100
    });
  })

  this.bar.setLabel(`Transaction Fees`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Fees;
