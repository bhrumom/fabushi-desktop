export interface SourceFailureDetail extends Record<string, unknown> {
  code: string;
}

export class SourceFailure extends Error {
  readonly failure: SourceFailureDetail;

  constructor(failure: SourceFailureDetail, message?: string) {
    super(message ?? failure.code);
    this.name = "SourceFailure";
    this.failure = failure;
  }
}

export function isSourceRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
