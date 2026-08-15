import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const packageJson = JSON.parse(
  await readFile(new URL("../package.json", import.meta.url), "utf8"),
);
const tsConfig = JSON.parse(
  await readFile(new URL("../tsconfig.json", import.meta.url), "utf8"),
);

const nativeTargets = [
  "aarch64-apple-darwin",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "x86_64-pc-windows-msvc",
  "x86_64-unknown-linux-gnu",
];

const optionalDependencies = {
  "@jabdori/maxt-darwin-arm64": packageJson.version,
  "@jabdori/maxt-darwin-x64": packageJson.version,
  "@jabdori/maxt-linux-arm64-gnu": packageJson.version,
  "@jabdori/maxt-linux-x64-gnu": packageJson.version,
  "@jabdori/maxt-win32-x64-msvc": packageJson.version,
};

test("publishes the @jabdori/maxt Node and browser package identity", () => {
  assert.equal(packageJson.name, "@jabdori/maxt");
  assert.match(packageJson.version, /^\d+\.\d+\.\d+$/);
  assert.equal(
    packageJson.description,
    "One TypeScript API for Upbit, Bithumb, Binance, and Hyperliquid.",
  );
  assert.equal(packageJson.type, "module");
  assert.equal(packageJson.license, "MIT");
  assert.equal(packageJson.main, "./dist/node.js");
  assert.equal(packageJson.types, "./dist/node.d.ts");
  assert.deepEqual(packageJson.sideEffects, ["./dist/browser.js", "./dist/node.js"]);
  assert.equal(
    packageJson.repository?.url,
    "git+https://github.com/jabdori/maxt.git",
  );
  assert.deepEqual(packageJson.engines, { node: ">=22.0.0" });
  assert.deepEqual(packageJson.files, [
    "dist/",
    "wasm/",
    "native.cjs",
    "native.d.ts",
    "examples/",
    "README.md",
    "README.ko.md",
    "CHANGELOG.md",
    "LICENSE",
  ]);
});

test("exports explicit Node and browser entry points", () => {
  assert.deepEqual(packageJson.exports, {
    ".": {
      types: "./dist/node.d.ts",
      browser: "./dist/browser.js",
      node: "./dist/node.js",
      default: "./dist/node.js",
    },
    "./browser": {
      types: "./dist/browser.d.ts",
      default: "./dist/browser.js",
    },
    "./node": {
      types: "./dist/node.d.ts",
      default: "./dist/node.js",
    },
    "./package.json": "./package.json",
  });
});

test("pins the TypeScript and napi-rs toolchain", () => {
  assert.deepEqual(packageJson.devDependencies, {
    "@napi-rs/cli": "3.8.2",
    "@playwright/test": "1.62.1",
    "@types/node": "24.13.3",
    typescript: "7.0.2",
  });
  assert.equal(
    packageJson.scripts.clean,
    "node -e \"require('node:fs').rmSync('dist',{recursive:true,force:true})\"",
  );
  assert.equal(packageJson.scripts.build, "npm run clean && tsc -p tsconfig.json");
  assert.equal(
    packageJson.scripts["build:browser"],
    "wasm-pack build rust --target web --out-dir ../wasm --out-name maxt_wasm --no-pack && node -e \"require('node:fs').writeFileSync('wasm/.npmignore','')\"",
  );
  assert.match(packageJson.scripts.typecheck, /^tsc -p tsconfig\.json --noEmit/);
  assert.match(packageJson.scripts.typecheck, /node -e/);
  assert.match(packageJson.scripts.typecheck, /--ignoreConfig/);
  assert.match(packageJson.scripts.typecheck, /--strict/);
  assert.match(packageJson.scripts.typecheck, /tests\/types\.ts/);
  assert.doesNotMatch(packageJson.scripts.typecheck, /\bif \[/);
  assert.equal(
    packageJson.scripts["build:node"],
    "napi build --manifest-path rust/Cargo.toml --platform --output-dir . --js native.cjs --dts native.d.ts --js-package-name @jabdori/maxt -- --locked",
  );
  assert.equal(
    packageJson.scripts["test:unit"],
    "npm run build && node --test tests/*.test.mjs",
  );
  assert.equal(
    packageJson.scripts["test:web"],
    "npm run build:browser && npm run build && playwright test -c tests/playwright.config.mjs",
  );
  assert.equal(
    packageJson.scripts["test:node"],
    "npm run build:node && npm run test:unit && npm run test:integration",
  );
  assert.equal(
    packageJson.scripts["test:integration"],
    "node --expose-gc --test tests/node.integration.mjs rust/tests/*.test.cjs",
  );
  assert.equal(
    packageJson.scripts.test,
    "npm run typecheck && npm run test:node",
  );
  assert.equal(packageJson.scripts["package:dirs"], "napi create-npm-dirs");
  assert.equal(packageJson.scripts.artifacts, "napi artifacts");
});

test("compiles the complete checked-in source tree with strict settings", () => {
  assert.deepEqual(tsConfig, {
    compilerOptions: {
      target: "ES2022",
      module: "NodeNext",
      moduleResolution: "NodeNext",
      rootDir: "src",
      outDir: "dist",
      declaration: true,
      strict: true,
      exactOptionalPropertyTypes: true,
      noUncheckedIndexedAccess: true,
      verbatimModuleSyntax: true,
      skipLibCheck: false,
      types: ["node"],
    },
    include: ["src/**/*.ts"],
  });
});

test("supports exactly five native packages and Rust targets", () => {
  assert.deepEqual(packageJson.optionalDependencies, optionalDependencies);
  assert.equal(packageJson.napi?.binaryName, "maxt");
  assert.deepEqual([...packageJson.napi.targets].sort(), nativeTargets);
});
