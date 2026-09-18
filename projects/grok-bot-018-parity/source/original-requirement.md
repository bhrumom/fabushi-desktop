# Original requirement

Refactor bhrumom/fabushi-desktop around bhrum/grok-bot-0.18-reconstructed, reproducing Grok Bot's UI, interactions, plugin experience, and especially its agent architecture. Hide product modules not present in Grok Bot (including current contacts/Messenger-specific surfaces, Telegram and payment UI during this phase). Do not implement the agent as a CLI wrapper. If Rust cannot reproduce the Grok Bot TypeScript agent architecture faithfully, use TypeScript. The only intentional product difference is that each agent controls the computer where Fabushi is installed rather than a provided cloud computer. macOS is first; package/release in GitHub Actions; manual testing is performed by humans.

Conversation task marker: Fabushi:8654bc3c-a0c3-4d12-9f3e-ca2c2a4ba889

## Reference provenance constraint

The reference repository's PROVENANCE.md/NOTICE.md states that it is an unofficial evidence-backed reconstruction of a public Grok Bot 0.18.0 binary, not official upstream source, and does not assert an upstream source-code license. Any public redistribution needs an independent rights/trademark review.
