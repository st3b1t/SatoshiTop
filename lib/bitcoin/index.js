
const {connect} = require('./rpc');

module.exports = {
  connect,
  fees: require('./fees'),
  log: require('./log'),
  mempool: require('./mempool'),
  nodeinfo: require('./nodeinfo'),
  peers: require('./peers'),
  tor: require('./tor'),
};
