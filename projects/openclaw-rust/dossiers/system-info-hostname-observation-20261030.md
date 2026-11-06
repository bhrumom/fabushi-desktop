# system.info hostname observation — 2026-10-30

The Gateway `system.info` projection previously used `HOSTNAME` or
`COMPUTERNAME` and otherwise returned the synthetic value `unknown`.

The existing host-system observation owner now reads the hostname directly
from Unix `gethostname()` or Windows `GetComputerNameW()` when the environment
does not provide a non-empty value. Sampling failure is returned as an error;
the Gateway no longer publishes an unobserved hostname.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; local OCRM tests/builds and cross-platform packaged
acceptance remain pending.
