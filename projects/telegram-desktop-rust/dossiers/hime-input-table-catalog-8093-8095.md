# Hime input-table initialization and method catalog — orders 8,093–8,095

Exact pinned-source reading shows 8,093 resolves user/system tables, rebuilds a newer merged append table only when needed, validates header/key widths, derives wildcard/selection/key metadata and behavior flags, reuses already-loaded tables, and fails on missing/invalid data. These are canonical Resource/input-data loader plus platform-input Settings responsibilities, not a Hime runtime.

8,094 frees/rebuilds the input-method catalog, parses method key/file/icon/disabled/cycle metadata, and selects the configured default only when the matching method is enabled. 8,095 is its tiny code/name schema. The applicable product responsibility is one canonical input-method catalog/search/settings owner with source-neutral metadata and truthful availability state.

All three rows remain `mapped-open`; unknown remains unchanged.
