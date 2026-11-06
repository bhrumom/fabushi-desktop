import { appendFileSync } from "node:fs";
import { pathToFileURL } from "node:url";

const BOOLEAN_INPUTS = [
  "PACKAGING_ARTIFACT_ONLY",
  "PACKAGING_PUBLISH_RELEASE",
  "PACKAGING_PROMOTE_EXISTING_CANDIDATE",
  "PACKAGING_ITERATE_EXISTING_MINIAPP_CANDIDATE",
];

function parseBooleanInput(environment, name) {
  const value = environment[name];
  if (value === undefined || value === "" || value === "false") return false;
  if (value === "true") return true;
  throw new Error(`${name} must be empty, "true", or "false"`);
}

/**
 * Resolve the existing macOS packaging owner's CI scope from raw environment
 * strings. This selects required work; it does not attest that acceptance ran.
 * Full-mode route admission remains with the existing release/promotion owners.
 */
export function resolvePackageAcceptanceScope(environment = {}) {
  const event = environment.PACKAGING_EVENT_NAME;
  if (event !== "push" && event !== "workflow_dispatch") {
    throw new Error('PACKAGING_EVENT_NAME must be "push" or "workflow_dispatch"');
  }

  const [artifactOnly, publishRelease, promoteExisting, iterateExisting] =
    BOOLEAN_INPUTS.map((name) => parseBooleanInput(environment, name));

  if (artifactOnly) {
    if (event !== "workflow_dispatch") {
      throw new Error("PACKAGING_ARTIFACT_ONLY requires workflow_dispatch");
    }
    const conflicts = [publishRelease, promoteExisting, iterateExisting]
      .flatMap((enabled, index) => enabled ? [BOOLEAN_INPUTS[index + 1]] : []);
    if (conflicts.length !== 0) {
      throw new Error(
        `PACKAGING_ARTIFACT_ONLY cannot be combined with ${conflicts.join(", ")}`,
      );
    }
  }

  return { scope: artifactOnly ? "artifact-only" : "full", artifactOnly };
}

function main() {
  // Resolve every input before touching the output file. Invalid modes must not
  // leave a partial scope for downstream jobs to consume.
  const policy = resolvePackageAcceptanceScope(process.env);
  const output = process.env.GITHUB_OUTPUT;
  if (output === undefined || output === "") {
    throw new Error("GITHUB_OUTPUT must name the workflow output file");
  }
  appendFileSync(
    output,
    `scope=${policy.scope}\nartifact_only=${policy.artifactOnly}\n`,
    "utf8",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    main();
  } catch (error) {
    process.stderr.write(
      `package acceptance scope: ${error instanceof Error ? error.message : String(error)}\n`,
    );
    process.exitCode = 1;
  }
}
