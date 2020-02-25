import { JSONDecoder, JSONHandler } from "assemblyscript-json";

/**
 * Types as defined in RFC 8259
 *
 * @see https://tools.ietf.org/html/rfc8259
 */
enum JsonType {
  // primitive types
  String,
  Number,
  Boolean,
  Null,
  // structured types
  Array,
  Object,
}

export abstract class JsonValue {
  public constructor(public readonly type: JsonType) {}

  public asString(): JsonString {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonString(this)) {
      throw new Error("Expected JSON type string, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonString;
  }

  public asNumber(): JsonNumber {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonNumber(this)) {
      throw new Error("Expected JSON type number, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonNumber;
  }

  public asBoolean(): JsonBoolean {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonBoolean(this)) {
      throw new Error("Expected JSON type boolean, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonBoolean;
  }

  public asNull(): JsonNull {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonNull(this)) {
      throw new Error("Expected JSON type null, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonNull;
  }

  public asArray(): JsonArray {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonArray(this)) {
      throw new Error("Expected JSON type array, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonArray;
  }

  public asObject(): JsonObject {
    // eslint-disable-next-line @typescript-eslint/no-use-before-define
    if (!isJsonObject(this)) {
      throw new Error("Expected JSON type object, but got " + this.type.toString());
    }
    // @ts-ignore insufficient overlap for cast
    return this as JsonObject;
  }
}

export class JsonString extends JsonValue {
  public constructor(private readonly data: string) {
    super(JsonType.String);
  }

  public toString(): string {
    return this.data;
  }
}

export class JsonNumber extends JsonValue {
  public constructor(private readonly data: i64) {
    super(JsonType.Number);
  }

  public toI64(): i64 {
    return this.data;
  }
}

export class JsonBoolean extends JsonValue {
  public constructor(private readonly data: bool) {
    super(JsonType.Boolean);
  }

  public toBool(): bool {
    return this.data;
  }
}

export class JsonNull extends JsonValue {
  public constructor() {
    super(JsonType.Null);
  }
}

export class JsonArray extends JsonValue {
  private readonly data: Array<JsonValue> = [];

  public constructor() {
    super(JsonType.Array);
  }

  public get length(): usize {
    return this.data.length;
  }

  public get(key: i32): JsonValue {
    return this.data[key];
  }

  public push(element: JsonValue): void {
    this.data.push(element);
  }
}

export class JsonObject extends JsonValue {
  private readonly data: Map<string, JsonValue> = new Map<string, JsonValue>();

  public constructor() {
    super(JsonType.Object);
  }

  public get size(): usize {
    return this.data.size;
  }

  public set(key: string, value: JsonValue): void {
    if (!key) throw new Error("JsonObject.set was called with empty key");
    this.data.set(key, value);
  }

  public has(key: string): bool {
    return this.data.has(key);
  }

  public get(key: string): JsonValue {
    return this.data.get(key);
  }
}

type JsonStringCheck = boolean; // shoule be `value is JsonString`
type JsonNumberCheck = boolean; // shoule be `value is JsonNumber`
type JsonBooleanCheck = boolean; // shoule be `value is JsonBoolean`
type JsonNullCheck = boolean; // shoule be `value is JsonNull`
type JsonArrayCheck = boolean; // shoule be `value is JsonArray`
type JsonObjectCheck = boolean; // shoule be `value is JsonObject`

export function isJsonString(value: JsonValue): JsonStringCheck {
  return value.type === JsonType.String;
}

export function isJsonNumber(value: JsonValue): JsonNumberCheck {
  return value.type === JsonType.Number;
}

export function isJsonBoolean(value: JsonValue): JsonBooleanCheck {
  return value.type === JsonType.Boolean;
}

export function isJsonNull(value: JsonValue): JsonNullCheck {
  return value.type === JsonType.Null;
}

export function isJsonArray(value: JsonValue): JsonArrayCheck {
  return value.type === JsonType.Array;
}

export function isJsonObject(value: JsonValue): JsonObjectCheck {
  return value.type === JsonType.Object;
}

export class ReadAllHandler extends JSONHandler {
  private readonly stack: Array<JsonValue> = [];

  get root(): JsonValue {
    return this.stack[0];
  }

  get top(): JsonValue | null {
    if (this.stack.length < 1) return null;
    else return this.stack[this.stack.length - 1];
  }

  // begin implementation of JSONHandler

  public setString(name: string, value: string): void {
    const obj = new JsonString(value);
    this.addValue(name, obj);
  }

  public setBoolean(name: string, value: bool): void {
    const obj = new JsonBoolean(value);
    this.addValue(name, obj);
  }

  public setNull(name: string): void {
    const obj = new JsonNull();
    this.addValue(name, obj);
  }

  public setInteger(name: string, value: i64): void {
    const obj = new JsonNumber(value);
    this.addValue(name, obj);
  }

  public pushArray(name: string): bool {
    const obj = new JsonArray();
    this.addValue(name, obj);
    this.stack.push(obj);
    return true;
  }

  public popArray(): void {
    if (this.stack.length > 1) {
      this.stack.pop();
    }
  }

  public pushObject(name: string): bool {
    const obj = new JsonObject();
    this.addValue(name, obj);
    this.stack.push(obj);
    return true;
  }

  public popObject(): void {
    if (this.stack.length > 1) {
      this.stack.pop();
    }
  }

  // end implementation of JSONHandler

  private addValue(name: string, obj: JsonValue): void {
    const top = this.top;
    if (top) {
      if (isJsonArray(top)) {
        (top as JsonArray).push(obj);
      } else if (isJsonObject(top)) {
        (top as JsonObject).set(name, obj);
      } else {
        // hmm, what does this case mean?
      }
    } else {
      this.stack.push(obj);
    }
  }
}

export function parse(jsonString: Uint8Array): JsonValue {
  const handler = new ReadAllHandler();
  const decoder = new JSONDecoder<ReadAllHandler>(handler);
  decoder.deserialize(jsonString);
  return handler.root;
}
