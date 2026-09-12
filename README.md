# TTRX

TTRX is a Rust converter for TETR.IO `.ttr` and `.ttrm` replay files. It writes a versioned `.ttrx` binary container and converts that container back to canonical JSON.

The first implemented profile is `source-semantic`: it preserves the parsed JSON data model, including object and array order, duplicate object keys, exact number lexemes, unknown fields, and unknown events. Whitespace and equivalent string escaping are canonicalized when decoding. Because input events and their values are retained, the reverse conversion preserves the source behavior without inventing game state.

This profile is the first vertical implementation of the converter. Deterministic action compilation and board regeneration described in the design documents are future format profiles; they are not claimed by the current executable.

## Build and use

```powershell
cargo build --release -p ttrx-cli

target\release\ttrx encode replay.ttr
target\release\ttrx encode match.ttrm -o match.ttrx
target\release\ttrx decode match.ttrx
target\release\ttrx inspect match.ttrx
target\release\ttrx verify replay.ttr
```

Output files are not overwritten unless `--force` is supplied, and `--force` only replaces a regular file. `encode` infers `.ttr` or `.ttrm` from the extension and also checks the replay shape. `decode` restores the extension recorded in the container. Use `-o` to choose another output path.

On Windows, replacing an existing file uses a uniquely named sibling backup because the standard rename operation cannot replace a destination. Ordinary failures restore that backup. If the process or machine stops between the two rename operations, the previous file may remain beside the output under a hidden `previous` temporary name and can be restored manually.

The core crate has no external dependencies. The WebAssembly wrapper uses `wasm-bindgen`; the CLI uses only the standard library and `ttrx-core`.

## WebAssembly

The wrapper exports `encode_ttr`, `encode_ttrm`, `decode_ttrx`, `ttrx_source_extension`, source/container inspection and verification functions, plus `ttrx_version`. `ttrx_source_extension` tells a browser caller whether decoded bytes should be saved as `.ttr`, `.ttrm`, or `.json`. Build it and generate Node.js glue with the pinned `wasm-bindgen` schema version:

```powershell
rustup target add wasm32-unknown-unknown
cargo install --locked wasm-bindgen-cli --version 0.2.126
wasm-bindgen --version # must report wasm-bindgen 0.2.126
cargo build --release -p ttrx-wasm --target wasm32-unknown-unknown
wasm-bindgen target\wasm32-unknown-unknown\release\ttrx_wasm.wasm `
  --target nodejs --out-dir work\wasm
node scripts\wasm-smoke.cjs work\wasm\ttrx_wasm.js replay.ttr match.ttrm
```

The exported encode/decode functions accept and return JavaScript `Uint8Array` values. Browser bundlers can use the same Wasm artifact with the matching `wasm-bindgen` target and CLI version.

## npm package

The `ttrx` npm package ships the WebAssembly converter for Node.js and browser-aware bundlers. It exposes the same eleven functions documented above, with CommonJS and ES module entry points for Node.js and an ES module WebAssembly entry point selected by browser bundlers.

```sh
npm install ttrx
```

```js
import { readFile, writeFile } from 'node:fs/promises';
import { decode_ttrx, encode_ttr } from 'ttrx';

const source = await readFile('replay.ttr');
const container = encode_ttr(source);
await writeFile('replay.ttrx', container);

const canonicalJson = decode_ttrx(container);
await writeFile('replay.decoded.ttr', canonicalJson);
```

CommonJS consumers can use `const ttrx = require('ttrx')`. The `inspect_*` and `verify_*` functions return human-readable strings; the encode/decode functions accept and return `Uint8Array` values and throw JavaScript errors for invalid inputs.

To build the npm package from source, install the `wasm32-unknown-unknown` Rust target and `wasm-bindgen-cli` 0.2.126, then run `npm run build`. `npm test` runs the Rust workspace tests, and `npm run test:package` checks both Node.js module formats after a build.

## Fidelity

The current profile is stricter than placement-only equivalence: it round-trips the complete parsed JSON tree. It does not promise byte-for-byte JSON identity because insignificant whitespace and equivalent escapes are not retained. No TETR.IO game engine is run during conversion, so `verify` proves container/data-model round-trip, not independent board simulation.

See [project requirements](docs/ttrx/requirements.md), [TTRX 1.0 binary format](docs/ttrx/format-v1.md), [implementation status](docs/ttrx/implementation-status.md), [validation evidence](docs/ttrx/validation.md), [TETR.IO source audit](docs/tetrio/README.md), and [open verification work](docs/tetrio/verification/coverage-and-open-items.md).
