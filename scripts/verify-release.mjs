import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { join, relative, resolve } from "node:path";

function git(root, args) {
  try {
    return execFileSync("git", args, { cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
  } catch (error) {
    const detail = error?.stderr?.toString().trim();
    throw new Error(detail || `git ${args.join(" ")} failed`);
  }
}

export function verifyVersions(versions) {
  const values = Object.values(versions);
  if (values.some((value) => typeof value !== "string" || value.length === 0) || new Set(values).size !== 1) {
    throw new Error(`version mismatch: ${JSON.stringify(versions)}`);
  }
  return values[0];
}

export function hashMigrationManifest(files) {
  const hash = createHash("sha256");
  for (const file of [...files].sort((a, b) => a.path.localeCompare(b.path))) {
    hash.update(file.path);
    hash.update("\0");
    hash.update(file.contents);
    hash.update("\0");
  }
  return hash.digest("hex");
}

function readVersions(root) {
  const rootPackage = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
  const desktopPackage = JSON.parse(readFileSync(join(root, "apps/desktop/package.json"), "utf8"));
  const tauri = JSON.parse(readFileSync(join(root, "apps/desktop/src-tauri/tauri.conf.json"), "utf8"));
  const cargo = readFileSync(join(root, "Cargo.toml"), "utf8").match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  return { root: rootPackage.version, desktop: desktopPackage.version, tauri: tauri.version, cargo };
}

function readMigrationManifest(root) {
  const directory = join(root, "crates/prompt-store/migrations");
  return readdirSync(directory)
    .filter((name) => name.endsWith(".sql"))
    .sort()
    .map((name) => ({ path: relative(root, join(directory, name)).replaceAll("\\", "/"), contents: readFileSync(join(directory, name), "utf8") }));
}

export function verifyRelease({ root = process.cwd(), channel = "candidate", jsonOut } = {}) {
  if (!["candidate", "release"].includes(channel)) throw new Error("channel must be candidate or release");
  const resolvedRoot = resolve(root);
  const version = verifyVersions(readVersions(resolvedRoot));
  if (git(resolvedRoot, ["status", "--porcelain"])) throw new Error("working tree must be clean");

  const tag = `v${version}`;
  let tagCommit = null;
  if (channel === "release") {
    if (git(resolvedRoot, ["cat-file", "-t", `refs/tags/${tag}`]) !== "tag") {
      throw new Error(`release requires annotated tag ${tag}`);
    }
    tagCommit = git(resolvedRoot, ["rev-parse", `refs/tags/${tag}^{commit}`]);
    try {
      execFileSync("git", ["show-ref", "--verify", "--quiet", "refs/remotes/origin/main"], { cwd: resolvedRoot, stdio: "ignore" });
    } catch {
      throw new Error("release requires origin/main");
    }
    try {
      execFileSync("git", ["merge-base", "--is-ancestor", tagCommit, "origin/main"], { cwd: resolvedRoot, stdio: "ignore" });
    } catch {
      throw new Error(`${tag} is not an ancestor of origin/main`);
    }
  }

  const result = {
    version,
    channel,
    gitCommit: git(resolvedRoot, ["rev-parse", "HEAD"]),
    tagCommit,
    migrationManifestSha256: hashMigrationManifest(readMigrationManifest(resolvedRoot)),
    builtAt: new Date().toISOString(),
  };
  if (jsonOut) {
    const output = resolve(resolvedRoot, jsonOut);
    mkdirSync(resolve(output, ".."), { recursive: true });
    writeFileSync(output, `${JSON.stringify(result, null, 2)}\n`, "utf8");
  }
  return result;
}

function parseArgs(argv) {
  const options = { channel: "candidate", jsonOut: undefined };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument.startsWith("--channel=")) options.channel = argument.slice("--channel=".length);
    else if (argument === "--channel") options.channel = argv[++index];
    else if (argument.startsWith("--json-out=")) options.jsonOut = argument.slice("--json-out=".length);
    else if (argument === "--json-out") options.jsonOut = argv[++index];
    else throw new Error(`unknown option ${argument}`);
  }
  return options;
}

const entry = process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url;
if (entry) {
  try {
    const result = verifyRelease(parseArgs(process.argv.slice(2)));
    process.stdout.write(`${JSON.stringify(result, null, 2)}\n`);
  } catch (error) {
    process.stderr.write(`Release verification failed: ${error.message}\n`);
    process.exitCode = 1;
  }
}
