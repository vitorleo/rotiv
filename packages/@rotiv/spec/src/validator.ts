import type { RotivSpec } from "@rotiv/types";

export interface ValidationError {
  path: string;
  message: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

/**
 * Validates a parsed spec object against the RotivSpec schema.
 * Phase 1: manual validation (no ajv dependency yet).
 * Phase 2: swap in ajv with the JSON Schema from schema.ts.
 */
export function validateSpec(spec: unknown): ValidationResult {
  const errors: ValidationError[] = [];

  if (typeof spec !== "object" || spec === null) {
    return {
      valid: false,
      errors: [{ path: "$", message: "spec must be an object" }],
    };
  }

  const s = spec as Record<string, unknown>;

  if (s["version"] !== "1") {
    errors.push({
      path: "$.version",
      message: `expected "1", got ${JSON.stringify(s["version"])}`,
    });
  }

  if (typeof s["framework_version"] !== "string") {
    errors.push({
      path: "$.framework_version",
      message: "must be a string",
    });
  }

  if (typeof s["project"] !== "object" || s["project"] === null) {
    errors.push({ path: "$.project", message: "must be an object" });
  } else {
    const project = s["project"] as Record<string, unknown>;
    if (typeof project["name"] !== "string" || project["name"].length === 0) {
      errors.push({ path: "$.project.name", message: "must be a non-empty string" });
    }
    if (typeof project["created_at"] !== "string") {
      errors.push({ path: "$.project.created_at", message: "must be a string" });
    }
  }

  for (const field of ["routes", "models", "modules"] as const) {
    if (!Array.isArray(s[field])) {
      errors.push({ path: `$.${field}`, message: "must be an array" });
    }
  }

  return { valid: errors.length === 0, errors };
}

/**
 * Asserts that a value is a valid RotivSpec.
 * Throws if validation fails.
 */
export function assertValidSpec(spec: unknown): asserts spec is RotivSpec {
  const result = validateSpec(spec);
  if (!result.valid) {
    const messages = result.errors.map((e) => `  ${e.path}: ${e.message}`).join("\n");
    throw new Error(`Invalid RotivSpec:\n${messages}`);
  }
}
