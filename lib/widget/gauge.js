/**
 * customized version of blessed-contrib Bar
 * https://github.com/yaronn/blessed-contrib/blob/master/lib/widget/charts/bar.js
 * by st3b1t
 */
const {Node} = require('blessed')
    , {gauge} = require('blessed-contrib');


/*
function Gauge(options) {
  if (!(this instanceof Node)) {
    return new Gauge(options);
  }

  gauge.prototype.setPercent(25)

  this.on('attach', function() {
    if (self.options.data) {
      self.setData(self.options.data);
    }
  });
}*/

//Gauge.prototype = Object.create(canvas.prototype);

/*Gauge.prototype.calcSize = function() {
  this.canvasSize = {width: this.width-2, height: this.height};
};*/
/*
Gauge.prototype.setData = function(bar) {

  if (!this.ctx) {
    throw 'error: canvas context does not exist. setData() for bar charts must be called after the chart has been added to the screen via screen.append()';
  }

  this.clear();

  var c = this.ctx;

  //var max = Math.max.apply(Math, bar.data);
  const values = bar.data.map(d => {
    return typeof(d.value)==='number'? Number(d.value) : d;
  });
  var max = Math.max.apply(Math, values);
  max = Math.max(max, this.options.maxHeight);
  var x = this.options.xOffset;
  var barY = this.canvasSize.height - 5;

  for (var i = 0; i < bar.data.length; i++) {


    const value = typeof(bar.data[i].value)==='number' ? bar.data[i].value : Number(bar.data[i]);
    const label = typeof(bar.data[i].label)==='string' ? bar.data[i].label : bar.data[i].toString();

    var h = Math.round(barY * (value / max));

    if (value > 0) {
      c.strokeStyle = 'blue';
      if (this.options.barBgColor)
        c.strokeStyle = this.options.barBgColor;
      c.fillRect(x, barY - h + 1, this.options.barWidth, h);
    } else {
      c.strokeStyle = 'normal';
    }

    c.fillStyle = 'white';
    if (this.options.barFgColor)
      c.fillStyle = this.options.barFgColor;
    if (this.options.showText) {
      label.split('\n').reverse().forEach((l, i) => {
        //console.log(l,i)
        c.fillText(l, x, this.canvasSize.height - 4 - i);
      })


    }
    c.strokeStyle = 'normal';
    c.fillStyle = 'white';
    if (this.options.labelColor)
      c.fillStyle = this.options.labelColor;

    if (this.options.showText)
      c.fillText(bar.titles[i], x, this.canvasSize.height - 3);

    x += this.options.barSpacing;
  }
};

Gauge.prototype.getOptionsPrototype = function() {
  return  {  barWidth: 1
    ,  barSpacing: 1
    ,  xOffset: 1
    ,  maxHeight: 1
    ,  data: {
      titles: ['s'],
      data: [1]
    }
  };
};*/

//Gauge.prototype.type = 'bar';

module.exports = gauge;
