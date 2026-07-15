export type PromptProvenance = {
  name: string;
  location: string;
  collectedAt: string;
};

const trailingProvenance = /\n{2,}参考来源[：:]\s*(?<name>[^\n]+)\s*\n(?<location>https?:\/\/\S+)\s*\n采集时间[：:]\s*(?<collectedAt>\d{4}-\d{2}-\d{2})\s*$/u;

export function separatePromptProvenance(value: string): { body: string; provenance?: PromptProvenance } {
  const match = trailingProvenance.exec(value);
  if (!match?.groups) return { body: value };

  return {
    body: value.slice(0, match.index).trimEnd(),
    provenance: {
      name: match.groups.name.trim(),
      location: match.groups.location.trim(),
      collectedAt: match.groups.collectedAt,
    },
  };
}
