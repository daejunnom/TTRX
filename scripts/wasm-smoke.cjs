'use strict';

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

if (process.argv.length !== 5) {
  console.error('usage: node scripts/wasm-smoke.cjs <ttrx_wasm.js> <fixture.ttr> <fixture.ttrm>');
  process.exit(2);
}

const [, , modulePath, ttrPath, ttrmPath] = process.argv;
const ttrx = require(path.resolve(modulePath));

const fixtures = [
  { kind: 'ttr', sourcePath: ttrPath, encode: ttrx.encode_ttr, verify: ttrx.verify_ttr },
  { kind: 'ttrm', sourcePath: ttrmPath, encode: ttrx.encode_ttrm, verify: ttrx.verify_ttrm },
];

for (const fixture of fixtures) {
  const source = fs.readFileSync(fixture.sourcePath);
  assert.match(fixture.verify(source), /Verification: passed/);

  const container = fixture.encode(source);
  assert.equal(ttrx.ttrx_source_extension(container), fixture.kind);
  assert.match(ttrx.inspect_ttrx(container), new RegExp(`Source kind: ${fixture.kind}`));
  assert.match(ttrx.verify_ttrx(container), /Verification: passed/);

  const decoded = Buffer.from(ttrx.decode_ttrx(container));
  assert.match(fixture.verify(decoded), /Verification: passed/);
  const byteIdentical = source.equals(decoded);
  const ratio = (container.length / source.length * 100).toFixed(2);
  console.log(
    `${fixture.kind}: ${source.length} -> ${container.length} bytes (${ratio}%); ` +
      `decoded-byte-identical=${byteIdentical}`,
  );
}

console.log(`TTRX WASM smoke test passed; version=${ttrx.ttrx_version()}`);
