
const {processes} = require('systeminformation')
  , {colors, humanBytes} = require('../utils');

function Proc(table, opts) {
  this.table = table;
  this.pSort = 'cpu';

  var self = this;
  var updater = function() {
    processes(data => {
      self.updateData(data);
    });
  };
  updater();
  this.interval = setInterval(updater, opts.sysinterval);
}

Proc.prototype.updateData = function(data) {
  var par = this.pSort;

  var headers = ['%CPU', 'Command'];

  var data = data.list
    .sort(function(a, b) {
      return b[par] - a[par];
    })
    .map(p => {
      return [
        p.cpu.toFixed(1),
        p.command
      ];
    });

  this.table.setData({
    headers,
    data
  });

  this.table.screen.render();
};

module.exports = Proc;
