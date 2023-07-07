
const contrib = require('blessed-contrib')
    , {donut, lcd, line, sparkline, table} = contrib
    , blessed = require('blessed')
    , {box} = blessed;

const bitcoin = require('./bitcoin')
    , system = require('./system');

module.exports = {
  init: async opts => {

    const screen = blessed.screen({
      title: 'SatoshiTop',
      fastCSR: true
    });

    const grid = new contrib.grid({
      rows: 10,
      cols: 12,
      screen,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const cpuLine = grid.set(0, 0, 3, 6, line, {
      label: 'CPU',
      showNthLabel: 5,
      maxY: 100,
      showLegend: false,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const netSpark = grid.set(3, 0, 3, 6, sparkline, {
      label: 'Network',
      tags: true,
      style: {
        fg: 'blue',
      },
    });

    //row, col, rowSpan, colSpan, obj, opts
    const memDonut = grid.set(0, 6, 3, 3, donut, {
      label: 'Memory',
      radius: 10,
      arcWidth: 6,
      yPadding: 1,
      remainColor: 'black',
    });

    //row, col, rowSpan, colSpan, obj, opts
    const diskDonut = grid.set(3, 6, 3, 3, donut, {
      label: 'Disk',
      radius: 10,
      arcWidth: 6,
      yPadding: -1,
      remainColor: 'black',
    });

    //row, col, rowSpan, colSpan, obj, opts
    const tempLcd = grid.set(0, 9, 2, 3, lcd, {
      label: 'Temperature',
      elements: 4,
      display: '00.0C',
      elementPadding: 0,
      color: 'red'
    });

    //row, col, rowSpan, colSpan, obj, opts
    const procTable = grid.set(2, 9, 4, 3, table, {
      label: 'Processes',
      keys: true,
      interactive: false,
      columnSpacing: 2,
      columnWidth: [4, 20],
    });

    //row, col, rowSpan, colSpan, obj, opts
    const torInfo = grid.set(6, 0, 1, 12, box, {
      label: 'Tor Network',
      tags: true,
      style: {
        fg: 'blue'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const nodeInfo = grid.set(7, 0, 3, 3, box, {
      label: 'Bitcoin Node',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const mempool = grid.set(7, 3, 3, 3, box, {
      label: 'Mempool',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const peersTable = grid.set(7, 6, 3, 6, table, {
      label: 'Peers',
      keys: true,
      interactive: false,
      columnSpacing: 0,
      columnWidth: [20, 30],
    });

    screen.render();

    screen.on('resize', e => {
      cpuLine.emit('attach');
      tempLcd.emit('attach');
      memDonut.emit('attach');
      netSpark.emit('attach');
      diskDonut.emit('attach');
      procTable.emit('attach');
      //bitcoin node
      torInfo.emit('attach');
      nodeInfo.emit('attach');
      mempool.emit('attach');
      peersTable.emit('attach');
    });

    screen.key(['escape', 'q', 'C-c'], (ch, key) => {
      return process.exit(0);
    });

    new system.cpu(cpuLine, opts);
    new system.net(netSpark, opts);
    new system.mem(memDonut, opts);
    new system.disk(diskDonut, opts);
    new system.proc(procTable, opts);
    new system.temp(tempLcd, opts);
    //bitcoin node

    await bitcoin.connect(opts);

    new bitcoin.tor(torInfo, opts);
    new bitcoin.nodeinfo(nodeInfo, opts);
    new bitcoin.mempool(mempool, opts);
    new bitcoin.peers(peersTable, opts);
  }
};
