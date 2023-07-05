const si = require('systeminformation')
  , {colors, humanBytes} = require('../utils');

function Temp(lcd) {
  this.lcd = lcd;

  si.cpuTemperature(data => {
    this.updateData(data);
    this.interval = setInterval(() => {
      si.cpuTemperature(data => {
        this.updateData(data);
      });
    }, 1000);
  });
}

Temp.prototype.updateData = function(data) {
  const {main} = data;
  this.lcd.setDisplay(`${main}C`);
};

module.exports = Temp;
