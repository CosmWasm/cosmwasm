import { Hex, toUtf8 } from "../cosmwasm-encoding";
import { Extern, Storage } from "../cosmwasm-std";

// TODO: find a way to use a fresh instance per makeTestingExtern() call
const testData: Map<string, Uint8Array> = new Map<string, Uint8Array>();

function canonicalizeImpl(human: string): Uint8Array {
  const addressLength = 20;
  const encoded = toUtf8(human).slice(0, addressLength);
  const out = new Uint8Array(addressLength);
  for (let i = 0; i < encoded.length; i++) out[i] = encoded[i];
  return out;
}

export function makeTestingExtern(): Extern {
  const storage = new Storage(
    (key: Uint8Array): Uint8Array => {
      return testData.get(Hex.encode(key));
    },
    (key: Uint8Array, value: Uint8Array): void => {
      testData.set(Hex.encode(key), value);
    },
  );
  return new Extern(canonicalizeImpl, storage);
}
