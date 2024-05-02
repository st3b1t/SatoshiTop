
function humanSubver(subver) {
  return `${subver}`.replace(/\//g,'')
}

function humanBytes(bytes, isDecimal = true) {
  if (bytes == 0) {
    return '0.00 B';
  }
  var base = isDecimal ? 1000 : 1024;
  var e = Math.floor(Math.log(bytes) / Math.log(base));
  return (
    (bytes / Math.pow(base, e)).toFixed(1) +
    ' KMGTP'.charAt(e) +
    (isDecimal || e == 0 ? '' : 'i') +
    'B'
  );
}

function humanSecs(sec, ago, long = false) {

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
    ret.push(y+(long?'year':'Y'));
  }
  if(m>0) {
    ret.push(m+(long?'month':'M'));
  }
  if(d>0) {
    ret.push(d+(long?'day':'D'));
  }
  if(h>0) {
    ret.push(h+(long?'hour':'h'));
  }
  if(i>0) {
    ret.push(i+(long?'min':'m'));
  }
  if(s>0) {
    ret.push(s+(long?'sec':'s'));
  }

  return ret.join(' ');
}


function humanMinAgo(sec) {

  let msec = 0;

  if(sec) {
    const date = new Date()
        , time = date.getTime();
    sec = (time/1000) - sec;
  }

  const i = Math.floor((((sec % 31536000) % 86400) % 3600) / 60)
      , s = Math.floor((((sec % 31536000) % 86400) % 3600) % 60)
      , ret = [];

  if(i>0) {
    ret.push(i+'min');
  }
  else if(s>0) {
    ret.push(s+'sec');
  }
  return ret.join(' ');
}

function humanBlocks(blocks, long) {
  return humanSecs(blocks * 600, false, long);
}

module.exports = {
  humanSubver,
  humanBytes,
  humanMinAgo,
  humanSecs,
  humanBlocks
}