import { toUtf8 } from "../cosmwasm-encoding";
import { Extern } from "../cosmwasm-std";

export function makeTestingExtern(): Extern {
  return new Extern((address: string) => {
    const addressLength = 20;
    const encoded = toUtf8(address).slice(0, addressLength);
    const out = new Uint8Array(addressLength);
    for (let i = 0; i < encoded.length; i++) out[i] = encoded[i];
    return out;
  });
}
