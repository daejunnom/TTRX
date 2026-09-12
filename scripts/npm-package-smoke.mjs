import packageJson from '../package.json' with { type: 'json' };
import checks from './npm-package-check.cjs';

const binding = await import(packageJson.name);

checks.runPackageChecks(binding, packageJson.version, 'ES module');
