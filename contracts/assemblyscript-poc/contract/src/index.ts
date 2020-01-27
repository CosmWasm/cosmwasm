// The entry file of your WebAssembly module.
import { JSONEncoder } from "assemblyscript-json";

import * as contract from "./contract";
import { log, releaseOwnership, takeOwnership } from "./cosmwasm";
import { parse } from "./encoding/json";
import { Encoding } from "./utils";

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

export function query(messagePtr: usize): usize {
  const msgJson = takeOwnership(messagePtr);
  log("JSON query request: " + Encoding.fromUtf8(msgJson));
  const msg = parse(msgJson).asObject();
  return wrapSuccessData(contract.query(msg));
}

// eslint-disable-next-line @typescript-eslint/camelcase
export function cosmwasm_api_0_6(): i32 {
  return 0x0603;
}
