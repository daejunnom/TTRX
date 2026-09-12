'use strict';

const assert = require('node:assert/strict');

const apiNames = [
  'decode_ttrx',
  'encode_ttr',
  'encode_ttrm',
  'inspect_ttr',
  'inspect_ttrm',
  'inspect_ttrx',
  'ttrx_source_extension',
  'ttrx_version',
  'verify_ttr',
  'verify_ttrm',
  'verify_ttrx',
];

const single = {
  version: 1,
  replay: {
    frames: 2,
    options: { version: 19, g: 0.02 },
    events: [
      { frame: 0, type: 'start', data: {} },
      { frame: 1, type: 'end', data: { reason: 'clear' } },
    ],
  },
};

const multi = {
  version: 1,
  replay: {
    rounds: [
      [
        { id: 'a', replay: { events: [{ frame: 0, type: 'start' }] } },
        { id: 'b', replay: { events: [{ frame: 1, type: 'end' }] } },
      ],
    ],
  },
};

function runPackageChecks(binding, expectedVersion, loader) {
  for (const name of apiNames) {
    assert.equal(typeof binding[name], 'function', `${loader} export ${name}`);
  }
  assert.equal(binding.ttrx_version(), expectedVersion);

  for (const [kind, value, encode, verify] of [
    ['ttr', single, binding.encode_ttr, binding.verify_ttr],
    ['ttrm', multi, binding.encode_ttrm, binding.verify_ttrm],
  ]) {
    const source = Buffer.from(JSON.stringify(value));
    assert.match(verify(source), /Verification: passed/);

    const container = encode(source);
    assert.ok(container instanceof Uint8Array);
    assert.equal(binding.ttrx_source_extension(container), kind);
    assert.match(binding.inspect_ttrx(container), new RegExp(`Source kind: ${kind}`));
    assert.match(binding.verify_ttrx(container), /Verification: passed/);

    const decoded = JSON.parse(Buffer.from(binding.decode_ttrx(container)).toString('utf8'));
    assert.deepEqual(decoded, value);

    const corrupted = Uint8Array.from(container);
    corrupted[corrupted.length - 1] ^= 0xff;
    assert.throws(() => binding.decode_ttrx(corrupted));
  }

  console.log(`TTRX npm ${loader} smoke test passed; version=${expectedVersion}`);
}

module.exports = { apiNames, runPackageChecks };
