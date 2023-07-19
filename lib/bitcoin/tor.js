
const {colors} = require('../utils')
    , {networkinfo} = require('./rpc');

function Tor(torBox, opts) {
  this.opts = opts;
  this.torBox = torBox;

  var self = this;
  var updater = async function() {
    const ni = await networkinfo();
    self.updateData({...ni})
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Tor.prototype.updateData = function(data) {
  let address = 'unknown'
    , port = 'unknown';

  if (data.networkinfo?.localaddresses?.length > 0) {
    ({address, port} = data.networkinfo.localaddresses[0]);
  }

  this.torBox.setContent(`${address}:${port}`);
  this.torBox.screen.render();
};

module.exports = Tor;
