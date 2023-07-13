
const {colors, humanSecs, humanSubver} = require('../utils')
    , {status, uptime, blockchaininfo, networkinfo} = require('./rpc');

function Nodeinfo(nodeBox, opts) {
  this.opts = opts;
  this.nodeBox = nodeBox;

  var self = this;
  var updater = async function() {
    const bi = await blockchaininfo();
    const ni = await networkinfo();
    const up = await uptime();
    const st = await status(bi.blockchaininfo);
    self.updateData({...st, ...up, ...ni, ...bi})
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Nodeinfo.prototype.updateData = function(data) {
  const labels = [
        'Status',
        'Blocks',
        'UpTime',
        'UserAgent',
        //'ProtocolVersion'
      ]
      , vals = [
        data.status,
        data.blockchaininfo.blocks,
        humanSecs(data.uptime),
        humanSubver(data.networkinfo.subversion),
        //data.networkinfo.protocolversion
      ];

  const nodeText = labels.map((label,i) => {
    let val = vals[i] ? `{green-fg}${vals[i]}{/green-fg}`: `{white-fg}unknown{/white-fg}`;
    return `${label}: ${val}`;
  });

  this.nodeBox.setContent(nodeText.join('\n'));
  this.nodeBox.screen.render();
};

module.exports = Nodeinfo;
