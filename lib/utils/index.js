
const human = require('./human');
const blocks = require('./blocks');
const fees = require('./fees');
const options = require('./options');
const loadAuth = require('./load-auth');
const rpcClient = require('./rpc-client');
const printHelp = require('./print-help');
const printHelpRpc = require('./print-help-rpc');
const confGen = require('./conf-gen');
const confParser = require('./conf-parser');

const colors = [
  'magenta', 'cyan', 'blue', 'yellow', 'green', 'red'
];

function truncate(str, n) {
  return (str.length > n) ? str.slice(0, n-3) + '...' : str;
}

module.exports = {
  colors,
  options,
  loadAuth,
  truncate,
  rpcClient,
  printHelp,
  printHelpRpc,
  confParser,
  confGen,
  ...human,
  ...blocks,
  ...fees,
}
