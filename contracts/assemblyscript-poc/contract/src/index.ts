// The entry file of your WebAssembly module.
import { JSONEncoder } from "assemblyscript-json";

import * as contract from "./contract";
import { log, releaseOwnership, takeOwnership } from "./cosmwasm";
import { Base64 } from "./encoding/base64";
import { parse } from "./encoding/json";
import { Encoding } from "./utils";

export { allocate, deallocate } from "./cosmwasm";

function wrapSuccessData(data: Uint8Array): usize {
  const encoder = new JSONEncoder();
  encoder.pushObject(null);
  encoder.setString("ok", Base64.encode(data));
  encoder.popObject();
  const json = encoder.toString();
  const result = Encoding.toUtf8(json);
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
