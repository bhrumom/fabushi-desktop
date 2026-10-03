export type InstructionBehavior = "allow" | "ask";

export interface AutoReviewInstructions {
  allowInstructions: string[];
  blockInstructions: string[];
}

export interface InstructionRow {
  behavior: InstructionBehavior;
  text: string;
  listIndex: number;
}

export const MAX_INSTRUCTIONS_PER_BEHAVIOR = 20;

export function parseCursorAuthId(
  authId: string | null | undefined,
): { subject: string } | null {
  if (authId == null) return null;
  const separator = authId.indexOf("|");
  if (separator <= 0 || separator >= authId.length - 1) return null;
  return { subject: authId.slice(separator + 1) };
}

export function instructionRows(
  instructions: AutoReviewInstructions,
): InstructionRow[] {
  const rows: InstructionRow[] = [];
  instructions.allowInstructions.forEach((text, listIndex) => {
    rows.push({ behavior: "allow", text, listIndex });
  });
  instructions.blockInstructions.forEach((text, listIndex) => {
    rows.push({ behavior: "ask", text, listIndex });
  });
  return rows;
}

function listKey(behavior: InstructionBehavior): keyof AutoReviewInstructions {
  return behavior === "allow" ? "allowInstructions" : "blockInstructions";
}

export function removeInstruction(
  instructions: AutoReviewInstructions,
  row: InstructionRow,
): AutoReviewInstructions {
  const key = listKey(row.behavior);
  return {
    ...instructions,
    [key]: instructions[key].filter((_, index) => index !== row.listIndex),
  };
}

export function reconcileInstructionRow(
  instructions: AutoReviewInstructions,
  row: InstructionRow,
): InstructionRow | null {
  const list = instructions[listKey(row.behavior)];
  const index = list.indexOf(row.text);
  if (index < 0) return null;
  return index === row.listIndex ? row : { ...row, listIndex: index };
}

export function saveInstruction(
  instructions: AutoReviewInstructions,
  text: string,
  behavior: InstructionBehavior,
  editing: InstructionRow | null,
): AutoReviewInstructions | null {
  const key = listKey(behavior);
  const target = instructions[key];
  const sameList = editing?.behavior === behavior;
  const duplicate = target.some(
    (item, index) => item === text && !(sameList && index === editing?.listIndex),
  );
  if (duplicate) return null;

  if (editing == null) {
    if (target.length >= MAX_INSTRUCTIONS_PER_BEHAVIOR) return null;
    return { ...instructions, [key]: [...target, text] };
  }

  if (sameList) {
    if (editing.listIndex < 0 || editing.listIndex >= target.length) return null;
    const next = [...target];
    next[editing.listIndex] = text;
    return { ...instructions, [key]: next };
  }

  if (target.length >= MAX_INSTRUCTIONS_PER_BEHAVIOR) return null;
  const withoutPrevious = removeInstruction(instructions, editing);
  return {
    ...withoutPrevious,
    [key]: [...withoutPrevious[key], text],
  };
}
