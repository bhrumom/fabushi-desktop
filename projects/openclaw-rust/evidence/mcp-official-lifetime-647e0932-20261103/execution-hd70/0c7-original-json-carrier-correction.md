# Correction: original 0c current-consumer DB byte carriers

The earlier `0c7-evidence-audit` / `0c7-prior-root-capture-gap` claim that the current-consumer database raw bytes were absent or unrecoverable was **incorrect**. The original upload omitted the standalone hidden database paths, but **both complete database byte sequences are already present in the original uploaded JSON containers**. The previous reports remain unchanged as history; this append-only correction supersedes that specific raw-absence conclusion.

## Original authority and decoding

The source is original Actions artifact **13204320664**, runtime **41574595343**, attempt **1**, Linux job **125715057540**, subject **0c7da50fcf457184f38d6a61b73f8e32810cff49**, tree **77aee57690a8d09cdf794f6459412ae26f604f1a**. The ZIP remains exactly **4,436,927 bytes**, SHA-256 `f3890cd4665c68767fb4f529a45dc5b9dd2ed8a8b94734eb8cbe61f9ff7bfd5d`.

| Current-consumer case | Original JSON container size | Original JSON SHA-256 | Decoded DB bytes |
|---|---:|---|---:|
| present-empty | 557,063 | `4b42bf58ccd141058bc18bf88fa9120af55f917bb51e3d79148d56de9fd94167` | 32,768 |
| legacy-nonempty-no-alias | 579,781 | `05dc9e47ad45e13efae75cf664a997522ecc8fd254cbd3bb96d85e1940c44522` | 32,768 |

Each original `current-consumer/cases/<case>/persistent-image-after.json` contains `other_files["agents/.call-sessions.db"]`. Every element is an integer in 0–255. Directly converting each original array to bytes gives SHA-256 **`678a56ce2376b5996556e3f692199423d285feaa36a55aa04076b25370e63974`**, exactly matching that case's original consumer inventory entry. No producer tar, fixture regeneration, SQLite operation, new executable or test run supplied these bytes.

The original ZIP member bytes match their inventory hashes and lengths. The inventory SHA-256 `5fb00c8e7f5b0056149b49fc98dd1617f763a56e3daeba497f422b3b0f1532b6` matches the original PRID receipt. The inventory-bound consumer manifest names the exact H/tree and `41574595343:1:sqlite-native-path`, identifies the `head-consume` phase and original Host/App executable hashes. The support-module source at immutable H uses `BTreeMap<String, Vec<u8>>` and reads every other persistent file directly; its hash matches both original source inventory and capture metadata. Its consumer loop writes this after-image after the four current-reader/App children and compares unchanged file bytes except the intended marker removal.

The original App receipt hash is `5723e02a5482a0a529fbd7067fe25c8203b644889c10273d9569431e980b96ab`; the App binary digest is `00ef5e876e23e37ee7a0a19fca98b03fd0d4c9386eb91333f08710fd2426cd1c` and Host digest is `2de306f88c33b8275e7676b72849eb767fdd14603ced8037a6e7b31fc79dd950`. These are original receipt-bound executable identities; this audit does not newly hash unavailable executable bytes.

## Corrected conclusion

All other consumer inventory files match their direct uploaded members. The only two missing standalone paths can be recovered exactly from the original JSON representations, so **current-consumer inventory byte coverage is complete using the original artifact**. Standalone hidden files and a dedicated consumer tar were still omitted; a narrowly allowlisted archive remains useful packaging, but it is not necessary to reconstruct these particular original bytes. The earlier statement of an unrecoverable raw-data gap was an auditor error, not evidence of absent product output.

[The machine proof](0c7-original-json-carrier-correction.json) records the complete inventory comparison, exact source identities, original capture context, recovered file hashes and unchanged prior report hashes. Decoded copies are under `0c7-json-carrier-recovery/`. They are audit reconstructions of original byte arrays, not newly executed product output. This correction grants no d70 or later-H runtime credit; d70's new PRID archive step was skipped.
