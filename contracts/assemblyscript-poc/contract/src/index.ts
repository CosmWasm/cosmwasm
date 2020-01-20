// The entry file of your WebAssembly module.
import { JSONEncoder } from "assemblyscript-json";

import * as contract from "./contract";
import { log, releaseOwnership } from "./cosmwasm";

export { allocate, deallocate } from "./cosmwasm";

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
  return releaseOwnership(result);
}

export function init(_paramsPtr: usize, _messagePtr: usize): usize {
  throw new Error("Not implemented");
}

export function handle(_paramsPtr: usize, _messagePtr: usize): usize {
  throw new Error("Not implemented");
}

export function query(_messagePtr: usize): usize {
  log("buh!");
  return wrapSuccessData(contract.query());
}
