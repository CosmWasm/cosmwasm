// The entry file of your WebAssembly module.
import { JSONEncoder } from "assemblyscript-json";

import * as contract from "./contract";
import { getDataPtr } from "./utils";

/**
 * Slice refers to some heap allocated data in wasm.
 * A pointer to this can be returned over ffi boundaries.
 */
@unmanaged
class Slice {
  offset: u32;
  len: u32;
}

function wrapSuccessData(data: Uint8Array): usize {
  const encoder = new JSONEncoder();

  // Construct necessary object
  encoder.pushObject(null);
  encoder.pushArray("ok");
  for (let i = 0; i < data.length; i++) {
    encoder.setInteger(null, data[i]);
  }
  encoder.popArray();
  encoder.popObject();

  const result = encoder.serialize();
  const resultPtr = getDataPtr(result);

  // do not remove result before caller got the chance to copy it
  __retain(resultPtr);

  const out: Slice = {
    offset: resultPtr,
    len: result.byteLength,
  };

  return changetype<usize>(out);
}

export function init(_paramsPtr: usize, _messagePtr: usize): usize {
  throw new Error("Not implemented");
}

export function handle(_paramsPtr: usize, _messagePtr: usize): usize {
  throw new Error("Not implemented");
}

export function query(_messagePtr: usize): usize {
  return wrapSuccessData(contract.query());
}

/**
 * allocate reserves the given number of bytes in wasm memory and returns a pointer
 * to a slice defining this data. This space is managed by the calling process
 * and should be accompanied by a corresponding deallocate
 */
export function allocate(size: usize): usize {
  const obj = __alloc(size, idof<ArrayBuffer>());
  __retain(obj);
  return obj;
}

/**
 * deallocate expects a pointer to a Slice created with allocate.
 * It will free both the Slice and the memory referenced by the slice.
 */
export function deallocate(pointer: usize): void {
  __release(pointer);
}
