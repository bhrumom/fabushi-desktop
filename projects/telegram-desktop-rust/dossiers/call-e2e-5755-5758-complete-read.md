# Call E2EE — deterministic orders 5755-5758 complete read

These blobs own call end-to-end crypto semantics: temporary key generation/destruction, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, outbound/inbound queues, subchain reconciliation, failure terminality and emoji verification hash. Protocol type conversion is an adapter, not a second owner.

They remain mapped-open to canonical Calls E2EE/Crypto and Call Session owners. TLS/transport encryption cannot satisfy this responsibility.

In parallel this exact implementation batch hardens the existing canonical Wallet ledger: request-id reuse now must match operation kind, accounts, normalized currency, amount and reference; conflicting reuse fails closed with `RequestConflict`, with unit coverage for credit/transfer/refund conflicts. This production change does not by itself close the broader Credits/Wallet/Payments ledger rows.

Accounting: **5,758/16,123 read; 10,365 unread; 15,844 unknown; 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.
