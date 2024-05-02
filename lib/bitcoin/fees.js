
const {colors, virtualBytes} = require('../utils')
    , {estimatefees} = require('./rpc');

const _ =require('lodash');

function Fees(bar, opts) {
  this.opts = opts;
  this.bar = bar;

  const maxFees = 4;

  var self = this;
  var updater = async function() {

    const [uno, tre, sei] = await estimatefees();
console.log(uno)
    self.updateData([
      {block: uno.blocks, fee: virtualBytes(uno.feerate)},
      {block: tre.blocks, fee: virtualBytes(tre.feerate)},
      {block: sei.blocks, fee: virtualBytes(sei.feerate)},
    ])
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Fees.prototype.updateData = function(fees) {

  const titles = []
      , data = [];

  fees.forEach(f => {
console.log(f,'---')
    const label = `${f.fee} sat/vB`;

    titles.push(`${f.block} block`);
    data.push({
      label
    });
  })

  this.bar.setLabel(`Transaction Fees`);
  this.bar.setData({
    titles,
    data
  });
};

module.exports = Fees;
