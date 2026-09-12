import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const destination = resolve(process.argv[2] ?? resolve(root, 'work', 'npm-pack'));
mkdirSync(destination, { recursive: true });

const npmCliCandidates = [
  process.env.npm_execpath,
  join(dirname(process.execPath), 'node_modules', 'npm', 'bin', 'npm-cli.js'),
  resolve(dirname(process.execPath), '..', 'lib', 'node_modules', 'npm', 'bin', 'npm-cli.js'),
].filter(Boolean);
const npmCli = npmCliCandidates.find((candidate) => existsSync(candidate));
if (!npmCli) {
  throw new Error('could not locate npm-cli.js; run this script through an npm installation');
}

const result = spawnSync(
  process.execPath,
  [npmCli, 'pack', '--json', '--pack-destination', destination],
  { cwd: root, encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] },
);
if (result.error) {
  throw new Error(`could not run npm pack: ${result.error.message}`);
}
if (result.status !== 0) {
  throw new Error(`npm pack exited with status ${result.status}`);
}

let packed;
const stdout = result.stdout.trim();
for (let index = stdout.indexOf('['); index !== -1; index = stdout.indexOf('[', index + 1)) {
  try {
    packed = JSON.parse(stdout.slice(index));
    break;
  } catch {
    // Lifecycle scripts may write to stdout before npm emits its final JSON array.
  }
}
if (!packed) {
  throw new Error('npm pack did not return a parseable JSON result');
}
if (!Array.isArray(packed) || packed.length !== 1) {
  throw new Error(`expected one npm tarball, received ${packed.length}`);
}

const [entry] = packed;
const packageJson = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
if (entry.name !== packageJson.name || entry.version !== packageJson.version) {
  throw new Error(
    `packed identity ${entry.name}@${entry.version} does not match ` +
      `${packageJson.name}@${packageJson.version}`,
  );
}

const exactFiles = new Set(['LICENSE', 'README.md', 'package.json']);
for (const file of entry.files ?? []) {
  if (!exactFiles.has(file.path) && !file.path.startsWith('dist/')) {
    throw new Error(`unexpected file in npm tarball: ${file.path}`);
  }
}
for (const required of exactFiles) {
  if (!(entry.files ?? []).some((file) => file.path === required)) {
    throw new Error(`required file missing from npm tarball: ${required}`);
  }
}
for (const required of [
  'dist/node/ttrx_wasm.js',
  'dist/node/ttrx_wasm_bg.wasm',
  'dist/bundler/ttrx_wasm.js',
  'dist/bundler/ttrx_wasm_bg.wasm',
]) {
  if (!(entry.files ?? []).some((file) => file.path === required)) {
    throw new Error(`generated binding missing from npm tarball: ${required}`);
  }
}

const tarball = basename(entry.filename);
console.log(`tarball=${tarball}`);
console.log(`integrity=${entry.integrity}`);
