
function p(t) {
  process.stdout.write(t);
}

module.exports = (pkg, options) => {

  const {name, version, description, homepage, author} = pkg;

  p(`${name} v${version} - (by ${author})\n${description}\n${homepage}\n`);

  if (options) {
    p('\nUsage:\n');
    p('\tsatop <option> \n');
    p('\tsatop --help \n');
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