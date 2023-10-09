
function p(t) {
  process.stdout.write(t);
}

module.exports = (pkg, fullOptions) => {

  const {version, homepage} = pkg;

  p(`SatoshiTop RPC client v${version} - by st3b1t\n${homepage}\n`);

  if (fullOptions) {
    const {
      intervalsys, intervalrpc, confgen, verbose,
      ...options } = fullOptions;

    //TODO add parameter to mange rpc bitcoin-core or electrum

    p('\nUsage:\n');
    p('\tsatoprpc <command> [params] (send command to rpc server)\n');
    p('\tsatoprpc help (show all rpc commands in server)\n');
    p('\nCommands and Params:\n\tRPC API Reference: https://developer.bitcoin.org/reference/rpc/\n');
    //p('\nEnvironment:\n\tSome of the options can be set by the related environment variable `env: <NAMEVAR>`\n');
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