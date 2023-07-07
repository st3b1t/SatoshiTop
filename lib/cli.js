
const fs = require('fs')
    , { resolve } = require('path')
    , { homedir } = require('os')
    , { parseArgs } = require('node:util')
    , ini = require('ini');

const pkg = require('../package.json')
    , { options, printHelp } = require('./utils')
    , satop = require('./satop');

module.exports = env => {

  //TODO parse envs

  const defaultOpts = {};

  for (let key in options) {
    defaultOpts[key] = options[key].default;

    if (key!=='conf') {
      //work around to allow override of fileOpts by cliOpts
      delete options[key].default;
    }
  }

  //TODO move in options.js
  defaultOpts.sysinterval = 3000;
  defaultOpts.rpcinterval = 5000;

  let initOpts = Object.assign({}, defaultOpts);

  const cliOpts = parseArgs({options}).values
      , { version, help, conf } = cliOpts;

  if (conf) {
    const fileConf = resolve(homedir(), conf);

    if(fs.existsSync(fileConf)) {
      const fileOpts = ini.parse(fs.readFileSync(fileConf, 'utf8'));
      Object.assign(initOpts, fileOpts);
    }
  }

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