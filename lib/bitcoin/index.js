
const {connect} = require('./rpc');

module.exports = {
  connect,
  log: require('./log'),
  tor: require('./tor'),
  fees: require('./fees'),
  nodeinfo: require('./nodeinfo'),
  mempool: require('./mempool'),
  peers: require('./peers'),
};
