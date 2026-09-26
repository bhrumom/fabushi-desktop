export const LOCAL_EXEC_GENERATION_TOKEN_ENV = "SAND_LOCAL_EXEC_GENERATION_TOKEN";
export const LOCAL_EXEC_GENERATION_TOKEN_ARG = "--sand-local-exec-generation=";
export const LOCAL_EXEC_DAEMON_PUBLICATION_LAG_MS = 60_000;

export interface LocalExecProcessIdentity {
  readonly pid: number;
  readonly startEpochMs: number;
  readonly command: string;
  readonly entryRealpath: string;
  readonly generationToken: string;
}

export interface ExpectedLocalExecProcessIdentity {
  readonly pid: number;
  readonly entryRealpath: string;
  readonly generationToken: string;
  readonly startEpochMs?: number;
  readonly command?: string;
  readonly discoveryStartedAt?: number;
}

function commandHasArgument(command: string, argument: string): boolean {
  if (argument.length === 0) return false;
  let cursor = 0;
  while (cursor <= command.length - argument.length) {
    const found = command.indexOf(argument, cursor);
    if (found < 0) return false;
    const leftBoundary = found === 0 || /\s/.test(command.charAt(found - 1));
    const right = found + argument.length;
    const rightBoundary = right === command.length || /\s/.test(command.charAt(right));
    if (leftBoundary && rightBoundary) return true;
    cursor = found + 1;
  }
  return false;
}

export function commandCarriesLocalExecGeneration(
  command: string,
  entryRealpath: string,
  generationToken: string,
): boolean {
  if (entryRealpath.length === 0 || generationToken.length === 0) return false;
  return commandHasArgument(command, entryRealpath)
    && commandHasArgument(command, `${LOCAL_EXEC_GENERATION_TOKEN_ARG}${generationToken}`);
}

export function sameLocalExecProcessIdentity(
  left: LocalExecProcessIdentity,
  right: LocalExecProcessIdentity,
): boolean {
  return left.pid === right.pid
    && left.startEpochMs === right.startEpochMs
    && left.command === right.command
    && left.entryRealpath === right.entryRealpath
    && left.generationToken === right.generationToken;
}

export function localExecDiscoveryTimeMatchesProcess(
  discoveryStartedAt: number,
  processStartEpochMs: number,
  observedAtMs: number,
): boolean {
  if (![discoveryStartedAt, processStartEpochMs, observedAtMs].every(Number.isFinite)) return false;
  const publicationDelay = discoveryStartedAt - processStartEpochMs;
  return publicationDelay >= 0
    && publicationDelay <= LOCAL_EXEC_DAEMON_PUBLICATION_LAG_MS
    && discoveryStartedAt <= observedAtMs;
}
