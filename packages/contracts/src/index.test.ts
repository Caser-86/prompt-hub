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

  it("creates manual drafts through the approved command boundary", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return { id: "draft-1" };
    });
    const draft = {
      title: "代码审查",
      body: "审查当前变更",
      description: null,
      category: "开发",
      tags: ["审查"],
    };

    await expect(client.createManualPromptDraft(draft)).resolves.toEqual({ id: "draft-1" });
    expect(calls).toEqual([{ command: "create_manual_prompt_draft", args: { draft } }]);
  });

  it("searches with explicit pagination through the approved command boundary", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return {
        hits: [
          {
            id: "search-result-1",
            title: "代码审查",
            snippet: "审查当前变更",
            status: "published",
            effectiveness: "effective",
            rating: 5,
            updatedAt: "2026-07-15T00:00:00Z",
          },
        ],
        total: 1,
      };
    });

    await expect(client.searchPrompts("审查", 10, 20)).resolves.toMatchObject({ total: 1 });
    expect(calls).toEqual([
      { command: "search_prompts", args: { text: "审查", limit: 10, offset: 20 } },
    ]);
  });
});
