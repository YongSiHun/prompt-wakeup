const fs = require("node:fs/promises");
const path = require("node:path");
const cp = require("node:child_process");
const assert = require("node:assert/strict");
const test = require("node:test");
const argumentsByCli = {
  codex: ["exec", "-"],
  claude: ["-p"],
  gemini: ["-p", "Execute the task supplied on stdin"],
};
const prompt =
  '한글 "따옴표" & echo HACKED > pwned\n$(touch pwned) %PATH% ! ^ | < >\n둘째 줄';

async function fixture() {
  const root = await fs.mkdtemp(path.join(__dirname, ".shim-"));
  const directory = path.join(root, "한글 폴더");
  await fs.mkdir(directory);
  const source =
    "const fs=require('node:fs');const c=[];process.stdin.on('data',d=>c.push(d));process.stdin.on('end',()=>{fs.writeFileSync('received.txt',Buffer.concat(c));fs.writeFileSync('arguments.json',JSON.stringify(process.argv.slice(2)));process.exit(Number(process.env.FIXTURE_EXIT))})";
  await fs.writeFile(path.join(directory, "fixture.cjs"), source);
  await fs.writeFile(
    path.join(directory, "codex.cmd"),
    '@echo off\r\n"' + process.execPath + '" "%~dp0fixture.cjs" %*\r\n',
  );
  return { root, directory };
}
async function cleanup(directory) {
  const target = await fs.realpath(directory);
  assert.equal(path.dirname(target), await fs.realpath(__dirname));
  assert.ok(path.basename(target).startsWith(".shim-"));
  await fs.rm(target, { recursive: true, force: true });
}
function launch(directory, args, exitCode) {
  const shim = path.join(directory, "codex.cmd");
  const command =
    '""' + shim + '" ' + args.map((value) => '"' + value + '"').join(" ") + '"';
  return new Promise((resolve, reject) => {
    const child = cp.spawn(
      path.join(process.env.SystemRoot, "System32/cmd.exe"),
      ["/d", "/s", "/c", command],
      {
        cwd: directory,
        env: { ...process.env, FIXTURE_EXIT: String(exitCode) },
        windowsHide: true,
        windowsVerbatimArguments: true,
        timeout: 10000,
      },
    );
    child.on("error", reject);
    child.stdout.resume();
    child.stderr.resume();
    child.stdin.on("error", reject);
    child.stdin.end(Buffer.from(prompt));
    child.on("close", resolve);
  });
}
async function assertTransport(directory, args, exitCode) {
  assert.equal(await launch(directory, args, exitCode), exitCode);
  assert.deepEqual(
    JSON.parse(
      await fs.readFile(path.join(directory, "arguments.json"), "utf8"),
    ),
    args,
  );
  assert.equal(
    await fs.readFile(path.join(directory, "received.txt"), "utf8"),
    prompt,
  );
  await assert.rejects(fs.access(path.join(directory, "pwned")), {
    code: "ENOENT",
  });
}
for (const [cli, args] of Object.entries(argumentsByCli)) {
  for (const exitCode of [0, 7, 13]) {
    test(
      cli + " shim preserves arguments/stdin; exit " + exitCode,
      { skip: process.platform !== "win32" },
      async (context) => {
        const data = await fixture();
        context.after(() => cleanup(data.root));
        await assertTransport(data.directory, args, exitCode);
      },
    );
  }
}
