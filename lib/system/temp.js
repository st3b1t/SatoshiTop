
//const blessed = require('blessed')

const {cpuTemperature} = require('systeminformation')
    , {colors, humanBytes} = require('../utils');

function Temp(lcd, opts) {
  this.opts = opts;
  this.lcd = lcd;

//TODO progressbar bottom lcd
  /*this.bar = blessed.progressbar({
    border: 'line',
    ch: ':',
    //orientation: 'horizontal',
    //height: 10,
    //width: 3,
    width: '50%',
    height: 3,
    right: 0,
    bottom: 0,
    filled: 50
  });

  this.lcd.screen.append(this.bar);*/

  cpuTemperature(data => {
    this.updateData(data);
    this.interval = setInterval(() => {
      cpuTemperature(data => {
        this.updateData(data);
      });
    }, opts.intervalsys);
  });
}

Temp.prototype.updateData = function(data) {
  const {main} = data;
  this.lcd.setDisplay(`${main}C`);
  //this.bar.setProgress(main);
};

module.exports = Temp;
