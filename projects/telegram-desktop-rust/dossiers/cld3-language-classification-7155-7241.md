# CLD3 language-classification authority — deterministic read 7,155–7,241

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` → `google/cld3@b48dc46512566f5a2d41118c8c1116c4f96dc661` (Apache-2.0).

All **87** component entries are exact-identified and semantically classified. Text sources were fetched directly at the pinned commit; the 1.76 MB generated `lang_id_nn_params.cc` was fetched by exact blob SHA after the contents API suppressed it; `model.png` is a documentation/model-diagram binary asset classified by exact blob identity and README context. No source row is promoted to verified by reading.

## Behavior

`NNetLanguageIdentifier` accepts bounded text and returns language, probability and reliability. Its pipeline validates/repairs interchange-invalid input, segments scripts while preserving offsets, derives character n-gram and relevant-script features, runs a statically embedded feed-forward network and aggregates script-span predictions. Reference tests cover supported languages, script detection, invalid UTF-8, empty strings and confidence.

## Shipping consumer trace

Accepted Telegram source calls `Platform::Language::Recognize` from:
- `boxes/translate_box.cpp` — whether to offer/skip translation;
- `history/view/history_view_translate_tracker.cpp` — chat translation tracking;
- `boxes/compose_ai_box.cpp` — default detected source language;
- `lang/translate_url_provider.cpp` — source language for external translation;
- `platform/linux/translate_provider_linux.cpp` — local source-language selection.

## Existing-owner-first / owner absence

Current Fabushi exact-head shipping tree has no path containing `language`, `translate`, `spellcheck` or `i18n`. Existing candidate review:
- **Canonical Translation service gateway**: correct higher-level consumer owner, but currently documentation-only/mapped-open, no shipping path.
- **Localization/Settings**: owns product locale/preferences, not arbitrary text classification.
- **Message**: may store/project detected language, but should not own model inference.
- **Search**: no ownership justification.

Thus the production capability is **blocked on an approved minimal owner**, not N/A. Proposed ADR boundary: source-neutral `TextLanguageClassifier`, local/private by default, bounded input, typed `unknown`, probability/reliability, pinned model/dependency/license, no remote data exfiltration. This is a proposal only; no CLD3/TMC/Telegram runtime/provider is created.

## Exact entries

| order | path | blob | responsibility | status |
| ---: | --- | --- | --- | --- |
| 7155 | `.github/workflows/main.yml` | `b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` | classifier-support | mapped-open |
| 7156 | `.gitignore` | `378eac25d311703f3f2cd456d8036da525cd0366` | build-doc-binding | mapped-open |
| 7157 | `CMakeLists.txt` | `2fa3908799c2cef538013ffe1deedaa1f7b0e7ee` | build-doc-binding | mapped-open |
| 7158 | `CONTRIBUTING.md` | `ade29dbfc344f5405e43aea448f5b2f013978f89` | build-doc-binding | mapped-open |
| 7159 | `LICENSE` | `c5899b26febde1d80fadd24c32254e70c68544ae` | build-doc-binding | mapped-open |
| 7160 | `MANIFEST.in` | `9fb3e4859cd624888f59f61f73fa66e418eaeb18` | build-doc-binding | mapped-open |
| 7161 | `README.md` | `462dd8e60e3c01dfaf886fa520b7288fc70513a8` | build-doc-binding | mapped-open |
| 7162 | `gcld3/__init__.py` | `b7d4dcc3f7576e4a8d3f4eb52a72559309ef9d0e` | build-doc-binding | mapped-open |
| 7163 | `gcld3/pybind_ext.cc` | `024d6d56d8973534c41799cae04bc101aa89f30e` | build-doc-binding | mapped-open |
| 7164 | `gcld3/tests/gcld3_test.py` | `066e08e8e22cd59ea7a3d838be0470f917b08fd5` | language-classifier-oracle | mapped-open |
| 7165 | `misc/myprotobuf.cmake` | `c8d42427dc5c14b5047813d23ec0a83df0e56a69` | classifier-support | mapped-open |
| 7166 | `model.png` | `87f0f14db0bfd58c23189de070ed03285c8c17bf` | build-doc-binding | mapped-open |
| 7167 | `requirements.txt` | `8d0ba11fa121536ddb656bc29a9d7c93ed6983f7` | build-doc-binding | mapped-open |
| 7168 | `setup.py` | `385189fc99e881b6a9fd5ff32549e09505507104` | build-doc-binding | mapped-open |
| 7169 | `src/BUILD.gn` | `80b912ee71abd836b57ff8b507e7a6562a87ba8b` | build-doc-binding | mapped-open |
| 7170 | `src/DEPS` | `e00022d82b957e0b782e65b0b83e7756b942f754` | build-doc-binding | mapped-open |
| 7171 | `src/base.cc` | `aaa363c9c6d8eb3ff9401ca08a833f9717ffe682` | classifier-support | mapped-open |
| 7172 | `src/base.h` | `20189e2cd09e4736a2d0e7fbd0facbe0f3941502` | classifier-support | mapped-open |
| 7173 | `src/casts.h` | `4c9ecd788a18fc9f65a8866b661491a80363a591` | classifier-support | mapped-open |
| 7174 | `src/embedding_feature_extractor.cc` | `16692b3937d6b63e618e13b1992cd856ecb097b4` | language-classifier-core | mapped-open |
| 7175 | `src/embedding_feature_extractor.h` | `4ff3e52f740457f14e5eed3b131b46b654ad6934` | language-classifier-core | mapped-open |
| 7176 | `src/embedding_network.cc` | `2296ea3f4f8a666e2f287ac6b7e0812abba10c47` | language-classifier-core | mapped-open |
| 7177 | `src/embedding_network.h` | `af11e89e3ddb2c4c909d7137e5474cb822ea6c0b` | language-classifier-core | mapped-open |
| 7178 | `src/embedding_network_params.h` | `447e0bb2fa08483b2f7250f20ead5d11a12a5db5` | language-classifier-core | mapped-open |
| 7179 | `src/feature_extractor.cc` | `10d1348725a3d51024dea0a0f02d062254ed86dd` | classifier-support | mapped-open |
| 7180 | `src/feature_extractor.h` | `81c7766a1ee19564b7309002f492f11dc23da3b2` | classifier-support | mapped-open |
| 7181 | `src/feature_extractor.proto` | `50a4c83ce68be4be35d2558fddc81604578d36ad` | classifier-support | mapped-open |
| 7182 | `src/feature_types.cc` | `059a7dd6d4b9402672fde3803e4e96366d10bad5` | classifier-support | mapped-open |
| 7183 | `src/feature_types.h` | `6e1389043280617c655b1f2fdab92529dd2cc8d8` | classifier-support | mapped-open |
| 7184 | `src/float16.h` | `7bed57da42d6d85b27578ec4fe2659353173be3d` | classifier-support | mapped-open |
| 7185 | `src/fml_parser.cc` | `c9cb4d8bf1338579196fc033950e1e1792e5f647` | classifier-support | mapped-open |
| 7186 | `src/fml_parser.h` | `ba87b4504ca8a534e9e2e094e9cded85c57cbb5c` | classifier-support | mapped-open |
| 7187 | `src/lang_id_nn_params.cc` | `af286199962b168e7a1dccb7f19c7f22f9f4f7fe` | language-classifier-core | mapped-open |
| 7188 | `src/lang_id_nn_params.h` | `890da7461f7ea17dfeb3c005a33e899a8cdf70b6` | language-classifier-core | mapped-open |
| 7189 | `src/language_identifier_features.cc` | `d006a32ec406d4777b87493a5814538887aad18f` | language-classifier-core | mapped-open |
| 7190 | `src/language_identifier_features.h` | `476ca00634bdd5009ced518dc98f096d43258b86` | language-classifier-core | mapped-open |
| 7191 | `src/language_identifier_features_test.cc` | `05fb86c82b12337b4f9e6fe4801a284a962e202b` | language-classifier-core | mapped-open |
| 7192 | `src/language_identifier_main.cc` | `b44f785bf2304c563b876e9bb8cc6b021dd3d431` | classifier-support | mapped-open |
| 7193 | `src/nnet_lang_id_test.cc` | `a7a252076d70ad5a71bd8422f3c797c912c68e26` | language-classifier-oracle | mapped-open |
| 7194 | `src/nnet_lang_id_test_data.cc` | `e221afcd897d56cc4c52ccdce23f449e917d811e` | language-classifier-oracle | mapped-open |
| 7195 | `src/nnet_lang_id_test_data.h` | `7377f1daa894763af7779c926dc21dda23166662` | language-classifier-oracle | mapped-open |
| 7196 | `src/nnet_language_identifier.cc` | `a878cf582ba4c6950d2b6c2d4481ac50ebeecb21` | language-classifier-core | mapped-open |
| 7197 | `src/nnet_language_identifier.h` | `e5eb8627efe896a9f335245777440de15c75edbd` | language-classifier-core | mapped-open |
| 7198 | `src/registry.cc` | `a4567a22a7dce7b031b8db48290134a7d0c2fc05` | classifier-support | mapped-open |
| 7199 | `src/registry.h` | `ff0426b1efddc25a81a79b9b790bad7d64a57d2d` | classifier-support | mapped-open |
| 7200 | `src/relevant_script_feature.cc` | `0cb6d559ce44f2a4ea50a14c41ef245ae9f5c06c` | unicode-script-normalization | mapped-open |
| 7201 | `src/relevant_script_feature.h` | `ce808105769bb320ceb8e1c40aab698edcc06860` | unicode-script-normalization | mapped-open |
| 7202 | `src/relevant_script_feature_test.cc` | `cfa56df6928edad2c6ed2e266730f46b5b1153f4` | unicode-script-normalization | mapped-open |
| 7203 | `src/script_detector.h` | `b3c4f6a7d2c97f3f739315580b19eae84c21379e` | unicode-script-normalization | mapped-open |
| 7204 | `src/script_detector_test.cc` | `50bea8f351b4996d449b0857880e4841c980f7c7` | unicode-script-normalization | mapped-open |
| 7205 | `src/script_span/README.md` | `86578c1821b4671dbfc73d9a02d7853d865e61c6` | unicode-script-normalization | mapped-open |
| 7206 | `src/script_span/fixunicodevalue.cc` | `373e5ee696af37e21673ade223d01f844f4c9112` | unicode-script-normalization | mapped-open |
| 7207 | `src/script_span/fixunicodevalue.h` | `ec90a9f625624c4be335cb8278afbca21cb466cc` | unicode-script-normalization | mapped-open |
| 7208 | `src/script_span/generated_entities.cc` | `3f3bacf178cc85940a1112805b79d7f1f7c193ac` | unicode-script-normalization | mapped-open |
| 7209 | `src/script_span/generated_ulscript.cc` | `8a2b39b68f0903b6c157dc0b9bd8e0f196dde288` | unicode-script-normalization | mapped-open |
| 7210 | `src/script_span/generated_ulscript.h` | `f2ce51dd16f034cc8605633ca49992c229bf2436` | unicode-script-normalization | mapped-open |
| 7211 | `src/script_span/getonescriptspan.cc` | `715616d8dfad42f459cfa5c420008add5b3127a4` | unicode-script-normalization | mapped-open |
| 7212 | `src/script_span/getonescriptspan.h` | `004f903ea87a684d1dc88b6e38751a1aeb5bcc5f` | unicode-script-normalization | mapped-open |
| 7213 | `src/script_span/getonescriptspan_test.cc` | `cb22921f31d851ca906c3ee4aeb72677b86ba9ff` | unicode-script-normalization | mapped-open |
| 7214 | `src/script_span/integral_types.h` | `0845579715d663be3361c1ed47b46c3a7253c89f` | unicode-script-normalization | mapped-open |
| 7215 | `src/script_span/offsetmap.cc` | `639fbe16d4975d87d60c2a649617a154583c0e69` | unicode-script-normalization | mapped-open |
| 7216 | `src/script_span/offsetmap.h` | `9cfe4121991d8ce564387c6efcfbbe590604b689` | unicode-script-normalization | mapped-open |
| 7217 | `src/script_span/port.h` | `2b3bc515ae89cce48414f6683a245b22f367c080` | unicode-script-normalization | mapped-open |
| 7218 | `src/script_span/stringpiece.h` | `8b80f81641bc6e3c5fb3e24d218bbb74df90cd41` | unicode-script-normalization | mapped-open |
| 7219 | `src/script_span/text_processing.cc` | `ec64ffa8725303f009ce16a07e394eb625115d54` | unicode-script-normalization | mapped-open |
| 7220 | `src/script_span/text_processing.h` | `12c5ab8e20e4998a2c97375d0481f0a76f8ec0d1` | unicode-script-normalization | mapped-open |
| 7221 | `src/script_span/utf8acceptinterchange.h` | `59adca85e0f26e78fa5343a5ca208cd50347527e` | unicode-script-normalization | mapped-open |
| 7222 | `src/script_span/utf8prop_lettermarkscriptnum.h` | `5ed3ec6905d14eecf204ec83b136c4dc071634d8` | unicode-script-normalization | mapped-open |
| 7223 | `src/script_span/utf8repl_lettermarklower.h` | `adc59d4c0dc515a93e884db1899659427b122c7e` | unicode-script-normalization | mapped-open |
| 7224 | `src/script_span/utf8scannot_lettermarkspecial.h` | `2ddad2fb6af856eea3ecfece167e9a91634d9e95` | unicode-script-normalization | mapped-open |
| 7225 | `src/script_span/utf8statetable.cc` | `8fcfb1e8f9d2bb59602b14a5e5bff91e715cc90f` | unicode-script-normalization | mapped-open |
| 7226 | `src/script_span/utf8statetable.h` | `5817c410ba16a18e68035a0be69d98ec4d7f8fa3` | unicode-script-normalization | mapped-open |
| 7227 | `src/sentence.proto` | `a5b71db6c44ee611c64e6db591e120d60c706a78` | classifier-support | mapped-open |
| 7228 | `src/sentence_features.cc` | `70d64f40ccf808582e13b052095371b7a5a697f0` | classifier-support | mapped-open |
| 7229 | `src/sentence_features.h` | `cc0be88330548dd0876d6c6919d0fec48498ddb5` | classifier-support | mapped-open |
| 7230 | `src/simple_adder.h` | `f70665eff782bb53d5b1ef96c2e77ece583847e9` | classifier-support | mapped-open |
| 7231 | `src/task_context.cc` | `4f9636882f4b769f0dd9aa69f354838486809b1e` | classifier-support | mapped-open |
| 7232 | `src/task_context.h` | `d3a5eb6c0664a2d59b7f3b579e4adefb56b93c86` | classifier-support | mapped-open |
| 7233 | `src/task_context_params.cc` | `27cf89d36bbe7c50a7e8884b2c1feed8a9890534` | language-classifier-core | mapped-open |
| 7234 | `src/task_context_params.h` | `95d865b3be5fb3afcdb5948f8e1d383e0a2a52ba` | language-classifier-core | mapped-open |
| 7235 | `src/task_spec.proto` | `b91bb1261cae829f278ee247f11b4dfb72be8683` | classifier-support | mapped-open |
| 7236 | `src/unicodetext.cc` | `67f52a0981f49d9ab00906b7e2584f3b99c9f632` | unicode-script-normalization | mapped-open |
| 7237 | `src/unicodetext.h` | `e53c870fe7e07d6e14b4d31f12eb58b54d5f8405` | unicode-script-normalization | mapped-open |
| 7238 | `src/utils.cc` | `3268a9bd0945fd12ac401328dc43763ca52f3319` | unicode-script-normalization | mapped-open |
| 7239 | `src/utils.h` | `60845cb4bb3d13927fb7bcd7237dd88eda934f1a` | unicode-script-normalization | mapped-open |
| 7240 | `src/workspace.cc` | `e48b511606885a16a3307cd41b73dd9c7ed04395` | classifier-support | mapped-open |
| 7241 | `src/workspace.h` | `d31e9ca67929da87ae7d047d45f57d2c1b463590` | classifier-support | mapped-open |

## Accounting

Read-through **7,241/16,125**; unread **8,884**; unknown **15,845**; omitted **0**. Next: **7,242** `Telegram/ThirdParty/cmark-gfm::.editorconfig@12032e647502de7a91f241f75fcd7666be375875`.