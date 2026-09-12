import { copyFileSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dist = join(root, 'dist');
const nodeOutput = join(dist, 'node');
const bundlerOutput = join(dist, 'bundler');
const expectedBindgenVersion = '0.2.126';

function run(command, args, { capture = false } = {}) {
  const result = spawnSync(command, args, {
    cwd: root,
    encoding: 'utf8',
    stdio: capture ? ['ignore', 'pipe', 'inherit'] : 'inherit',
  });

  if (result.error) {
    throw new Error(`could not run ${command}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`${command} exited with status ${result.status}`);
  }
  return capture ? result.stdout.trim() : '';
}

const packageJson = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'));
const metadata = JSON.parse(
  run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], {
    capture: true,
  }),
);
const wasmPackage = metadata.packages.find((entry) => entry.name === 'ttrx-wasm');
if (!wasmPackage) {
  throw new Error('cargo metadata did not contain the ttrx-wasm package');
}
if (wasmPackage.version !== packageJson.version) {
  throw new Error(
    `version mismatch: package.json=${packageJson.version}, ttrx-wasm=${wasmPackage.version}`,
  );
}

const bindgenVersion = run('wasm-bindgen', ['--version'], { capture: true });
if (bindgenVersion !== `wasm-bindgen ${expectedBindgenVersion}`) {
  throw new Error(
    `wasm-bindgen ${expectedBindgenVersion} is required; found ${bindgenVersion || '<no version>'}`,
  );
}

if (dirname(dist) !== root || resolve(dist) !== join(root, 'dist')) {
  throw new Error(`refusing to clean unexpected output directory: ${dist}`);
}
rmSync(dist, { recursive: true, force: true });
mkdirSync(nodeOutput, { recursive: true });
mkdirSync(bundlerOutput, { recursive: true });

run('cargo', [
  'build',
  '--locked',
  '--release',
  '-p',
  'ttrx-wasm',
  '--target',
  'wasm32-unknown-unknown',
]);

const wasmArtifact = join(
  root,
  'target',
  'wasm32-unknown-unknown',
  'release',
  'ttrx_wasm.wasm',
);
for (const [target, output] of [
  ['nodejs', nodeOutput],
  ['bundler', bundlerOutput],
]) {
  run('wasm-bindgen', [wasmArtifact, '--target', target, '--out-dir', output]);
}

copyFileSync(join(root, 'npm', 'node.mjs'), join(nodeOutput, 'index.mjs'));
writeFileSync(join(nodeOutput, 'package.json'), '{"type":"commonjs"}\n');
writeFileSync(join(bundlerOutput, 'package.json'), '{"type":"module"}\n');

console.log(`Built ttrx ${packageJson.version} npm bindings in ${dist}`);
