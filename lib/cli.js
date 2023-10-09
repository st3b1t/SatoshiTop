
const pkg = require('../package.json')
    , { options, printHelp, confGen, confParser } = require('./utils')
    , satop = require('./satop');

module.exports = env => {

  const {version, help, confgen, initOpts} = confParser(env);

  if (initOpts.verbose) {
    process.stderr.write(JSON.stringify({initOpts},null,4));
  }

  if (version) {
    printHelp(pkg)
  }
  else if (help) {
    printHelp(pkg, options);
  }
  else if (confgen) {
    confGen(options);
  }
  else {
    satop.init(initOpts);
  }
}