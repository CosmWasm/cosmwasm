import { equalUint8Array } from "./memory";

describe("memory", () => {
  describe("equalUint8Array", () => {
    it("works for the same array", () => {
      const array1 = new Uint8Array(0);
      expect(equalUint8Array(array1, array1)).toStrictEqual(true);

      const array2 = new Uint8Array(1);
      array2[0] = 0x34;
      expect(equalUint8Array(array2, array2)).toStrictEqual(true);
    });

    it("works for equal arrays", () => {
      {
        const a = new Uint8Array(0);
        const b = new Uint8Array(0);
        expect(equalUint8Array(a, b)).toStrictEqual(true);
      }
      {
        const a = new Uint8Array(1);
        a[0] = 0x34;
        const b = new Uint8Array(1);
        b[0] = 0x34;
        expect(equalUint8Array(a, b)).toStrictEqual(true);
      }
      {
        const a = new Uint8Array(3);
        a[0] = 0x34;
        a[1] = 0xdd;
        a[2] = 0x6f;
        const b = new Uint8Array(3);
        b[0] = 0x34;
        b[1] = 0xdd;
        b[2] = 0x6f;
        expect(equalUint8Array(a, b)).toStrictEqual(true);
      }
    });

    it("works for unequal arrays", () => {
      // different length
      {
        const a = new Uint8Array(0);
        const b = new Uint8Array(2);
        expect(equalUint8Array(a, b)).toStrictEqual(false);
      }

      // different content
      {
        const a = new Uint8Array(1);
        a[0] = 0x34;
        const b = new Uint8Array(1);
        b[0] = 0x43;
        expect(equalUint8Array(a, b)).toStrictEqual(false);
      }

      // common prefix and suffix but different content
      {
        const a = new Uint8Array(3);
        a[0] = 0x34;
        a[1] = 0xdd;
        a[2] = 0x6f;
        const b = new Uint8Array(3);
        b[0] = 0x34;
        b[1] = 0x55;
        b[2] = 0x6f;
        expect(equalUint8Array(a, b)).toStrictEqual(false);
      }
    });
  });
});
