
const {colors, humanSecs} = require('../utils')
    , {connect, status, uptime, getblockchaininfo, getnetworkinfo} = require('./rpc');

function Nodeinfo(nodeBox, torBox, opts) {
  this.nodeBox = nodeBox;
  this.torBox = torBox;
  this.rpc = connect(opts);

  var self = this;
  var updater = async function() {
    const up = await uptime();
    const ni = await getnetworkinfo();
    const bi = await getblockchaininfo();
    const st = await status(bi);
    self.updateData({...st, ...up, ...ni, ...bi})
  };
  updater();
  this.interval = setInterval(updater, opts.bitcoinInterval);
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
        data.networkinfo.subversion.replaceAll('/',''),
        data.networkinfo.protocolversion
      ];

  const nodeText = labels.map((label,i) => {
    return `${label}: {red-fg}${vals[i]}{/red-fg}`;
  });

  const {address, port} = data.networkinfo.localaddresses[0]
      , torText = `${address}:${port}`;

  this.nodeBox.setContent(nodeText.join('\n'));
  this.nodeBox.screen.render();

  this.torBox.setContent(torText);
  this.torBox.screen.render();

};

module.exports = Nodeinfo;
