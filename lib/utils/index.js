
const options = require('./options');
const rpcClient = require('./rpc-client');
const printHelp = require('./print-help');

const colors = [
  'magenta', 'cyan', 'blue', 'yellow', 'green', 'red'
];

module.exports = {
  colors,
  options,
  rpcClient,
  printHelp,
  humanBytes: (bytes, isDecimal) => {
    isDecimal = typeof isDecimal !== 'undefined' ? isDecimal : false;
    if (bytes == 0) {
      return '0.00 B';
    }
    var base = isDecimal ? 1000 : 1024;
    var e = Math.floor(Math.log(bytes) / Math.log(base));
    return (
      (bytes / Math.pow(base, e)).toFixed(2) +
      ' ' +
      ' KMGTP'.charAt(e) +
      (isDecimal || e == 0 ? '' : 'i') +
      'B'
    );
  },
  humanSecs: (sec, ago) => {
    //http://goo.gl/8DqYS
    if(sec && ago)
      sec = (K.Util.time()/1000) - (sec/1000);

    var y = Math.floor(sec / 31536000),
      m = Math.floor(sec / 2629744),
      d = Math.floor((sec % 31536000) / 86400),
      h = Math.floor(((sec % 31536000) % 86400) / 3600),
      i = Math.floor((((sec % 31536000) % 86400) % 3600) / 60),
      s = Math.floor((((sec % 31536000) % 86400) % 3600) % 60),
      ret = '';

    if(y>0)
      ret+=' '+y+'year'+(y>1 && 's');

    if(y<2 && m>0)
      ret+=' '+m+'month'+(m>1 && 's');

    if(m<2 && d>0)
      ret+=' '+d+'day'+(d>1 && 's');

    if(d<2 && h>0)
      ret+=' '+h+'hour'+(h>1 && 's');

    if(d<1) {
      if(h<1 && i>0)
        ret+=' '+i+'minute'+(i>1 && 's');

      if(i<2 && s>0 && i<1)
        ret+=' '+s+'second'+(s>1 && 's');
    }

    return ret;
  },
}
