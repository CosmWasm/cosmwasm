import { toUtf8 } from "../cosmwasm-encoding";
import { allocate, deallocate, keepOwnership, readRegion } from "./cosmwasm";
import * as env from "./env";

export class Extern {
  constructor(public readonly canonicalize: (humanAddress: string) => Uint8Array) {}
}

export function canonicalizeImpl(human: string): Uint8Array {
  const humanEncoded = toUtf8(human);
  const resultPtr = allocate(50);
  const returnCode = env.canonicalize_address(keepOwnership(humanEncoded), resultPtr);
  if (returnCode < 0) {
    throw new Error(
      "Call to env.canonicalize_address failed with return code " + returnCode.toString(),
    );
  }
  const canonical = readRegion(resultPtr);
  deallocate(resultPtr);
  return canonical;
}

export function makeExtern(): Extern {
  return new Extern(canonicalizeImpl);
}
