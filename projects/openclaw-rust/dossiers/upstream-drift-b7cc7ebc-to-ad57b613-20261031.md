# OpenClaw live-candidate drift frontier

Status: read-only compare evidence; not source closure, implementation authority,
or acceptance evidence.

## Exact comparison

- base: `b7cc7ebc9b442ed7eb73a92336c2a9886b578f7b`
- subject: `ad57b6135a55a6792b6d703bc1750954d854d6aa`
- subject tree: `24204d2196587105bf3af93a999cea03e8bdfae5`
- compare status: ahead 4, behind 0
- changed files: 25

## Changed responsibility areas

- Gateway protocol/client: transport documentation, handshake and protocol
  client tests, pending requests, protocol request/guards, payload limits, and
  gateway server constants. This reopens Gateway compatibility and auth/session
  transport reads for the new exact subject.
- Speech/TTS: request preference/fallback routing and runtime routing tests,
  plus a speech allocation fixture. This reopens the speech/voice responsibility
  frontier; the fixture is not runtime evidence.
- Session/chat/UI: Gateway browser socket/API, session placement startup, chat
  history cursor/hydration/send queue and their tests. This reopens the UI and
  session consumer reads rather than proving any Rust parity.
- Plugins UI: plugin page view and generation tests. This reopens the plugin
  presentation/consumer frontier; it does not establish bundle-manifest ingress
  or activation wiring.
- Other changes are allocation/config or documentation support files and must
  still be classified under the exact inventory.

## Evidence boundary

The compare proves only that the live ref moved and identifies paths requiring
fresh exact-subject reading. It does not prove recursive dependency closure,
package/LFS integrity, source responsibility mapping, Rust implementation,
production wiring, Actions verification, or acceptance. No ledger status or
completion flag changes from this dossier.
