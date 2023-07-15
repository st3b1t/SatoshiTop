
//TODO make random data in each module follow the format:
// https://systeminformation.io

const rnd = Math.random;

let uptime = rnd()*1000000;

module.exports = {
    currentLoad: () => {
        //TODO make random data
    },
    mem: () => {
        //TODO make random data
    },
    fsSize,
    networkInterfaceDefault: () => {
        //TODO make random data
    },
    networkStats: () => {
        //TODO make random data
    },
    processes,
    cpuTemperature: () => {
        //TODO make random data
    },
}

function shuffleArray(arr) {
  // Start from the last element and swap
  // one by one. We don't need to run for
  // the first element that's why i > 0
  for (let i = arr.length - 1; i > 0; i--) {
    // pick a random index from 0 to i inclusive
    const j = Math.floor(Math.random() * (i + 1)); // at random index
    // Swap arr[i] with the element
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}

function processes(cb) {
    const pp = require('./sys-processes.json');
    pp.list.forEach(p => {
        p.cpu = rnd()*30;
        p.command = p.command.replace('e','i').replace(':','')
    })
    cb(pp)
}
function fsSize(cb) {
    cb([
  {
    fs: '/dev/nvme0n1p3',
    type: 'ext4',
    size: 398394933248,
    used: 370108530688,
    available: 7974023168,
    use: 97.89,
    mount: '/',
    rw: false
  },
  {
    fs: '/dev/nvme0n1p2',
    type: 'vfat',
    size: 824180736,
    used: 58933248,
    available: 765247488,
    use: 17.15,
    mount: '/boot/efi',
    rw: true
  },
  {
    fs: '/dev/dm-1',
    type: 'ext4',
    size: 97826279424,
    used: 89351196672,
    available: 3458433024,
    use: 26.27,
    mount: '/media/Blockchain',
    rw: false
  }
])
}
