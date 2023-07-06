/**
 * Copyright (c) 2023 st3b1t
 */
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
  }

  //TODO refact
  defaultOpts.sysInterval = 3000;
  defaultOpts.rpcInterval = 5000;

  let opts = Object.assign({}, defaultOpts);

  const args = parseArgs({options})
      , { values: cliOpts } = args
      , { version, help, conf } = cliOpts;

  if (conf) {
    const fileConf = resolve(homedir(), conf);

    if(fs.existsSync(fileConf)) {
      const fileOpts = ini.parse(fs.readFileSync(fileConf, 'utf8'));
      opts = Object.assign(opts, fileOpts);
    }
  }

  opts = Object.assign(opts, cliOpts);

  if (version) {
    printHelp(pkg)
  }
  else if (help) {
    printHelp(pkg, options);
  }
  else {
    satop.init(opts);
  }
}