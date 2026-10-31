# Third-party notices

## Telegram Database Library (TDLib)

The bootstrap data-center addresses and ports in
`../telegram-protocol/src/dc.rs` are transcribed from
`td/telegram/net/ConnectionCreator.cpp@4bfe3bbdaaa82a20772a7bc8b437321578d45c7e`
at TDLib commit
`a17f87c4cff7b90b278d12b91ba0614383aaee82`.

Upstream: <https://github.com/tdlib/td>

TDLib is distributed under the Boost Software License 1.0. The exact upstream
license object is
`LICENSE_1_0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`,
and the byte-identical license text is included at
`../telegram-protocol/LICENSE-TDLIB-BOOST-1.0.txt`.

The network crate itself is Fabushi-owned transport/adaptation code consuming
the shared Telegram protocol crate. This notice records the exact bootstrap
data transcription; it does not classify the whole network crate as a TDLib
copy. Stage A validation and release legal acceptance have not been run or
granted.
