# Fixed OpenClaw subject tree role census

**Observed:** 2026-10-31, read-only local Git inspection  
**Subject:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`  
**Tree:** `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
**Authority:** supporting exact-tree evidence; not an Actions inventory or source-closure receipt

## Deterministic top-level role census

The fixed commit's recursive Git path list was grouped by the first
repository-relative directory using a deterministic, mutually exclusive rule:
`.github`, `docs`, `src`, `extensions`, `packages`, `ui`, test/QA roots,
script/tool roots, and `root-or-other` for all remaining paths.

| Role bucket | Paths |
| --- | ---: |
| `src` | 27,856 |
| `extensions` | 12,311 |
| `ui` | 6,658 |
| `root-or-other` | 3,811 |
| `tests-qa` (`test`, `tests`, `qa`) | 3,046 |
| `scripts-tools` (`scripts`, `tools`) | 1,939 |
| `docs` | 1,456 |
| `packages` | 1,423 |
| `.github` | 250 |
| **Total** | **58,750** |

The bucket sum equals the fixed recursive tree total. This is path-role
evidence only: it does not infer applicability, semantic responsibility,
dependency closure, or production use from directory placement.

## File-kind census

The same exact path list reports the following largest suffix groups:

| Suffix | Paths |
| --- | ---: |
| `.ts` | 47,305 |
| `.md` | 2,104 |
| `.swift` | 1,591 |
| `.tsx` | 1,308 |
| `.json` | 1,012 |
| `.mjs` | 745 |
| `.mts` | 671 |
| `.yaml` | 639 |
| `.kt` | 626 |
| `.svg` | 392 |
| `.png` | 313 |
| `.sh` | 306 |
| `.txt` | 265 |
| `.css` | 261 |
| `.yml` | 189 |

The remaining suffix groups and 71 extensionless paths remain in the exact
tree and are not discarded by this summary. In particular, the dominance of
TypeScript paths is not a reason to exclude Swift/Kotlin/UI/docs/CI/release,
fixtures, examples, or generated-looking paths from the governed scope.

## Closure boundary

This census is a supporting measurement for the exact-subject materializer. It
does not modify `inventory/shards/manifest.json`, does not fill the currently
missing applicability/disposition fields, and does not change
`dependency_closure_enumerated`, `source_closure_ready`, or any OCRM completion
flag. The required next authority remains the exact-subject Actions
materialization that assigns explicit applicability and disposition to every
row and publishes its immutable evidence envelope.
