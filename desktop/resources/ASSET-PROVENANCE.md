# Distributed asset provenance

This file records independently reviewable provenance for non-generated files shipped from `desktop/resources`. Generated binaries keep provenance at their build inputs rather than at their staging directory.

| Packaged asset | Current Git blob | SHA-256 | Repository origin | Rights / license basis | Package scope | Attribution |
| --- | --- | --- | --- | --- | --- | --- |
| `desktop/resources/icon.png` | `8245ffe5e42e360fbcdb5e2f67c3dad6a13c39bf` | `14b79d25d2ce6e30db0e8b412ee1524c29b0e9ba03b4d2de1a19f7c435577d39` | First tracked in Fabushi commit `c4b6cfbb99655b7406a27a6f7405a7adc4fd259e` by project maintainer `bhrum`; no external source is recorded in repository history. | Fabushi package branding asset covered by the package-level `Copyright © 2026 Fabushi. All rights reserved.` declaration in `desktop/package.json`. This is a proprietary project-distribution basis, not an open-source license grant. If an external source or contributor-specific rights constraint is later identified, this row must be reopened before release. | Electron Builder Linux/Windows/macOS application icon inputs. | No third-party attribution is recorded by current repository evidence. |

The exact candidate release must verify the asset digest above against the file packaged from the same exact HEAD.
