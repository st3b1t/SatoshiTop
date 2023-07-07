
const {colors, humanSecs} = require('../utils')
    , {status, uptime, blockchaininfo, networkinfo} = require('./rpc');

function Nodeinfo(nodeBox, opts) {
  this.nodeBox = nodeBox;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    const ni = await networkinfo();
    const up = await uptime();
    const st = await status(bi);
    self.updateData({...st, ...up, ...ni, ...bi})
  };
  updater();
  this.interval = setInterval(updater, opts.rpcinterval);
}

Nodeinfo.prototype.updateData = function(data) {
  const labels = [
        'Status',
        'UpTime',
        'Version',
        'ProtocolVersion'
      ]
      , vals = [
        data.status,
        humanSecs(data.uptime),
        data.networkinfo.subversion,
        data.networkinfo.protocolversion
      ];

  const nodeText = labels.map((label,i) => {
    let val = vals[i] ? `{red-fg}${vals[i]}{/red-fg}`: `{white-fg}unknown{/white-fg}`;
    return `${label}: ${val}`;
  });

  this.nodeBox.setContent(nodeText.join('\n'));
  this.nodeBox.screen.render();
};

module.exports = Nodeinfo;
