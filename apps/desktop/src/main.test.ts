import { readFileSync } from "node:fs";
import { resolve } from "node:path";

import { describe, expect, it } from "vitest";

describe("desktop entrypoint", () => {
  it("loads the application stylesheet", () => {
    const source = readFileSync(resolve(process.cwd(), "src/main.tsx"), "utf8");

    expect(source).toContain('import "./styles.css";');
  });
});
