const blessed = require('blessed')
    , contrib = require('blessed-contrib');

const system = require('./system')
    , bitcoin = require('./bitcoin');

module.exports = {
  init: opts => {

    const screen = blessed.screen();

    const grid = new contrib.grid({
      rows: 12,
      cols: 12,
      screen,
    });

    //grid.set(row, col, rowSpan, colSpan, obj, opts)

    const cpuLine = grid.set(0, 0, 3, 6, contrib.line, {
      label: 'CPU',
      showNthLabel: 5,
      maxY: 100,
      showLegend: false,
    });

    const netSpark = grid.set(3, 0, 3, 6, contrib.sparkline, {
      label: 'Network',
      tags: true,
      style: {
        fg: 'blue',
      },
    });

    const memDonut = grid.set(0, 6, 3, 3, contrib.donut, {
      label: 'Memory',
      radius: 10,
      arcWidth: 6,
      yPadding: 1,
      remainColor: 'black',
    });

    const diskDonut = grid.set(3, 6, 3, 3, contrib.donut, {
      label: 'Disk',
      radius: 10,
      arcWidth: 6,
      yPadding: -1,
      remainColor: 'black',
    });

    const tempLcd = grid.set(0, 9, 2, 3, contrib.lcd, {
      label: 'Temperature',
      elements: 4,
      display: '00.0C',
      elementPadding: 0,
      color: 'red',
    });

    const procTable = grid.set(2, 9, 4, 3, contrib.table, {
      label: 'Processes',
      keys: true,
      columnSpacing: 2,
      columnWidth: [4, 20],
    });

    //bitcoin node info

    //grid.set(row, col, rowSpan, colSpan, obj, opts)

    const nodeInfo = grid.set(6, 0, 2, 4, blessed.box, {
      label: 'Bitcoin Node Info',
      tags: true,
      style: {
        fg: 'green'
      }
    });

    const torInfo = grid.set(11, 0, 1, 12, blessed.box, {
      label: 'Tor Address',
      tags: true,
      style: {
        fg: 'violet'
      }
    });

    screen.render();

    screen.on('resize', function(a) {
      cpuLine.emit('attach');
      tempLcd.emit('attach');
      memDonut.emit('attach');
      netSpark.emit('attach');
      diskDonut.emit('attach');
      procTable.emit('attach');
      //bitcoin node
      nodeInfo.emit('attach');
      torInfo.emit('attach');
    });

    screen.key(['escape', 'q', 'C-c'], function(ch, key) {
      return process.exit(0);
    });

    new system.cpu(cpuLine);
    new system.net(netSpark);
    new system.mem(memDonut);
    new system.disk(diskDonut);
    new system.proc(procTable);
    new system.temp(tempLcd);
    //bitcoin node
    new bitcoin.nodeinfo(nodeInfo, torInfo, opts);
  }
};
