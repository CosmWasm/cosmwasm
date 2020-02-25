import { query } from "./contract";
import { makeTestingExtern } from "./cosmwasm-testing";
import { Encoding } from "./utils";

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
