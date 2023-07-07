
const fs = require('fs')
    , { resolve } = require('path')
    , { homedir } = require('os')
    , { parseArgs } = require('node:util')
    , ini = require('ini');

const { options, printHelp } = require('./utils')
    , pkg = require('../package.json')
    , satop = require('./satop');

module.exports = env => {

  const defaultOpts = {};

  for (let key in options) {
    defaultOpts[key] = options[key].default;

    if (key !== 'conf') {
      //work around to allow override of fileOpts by cliOpts
      delete options[key].default;
    }
  }

  const initOpts = Object.assign({}, defaultOpts)
      , cliOpts = parseArgs({options, strict: false}).values
      , { conf, version, help } = cliOpts;

  if (conf) {
    const fileConf = resolve(homedir(), conf);

    if(fs.existsSync(fileConf)) {
      const fileOpts = ini.parse(fs.readFileSync(fileConf, 'utf8'));
      Object.assign(initOpts, fileOpts);
    }
  }

  //TODO override fileOpts by envOpts

  Object.assign(initOpts, cliOpts);

  if (version) {
    printHelp(pkg)
  }
  else if (help) {
    printHelp(pkg, options);
  }
  else {
    satop.init(initOpts);
  }
}