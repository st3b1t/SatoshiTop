
const {processes} = require('./sys')
    , {colors, humanBytes} = require('../utils');

function Proc(table, opts) {
  this.opts = opts;
  this.table = table;

  var self = this;
  var updater = function() {
    processes(data => {
      self.updateData(data);
    });
  };
  updater();
  this.interval = setInterval(updater, opts.intervalsys);
}

Proc.prototype.updateData = function(data) {
  const sortby = 'cpu';
  const headers = ['%CPU', '%Ram', 'User', 'Command'];

  var data = data.list
    .sort(function(a, b) {
      return b[sortby] - a[sortby];
    })
    .map(p => {
      return [
        p.cpu.toFixed(1),
        p.mem.toFixed(1),
        p.user,
        p.command,
        //TODO show also path is screen is enought large (p.path, p.command),
      ];
    });

  this.table.setData({
    headers,
    data
  });

  this.table.screen.render();
};

module.exports = Proc;
