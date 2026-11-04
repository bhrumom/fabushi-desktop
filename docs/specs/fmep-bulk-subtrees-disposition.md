# FMEP-BULK-01: 非产品子树批量分类裁决规范

**Status:** 规范性执行细则（Normative execution addendum to FMEP-001 Revision 4 §15）  
**Authority:** `docs/specs/fabushi-deferred-ci-migration.md` §13 (`FMEP-SOURCE-01`) 与 §15 (`FMEP-BULK-01`)  
**Upstream Fixed Anchor:** `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

---

## 1. 范围与架构替代裁决

依据 FMEP-001 §15，固定 U 的 16,381 个递归对象中，**9,486 个第三方 gitlink 子模块对象**（分布在 39 个独立仓库中，如 `range-v3`、`hunspell`、`lib_ui`、`lib_base`、`tgcalls`、`lib_webrtc`、`MicroTeX` 等）统一归类为：
- `category: "dependency"`
- `risk: "low"`
- `review_depth: "contract"`
- `status: "complete"`

### 架构替代裁决理由
在 Fabushi 的 Rust 生产宿主 + Electron/Chromium 平台 + React/TypeScript 桌面架构下：
1. **基础工具与算法库**（`ericniebler/range-v3`、`Microsoft/GSL`、`TartanLlama/expected`、`desktop-app/lib_crl`、`desktop-app/lib_rpl`）：由 Rust 标准库迭代器、`itertools`、标准错误处理及 Tokio 异步原语替代；
2. **多媒体与音视频通话**（`TelegramMessenger/tgcalls`、`desktop-app/lib_webrtc`、`MicroTeX`、`desktop-app/lib_lottie`）：由 Chromium 原生 WebRTC 引擎、HTML5/Canvas/SVG 渲染与音频管线替代；
3. **平台与输入法绑定**（`flatpak/xdg-desktop-portal`、`KDE/kcoreaddons`、`fcitx/fcitx5-qt`、`hime-ime/hime`、`hamonikr/nimf`）：由 Electron 跨平台桌面 API、Chromium 原生 IME 与 AppHost 平台桥接替代；
4. **解析、压缩、安全与系统库**（`hunspell`、`Yubico/libfido2`、`PJK/libcbor`、`lz4`、`Cyan4973/xxHash`、`cmark-gfm`、`cld3`、`zxcvbn-c`）：由成熟 Rust 依赖生态（如 `sha2`、`pulldown-cmark`、标准压缩/加密 crate）与 Web 标准能力替代；
5. **Qt 视图与窗口构建壳**（`desktop-app/lib_ui`、`desktop-app/lib_base`、`desktop-app/lib_webview`、`codegen`、`cppgir` 等）：由 React/TypeScript 生产渲染层、统一设计系统组件及 Electron WebView 宿主替代。

---

## 2. 两个挂载方案对比与 FMEP-STATE-01 影响

校验器 `scripts/tdrp-source-review.mjs`（`validateSourceReviewRecords`，约 164–199 行）要求：每条 review 的 identity 必须在 inventory 中；category/risk/review_depth 必须取枚举值；dependency/generated 不能用 disposition 深度；product/mixed-ui 必须 deep 且 `responsibility_ids` 非空；且每个声明的 `responsibility_id` 都必须通过 row 的 `source_*` 或 `additional_source_identities` 覆盖该 identity。

针对 9,486 个 dependency 对象的挂载，对比以下两种方案：

| 维度 | 方案 1：子模块治理职责行挂载 (Governance Rows) | 方案 2：空责任链接直接契约审查 (Empty Responsibility IDs) |
| --- | --- | --- |
| **挂载形式** | 在 `ledger.rows` 新增 39 行治理职责（如 `TDRP-DEP-RANGE-V3-001`），并将子模块内数千个 identity 填入 `additional_source_identities`；review 的 `responsibility_ids` 挂载对应行。 | 不在 `ledger.rows` 新建任何行；在 `ledger.source_reviews` 中声明 `responsibility_ids: []`。 |
| **校验器规则** | 满足 `validateSourceReviewRecords`（因每条 review 的 identity 均被对应 row 覆盖），但显著加重关系遍历与哈希开销。 | **原生满足**：校验器第 188 行仅对 `product/mixed-ui` 强制非空；第 191 行循环在数组为空时跳过，直接通过校验。 |
| **Schema 契约** | 符合 `parity-ledger.schema.json`，但 `additional_source_identities` 会包含巨型数组。 | **原生符合**：Schema 仅在 `category in ["product", "mixed-ui"]` 时要求 `minItems: 1`，对 dependency 完全允许 `[]`。 |
| **FMEP-STATE-01 计数影响** | **严重污染**：`rows` 虚增 39 行治理职责，稀释责任总数；违背“分母尚未完整分解时不得混入产品完成率”的治理原则。 | **零污染**：`rows` 保持现存 187 行不变，产品职责（19 行）与领域库接线（3 行）分母绝对纯净，进度总览表保持客观。 |
| **FMEP-HYGIENE-01 卫生门禁** | **严重违规**：单行包含多达 3,201 个对象（产生超级行，违反 Super-Row Decomposition 门禁），且必须虚构本地目标文件与符号。 | **完全合规**：零超级行，无须伪造虚假目标路径或符号，账本增量完全局限于 `source_reviews` 数组内。 |
| **体积与维护开销** | 9,486 个 identity 对象在 `rows` 与 `source_reviews` 中双重存储，使 ledger 体积膨胀 >3.5 MB。 | 单重存储，产物确定且结构紧凑，生成与校验耗时 < 0.2 秒。 |

---

## 3. 最终决策与推荐方案

**采纳推荐方案 2（空责任链接直接契约审查）**：
1. **免除修改**：现存 JSON Schema 与 `tdrp-source-review.mjs` 校验器已 100% 原生支持该方案，无须引入任何破坏性代码变更；
2. **架构隔离**：将上游 C++ 依赖处置严格收敛在 `source_reviews` 层级，不侵入 Fabushi 产品消息的生产责任图；
3. **闭环取证**：生成器按规范输出确定性 records，并在 `evidence` 中统一锚定本规范小节。

<a id="submodule-contract-disposition"></a>
### 规范性证据锚点（Normative Evidence Anchor）
所有通过 `scripts/tdrp-bulk-disposition.py` 生成的第三方子模块 `source_reviews`，其 `evidence` 字段统一引用：
`docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition`
