
const {mem} = require('systeminformation')
  , {colors, humanBytes} = require('../utils');

function Mem(donut, opts) {
  this.donut = donut;

  mem(data => {
    this.memData = [
      {
        title: 'Memory',
        style: {
          line: colors[0],
        },
        x: Array(61)
          .fill()
          .map((_, i) => 60 - i),
        y: Array(61).fill(0),
      }
    ];
    this.updateData(data);
    this.interval = setInterval(() => {
      mem(data => {
        this.updateData(data);
      });
    }, opts.sysinterval/3);
  });
}

Mem.prototype.updateData = function(data) {
  const {available, total} = data
      , percent = (100 * (1 - available / total)).toFixed() / 100
      , used = total - available;

  var label = `${humanBytes(used)} of ${humanBytes(total)}`;

  this.donut.setData([
    {
      label,
      percent,
      color: colors[0],
    },
  ]);
};

module.exports = Mem;
