
const {fsSize} = require('systeminformation')
    , {colors, humanBytes} = require('../utils');

function Disk(donut, opts) {
  this.opts = opts;
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

Disk.prototype.updateData = function(disks) {

  const data = [];

  disks
  .filter(disk => {
    //hide boot partition
    return !disk.mount.includes('/boot')
  })
  .forEach((disk,i) => {
    const {fs, mount, use, used, size} = disk
        , percent = use / 100
        , dev = fs.replace('/dev/','')
        , color = colors[i];
    //const usage = ''//[used, size].map(humanBytes).join('/');
    data.push({
      label: mount,
      percent,
      color,
    })
  });

  this.donut.setData(data);
  this.donut.screen.render();
};

module.exports = Disk;
