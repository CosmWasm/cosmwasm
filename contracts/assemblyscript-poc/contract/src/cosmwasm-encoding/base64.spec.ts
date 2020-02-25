import { Base64 } from "./base64";
import { Hex } from "./hex";

describe("Base64", () => {
  describe("encode", () => {
    it("works for empty input", () => {
      const data = new Uint8Array(0);
      expect(Base64.encode(data)).toStrictEqual("");
    });

    it("works for one byte", () => {
      {
        // head -c 1 /dev/zero | base64
        const data = new Uint8Array(1);
        data[0] = 0x00;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("AA==");
      }
      {
        // echo -n "a" | base64
        const data = new Uint8Array(1);
        data[0] = 0x61;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("YQ==");
      }
      {
        // echo "ff" | xxd -r -p - | base64
        const data = new Uint8Array(1);
        data[0] = 0xff;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("/w==");
      }
    });

    it("works for two bytes", () => {
      {
        // head -c 2 /dev/zero | base64
        const data = new Uint8Array(2);
        data[0] = 0x00;
        data[0] = 0x00;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("AAA=");
      }
      {
        // echo -n "ab" | base64
        const data = new Uint8Array(2);
        data[0] = 0x61;
        data[1] = 0x62;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("YWI=");
      }
      {
        // echo "ffff" | xxd -r -p - | base64
        const data = new Uint8Array(2);
        data[0] = 0xff;
        data[1] = 0xff;
        const encoded = Base64.encode(data);
        expect(encoded).toStrictEqual("//8=");
      }
    });

    it("works for three bytes (no padding)", () => {
      {
        // head -c 3 /dev/zero | base64
        const data = new Uint8Array(3);
        data[0] = 0x00;
        data[1] = 0x00;
        data[2] = 0x00;
        expect(Base64.encode(data)).toStrictEqual("AAAA");
      }
      {
        // echo -n "abc" | base64
        const data = new Uint8Array(3);
        data[0] = 0x61;
        data[1] = 0x62;
        data[2] = 0x63;
        expect(Base64.encode(data)).toStrictEqual("YWJj");
      }
      {
        // echo "ffffff" | xxd -r -p - | base64
        const data = new Uint8Array(3);
        data[0] = 0xff;
        data[1] = 0xff;
        data[2] = 0xff;
        expect(Base64.encode(data)).toBlockEqual("////");
      }
    });

    it("works for more than one block", () => {
      {
        // echo "aabbccddee" | xxd -r -p - | base64
        const data = Hex.decode("aabbccddee");
        expect(Base64.encode(data)).toStrictEqual("qrvM3e4=");
      }
      {
        // echo "aabbccddee00" | xxd -r -p - | base64
        const data = Hex.decode("aabbccddee00");
        expect(Base64.encode(data)).toStrictEqual("qrvM3e4A");
      }
      {
        // echo "aabbccddee003a" | xxd -r -p - | base64
        const data = Hex.decode("aabbccddee003a");
        expect(Base64.encode(data)).toStrictEqual("qrvM3e4AOg==");
      }
    });
  });
});
