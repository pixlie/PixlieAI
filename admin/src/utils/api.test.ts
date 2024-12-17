// We test the snakeCasedKeys and camelCasedKeys functions
import { snakeCasedKeys, camelCasedKeys } from "./api";
import { describe, expect, it } from "vitest";

describe("snakeCasedKeys", () => {
  it("should convert keys to snake case", () => {
    const obj = {
      helloWorld: "hello world",
      hello_world: "hello world",
      helloWorld123: "hello world",
      hello_world123: "hello world",
    };
    const expectedObj = {
      hello_world: "hello world",
      hello_world123: "hello world",
    };

    expect(snakeCasedKeys(obj)).toEqual(expectedObj);
  });

  it("should not convert keys that are already in snake case", () => {
    const obj = {
      hello_world: "hello world",
      hello_world123: "hello world",
    };
    const expectedObj = {
      hello_world: "hello world",
      hello_world123: "hello world",
    };

    expect(snakeCasedKeys(obj)).toEqual(expectedObj);
  });

  it("should convert keys in arrays", () => {
    const obj = [
      { helloWorld: "hello world" },
      { hello_world: "hello world" },
      { helloWorld123: "hello world" },
      { hello_world123: "hello world" },
    ];
    const expectedObj = [
      { hello_world: "hello world" },
      { hello_world: "hello world" },
      {
        hello_world123: "hello world",
      },
      {
        hello_world123: "hello world",
      },
    ];

    expect(snakeCasedKeys(obj)).toEqual(expectedObj);
  });
});

describe("camelCasedKeys", () => {
  it("should convert keys to camel case", () => {
    const obj = {
      hello_world: "hello world",
      hello_world123: "hello world",
      helloWorld: "hello world",
      helloWorld123: "hello world",
    };
    const expectedObj = {
      helloWorld: "hello world",
      helloWorld123: "hello world",
    };

    expect(camelCasedKeys(obj)).toEqual(expectedObj);
  });

  it("should not convert keys that are already in camel case", () => {
    const obj = {
      helloWorld: "hello world",
      helloWorld123: "hello world",
    };
    const expectedObj = {
      helloWorld: "hello world",
      helloWorld123: "hello world",
    };

    expect(camelCasedKeys(obj)).toEqual(expectedObj);
  });

  it("should convert keys in arrays", () => {
    const obj = [
      { hello_world: "hello world" },
      { hello_world123: "hello world" },
      { helloWorld: "hello world" },
      { helloWorld123: "hello world" },
    ];
    const expectedObj = [
      { helloWorld: "hello world" },
      { helloWorld123: "hello world" },
      { helloWorld: "hello world" },
      { helloWorld123: "hello world" },
    ];

    expect(camelCasedKeys(obj)).toEqual(expectedObj);
  });
});
