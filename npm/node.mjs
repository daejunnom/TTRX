import binding from './ttrx_wasm.js';

export const {
  decode_ttrx,
  encode_ttr,
  encode_ttrm,
  inspect_ttr,
  inspect_ttrm,
  inspect_ttrx,
  ttrx_source_extension,
  ttrx_version,
  verify_ttr,
  verify_ttrm,
  verify_ttrx,
} = binding;

export default binding;
