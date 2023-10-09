
const pkg = require('../package.json')
    , { options, printHelpRpc, confParser, loadAuth, rpcClient } = require('./utils')
    , satop = require('./satop');

const  { parseArgs } = require('node:util')

module.exports = async env => {

  const {version, verbose, help, confgen, initOpts, cliParams} = confParser(env);

  const [command, ...params] = cliParams;

  if (verbose) {
    process.stderr.write(JSON.stringify({initOpts})+'\n\n');
  }

  if (version) {
    printHelpRpc(pkg)
  }
  else if (help) {
    printHelpRpc(pkg, options);
  }
  else if (command) {
    const optsAuth = await loadAuth(initOpts);
    await rpcClient.connect(optsAuth);

    try {
      const res = await rpcClient.request(command, params);
      console.log(res)
    }
    catch(e) {
      const {error} = JSON.parse(e.message);
      console.log('error:', error.message)
    }
  }
  else {
    printHelpRpc(pkg, options);
  }
}