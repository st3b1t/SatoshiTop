/*
  average cpu loads

  ...work in progress...
*/
const {currentLoad} = require('systeminformation')
    , {colors} = require('../utils');

function Cpu(line, opts) {
  this.line = line;
  currentLoad(data => {
    //TODO convert in single
    this.cpuData = {
        title: 'CPU load',
        style: {
          line: colors[0],
        },
        x: Array(61)
          .fill()
          .map((_, i) => 60 - i),
        y: Array(61).fill(0),
      };

    this.updateData(data);
    this.interval = setInterval(() => {
      currentLoad(data => {
        this.updateData(data);
      });
    }, opts.intervalsys/3);
  });
}

Cpu.prototype.updateData = function(data) {

  //TODO convert in single

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
