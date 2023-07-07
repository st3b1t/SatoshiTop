
const {connect} = require('./rpc');

module.exports = {
  connect,
  log: require('./log'),
  tor: require('./tor'),
  blockchain: require('./blockchain'),
  nodeinfo: require('./nodeinfo'),
  mempool: require('./mempool'),
  peers: require('./peers'),
};
