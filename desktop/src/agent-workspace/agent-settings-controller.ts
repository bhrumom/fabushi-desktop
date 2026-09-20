import type { BotSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import type { AgentDirectoryController } from './use-agent-directory-controller';

export interface AgentSettingsProfileValue {
  readonly name: string;
  readonly title?: string;
  readonly description: string;
  readonly avatarShape: string;
  readonly avatarColor: string;
  readonly notifyOnUpdatesEnabled: boolean;
}

export interface AgentSettingsProfileUpdate {
  readonly name: string;
  readonly title?: string;
  readonly description: string;
  readonly avatarShape: string;
  readonly avatarColor: string;
}

export type AgentSettingsPending = 'profile' | 'notifications' | null;

export interface AgentSettingsSnapshot {
  readonly agent: BotSummary | null;
  readonly value: AgentSettingsProfileValue | null;
  readonly pending: AgentSettingsPending;
  readonly error: string | null;
  readonly generation: number;
}

export interface AgentSettingsSource {
  update: AgentDirectoryController['update'];
}

function profileValue(agent: BotSummary | null): AgentSettingsProfileValue | null {
  if (!agent) return null;
  return {
    name: agent.name,
    title: agent.title,
    description: agent.description,
    avatarShape: agent.avatarShape?.trim() ?? '',
    avatarColor: agent.avatarColor?.trim() ?? '',
    notifyOnUpdatesEnabled: agent.notifyOnUpdates ?? agent.notificationsEnabled,
  };
}

function sameAgent(left: BotSummary | null, right: BotSummary | null): boolean {
  if (left === right) return true;
  if (!left || !right || left.id !== right.id) return false;
  return left.name === right.name
    && left.title === right.title
    && left.description === right.description
    && (left.avatarShape ?? '') === (right.avatarShape ?? '')
    && (left.avatarColor ?? '') === (right.avatarColor ?? '')
    && left.notifyOnUpdates === right.notifyOnUpdates
    && left.notificationsEnabled === right.notificationsEnabled;
}

/**
 * Fabu-style Agent Settings controller.
 *
 * Mutation fencing lives outside the view. Directory bot.* compatibility is
 * hidden behind AgentDirectoryController, while authoritative profile changes
 * still arrive through its roster projection.
 */
export function createAgentSettingsController(source: AgentSettingsSource, initialAgent: BotSummary | null) {
  let agent = initialAgent;
  let pending: AgentSettingsPending = null;
  let error: string | null = null;
  let generation = 0;
  let snapshot: AgentSettingsSnapshot = {
    agent,
    value: profileValue(agent),
    pending,
    error,
    generation,
  };
  const listeners = new Set<() => void>();

  const emit = () => {
    snapshot = { agent, value: profileValue(agent), pending, error, generation };
    for (const listener of [...listeners]) listener();
  };

  return {
    subscribe(listener: () => void): () => void {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    getSnapshot(): AgentSettingsSnapshot {
      return snapshot;
    },
    setAgent(next: BotSummary | null): void {
      if (sameAgent(agent, next)) return;
      if (agent?.id !== next?.id) {
        generation += 1;
        pending = null;
        error = null;
      }
      agent = next;
      emit();
    },
    async updateProfile(profile: AgentSettingsProfileUpdate): Promise<boolean> {
      if (!agent || pending) return false;
      const normalized: AgentSettingsProfileUpdate = {
        name: profile.name.trim(),
        ...(profile.title === undefined ? {} : { title: profile.title.trim() }),
        description: profile.description.trim(),
        avatarShape: profile.avatarShape.trim(),
        avatarColor: profile.avatarColor.trim(),
      };
      if (!normalized.name) return false;
      if (
        normalized.name === agent.name
        && (normalized.title ?? '') === (agent.title ?? '')
        && normalized.description === agent.description
        && normalized.avatarShape === (agent.avatarShape ?? '')
        && normalized.avatarColor === (agent.avatarColor ?? '')
      ) return false;
      const requestGeneration = generation;
      const agentId = agent.id;
      pending = 'profile';
      error = null;
      emit();
      try {
        await source.update(agentId, {
          name: normalized.name,
          title: normalized.title ?? '',
          description: normalized.description,
          avatarShape: normalized.avatarShape,
          avatarColor: normalized.avatarColor,
        });
        return requestGeneration === generation && agent?.id === agentId;
      } catch (cause) {
        if (requestGeneration === generation && agent?.id === agentId) {
          error = cause instanceof Error ? cause.message : String(cause);
          emit();
        }
        return false;
      } finally {
        if (requestGeneration === generation && agent?.id === agentId) {
          pending = null;
          emit();
        }
      }
    },
    async setNotifications(enabled: boolean): Promise<boolean> {
      if (!agent || pending || (agent.notifyOnUpdates ?? agent.notificationsEnabled) === enabled) return false;
      const requestGeneration = generation;
      const agentId = agent.id;
      pending = 'notifications';
      error = null;
      emit();
      try {
        await source.update(agentId, {
          notifyOnUpdates: enabled,
          notificationsEnabled: enabled,
        });
        return requestGeneration === generation && agent?.id === agentId;
      } catch (cause) {
        if (requestGeneration === generation && agent?.id === agentId) {
          error = cause instanceof Error ? cause.message : String(cause);
          emit();
        }
        return false;
      } finally {
        if (requestGeneration === generation && agent?.id === agentId) {
          pending = null;
          emit();
        }
      }
    },
  };
}

export type AgentSettingsController = ReturnType<typeof createAgentSettingsController>;
