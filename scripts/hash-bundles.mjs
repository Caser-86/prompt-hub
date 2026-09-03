import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import process from "node:process";

const root = resolve(process.cwd(), process.argv[2] ?? "target/release/bundle");
const outputName = "SHA256SUMS.txt";

function filesUnder(directory) {
  if (!existsSync(directory)) throw new Error(`bundle directory does not exist: ${directory}`);
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path));
    else if (entry.isFile() && entry.name !== outputName) files.push(path);
  }
  return files;
}

const files = filesUnder(root).sort((left, right) => relative(root, left).localeCompare(relative(root, right)));
if (files.length === 0) throw new Error(`no bundle files found in ${root}`);

const lines = files.map((path) => {
  const digest = createHash("sha256").update(readFileSync(path)).digest("hex").toUpperCase();
  return `${digest}  ${relative(root, path).replaceAll("\\", "/")}`;
});
writeFileSync(join(root, outputName), `${lines.join("\n")}\n`, "utf8");
process.stdout.write(`${lines.join("\n")}\n`);
