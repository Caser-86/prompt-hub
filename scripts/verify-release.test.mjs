import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { hashMigrationManifest, verifyVersions } from "./verify-release.mjs";

describe("verify-release", () => {
  it("rejects mismatched package versions", () => {
    assert.throws(
      () => verifyVersions({ root: "0.1.10", desktop: "0.1.9", tauri: "0.1.10", cargo: "0.1.10" }),
      /version mismatch/,
    );
  });

  it("hashes migration definitions in a deterministic order", () => {
    const first = hashMigrationManifest([
      { path: "0002.sql", contents: "two" },
      { path: "0001.sql", contents: "one" },
    ]);
    const second = hashMigrationManifest([
      { path: "0001.sql", contents: "one" },
      { path: "0002.sql", contents: "two" },
    ]);
    assert.equal(first, second);
    assert.equal(first.length, 64);
  });
});
