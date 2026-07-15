import { describe, expect, it } from "vitest";

import { createDesktopCommandClient } from "./index";

describe("desktop command client", () => {
  it("uses the stable service command and returns its typed status payload", async () => {
    const calls: string[] = [];
    const client = createDesktopCommandClient(async (command) => {
      calls.push(command);
      return {
        appVersion: "0.1.0",
        databaseSchemaVersion: 2,
        offlineCapable: true,
      };
    });

    await expect(client.getApplicationStatus()).resolves.toEqual({
      appVersion: "0.1.0",
      databaseSchemaVersion: 2,
      offlineCapable: true,
    });
    expect(calls).toEqual(["get_application_status"]);
  });

  it("uses the service-only list command for prompt library reads", async () => {
    const calls: string[] = [];
    const client = createDesktopCommandClient(async (command) => {
      calls.push(command);
      return [];
    });

    await expect(client.listPrompts()).resolves.toEqual([]);
    expect(calls).toEqual(["list_prompts"]);
  });
});
