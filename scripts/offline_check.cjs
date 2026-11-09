const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const assert = require("node:assert/strict");
const test = require("node:test");
const ts = require(process.argv[2] || "typescript");
const root = path.resolve(__dirname, "..");
function walk(dir) {
  return fs
    .readdirSync(dir, { withFileTypes: true })
    .flatMap((e) =>
      e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)],
    );
}
const files = [
  ...walk(path.join(root, "src")),
  ...walk(path.join(root, "tests")),
].filter((p) => /\.tsx?$/.test(p));
for (const file of files) {
  const result = ts.transpileModule(fs.readFileSync(file, "utf8"), {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
      jsx: ts.JsxEmit.ReactJSX,
    },
    fileName: file,
    reportDiagnostics: true,
  });
  assert.equal(result.diagnostics?.length || 0, 0, file);
}
console.log("TypeScript syntax: " + files.length + " files OK");
const compiled = ts.transpileModule(
  fs.readFileSync(path.join(root, "src/lib/time.ts"), "utf8"),
  {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.CommonJS,
    },
  },
).outputText;
const ctx = { exports: {}, Date, Error };
vm.runInNewContext(compiled, ctx);
const { localInput, utcFromLocal, remaining } = ctx.exports;
test("UTC-local seconds roundtrip", () => {
  const d = new Date("2030-04-01T10:22:33Z");
  assert.equal(utcFromLocal(localInput(d)), d.toISOString());
});
test("past schedules rejected", () =>
  assert.throws(() => utcFromLocal("2000-01-01T00:00:00")));
test("invalid datetime rejected", () =>
  assert.throws(() => utcFromLocal("bad")));
test("invalid calendar day rejected", () =>
  assert.throws(() => utcFromLocal("2030-02-30T12:00:00")));
test("seconds remaining", () =>
  assert.equal(
    remaining("2030-01-01T00:00:59Z", Date.parse("2030-01-01T00:00:00Z")),
    "59초",
  ));
test("hours and minutes remaining", () =>
  assert.equal(
    remaining("2030-01-01T01:02:03Z", Date.parse("2030-01-01T00:00:00Z")),
    "1시간 2분 3초",
  ));
test("overdue label", () =>
  assert.equal(
    remaining("2030-01-01T00:00:00Z", Date.parse("2030-01-01T00:00:01Z")),
    "실행 시각 도달",
  ));
