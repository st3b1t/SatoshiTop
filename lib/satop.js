
const contrib = require('blessed-contrib')
    , {donut, lcd, line, sparkline, table} = contrib
    , blessed = require('blessed')
    , {box, text} = blessed;

const {block} = require('./widget') //customized Blessed widgets
    , bitcoin = require('./bitcoin')
    , system = require('./system');


module.exports = {
  init: async opts => {

//DEBUG
/*const si = require('systeminformation');
const out = await si.processes()
console.log(JSON.stringify(out, null,4))
return
*/

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
    const cpu = grid.set(0, 0, 3, 4, line, {
      label: 'Cpu',
      showNthLabel: 5,
      maxY: 100,
      showLegend: false,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const net = grid.set(3, 0, 3, 4, sparkline, {
      label: 'Network',
      tags: true,
      style: {
        fg: 'blue',
      },
    });

    //row, col, rowSpan, colSpan, obj, opts
    const mem = grid.set(0, 4, 4, 3, donut, {
      label: 'Ram',
      radius: 10,
      arcWidth: 6,
      yPadding: 0,
      remainColor: 'black',
    });

    //row, col, rowSpan, colSpan, obj, opts
    const disk = grid.set(0, 7, 4, 5, donut, {
      label: 'Disks',
      radius: 10,
      arcWidth: 6,
      yPadding: 0,
      remainColor: 'black',
      showLegend: true,
    });

    //row, col, rowSpan, colSpan, obj, opts
    const procs = grid.set(4, 7, 4, 5, table, {
      label: 'Processes',
      keys: true,
      interactive: false,
      columnSpacing: 2,
      columnWidth: [4, 4, 6, 20],
    });

    //row, col, rowSpan, colSpan, obj, opts
    const temp = grid.set(4, 4, 2, 3, lcd, {
      label: 'Temperature',
      elements: 3,
      display: '00.0C',
      elementPadding: 0,
      color: 'red'
    });

    //row, col, rowSpan, colSpan, obj, opts
    const node = grid.set(6, 0, 2, 3, box, {
      label: 'Bitcoin Node',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });
/*
    //row, col, rowSpan, colSpan, obj, opts
    const fees = grid.set(6, 3, 2, 4, box, {
      label: 'Fees',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });*/
    const fees = grid.set(6, 3, 2, 4, block, {
      label: 'Fees',
      tags: true,
      style: {
        fg: 'yellow'
      },
      barBgColor: 'yellow',
      barFgColor: 'black',
      labelColor: 'yellow',
      showText: true,
      barWidth: 7,
      barSpacing: 3,
      xOffset: 0,
      maxHeight: 100
    });

    //row, col, rowSpan, colSpan, obj, opts
    const mempool = grid.set(8, 0, 3, 3, box, {
      label: 'Mempool',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    //row, col, rowSpan, colSpan, obj, opts
    const blocks = grid.set(8, 3, 3, 4, block, {
      label: 'Blocks',
      tags: true,
      style: {
        fg: 'yellow'
      },
      barFgColor: 'white',
      labelColor: 'blue',
      showText: true,
      barWidth: 5,
      barSpacing: 3,
      xOffset: 0,
      maxHeight: 2097152,//2MB block size
    });

    //row, col, rowSpan, colSpan, obj, opts
    const peers = grid.set(8, 7, 3, 5, table, {
      label: 'Peers',
      keys: true,
      interactive: false,
      columnSpacing: 0,
      columnWidth: [18, 30],
    });

    //row, col, rowSpan, colSpan, obj, opts
    const tor = grid.set(11, 0, 1, 12, box, {
      label: 'Tor Network',
      tags: true,
      style: {
        fg: 'blue'
      }
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
      blocks.emit('attach');
      peers.emit('attach');
      mempool.emit('attach');
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
    await bitcoin.rpc.connect(opts);
    new bitcoin.nodeinfo(node, opts);
    new bitcoin.fees(fees, opts);
    new bitcoin.blocks(blocks, opts);
    new bitcoin.mempool(mempool, opts);
    new bitcoin.peers(peers, opts);
    new bitcoin.tor(tor, opts);
  }
};
