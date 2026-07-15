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

  it("rejects malformed list entries instead of leaking an unstable backend shape", async () => {
    const client = createDesktopCommandClient(async () => [{ id: "prompt-1" }]);

    await expect(client.listPrompts()).rejects.toThrow("list_prompts returned an invalid response");
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
      variables: [],
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

  it("passes structured search filters through the service boundary", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return { hits: [], total: 0 };
    });

    await client.searchPrompts("审查", 20, 0, { effectiveness: "effective", minimumRating: 4 });

    expect(calls).toEqual([
      {
        command: "search_prompts",
        args: {
          text: "审查", limit: 20, offset: 0,
          filters: { effectiveness: "effective", minimumRating: 4 },
        },
      },
    ]);
  });

  it("records compatibility and validation metadata through approved commands", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return { id: "prompt-1" };
    });
    const compatibility = {
      tool: "Codex",
      model: "gpt-5",
      status: "confirmed" as const,
      notes: "已验证",
    };
    const validation = {
      status: "effective" as const,
      rating: 5,
      notes: "输出可用",
    };

    await client.recordPromptCompatibility("prompt-1", compatibility);
    await client.recordPromptValidation("prompt-1", validation);

    expect(calls).toEqual([
      { command: "record_prompt_compatibility", args: { id: "prompt-1", metadata: compatibility } },
      { command: "record_prompt_validation", args: { id: "prompt-1", metadata: validation } },
    ]);
  });

  it("loads immutable history and restores a version through approved commands", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return command === "prompt_history"
        ? [{ number: 1, body: "第一版", createdAt: "2026-07-15T00:00:00Z" }]
        : { id: "prompt-1" };
    });

    await expect(client.promptHistory("prompt-1")).resolves.toHaveLength(1);
    await client.restorePromptVersion("prompt-1", 1);

    expect(calls).toEqual([
      { command: "prompt_history", args: { id: "prompt-1" } },
      { command: "restore_prompt_version", args: { id: "prompt-1", versionNumber: 1 } },
    ]);
  });

  it("uses service commands for archive, soft delete, and recovery", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return { id: "prompt-1" };
    });

    await client.archivePrompt("prompt-1");
    await client.softDeletePrompt("prompt-1");
    await client.recoverPrompt("prompt-1");

    expect(calls).toEqual([
      { command: "archive_prompt", args: { id: "prompt-1" } },
      { command: "soft_delete_prompt", args: { id: "prompt-1" } },
      { command: "recover_prompt", args: { id: "prompt-1" } },
    ]);
  });

  it("archives selected prompts through a dedicated batch command", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => { calls.push({ command, args }); return undefined; });
    await client.batchArchivePrompts(["prompt-1", "prompt-2"]);
    expect(calls).toEqual([{ command: "batch_archive_prompts", args: { ids: ["prompt-1", "prompt-2"] } }]);
  });

  it("creates and previews local backups only through the service boundary", async () => {
    const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
    const client = createDesktopCommandClient(async (command, args) => {
      calls.push({ command, args });
      return command === "create_manual_backup"
        ? { path: "C:/data/backups/manual.db", byteLen: 512, schemaVersion: 2 }
        : { targetExists: true, backupSchemaVersion: 2, backupByteLen: 512 };
    });

    await expect(client.createManualBackup()).resolves.toMatchObject({ schemaVersion: 2 });
    await expect(client.previewBackupRestore("C:/data/backups/manual.db")).resolves.toMatchObject({ targetExists: true });

    expect(calls).toEqual([
      { command: "create_manual_backup", args: undefined },
      { command: "preview_backup_restore", args: { path: "C:/data/backups/manual.db" } },
    ]);
  });
});
