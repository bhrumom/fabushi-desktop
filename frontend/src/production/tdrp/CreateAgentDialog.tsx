import React from "react";
import { OverlayDialog } from "../../recovered/ui/overlay-primitives";
import { AgentAvatar } from "../../recovered/features/conversation/workspace/agent-avatar";
import { SandButton } from "../../recovered/ui/sand-kit-primitives";
import { AVATAR_COLORS, AVATAR_SHAPES } from "../../recovered/features/agent-info/avatar-editor/model";

export type AvatarShapeType = (typeof AVATAR_SHAPES)[number];
export type AvatarColorType = (typeof AVATAR_COLORS)[number]["id"];

export interface CreateAgentDialogProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly busy: boolean;
  readonly name: string;
  readonly onNameChange: (name: string) => void;
  readonly description: string;
  readonly onDescriptionChange: (description: string) => void;
  readonly avatarShape: AvatarShapeType;
  readonly onAvatarShapeChange: (shape: AvatarShapeType) => void;
  readonly avatarColor: AvatarColorType;
  readonly onAvatarColorChange: (color: AvatarColorType) => void;
  readonly onSubmit: () => void;
}

export const CreateAgentDialog: React.FC<CreateAgentDialogProps> = ({
  isOpen,
  onClose,
  busy,
  name,
  onNameChange,
  description,
  onDescriptionChange,
  avatarShape,
  onAvatarShapeChange,
  avatarColor,
  onAvatarColorChange,
  onSubmit,
}) => {
  return (
    <OverlayDialog
      className="sand-new-agent-dialog"
      label="Create agent"
      onClose={() => {
        if (!busy) onClose();
      }}
      open={isOpen}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          if (!busy) onSubmit();
        }}
      >
        <header>
          <div>
            <h2>Create agent</h2>
            <p>Choose the agent identity before its first conversation.</p>
          </div>
          <AgentAvatar
            agentId="new-agent-avatar-preview"
            color={avatarColor}
            shape={avatarShape}
            size="xl"
            state="thinking"
          />
        </header>
        <div className="sand-new-agent-dialog__body">
          <label>
            <span>Name</span>
            <input
              autoFocus
              maxLength={120}
              onChange={(event) => onNameChange(event.currentTarget.value)}
              placeholder="Agent name"
              value={name}
            />
          </label>
          <label>
            <span>Description</span>
            <textarea
              maxLength={500}
              onChange={(event) => onDescriptionChange(event.currentTarget.value)}
              placeholder="What should this agent help with?"
              rows={3}
              value={description}
            />
          </label>
          <fieldset>
            <legend>Avatar shape</legend>
            <div className="sand-new-agent-dialog__shape-grid">
              {AVATAR_SHAPES.map((shape: AvatarShapeType) => (
                <button
                  aria-label={`Avatar shape ${shape}`}
                  aria-pressed={avatarShape === shape}
                  className="sand-new-agent-dialog__shape"
                  key={shape}
                  onClick={() => onAvatarShapeChange(shape)}
                  type="button"
                >
                  <AgentAvatar
                    agentId={`new-agent-shape-${shape}`}
                    color={avatarColor}
                    isStatic
                    shape={shape}
                    size="lg"
                  />
                </button>
              ))}
            </div>
          </fieldset>
          <fieldset>
            <legend>Avatar color</legend>
            <div className="sand-new-agent-dialog__color-grid">
              {AVATAR_COLORS.map((color: (typeof AVATAR_COLORS)[number]) => (
                <button
                  aria-label={`Avatar color ${color.label}`}
                  aria-pressed={avatarColor === color.id}
                  className="sand-new-agent-dialog__color"
                  key={color.id}
                  onClick={() => onAvatarColorChange(color.id)}
                  style={{ background: color.value }}
                  title={color.label}
                  type="button"
                />
              ))}
            </div>
          </fieldset>
          <p className="sand-new-agent-dialog__hint">
            You can upload or generate an image later from Agent settings.
          </p>
        </div>
        <footer>
          <SandButton disabled={busy} onClick={onClose} size="sm" variant="secondary">
            Cancel
          </SandButton>
          <SandButton disabled={busy || name.trim().length === 0} size="sm" type="submit">
            Create
          </SandButton>
        </footer>
      </form>
    </OverlayDialog>
  );
};
