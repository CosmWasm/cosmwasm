/* eslint-disable @typescript-eslint/triple-slash-reference */

/// <reference path="../node_modules/@as-pect/assembly/types/as-pect.d.ts" />

import { Encoding } from "./utils";

describe("Encoding", () => {
  describe("toUtf8", () => {
    it("works for empty input", () => {
      expect(Encoding.toUtf8("")).toStrictEqual(new Uint8Array(0));
    });

    it("works for simmple examples", () => {
      {
        const expected = new Uint8Array(3);
        expected[0] = 0x61;
        expected[1] = 0x62;
        expected[2] = 0x63;
        expect(Encoding.toUtf8("abc")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(12);
        let i = 0;
        expected[i++] = 0x20;
        expected[i++] = 0x3f;
        expected[i++] = 0x3d;
        expected[i++] = 0x2d;
        expected[i++] = 0x6e;
        expected[i++] = 0x7c;
        expected[i++] = 0x7e;
        expected[i++] = 0x2b;
        expected[i++] = 0x2d;
        expected[i++] = 0x2a;
        expected[i++] = 0x2f;
        expected[i++] = 0x5c;
        expect(Encoding.toUtf8(" ?=-n|~+-*/\\")).toStrictEqual(expected);
      }
    });
  });
});
