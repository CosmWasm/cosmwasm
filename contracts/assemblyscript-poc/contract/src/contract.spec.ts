import { query } from "./contract";
import { Extern } from "./cosmwasm";
import { Encoding } from "./utils";

function makeTestingExtern(): Extern {
  return new Extern((address: string) => {
    const addressLength = 20;
    const encoded = Encoding.toUtf8(address).slice(0, addressLength);
    const out = new Uint8Array(addressLength);
    for (let i = 0; i < encoded.length; i++) out[i] = encoded[i];
    return out;
  });
}

describe("contract", () => {
  describe("query", () => {
    it("works for address with balance", () => {
      const extern = makeTestingExtern();
      const res = query(extern, Encoding.toUtf8('{"balance": {"address": "addr4321"}}'));
      expect(res).toStrictEqual('{"balance":"22"}');
    });

    it("works for address without balance", () => {
      const extern = makeTestingExtern();
      const res = query(extern, Encoding.toUtf8('{"balance": {"address": "broke"}}'));
      expect(res).toStrictEqual('{"balance":"0"}');
    });
  });
});
