import { Hex } from "./hex";

describe("Hex", () => {
  describe("encode", () => {
    it("works for empty input", () => {
      const data = new Uint8Array(0);
      expect(Hex.encode(data)).toStrictEqual("");
    });

    it("works for one byte", () => {
      {
        const data = new Uint8Array(1);
        data[0] = 0x00;
        expect(Hex.encode(data)).toStrictEqual("00");
      }
      {
        const data = new Uint8Array(1);
        data[0] = 0x01;
        expect(Hex.encode(data)).toStrictEqual("01");
      }
      {
        const data = new Uint8Array(1);
        data[0] = 0x23;
        expect(Hex.encode(data)).toStrictEqual("23");
      }
      {
        const data = new Uint8Array(1);
        data[0] = 0x3f;
        expect(Hex.encode(data)).toStrictEqual("3f");
      }
    });

    it("works for multiple bytes", () => {
      {
        const data = new Uint8Array(2);
        data[0] = 0xaa;
        data[1] = 0xbb;
        expect(Hex.encode(data)).toStrictEqual("aabb");
      }
    });
  });

  describe("decode", () => {
    it("works for empty input", () => {
      const expected = new Uint8Array(0);
      expect(Hex.decode("")).toStrictEqual(expected);
    });

    it("works for single byte", () => {
      {
        const expected = new Uint8Array(1);
        expected[0] = 0x00;
        expect(Hex.decode("00")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(1);
        expected[0] = 0x01;
        expect(Hex.decode("01")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(1);
        expected[0] = 0x0a;
        expect(Hex.decode("0a")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(1);
        expected[0] = 0x23;
        expect(Hex.decode("23")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(1);
        expected[0] = 0xab;
        expect(Hex.decode("ab")).toStrictEqual(expected);
      }
      {
        const expected = new Uint8Array(1);
        expected[0] = 0x6c;
        expect(Hex.decode("6c")).toStrictEqual(expected);
      }
    });

    it("works for multiple bytes", () => {
      {
        const expected = new Uint8Array(2);
        expected[0] = 0x78;
        expected[1] = 0x9a;
        expect(Hex.decode("789a")).toStrictEqual(expected);
      }
    });
  });
});
