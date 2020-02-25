import { fromUtf8 } from "./utf8";

function charToNumicValue(char: i32): u8 {
  // 0-9
  if (char >= 48 && char <= 57) return (char - 48) as u8;
  // A-F
  if (char >= 65 && char <= 70) return (char - 65 + 10) as u8;
  // a-f
  if (char >= 97 && char <= 102) return (char - 97 + 10) as u8;

  throw new Error("Not a valid hex character");
}

export class Hex {
  public static encode(data: Uint8Array): string {
    const out = new Uint8Array(data.length * 2); // preallocate all space we will need
    for (let i = 0; i < data.length; i++) {
      const leftFourBits = data[i] >> 4;
      const rightFourBits = data[i] & 0x0f;
      // 0-9 map to ASCII bytes 48-57, 10-15 map to ASCII bytes 97-102
      out[i * 2 + 0] = leftFourBits < 10 ? leftFourBits + 48 : leftFourBits - 10 + 97;
      out[i * 2 + 1] = rightFourBits < 10 ? rightFourBits + 48 : rightFourBits - 10 + 97;
    }
    return fromUtf8(out);
  }

  public static decode(hex: string): Uint8Array {
    if (hex.length % 2 !== 0) throw new Error("Hex string must be odd");
    const out = new Uint8Array(hex.length / 2);
    for (let i = 0; i < out.length; i++) {
      const left = charToNumicValue(hex.charCodeAt(i * 2));
      const right = charToNumicValue(hex.charCodeAt(i * 2 + 1));
      out[i] = (left << 4) | right;
    }
    return out;
  }
}
