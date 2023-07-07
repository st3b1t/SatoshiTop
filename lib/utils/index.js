
const human = require('./human');
const options = require('./options');
const rpcClient = require('./rpc-client');
const printHelp = require('./print-help');

const colors = [
  'magenta', 'cyan', 'blue', 'yellow', 'green', 'red'
];

function truncate(str, n) {
  return (str.length > n) ? str.slice(0, n-3) + '...' : str;
}

module.exports = {
  colors,
  options,
  truncate,
  rpcClient,
  printHelp,
  ...human
}
