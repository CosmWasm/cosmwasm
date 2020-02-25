import {
  isJsonArray,
  isJsonBoolean,
  isJsonNull,
  isJsonNumber,
  isJsonObject,
  isJsonString,
  JsonArray,
  JsonBoolean,
  JsonNull,
  JsonNumber,
  JsonObject,
  JsonString,
  JsonValue,
  parse,
} from "./json";
import { toUtf8 } from "./utf8";

describe("json", () => {
  describe("JsonString", () => {
    it("can be created", () => {
      const obj1 = new JsonString("foo");
      expect(obj1.toString()).toStrictEqual("foo");
    });
  });

  describe("JsonNumber", () => {
    it("can be created", () => {
      const zero = new JsonNumber(0);
      expect(zero.toI64()).toStrictEqual(0);
      const positive = new JsonNumber(42);
      expect(positive.toI64()).toStrictEqual(42);
      const negative = new JsonNumber(-5);
      expect(negative.toI64()).toStrictEqual(-5);

      // Those values exceed the JavaScript safe integer range but work here
      const min = new JsonNumber(i64.MIN_VALUE);
      expect(min.toI64()).toStrictEqual(-9223372036854775808);
      const max = new JsonNumber(i64.MAX_VALUE);
      expect(max.toI64()).toStrictEqual(9223372036854775807);
    });
  });

  describe("JsonBoolean", () => {
    it("can be created", () => {
      const obj1 = new JsonBoolean(true);
      expect(obj1.toBool()).toStrictEqual(true);
      const obj2 = new JsonBoolean(false);
      expect(obj2.toBool()).toStrictEqual(false);
    });
  });

  describe("JsonNull", () => {
    it("can be created", () => {
      const obj1 = new JsonNull();
      expect(obj1).toBeTruthy();
    });
  });

  describe("JsonArray", () => {
    it("can be created", () => {
      const obj1 = new JsonArray();
      expect(obj1).toBeTruthy();
      expect(obj1.length).toStrictEqual(0);
    });
  });

  describe("JsonObject", () => {
    it("can be created", () => {
      const obj1 = new JsonObject();
      expect(obj1).toBeTruthy();
      expect(obj1.size).toStrictEqual(0);
    });

    it("has working size/set/has/get", () => {
      const uut = new JsonObject();
      expect(uut.size).toStrictEqual(0);

      // add
      uut.set("ab", new JsonNumber(7));
      expect(uut.size).toStrictEqual(1);
      expect(uut.has("ab")).toStrictEqual(true);
      expect(uut.has("ba")).toStrictEqual(false);
      expect(
        uut
          .get("ab")
          .asNumber()
          .toI64(),
      ).toStrictEqual(7);

      // override
      uut.set("ab", new JsonNumber(8));
      expect(uut.size).toStrictEqual(1);
      expect(uut.has("ab")).toStrictEqual(true);
      expect(uut.has("ba")).toStrictEqual(false);
      expect(
        uut
          .get("ab")
          .asNumber()
          .toI64(),
      ).toStrictEqual(8);
    });
  });

  describe("isJsonString", () => {
    it("works", () => {
      expect(isJsonString(new JsonString(""))).toStrictEqual(true);

      expect(isJsonString(new JsonNumber(4))).toStrictEqual(false);
      expect(isJsonString(new JsonNull())).toStrictEqual(false);
      expect(isJsonString(new JsonBoolean(true))).toStrictEqual(false);
      expect(isJsonString(new JsonArray())).toStrictEqual(false);
    });
  });

  describe("isJsonNumber", () => {
    it("works", () => {
      expect(isJsonNumber(new JsonNumber(4))).toStrictEqual(true);

      expect(isJsonNumber(new JsonString(""))).toStrictEqual(false);
      expect(isJsonNumber(new JsonNull())).toStrictEqual(false);
      expect(isJsonNumber(new JsonBoolean(true))).toStrictEqual(false);
      expect(isJsonNumber(new JsonArray())).toStrictEqual(false);
    });
  });

  describe("isJsonBoolean", () => {
    it("works", () => {
      expect(isJsonBoolean(new JsonBoolean(true))).toStrictEqual(true);
      expect(isJsonBoolean(new JsonBoolean(false))).toStrictEqual(true);

      expect(isJsonBoolean(new JsonString(""))).toStrictEqual(false);
      expect(isJsonBoolean(new JsonNull())).toStrictEqual(false);
      expect(isJsonBoolean(new JsonNumber(4))).toStrictEqual(false);
      expect(isJsonBoolean(new JsonArray())).toStrictEqual(false);
    });
  });

  describe("isJsonNull", () => {
    it("works", () => {
      expect(isJsonNull(new JsonNull())).toStrictEqual(true);

      expect(isJsonNull(new JsonBoolean(true))).toStrictEqual(false);
      expect(isJsonNull(new JsonString(""))).toStrictEqual(false);
      expect(isJsonNull(new JsonNumber(4))).toStrictEqual(false);
      expect(isJsonNull(new JsonArray())).toStrictEqual(false);
    });
  });

  describe("isJsonArray", () => {
    it("works", () => {
      expect(isJsonArray(new JsonArray())).toStrictEqual(true);

      expect(isJsonArray(new JsonNull())).toStrictEqual(false);
      expect(isJsonArray(new JsonBoolean(true))).toStrictEqual(false);
      expect(isJsonArray(new JsonString(""))).toStrictEqual(false);
      expect(isJsonArray(new JsonNumber(4))).toStrictEqual(false);
    });
  });

  describe("isJsonObject", () => {
    it("works", () => {
      expect(isJsonObject(new JsonObject())).toStrictEqual(true);

      expect(isJsonObject(new JsonNull())).toStrictEqual(false);
      expect(isJsonObject(new JsonBoolean(true))).toStrictEqual(false);
      expect(isJsonObject(new JsonString(""))).toStrictEqual(false);
      expect(isJsonObject(new JsonNumber(4))).toStrictEqual(false);
      expect(isJsonObject(new JsonArray())).toStrictEqual(false);
    });
  });

  describe("JsonValue", () => {
    describe("asString", () => {
      it("works", () => {
        const value: JsonValue = new JsonString("foo");
        expect(value.asString().toString()).toStrictEqual("foo");
      });
    });

    describe("asNumber", () => {
      it("works", () => {
        const value: JsonValue = new JsonNumber(4);
        expect(value.asNumber().toI64()).toStrictEqual(4);
      });
    });

    describe("asBoolean", () => {
      it("works", () => {
        const value: JsonValue = new JsonBoolean(true);
        expect(value.asBoolean().toBool()).toStrictEqual(true);
      });
    });

    describe("asNull", () => {
      it("works", () => {
        const value: JsonValue = new JsonNull();
        expect(value.asNull()).toBeTruthy();
      });
    });

    describe("asArray", () => {
      it("works", () => {
        const value: JsonValue = new JsonArray();
        expect(value.asArray()).toBeTruthy();
      });
    });

    describe("asObject", () => {
      it("works", () => {
        const value: JsonValue = new JsonObject();
        expect(value.asObject()).toBeTruthy();
      });
    });
  });

  describe("parse", () => {
    it("works for strings", () => {
      const result = parse(toUtf8('"foobar"'));
      expect(isJsonString(result)).toStrictEqual(true);
      expect((result as JsonString).toString()).toStrictEqual("foobar");
    });

    it("works for number", () => {
      const result = parse(toUtf8("123"));
      expect(isJsonNumber(result)).toStrictEqual(true);
      expect((result as JsonNumber).toI64()).toStrictEqual(123);
    });

    it("works for true/false", () => {
      {
        const result = parse(toUtf8("true"));
        expect(isJsonBoolean(result)).toStrictEqual(true);
        expect((result as JsonBoolean).toBool()).toStrictEqual(true);
      }
      {
        const result = parse(toUtf8("false"));
        expect(isJsonBoolean(result)).toStrictEqual(true);
        expect((result as JsonBoolean).toBool()).toStrictEqual(false);
      }
    });

    it("works for null", () => {
      const result = parse(toUtf8("null"));
      expect(isJsonNull(result)).toStrictEqual(true);
    });

    // Arrays

    it("works for empty array", () => {
      const result = parse(toUtf8("[]"));
      expect(isJsonArray(result)).toStrictEqual(true);
      expect((result as JsonArray).length).toStrictEqual(0);
    });

    it("works for array of numbers", () => {
      const result = parse(toUtf8("[1, 2, 3]"));
      expect(isJsonArray(result)).toStrictEqual(true);
      expect((result as JsonArray).length).toStrictEqual(3);
      expect(((result as JsonArray).get(0) as JsonNumber).toI64()).toStrictEqual(1);
      expect(((result as JsonArray).get(1) as JsonNumber).toI64()).toStrictEqual(2);
      expect(((result as JsonArray).get(2) as JsonNumber).toI64()).toStrictEqual(3);
    });

    it("works for array of strings", () => {
      const result = parse(toUtf8('["a", "2", ""]'));
      expect(isJsonArray(result)).toStrictEqual(true);
      expect((result as JsonArray).length).toStrictEqual(3);
      expect(((result as JsonArray).get(0) as JsonString).toString()).toStrictEqual("a");
      expect(((result as JsonArray).get(1) as JsonString).toString()).toStrictEqual("2");
      expect(((result as JsonArray).get(2) as JsonString).toString()).toStrictEqual("");
    });

    it("works for array of arrays", () => {
      const result = parse(toUtf8("[[], [1], [2, 3]]"));
      expect(isJsonArray(result)).toStrictEqual(true);
      expect((result as JsonArray).length).toStrictEqual(3);
      const inner0 = (result as JsonArray).get(0) as JsonArray;
      const inner1 = (result as JsonArray).get(1) as JsonArray;
      const inner2 = (result as JsonArray).get(2) as JsonArray;
      expect(inner0.length).toStrictEqual(0);
      expect(inner1.length).toStrictEqual(1);
      expect(inner2.length).toStrictEqual(2);
      expect((inner1.get(0) as JsonNumber).toI64()).toStrictEqual(1);
      expect((inner2.get(0) as JsonNumber).toI64()).toStrictEqual(2);
      expect((inner2.get(1) as JsonNumber).toI64()).toStrictEqual(3);
    });

    // Objects

    it("works for empty object", () => {
      const result = parse(toUtf8("{}"));
      expect(isJsonObject(result)).toStrictEqual(true);
      expect((result as JsonObject).size).toStrictEqual(0);
    });

    it("works for object with one field", () => {
      const result = parse(toUtf8('{"foo": true}'));
      expect(isJsonObject(result)).toStrictEqual(true);
      expect((result as JsonObject).size).toStrictEqual(1);
      expect((result as JsonObject).get("foo")).toStrictEqual(new JsonBoolean(true));
    });

    it("works for object with two fields", () => {
      const result = parse(toUtf8('{"foo": true, "bar": false}'));
      expect(isJsonObject(result)).toStrictEqual(true);
      expect((result as JsonObject).size).toStrictEqual(2);
      expect((result as JsonObject).get("foo")).toStrictEqual(new JsonBoolean(true));
      expect((result as JsonObject).get("bar")).toStrictEqual(new JsonBoolean(false));
    });

    it("works for object with nested fields", () => {
      const result = parse(toUtf8('{"foo": {"bar": 42}}'));
      expect(isJsonObject(result)).toStrictEqual(true);
      expect((result as JsonObject).size).toStrictEqual(1);
      const foo = (result as JsonObject).get("foo");
      expect(isJsonObject(foo)).toStrictEqual(true);
      expect((foo as JsonObject).size).toStrictEqual(1);
      expect((foo as JsonObject).get("bar")).toStrictEqual(new JsonNumber(42));
    });

    it("has nice API to read known nested field", () => {
      const parsed = parse(toUtf8('{"balance":{"address":"addr4321"}}'));
      const address = parsed
        .asObject()
        .get("balance")
        .asObject()
        .get("address")
        .asString();
      expect(address.toString()).toStrictEqual("addr4321");
    });
  });
});
