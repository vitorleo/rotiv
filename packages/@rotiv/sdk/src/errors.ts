/**
 * Structured error type mirroring the Rust `RotivError` struct.
 * Used for errors returned from the Rotiv runtime (Phase 2+).
 */
export interface RotivError {
  code: string;
  message: string;
  file?: string;
  line?: number;
  expected?: string;
  got?: string;
  suggestion?: string;
  corrected_code?: string;
}

export class RotivRuntimeError extends Error {
  readonly code: string;
  readonly details: RotivError;

  constructor(details: RotivError) {
    super(`[${details.code}] ${details.message}`);
    this.name = "RotivRuntimeError";
    this.code = details.code;
    this.details = details;
  }
}
