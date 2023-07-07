
function Log(log, opts) {
  this.log = log;

  var self = this;
  var updater = async function() {

    //TODO

    self.updateData();
  };
  updater();
  this.interval = setInterval(updater, opts.intervalrpc);
}

Log.prototype.updateData = function(data) {

}

module.exports = Log;