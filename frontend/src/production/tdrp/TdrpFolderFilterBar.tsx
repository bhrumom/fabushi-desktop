import React from "react";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import {
  FabushiCollectionFilterBar,
  resolveCollectionCustomFoldersStorageKey,
  type FabushiFilterFolder,
} from "./FabushiCollectionFilterBar";

/**
 * @deprecated Legacy folder filter wrapper (Blueprint 01 §2.1 BUG-01-02 & §3.2).
 * Consolidated into `FabushiCollectionFilterBar` to eliminate duplicate `localStorage` keys.
 * Upstream Source Alignment: `Telegram/SourceFiles/data/data_chat_filters.cpp`.
 */

export type TdrpFolder = FabushiFilterFolder;

export interface TdrpFolderFilterBarProps {
  readonly activeFolderId: string;
  readonly onSelectFolder: (folderId: string) => void;
  readonly onOpenNewChat?: () => void;
  readonly unreadCount?: number;
  readonly humanCount?: number;
  readonly groupCount?: number;
  readonly agentCount?: number;
  readonly accountId?: string | null;
  readonly accountScopeGeneration?: number;
  readonly client?: ProductionCoordinatorClient | null;
}

export { filterAgentsByFolder, type FilterableAgent } from "./tdrp-folder-filter";
export { resolveCollectionCustomFoldersStorageKey };

export const TdrpFolderFilterBar: React.FC<TdrpFolderFilterBarProps> = ({
  activeFolderId,
  onSelectFolder,
  onOpenNewChat = () => {},
  unreadCount = 0,
  humanCount = 0,
  groupCount = 0,
  agentCount = 0,
  accountId = null,
  accountScopeGeneration = 0,
  client,
}) => {
  return (
    <FabushiCollectionFilterBar
      activeFolderId={activeFolderId}
      onSelectFolder={onSelectFolder}
      onOpenNewChat={onOpenNewChat}
      unreadCount={unreadCount}
      directCount={humanCount}
      groupCount={groupCount}
      agentCount={agentCount}
      accountId={accountId}
      accountScopeGeneration={accountScopeGeneration}
      client={client}
    />
  );
};
