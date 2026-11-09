const fs = require("node:fs/promises");
const path = require("node:path");
const cp = require("node:child_process");
const assert = require("node:assert/strict");
(async () => {
  if (process.platform !== "win32") throw new Error("Windows only");
  const dir = path.resolve(__dirname, "../artifacts/smoke/한글 폴더");
  await fs.mkdir(dir, { recursive: true });
  const fixture = path.join(dir, "fixture.cjs");
  const shim = path.join(dir, "codex.cmd");
  await fs.writeFile(
    fixture,
    "const fs=require('node:fs');const c=[];process.stdin.on('data',d=>c.push(d));process.stdin.on('end',()=>{fs.writeFileSync('received.txt',Buffer.concat(c));fs.writeFileSync('arguments.json',JSON.stringify(process.argv.slice(2)));process.exit(Number(process.env.FIXTURE_EXIT))})",
  );
  await fs.writeFile(
    shim,
    '@echo off\r\n"' + process.execPath + '" "%~dp0fixture.cjs" %*\r\n',
  );
  const prompt =
    '한글 "따옴표" & echo HACKED > pwned\n$(touch pwned) %PATH% ! ^ | < >\n둘째 줄';
  for (const args of [
    ["exec", "-"],
    ["-p"],
    ["-p", "Execute the task supplied on stdin"],
  ])
    for (const expected of [0, 7, 13]) {
      const command =
        '""' + shim + '" ' + args.map((a) => '"' + a + '"').join(" ") + '"';
      const status = await new Promise((resolve, reject) => {
        const child = cp.spawn(
          path.join(process.env.SystemRoot, "System32/cmd.exe"),
          ["/d", "/s", "/c", command],
          {
            cwd: dir,
            env: { ...process.env, FIXTURE_EXIT: String(expected) },
            windowsHide: true,
            windowsVerbatimArguments: true,
            stdio: ["pipe", "pipe", "pipe"],
          },
        );
        child.on("error", reject);
        child.stdout.pipe(process.stdout);
        child.stderr.pipe(process.stderr);
        child.stdin.on("error", reject);
        child.stdin.end(Buffer.from(prompt));
        child.on("close", resolve);
      });
      assert.equal(status, expected);
      assert.deepEqual(
        JSON.parse(await fs.readFile(path.join(dir, "arguments.json"), "utf8")),
        args,
      );
      assert.equal(
        await fs.readFile(path.join(dir, "received.txt"), "utf8"),
        prompt,
      );
      assert.equal(
        await fs.access(path.join(dir, "pwned")).then(
          () => true,
          () => false,
        ),
        false,
      );
    }
  console.log(
    "PASS: Windows cmd shim; Unicode path; stdin exact bytes; exit 0/7/13; no prompt injection",
  );
})().catch((e) => {
  console.error(e);
  process.exitCode = 1;
});
