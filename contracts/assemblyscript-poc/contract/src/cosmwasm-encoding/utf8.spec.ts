/* eslint-disable @typescript-eslint/triple-slash-reference */

/// <reference path="../../node_modules/@as-pect/assembly/types/as-pect.d.ts" />

import { fromUtf8, toUtf8 } from "./utf8";

describe("utf8", () => {
  describe("toUtf8", () => {
    it("works for empty input", () => {
      expect(toUtf8("")).toStrictEqual(new Uint8Array(0));
    });

    it("works for simmple examples", () => {
      {
        const expected = new Uint8Array(3);
        expected[0] = 0x61;
        expected[1] = 0x62;
        expected[2] = 0x63;
        expect(toUtf8("abc")).toStrictEqual(expected);
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
        expect(toUtf8(" ?=-n|~+-*/\\")).toStrictEqual(expected);
      }
    });
  });

  describe("fromUtf8", () => {
    it("works for empty input", () => {
      expect(fromUtf8(new Uint8Array(0))).toStrictEqual("");
    });

    it("works for simmple examples", () => {
      {
        const encoded = new Uint8Array(3);
        encoded[0] = 0x61;
        encoded[1] = 0x62;
        encoded[2] = 0x63;
        expect(fromUtf8(encoded)).toStrictEqual("abc");
      }
      {
        const encoded = new Uint8Array(12);
        let i = 0;
        encoded[i++] = 0x20;
        encoded[i++] = 0x3f;
        encoded[i++] = 0x3d;
        encoded[i++] = 0x2d;
        encoded[i++] = 0x6e;
        encoded[i++] = 0x7c;
        encoded[i++] = 0x7e;
        encoded[i++] = 0x2b;
        encoded[i++] = 0x2d;
        encoded[i++] = 0x2a;
        encoded[i++] = 0x2f;
        encoded[i++] = 0x5c;
        expect(fromUtf8(encoded)).toStrictEqual(" ?=-n|~+-*/\\");
      }
    });
  });
});
