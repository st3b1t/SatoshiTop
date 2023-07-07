
const contrib = require('blessed-contrib')
    , {donut, lcd, line, sparkline, table} = contrib
    , blessed = require('blessed')
    , {box, text} = blessed;

const bitcoin = require('./bitcoin')
    , system = require('./system');

module.exports = {
  init: async opts => {

    const screen = blessed.screen({
      title: 'SatoshiTop',
      smartCSR: true,
      //fastCSR: true
    });

    const grid = new contrib.grid({
      screen,
      rows: 12,
      cols: 12,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const cpu = grid.set(0, 0, 3, 6, line, {
      label: 'CPU',
      showNthLabel: 5,
      maxY: 100,
      showLegend: false,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const net = grid.set(3, 0, 3, 6, sparkline, {
      label: 'Network',
      tags: true,
      style: {
        fg: 'blue',
      },
    });

    //row, col, rowSpan, colSpan, obj, opts
    const mem = grid.set(0, 6, 3, 3, donut, {
      label: 'Memory',
      radius: 10,
      arcWidth: 6,
      yPadding: 1,
      remainColor: 'black',
    });

    //row, col, rowSpan, colSpan, obj, opts
    const disk = grid.set(3, 6, 3, 3, donut, {
      label: 'Disk',
      radius: 10,
      arcWidth: 6,
      yPadding: -1,
      remainColor: 'black',
    });

    //row, col, rowSpan, colSpan, obj, opts
    const temp = grid.set(0, 9, 2, 3, lcd, {
      label: 'Temperature',
      elements: 4,
      display: '00.0C',
      elementPadding: 0,
      color: 'red'
    });

    //row, col, rowSpan, colSpan, obj, opts
    const procs = grid.set(2, 9, 4, 3, table, {
      label: 'Processes',
      keys: true,
      interactive: false,
      columnSpacing: 2,
      columnWidth: [4, 20],
    });

    //row, col, rowSpan, colSpan, obj, opts
    const tor = grid.set(6, 0, 1, 12, box, {
      label: 'Tor Network',
      tags: true,
      style: {
        fg: 'blue'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const node = grid.set(7, 0, 2, 3, box, {
      label: 'Bitcoin Node',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const fees = grid.set(9, 0, 3, 3, box, {
      label: 'Fees',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const mempool = grid.set(7, 3, 5, 4, box, {
      label: 'Mempool',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const peers = grid.set(7, 7, 5, 5, table, {
      label: 'Peers',
      keys: true,
      interactive: false,
      columnSpacing: 0,
      columnWidth: [18, 30],
    });

    screen.render();

    screen.on('resize', e => {
      //system
      cpu.emit('attach');
      net.emit('attach');
      mem.emit('attach');
      temp.emit('attach');
      disk.emit('attach');
      procs.emit('attach');

      //bitcoin
      tor.emit('attach');
      node.emit('attach');
      fees.emit('attach');
      mempool.emit('attach');
      peers.emit('attach');
    });

    screen.key(['escape', 'q', 'C-c'], (ch, key) => {
      return process.exit(0);
    });

    //system
    new system.cpu(cpu, opts);
    new system.net(net, opts);
    new system.mem(mem, opts);
    new system.disk(disk, opts);
    new system.proc(procs, opts);
    new system.temp(temp, opts);

    //bitcoin
    await bitcoin.connect(opts);
    new bitcoin.tor(tor, opts);
    new bitcoin.nodeinfo(node, opts);
    new bitcoin.fees(fees, opts);
    new bitcoin.mempool(mempool, opts);
    new bitcoin.peers(peers, opts);
  }
};
