import { useEffect, useRef, useSyncExternalStore } from 'react';
import type { BotSummary } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  createAgentSettingsController,
  type AgentSettingsController,
} from './agent-settings-controller';
import type { AgentDirectoryController } from './use-agent-directory-controller';

export function useAgentSettingsController(
  directory: Pick<AgentDirectoryController, 'update'>,
  agent: BotSummary | null,
): {
  readonly controller: AgentSettingsController;
  readonly snapshot: ReturnType<AgentSettingsController['getSnapshot']>;
} {
  const controllerRef = useRef<AgentSettingsController | null>(null);
  if (!controllerRef.current) {
    controllerRef.current = createAgentSettingsController({ update: directory.update }, agent);
  }
  const controller = controllerRef.current;

  useEffect(() => {
    controller.setAgent(agent);
  }, [agent, controller]);

  const snapshot = useSyncExternalStore(
    controller.subscribe,
    controller.getSnapshot,
    controller.getSnapshot,
  );

  return { controller, snapshot };
}
