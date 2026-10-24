import { useState } from "react";
// @evidence src/app/dist/renderer/assets/index-BlqerJhg.js#L1
import {
  MAX_INSTRUCTIONS_PER_BEHAVIOR,
  instructionRows,
  removeInstruction,
  saveInstruction,
  type AutoReviewInstructions,
  type InstructionBehavior,
  type InstructionRow
} from "./model";
import { SandButton } from "../../../ui/sand-kit-primitives";
import { settingsCopy, type SettingsAdvancedCopy } from "./localization";
import { SandSelect } from "../../../ui/sand-floating-primitives";
import { SandSwitch, SandTextField, SandTextarea } from "../../../ui/sand-form-primitives";
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=578220 (shared select trigger)

export interface AutoReviewSettings extends AutoReviewInstructions {
  isEnabled: boolean;
}

export interface AutoReviewRulesPanelProps {
  disabled?: boolean;
  settings: AutoReviewSettings;
  onChange(settings: AutoReviewSettings): void | Promise<unknown>;
  copy?: SettingsAdvancedCopy;
}

export function AutoReviewRulesPanel({ disabled = false, settings, onChange, copy }: AutoReviewRulesPanelProps) {
  const text = copy ?? settingsCopy("en").advancedSettings;
  const [savePending, setSavePending] = useState(false);
  const [draft, setDraft] = useState("");
  const [behavior, setBehavior] = useState<InstructionBehavior>("allow");
  const [editing, setEditing] = useState<InstructionRow | null>(null);
  const [editingText, setEditingText] = useState("");
  const [editingBehavior, setEditingBehavior] = useState<InstructionBehavior>("allow");
  const isDisabled = disabled || savePending;
  const instructions: AutoReviewInstructions = settings;
  const rows = instructionRows(instructions);
  const activeList = behavior === "allow" ? instructions.allowInstructions : instructions.blockInstructions;
  const canAdd = !isDisabled && editing == null && draft.trim().length > 0 && activeList.length < MAX_INSTRUCTIONS_PER_BEHAVIOR;

  const commitSettings = (next: AutoReviewSettings) => {
    setSavePending(true);
    void Promise.resolve()
      .then(() => onChange(next))
      .catch(() => undefined)
      .finally(() => setSavePending(false));
  };

  const addRule = () => {
    if (!canAdd) return;
    const next = saveInstruction(instructions, draft.trim(), behavior, null);
    if (next == null) return;
    commitSettings({ ...settings, ...next });
    setDraft("");
    setBehavior("allow");
  };

  const beginEdit = (row: InstructionRow) => {
    setEditing(row);
    setEditingText(row.text);
    setEditingBehavior(row.behavior);
  };

  const cancelEdit = () => {
    setEditing(null);
    setEditingText("");
    setEditingBehavior("allow");
  };

  const saveEdit = () => {
    if (editing == null || isDisabled || editingText.trim().length === 0) return;
    const next = saveInstruction(instructions, editingText.trim(), editingBehavior, editing);
    if (next == null) return;
    commitSettings({ ...settings, ...next });
    cancelEdit();
  };

  return (
    <section className="sand-auto-review">
      <SandSwitch
        checked={settings.isEnabled}
        disabled={isDisabled}
        label={<span><strong>{text.autoReviewTitle}</strong><small>{text.autoReviewDescription}</small></span>}
        onCheckedChange={(checked) => commitSettings({ ...settings, isEnabled: checked })}
      />

      {settings.isEnabled ? (
        <div>
          <div>
            <h3>{text.autoReviewRules}</h3>
            {/* @evidence recovered/frontend/app/assets/index-BlqerJhg.js#L291-L295 */}
            <p>{text.autoReviewInstructions}</p>
          </div>

          <div>
            <SandTextField
              aria-label={text.autoReviewDraftLabel}
              disabled={isDisabled || editing != null}
              maxLength={1000}
              onChange={(event) => setDraft(event.currentTarget.value.slice(0, 1000))}
              onKeyDown={(event) => {
                if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) addRule();
              }}
              placeholder={text.autoReviewExample}
              value={draft}
            />
            <SandSelect ariaLabel={text.ruleBehaviorLabel} className="ui-select-trigger" disabled={isDisabled || editing != null} onValueChange={setBehavior} options={[{ value: "allow" as const, label: text.allowAutomatically }, { value: "ask" as const, label: text.askFirst }]} placement="bottom-end" value={behavior} />
            <SandButton disabled={!canAdd} onClick={addRule} size="sm" variant="primary">{text.addRule}</SandButton>
          </div>
          {activeList.length >= MAX_INSTRUCTIONS_PER_BEHAVIOR ? <small>{text.maxRules.replace("{count}", String(MAX_INSTRUCTIONS_PER_BEHAVIOR))}</small> : null}

          {rows.length > 0 ? (
            <div aria-label={text.rulesTableLabel} className="sand-auto-review-rules-table" role="table">
              <div className="sand-auto-review-rule" role="row">
                <span role="columnheader">{text.actionColumn}</span>
                <span role="columnheader">{text.behaviorColumn}</span>
                <span aria-hidden="true" />
              </div>
              <div role="rowgroup">
                {rows.map((row, index) => editing != null && editing.behavior === row.behavior && editing.listIndex === row.listIndex && editing.text === row.text ? (
                  <div className="sand-auto-review-rule" key={`${row.behavior}:${row.listIndex}:${row.text}`} role="row">
                    <SandTextarea aria-label={text.editAction.replace("{index}", String(index + 1))} autoFocus autoResize={false} disabled={isDisabled} maxLength={1000} minRows={1} onChange={(event) => setEditingText(event.currentTarget.value.slice(0, 1000))} rows={1} value={editingText} />
                    <SandSelect ariaLabel={text.behaviorForRule.replace("{index}", String(index + 1))} className="ui-select-trigger" disabled={isDisabled} onValueChange={setEditingBehavior} options={[{ value: "allow" as const, label: "Allow automatically" }, { value: "ask" as const, label: "Ask first" }]} placement="bottom-end" value={editingBehavior} />
                    <span>
                      <SandButton aria-label={text.cancelEditingRule.replace("{index}", String(index + 1))} disabled={isDisabled} onClick={cancelEdit} size="sm" variant="secondary">{text.cancel}</SandButton>
                      <SandButton aria-label={text.saveRule.replace("{index}", String(index + 1))} disabled={isDisabled || editingText.trim().length === 0} onClick={saveEdit} size="sm" variant="primary">{text.saveRule}</SandButton>
                    </span>
                  </div>
                ) : (
                  <div className="sand-auto-review-rule" key={`${row.behavior}:${row.listIndex}:${row.text}`} role="row">
                    <span role="cell" title={row.text}>{row.text}</span>
                    <span role="cell">{row.behavior === "allow" ? text.allowAutomatically : text.askFirst}</span>
                    <span role="cell">
                      <SandButton aria-label={text.editRule.replace("{index}", String(index + 1))} disabled={isDisabled || editing != null} onClick={() => beginEdit(row)} size="sm" variant="secondary">{text.editRule.replace("{index}", String(index + 1))}</SandButton>
                      <SandButton
                        aria-label={text.deleteRule.replace("{index}", String(index + 1))}
                        disabled={isDisabled}
                        onClick={() => commitSettings({ ...settings, ...removeInstruction(instructions, row) })}
                        sentiment="danger"
                        size="sm"
                        variant="secondary"
                      >{text.delete}</SandButton>
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ) : null}
          <small>{text.rulesScope}</small>
        </div>
      ) : null}
    </section>
  );
}
