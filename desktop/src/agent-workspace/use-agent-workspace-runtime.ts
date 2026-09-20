import { useCallback, useEffect, useRef, useState } from 'react';
import type { ComputerStatus } from '../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import {
  createAgentSubmissionQueue,
  type AgentSubmission,
  type AgentSubmissionQueue,
} from '../fabu-runtime/submission-queue';
import { readAgentWorkspaceDrafts, persistAgentWorkspaceDrafts } from './agent-draft-store';
import { AgentRuntimeCoordinator } from './agent-runtime-coordinator';
import {
  AgentTranscriptStore,
  type AgentTranscriptTerminalStatus,
} from './agent-transcript-store';
import { AgentWorkspaceController } from './agent-workspace-controller';

export interface UseAgentWorkspaceRuntimeOptions {
  readonly hostReady: boolean;
  send(input: AgentSubmission): Promise<void>;
  onError?(peerKey: string, message: string): void;
  onComputerStatus?(status: ComputerStatus): void;
  onOperationStarted?(peerKey: string, operationId: string): void;
  onOperationTerminal?(
    peerKey: string,
    operationId: string,
    status: AgentTranscriptTerminalStatus,
    message?: string,
  ): void;
}

export interface AgentWorkspaceRuntimeFacade {
  readonly controller: AgentWorkspaceController;
  readonly transcriptStore: AgentTranscriptStore;
  readonly coordinator: AgentRuntimeCoordinator;
  readonly submissionQueue: AgentSubmissionQueue;
  readonly revision: number;
  notify(): void;
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : String(cause);
}

/**
 * Owns the long-lived Agent workspace runtime boundary for the renderer.
 *
 * Messenger compatibility code may resolve a peer and provide a transport
 * callback, but request/operation ownership, transcript reduction, queued
 * submissions and restart-safe drafts live here. This keeps React views from
 * recreating or coordinating those Agent lifecycles themselves.
 */
export function useAgentWorkspaceRuntime(
  options: UseAgentWorkspaceRuntimeOptions,
): AgentWorkspaceRuntimeFacade {
  const optionsRef = useRef(options);
  optionsRef.current = options;

  const [revision, setRevision] = useState(0);
  const notify = useCallback(() => {
    setRevision((value) => value + 1);
  }, []);

  const controllerRef = useRef<AgentWorkspaceController | null>(null);
  if (!controllerRef.current) {
    controllerRef.current = new AgentWorkspaceController(readAgentWorkspaceDrafts());
  }
  const controller = controllerRef.current;

  const transcriptStoreRef = useRef<AgentTranscriptStore | null>(null);
  if (!transcriptStoreRef.current) transcriptStoreRef.current = new AgentTranscriptStore();
  const transcriptStore = transcriptStoreRef.current;

  const submissionQueueRef = useRef<AgentSubmissionQueue | null>(null);
  if (!submissionQueueRef.current) {
    submissionQueueRef.current = createAgentSubmissionQueue({
      isBlocked: (input) => !optionsRef.current.hostReady || controller.isBusy(input.peerKey),
      send: (input) => optionsRef.current.send(input),
      onPhase: (submission) => {
        if (submission.phase === 'queued') {
          transcriptStore.appendUserMessage(submission.peerKey, {
            id: submission.messageId,
            text: submission.prompt,
            richText: submission.richText,
            createdAtMs: submission.createdAtMs,
            optimistic: true,
            queued: true,
            attachments: submission.attachments,
          });
          notify();
          return;
        }
        transcriptStore.removeQueuedUserMessage(submission.peerKey, submission.messageId);
        notify();
      },
      onFailure: (submission, cause) => {
        transcriptStore.removeQueuedUserMessage(submission.peerKey, submission.messageId);
        controller.restoreDraft(submission.peerKey, {
          text: submission.prompt,
          richText: submission.richText,
          attachments: submission.attachments ? [...submission.attachments] : [],
          references: submission.references ? [...submission.references] : [],
          replyTo: submission.replyTo,
        });
        notify();
        optionsRef.current.onError?.(submission.peerKey, errorMessage(cause));
      },
    });
  }
  const submissionQueue = submissionQueueRef.current;

  const coordinatorRef = useRef<AgentRuntimeCoordinator | null>(null);
  if (!coordinatorRef.current) {
    coordinatorRef.current = new AgentRuntimeCoordinator(
      controller,
      transcriptStore,
      {
        onTranscriptChanged: () => notify(),
        onOperationChanged: () => notify(),
        onComputerStatus: (status) => optionsRef.current.onComputerStatus?.(status),
        onOperationStarted: (peerKey, operationId) => {
          optionsRef.current.onOperationStarted?.(peerKey, operationId);
        },
        onRequestFailed: (peerKey, _requestId, message) => {
          optionsRef.current.onError?.(peerKey, message);
        },
        onOperationTerminal: (peerKey, operationId, status, message) => {
          submissionQueue.flush();
          optionsRef.current.onOperationTerminal?.(
            peerKey,
            operationId,
            status,
            message,
          );
        },
      },
    );
  }
  const coordinator = coordinatorRef.current;

  useEffect(() => {
    persistAgentWorkspaceDrafts(
      controller.draftSnapshot(),
      controller.attachmentSnapshot(),
      controller.replySnapshot(),
      controller.referenceSnapshot(),
      controller.richTextSnapshot(),
    );
  }, [controller, revision]);

  useEffect(() => {
    if (options.hostReady) submissionQueue.flush();
  }, [options.hostReady, submissionQueue]);

  useEffect(() => () => {
    submissionQueue.dispose();
    coordinator.dispose();
  }, [coordinator, submissionQueue]);

  return {
    controller,
    transcriptStore,
    coordinator,
    submissionQueue,
    revision,
    notify,
  };
}
