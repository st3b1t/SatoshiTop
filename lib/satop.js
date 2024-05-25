
const contrib = require('blessed-contrib')
    , {donut, lcd, sparkline} = contrib
    , blessed = require('blessed')
    , {box, text} = blessed;

const {block, line, table} = require('./widget') //customized Blessed widgets
    , bitcoin = require('./bitcoin')
    , system = require('./system');


module.exports = {
  init: async opts => {

    let cpu = 1,
        net = 1,
        ram = 1,
        disk = 1,
        procs = 1,
        temp = 1,
        nodeinfo = 1,
        fees = 1,
        mempool = 1,
        blocks = 1,
        peers = 1,
        tor = 1;

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
    if(cpu)
    cpu = grid.set(0, 0, 3, 5, line, {
      label: 'Cpu',
      showNthLabel: 5,
      maxY: 100,
      showLegend: false,
      yPadding: 0
    });

    if(net)
    net = grid.set(3, 0, 3.2, 5, sparkline, {
      label: 'Network',
      tags: true,
      yPadding: 0,
      style: {
        fg: 'blue',
      },
    });

    if(ram)
    ram = grid.set(0, 5, 4, 2, donut, {
      label: 'Ram',
      radius: 10,
      arcWidth: 6,
      yPadding: 0,
      remainColor: 'black',
    });

    if(disk)
    disk = grid.set(0, 7, 4, 5, donut, {
      label: 'Disks',
      radius: 10,
      arcWidth: 6,
      yPadding: 0,
      remainColor: 'black',
      showLegend: true,
    });

    if(procs)
    procs = grid.set(4, 8, 4.2, 4, table, {
      label: 'Processes',
      keys: true,
      interactive: false,
      columnSpacing: 2,
      columnWidth: [4, 4, 6, 20],
    });

    if(temp)
    temp = grid.set(4, 5, 2.2, 3, lcd, {
      label: 'Temperature',
      elements: 3,
      display: '00.0C',
      elementPadding: 0,
      color: 'red'
    });

    if(nodeinfo)
    nodeinfo = grid.set(6, 0, 2, 3, box, {
      label: 'Bitcoin Node',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    if(fees)
    fees = grid.set(6, 3, 2, 5, block, {
      label: 'Fees',
      tags: true,
      style: {
        fg: 'yellow'
      },
      barBgColor: 'yellow',
      barFgColor: 'black',
      labelColor: 'yellow',
      showText: true,
      barWidth: 8,
      barSpacing: 3,
      xOffset: 0,
      maxHeight: 100
    });

    if(mempool)
    mempool = grid.set(8, 0, 3, 4, box, {
      label: 'Mempool',
      tags: true,
      style: {
        fg: 'yellow'
      }
    });

    if(blocks)
    blocks = grid.set(8, 4, 3, 4, block, {
      label: 'Blocks',
      tags: true,
      style: {
        fg: 'yellow'
      },
      barFgColor: 'white',
      labelColor: 'blue',
      showText: true,
      barWidth: 7,
      barSpacing: 3,
      xOffset: 0,
      maxHeight: 2097152,
    });

    if(peers)
    peers = grid.set(8, 8, 3, 4, table, {
      label: 'Peers',
      keys: true,
      interactive: false,
      columnSpacing: 0,
      columnWidth: [18, 30],
    });

    if(tor)
    tor = grid.set(11, 0, 1, 12, box, {
      label: 'Tor Network',
      tags: true,
      style: {
        fg: 'blue'
      }
    });

    const attach = e => {
      //system
      if(cpu) cpu.emit('attach');
      if(net) net.emit('attach');
      if(ram) ram.emit('attach');
      if(temp) temp.emit('attach');
      if(disk) disk.emit('attach');
      if(procs) procs.emit('attach');

      //bitcoin
      if(tor) tor.emit('attach');
      if(nodeinfo) nodeinfo.emit('attach');
      if(fees) fees.emit('attach');
      if(blocks) blocks.emit('attach');
      if(peers) peers.emit('attach');
      if(mempool) mempool.emit('attach');
    }

    screen.on('resize', attach);
    //not work screen.on('render', attach);
    //https://github.com/chjj/blessed?tab=readme-ov-file#events-1

    screen.render();

    screen.key(['escape', 'q', 'C-c'], (ch, key) => {
      return process.exit(0);
    });

    //system
    new system.cpu(cpu, opts);
    new system.net(net, opts);
    new system.ram(ram, opts);
    new system.disk(disk, opts);
    new system.proc(procs, opts);
    new system.temp(temp, opts);

    //bitcoin
    const {error, connected} = await bitcoin.rpc.connect(opts);
    if(connected) {
      if(nodeinfo) new bitcoin.nodeinfo(nodeinfo, opts);
      if(fees) new bitcoin.fees(fees, opts);
      if(blocks) new bitcoin.blocks(blocks, opts);
      if(mempool) new bitcoin.mempool(mempool, opts);
      if(peers) new bitcoin.peers(peers, opts);
      if(tor) new bitcoin.tor(tor, opts);
    }
    else if(error) {
      nodeinfo.setContent(`{red-fg}Connection error: ${opts.rpcconnect}:${opts.rpcport} ${error.message}{/red-fg}`)
    }
  }
};
