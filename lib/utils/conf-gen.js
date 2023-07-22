
const fs = require('fs')
    , {homedir} = require('os')
    , {resolve} = require('path')
    , {createInterface} = require('node:readline/promises');

module.exports = async options => {

  const fileBase = resolve(__dirname,'../../dot.satoprc.example')
      , fileConf = resolve(homedir(), options.conf.default);

  let overwrite = true;

  if(fs.existsSync(fileConf)) {

    console.warn(`File '${fileConf}' just exists!`)

    const readline = createInterface({
      input: process.stdin,
      output: process.stdout
    });

    const resp = await readline.question('Do you want to overwrite it? [y/n]\n');

    overwrite = Boolean(resp === 'y');
  }

  if (overwrite) {
    fs.copyFile(fileBase, fileConf, err => {
      if (err) {
        console.error(`Error in config generation '${fileConf}'`, err);
        process.exit(1);
      }
      else {
        fs.chmod(fileConf, 0o600, err => {
          if (err) {
            console.error(`Error in chmod config '${fileConf}'`, err);
            process.exit(1);
          }
          else {
            console.log(`Config file generated in '${fileConf}'`);
            process.exit(0);
          }
        });
      }
    });
  }
}
