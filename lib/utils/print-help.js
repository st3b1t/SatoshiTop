
function p(t) {
  process.stdout.write(t);
}

module.exports = (pkg, options) => {

  const {version, description, homepage} = pkg;

  p(`SatoshiTop v${version} - by st3b1t\n${description}\n${homepage}\n`);

  if (options) {
    p('\nUsage:\n');
    p('\tsatop <option> \n');
    p('\tsatop --help \n');
    p('\nEnvironment:\n\tSome of the options can be set by the related environment variable `env: <NAMEVAR>`\n');
    p('\nOptions:\n\n');
    Object.entries(options).forEach(([key, val]) => {
      p(`\t`)
      if (val.short) {
        p(`-${val.short}, `)
      }
      p(`--${key}\n\t\t${val.description}`);
      if (val.default) {
        p(` (default: ${val.default})`);
      }
      if (val.env) {
        p(`\n\t\t(env: ${val.env})`)
      }
      p('\n\n');
    });
  }
}