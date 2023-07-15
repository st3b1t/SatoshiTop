
const sys = require('systeminformation');

const sysFake = require('../../test/sys-fake');

const {fsSize, processes} = process.env.FAKEMODE ? sysFake: sys;

module.exports = {
  ...sys,
  fsSize,
  processes
};
