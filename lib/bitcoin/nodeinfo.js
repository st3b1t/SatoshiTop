
const {colors, humanSecs} = require('../utils')
    , {connect, status, uptime, getnetworkinfo} = require('./rpc');

function Nodeinfo(box, opts) {
  this.box = box;
  this.rpc = connect(opts);

  var self = this;
  var updater = async function() {

    const st = await status();
    const up = await uptime();
    const ni = await getnetworkinfo();
    self.updateData({...st, ...up, ...ni})
  };
  updater();
  this.interval = setInterval(updater, 5000);
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

  const text = labels.map((label,i) => {
    return `${label}: {red-fg}${vals[i]}{/red-fg}`;
  });

  if (this.box) {
    this.box.setContent(text.join('\n'));
    this.box.screen.render();
  }
};

module.exports = Nodeinfo;
