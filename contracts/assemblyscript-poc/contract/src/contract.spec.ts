import { query } from "./contract";
import { toUtf8 } from "./cosmwasm-encoding";
import { makeTestingExtern } from "./cosmwasm-testing";

describe("contract", () => {
  describe("query", () => {
    it("works for address with balance", () => {
      const extern = makeTestingExtern();
      const res = query(extern, toUtf8('{"balance": {"address": "addr4321"}}'));
      expect(res).toStrictEqual('{"balance":"22"}');
    });

    it("works for address without balance", () => {
      const extern = makeTestingExtern();
      const res = query(extern, toUtf8('{"balance": {"address": "broke"}}'));
      expect(res).toStrictEqual('{"balance":"0"}');
    });
  });
});
