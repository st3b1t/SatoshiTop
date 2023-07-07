
const {fsSize} = require('systeminformation')
    , {colors, humanBytes} = require('../utils');

function Disk(donut, opts) {
  this.donut = donut;

  fsSize(data => {
    this.updateData(data);
  });

  this.interval = setInterval(() => {
    fsSize(data => {
      this.updateData(data);
    });
  }, opts.intervalsys*5);
}

Disk.prototype.updateData = function(data) {
  const {use, used, size} = data[0]
      , percent = use / 100;

  var label =
    humanBytes(used, true) +
    ' of ' +
    humanBytes(size, true);

  this.donut.setData([
    {
      label,
      percent,
      color: colors[5],
    }
  ]);
  this.donut.screen.render();
};

module.exports = Disk;
