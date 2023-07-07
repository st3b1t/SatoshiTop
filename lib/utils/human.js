
function humanSubver(subver) {
  return subver.replace(/\//g,'')
}

function humanBytes(bytes, isDecimal) {
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
}

function humanSecs(sec, ago) {

  let msec = 0;

  if(sec && ago) {
    const date = new Date()
        , time = date.getTime();
    sec = (time/1000) - sec;
  }

  const y = Math.floor(sec / 31536000)
      , m = Math.floor(sec / 2629744)
      , d = Math.floor((sec % 31536000) / 86400)
      , h = Math.floor(((sec % 31536000) % 86400) / 3600)
      , i = Math.floor((((sec % 31536000) % 86400) % 3600) / 60)
      , s = Math.floor((((sec % 31536000) % 86400) % 3600) % 60)
      , ret = [];

  if(y>0) {
    ret.push(y+'Y');
  }
  if(m>0) {
    ret.push(m+'M');
  }
  if(d>0) {
    ret.push(d+'D');
  }
  if(h>0) {
    ret.push(h+'h');
  }
  if(i>0) {
    ret.push(i+'m');
  }
  if(s>0) {
    ret.push(s+'s');
  }

  return ret.join(' ');
}

module.exports = {
  humanSubver,
  humanBytes,
  humanSecs
}