
const {mem} = require('./sys')
    , {colors, humanBytes} = require('../utils');

function Mem(donut, opts) {
  this.opts = opts;
  this.donut = donut;

  mem(data => {
    this.memData = [
      {
        title: 'Ram',
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
    }, opts.intervalsys/3);
  });
}

Mem.prototype.updateData = function(data) {
  const {available, total} = data
      , percent = (100 * (1 - available / total)).toFixed() / 100
      , used = total - available;

  const label = [used, total].map(humanBytes).join('/');

  //this.donut.setLabel(`Ram ${label}`);
  this.donut.setData([
    {
      label,
      percent,
      color: colors[3],
    },
  ]);
};

module.exports = Mem;
