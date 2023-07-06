
const human = require('./human');
const options = require('./options');
const rpcClient = require('./rpc-client');
const printHelp = require('./print-help');

const colors = [
  'magenta', 'cyan', 'blue', 'yellow', 'green', 'red'
];

module.exports = {
  colors,
  options,
  rpcClient,
  printHelp,
  ...human
}
