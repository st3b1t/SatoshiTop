
//const blessed = require('blessed')

const {cpuTemperature} = require('./sys')
    , {colors, humanBytes} = require('../utils');

//TODO usb box const {box, text} = require('blessed')

function Temp(lcd, opts) {
  this.opts = opts;
  this.lcd = lcd;
  this.max = 0;
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
  const {main, max} = data;
  this.max = Math.max(this.max, max);
  //TODO se color with value
  this.lcd.setDisplay(`${main}C`);
  this.lcd.setLabel(`Temp (max ${this.max}C)`);
  //TODO this.bar.setProgress(main);
};

module.exports = Temp;
