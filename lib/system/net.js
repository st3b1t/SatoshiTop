
const {networkInterfaceDefault, networkStats} = require('./sys')
    , {colors, humanBytes} = require('../utils');

function Net(sparkline, opts) {
  this.opts = opts;
  this.sparkline = sparkline;
  this.netData = [Array(61).fill(0), Array(61).fill(0)];

  networkInterfaceDefault(iface => {
    var self = this;
    var updater = function() {
      networkStats(iface, data => {
        self.updateData(data[0]);
      });
    };
    updater();
    this.interval = setInterval(updater, opts.intervalsys/3);
  });
}

Net.prototype.updateData = function(data) {
  const {rx_sec, tx_sec, rx_bytes, tx_bytes} = data
      , rxSec = Math.max(0, rx_sec)
      , txSec = Math.max(0, tx_sec);

  const rx_label = `In: ${humanBytes(rxSec)}/s Tot: ${humanBytes(rx_bytes)}`
      , tx_label = `Out: ${humanBytes(txSec)}/s Tot: ${humanBytes(tx_bytes)}`;

  this.netData[0].shift();
  this.netData[0].push(rxSec);

  this.netData[1].shift();
  this.netData[1].push(txSec);

  this.sparkline.setData([rx_label, tx_label], this.netData);
  this.sparkline.screen.render();
};

module.exports = Net;
