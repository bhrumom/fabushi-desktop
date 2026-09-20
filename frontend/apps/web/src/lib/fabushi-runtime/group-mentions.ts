export interface GroupMentionTargets {
  readonly isEveryone: boolean;
  readonly memberIds: readonly string[];
}

function mentionAliases(name: string): string[] {
  const normalized = name.normalize("NFKC").trim().toLocaleLowerCase();
  if (!normalized) return [];
  const words = normalized.split(/\s+/u).filter(Boolean);
  return [...new Set([
    normalized,
    words.join(""),
    words.join("-"),
    words[0] ?? "",
  ].filter(Boolean))];
}

function boundaryMatch(text: string, alias: string): boolean {
  const needle = `@${alias}`;
  let cursor = text.indexOf(needle);
  while (cursor >= 0) {
    const before = text[cursor - 1] ?? "";
    const after = text[cursor + needle.length] ?? "";
    const beforeWord = /[\p{L}\p{N}_]/u.test(before);
    const afterWord = /[\p{L}\p{N}_]/u.test(after);
    if (!beforeWord && !afterWord) return true;
    cursor = text.indexOf(needle, cursor + needle.length);
  }
  return false;
}

export function parseGroupMentions(
  rawText: string,
  members: readonly { readonly id: string; readonly name: string }[],
): GroupMentionTargets {
  const text = rawText.normalize("NFKC").toLocaleLowerCase();
  const isEveryone = /(^|[^\p{L}\p{N}_])@(everyone|all|全体|所有人)(?=$|[^\p{L}\p{N}_])/u.test(text);
  const memberIds: string[] = [];
  const emitted = new Set<string>();

  for (const member of members) {
    if (emitted.has(member.id)) continue;
    if (mentionAliases(member.name).some((alias) => boundaryMatch(text, alias))) {
      emitted.add(member.id);
      memberIds.push(member.id);
    }
  }

  return { isEveryone, memberIds };
}
