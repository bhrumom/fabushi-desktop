# Round 49 — MTProto detailed-answer resend evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 18
Stage A: implementation/contracts authored; executable validation not run.

Source: fixed U `f985723b552b30fe1c17d377d57f44e61a15a8c4`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`.
Fixed-U constructors: `msg_detailed_info#276d3ec6`, `msg_resend_req#7d861a08`.

Canonical spec: `9bea551d1e26a4ff5efa522417999388256486ff`.
Canonical implementation: `0536aa2ad9f93e59f1b1e3cdc19aabaf9a104fce`.
Canonical blobs: api `1feaefd062f4e8635bbf002a7dce30b804c9f6ed`, lib `3e17847045f3be606d8b821c85f7b8136b89474a`, network `581156c32289082ad1f53410ab0637bdeef85c66`.
Desktop synchronization: `3f5e2d3a8b2e0c2726a1b1dcd1ab1172a0c5758d` with the same three blobs.

The one existing `invoke_raw` outstanding request is the only request history used. A matching detailed-info message sends `msg_resend_req([answer_msg_id])` through the same encrypted non-content service-message path; unrelated detailed-info does not mutate the current call.

Authored-only contracts: protocol fixed-U builder encoding and network current-request matching. They were not run.

Open from the same fixed-U source: `msg_new_detailed_info`, `msgs_state_req`, `msgs_state_info`, `msgs_all_info`, and full multi-request state reconciliation.

Status: implemented / validation_status=not-run / accepted=false.
