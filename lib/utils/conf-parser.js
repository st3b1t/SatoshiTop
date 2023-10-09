
const fs = require('fs')
    , { resolve } = require('path')
    , { homedir } = require('os')
    , { parseArgs } = require('node:util')
    , ini = require('ini');

const options = require('./options');

module.exports = env => {

  const defaultOpts = {}
      , envOpts = {}
      , opts = {};

  for (let key in options) {
    let KEY = key.toUpperCase();

    defaultOpts[key] = options[key].default;

    if ('env' in options[key] && KEY in env) {
      envOpts[key] = env[KEY];
    }

    opts[key] = Object.assign({}, options[key]);

    if (key !== 'conf') {
      //work around to allow override of fileOpts by cliOpts
      delete opts[key].default;
    }
  }

  const initOpts = Object.assign({}, defaultOpts)
      , cliOpts = parseArgs({options: opts, strict: false}).values
      , { conf, version, help, confgen } = cliOpts;

  if (conf) {
    const fileConf = resolve(homedir(), conf);

    if(fs.existsSync(fileConf)) {
      const fileOpts = ini.parse(fs.readFileSync(fileConf, 'utf8'));
      Object.assign(initOpts, fileOpts);
    }
  }

  Object.assign(initOpts, envOpts, cliOpts);

  //manual cast type number because parseArgs is a stupid module -_-
  Object.assign(initOpts, {
    intervalsys: Number(initOpts.intervalsys),
    intervalrpc: Number(initOpts.intervalrpc),
  });

  //TODO validate initOpts values

  return {
    initOpts,
    version,
    confgen,
    help,
  };
}