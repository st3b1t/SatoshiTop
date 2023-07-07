/*
multiple cores cpu loads
*/
const {currentLoad} = require('systeminformation')
    , {colors} = require('../utils');

function Cpu(line, opts) {
  this.line = line;
  currentLoad(data => {
    this.cpuData = data.cpus.map((cpu, i) => {
      return {
        title: 'CPU' + (i + 1),
        style: {
          line: colors[i % colors.length],
        },
        x: Array(61)
          .fill()
          .map((_, i) => 60 - i),
        y: Array(61).fill(0),
      }
    });
    this.updateData(data);
    this.interval = setInterval(() => {
      currentLoad(data => {
        this.updateData(data);
      });
    }, opts.sysinterval/3);
  });
}

Cpu.prototype.updateData = function(data) {
  data.cpus.forEach((cpu, i) => {
    var loadString = cpu.load.toFixed(1).toString();
    while (loadString.length < 6) {
      loadString = ' ' + loadString;
    }
    loadString = loadString + '%';

    this.cpuData[i].title = 'CPU' + (i + 1) + loadString;
    this.cpuData[i].y.shift();
    this.cpuData[i].y.push(cpu.load);
  });

  this.line.setData(this.cpuData);
  this.line.screen.render();
};

module.exports = Cpu;
