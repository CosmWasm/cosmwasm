import { toUtf8 } from "../cosmwasm-encoding";
import { allocate, deallocate, keepOwnership, readRegion } from "./cosmwasm";
import * as env from "./env";

export class Storage {
  constructor(
    public readonly read: (key: Uint8Array) => Uint8Array,
    public readonly write: (key: Uint8Array, value: Uint8Array) => void,
  ) {}
}

export class Extern {
  constructor(
    public readonly canonicalize: (humanAddress: string) => Uint8Array,
    public readonly storage: Storage,
  ) {}
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
  const storage = new Storage(
    (key: Uint8Array): Uint8Array => {
      const keyPtr = keepOwnership(key);
      const resultPtr = allocate(2048);
      const readResult = env.read_db(keyPtr, resultPtr);
      if (readResult < 0) throw new Error("Error reading from database");
      const result = readRegion(resultPtr);
      return result.slice(0, readResult);
    },
    (key: Uint8Array, value: Uint8Array): void => {
      const keyPtr = keepOwnership(key);
      const valuePtr = keepOwnership(value);
      const writeResult = env.write_db(keyPtr, valuePtr);
      if (writeResult < 0) throw new Error("Error writing to database");
    },
  );
  return new Extern(canonicalizeImpl, storage);
}
