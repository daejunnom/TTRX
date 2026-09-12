import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const packageJson = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
const tag = process.argv[2] ?? process.env.GITHUB_REF_NAME;

if (!tag) {
  throw new Error('pass a release tag or set GITHUB_REF_NAME');
}

const expected = `v${packageJson.version}`;
if (tag !== expected) {
  throw new Error(`release tag ${tag} does not match package version ${expected}`);
}

console.log(`Release tag ${tag} matches ttrx ${packageJson.version}`);
