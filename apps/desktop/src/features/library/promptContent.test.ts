import { describe, expect, it } from "vitest";

import { separatePromptProvenance } from "./promptContent";

describe("separatePromptProvenance", () => {
  it("keeps a trailing collected-source block out of the copyable prompt body", () => {
    const result = separatePromptProvenance([
      "请将会议记录整理为行动项。",
      "",
      "参考来源： Google Prompt design strategies",
      "https://ai.google.dev/gemini-api/docs/prompting-strategies",
      "采集时间： 2026-07-16",
    ].join("\n"));

    expect(result.body).toBe("请将会议记录整理为行动项。");
    expect(result.provenance).toEqual({
      name: "Google Prompt design strategies",
      location: "https://ai.google.dev/gemini-api/docs/prompting-strategies",
      collectedAt: "2026-07-16",
    });
  });
});
