
const {connect} = require('./rpc');

module.exports = {
  connect,
  log: require('./log'),
  nodeinfo: require('./nodeinfo'),
  mempool: require('./mempool'),
  peers: require('./peers'),
};
