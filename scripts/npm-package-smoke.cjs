'use strict';

const packageJson = require('../package.json');
const binding = require('ttrx');
const { runPackageChecks } = require('./npm-package-check.cjs');

runPackageChecks(binding, packageJson.version, 'CommonJS');
