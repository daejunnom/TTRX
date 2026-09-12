import packageJson from '../package.json' with { type: 'json' };
import * as binding from 'ttrx';
import checks from './npm-package-check.cjs';

checks.runPackageChecks(binding, packageJson.version, 'ES module');
