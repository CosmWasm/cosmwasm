import { Extern } from "./cosmwasm-std";
import { parse } from "./encoding/json";
import { equalUint8Array } from "./encoding/memory";

export function query(extern: Extern, msgJson: Uint8Array): string {
  const msg = parse(msgJson).asObject();
  if (msg.has("balance")) {
    const address = extern.canonicalize(
      msg
        .get("balance")
        .asObject()
        .get("address")
        .asString()
        .toString(),
    );

    let balance: string;

    if (equalUint8Array(address, extern.canonicalize("addr4321"))) balance = "22";
    else balance = "0";

    return '{"balance":"' + balance + '"}';
  } else {
    throw new Error("Unsupported query method");
  }
}
