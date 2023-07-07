
const {connect} = require('./rpc');

module.exports = {
  connect,
  log: require('./log'),
  tor: require('./tor'),
  nodeinfo: require('./nodeinfo'),
  mempool: require('./mempool'),
  peers: require('./peers'),
};
