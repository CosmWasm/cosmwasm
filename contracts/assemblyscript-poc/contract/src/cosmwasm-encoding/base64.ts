import { fromUtf8 } from "./utf8";

const alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const equalsChar: u8 = 61; // ASCII value of =

export class Base64 {
  public static encode(data: Uint8Array): string {
    if (data.length === 0) return "";

    // Number of trailing = characters
    const paddingCount = data.length % 3 ? 3 - (data.length % 3) : 0;

    const outLen = (data.length / 3 + (data.length % 3 > 0 ? 1 : 0)) * 4;
    const out = new Uint8Array(outLen);
    let outPos = 0;

    for (let c = 0; c < data.length; c += 3) {
      // Compose to a single 24-bit number
      let n: u32 = (data[c] as u32) << 16;
      if (c + 1 < data.length) n += (data[c + 1] as u32) << 8;
      if (c + 2 < data.length) n += data[c + 2];

      // Split into four 6-bit numbers
      const o1 = (n >>> 18) & 63;
      const o2 = (n >>> 12) & 63;
      const o3 = (n >>> 6) & 63;
      const o4 = n & 63;

      out[outPos++] = alphabet.charCodeAt(o1);
      out[outPos++] = alphabet.charCodeAt(o2);
      out[outPos++] = alphabet.charCodeAt(o3);
      out[outPos++] = alphabet.charCodeAt(o4);
    }

    // replace last `paddingCount` positions with =
    for (let i = 0; i < paddingCount; i++) {
      out[out.length - 1 - i] = equalsChar;
    }

    return fromUtf8(out);
  }
}
