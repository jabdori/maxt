import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const packageJson = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);

const nativeTargets = [
  "aarch64-apple-darwin",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

const optionalDependencies = {
  "@jabdori/maxt-darwin-arm64": "0.1.0",
  "@jabdori/maxt-darwin-x64": "0.1.0",
  "@jabdori/maxt-linux-arm64-gnu": "0.1.0",
  "@jabdori/maxt-linux-x64-gnu": "0.1.0",
  "@jabdori/maxt-win32-x64-msvc": "0.1.0",
};

test("publishes the @jabdori/maxt Node package identity", () => {
  assert.equal(packageJson.name, "@jabdori/maxt");
  assert.equal(packageJson.version, "0.1.0");
  assert.equal(packageJson.type, "module");
  assert.equal(packageJson.license, "MIT");
  assert.equal(packageJson.repository?.url, "https://github.com/jabdori/maxt.git");
  assert.deepEqual(packageJson.engines, { node: ">=22.0.0" });
  assert.deepEqual(packageJson.files, [
    "dist/",
    "native.cjs",
    "native.d.ts",
    "README.md",
    "LICENSE",
  ]);
});

test("exports only the initial Node entry points", () => {
  assert.deepEqual(Object.keys(packageJson.exports).sort(), [
    ".",
    "./node",
    "./package.json",
  ]);

  for (const entry of [".", "./node"]) {
    assert.equal(packageJson.exports[entry].types, "./dist/node.d.ts");
    assert.equal(packageJson.exports[entry].import, "./dist/node.js");
    assert.equal(packageJson.exports[entry].default, "./dist/node.js");
  }
  assert.equal(packageJson.exports["./package.json"], "./package.json");
});

test("pins the TypeScript and napi-rs toolchain", () => {
  assert.deepEqual(packageJson.devDependencies, {
    "@napi-rs/cli": "3.8.2",
    "@types/node": "24.13.3",
    typescript: "7.0.2",
  });
  assert.equal(packageJson.scripts.build, "tsc -p tsconfig.json");
  assert.equal(
    packageJson.scripts.typecheck,
    "tsc -p tsconfig.json --noEmit --rootDir .",
  );
  assert.equal(
    packageJson.scripts["build:node"],
    "napi build --manifest-path rust/Cargo.toml --platform --output-dir . --js native.cjs --dts native.d.ts --js-package-name @jabdori/maxt -- --locked",
  );
  for (const script of [
    "test:unit",
    "test:node",
    "test",
    "package:dirs",
    "artifacts",
  ]) {
    assert.equal(typeof packageJson.scripts[script], "string");
    assert.notEqual(packageJson.scripts[script], "");
  }
});

test("supports exactly five native packages and Rust targets", () => {
  assert.deepEqual(packageJson.optionalDependencies, optionalDependencies);
  assert.equal(packageJson.napi?.binaryName, "maxt");
  assert.deepEqual([...packageJson.napi.targets].sort(), nativeTargets);
});
