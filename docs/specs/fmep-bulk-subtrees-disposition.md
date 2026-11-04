# FMEP-BULK-01: 非产品子树批量分类裁决规范

**Status:** 规范性执行细则（Normative addendum to FMEP-001 Revision 4 §15）  
**Authority:** `docs/specs/fabushi-deferred-ci-migration.md` §13 (`FMEP-SOURCE-01`) 与 §15 (`FMEP-BULK-01`)  
**Upstream Anchor:** `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`（完整递归清单 16,381 项）

---

## 1. 范围与风险分层原则

依据 FMEP-001 §15 与 FMEP-SOURCE-01 规范，固定 upstream 锚点（commit `f985723b`）的 16,381 个递归对象中，**9,486 个第三方 gitlink 子模块对象**（分布在 39 个独立仓库，覆盖 40 处子模块挂载实例）依据代码来源、产品语义关联度与安全影响划分为以下风险分层：

### 1.1 高风险 Telegram 专属子模块（High / Pending / Deep）
- **范围**：`desktop-app/lib_storage`、`TelegramMessenger/tgcalls`、`desktop-app/lib_tl`（3 个子模块，共 341 项）。
- **裁决**：`category: "dependency"`, `risk: "high"`, `review_depth: "deep"`, `status: "pending"`, `responsibility_ids: []`。
- **分层理由**：此类仓库为 Telegram 核心专属协议、媒体与存储实现，包含关键产品语义（本地加密存储格式、端到端音视频通话状态机与信令、TL 协议编译与反序列化）。依据 FMEP-SOURCE-01，不得仅凭目录位置免除产品责任，必须保持 `pending` 并升级为 `deep review`，待后续纵向切片（如 W1-B / Wave 2 通话与存储重构）专门评估。

### 1.2 中风险 Telegram 专属子模块（Medium / Pending / Deep）
- **范围**：`desktop-app/lib_ui`、`desktop-app/lib_base`、`desktop-app/codegen`、`desktop-app/lib_crl`、`desktop-app/lib_webrtc`、`desktop-app/lib_spellcheck`、`desktop-app/lib_lottie`、`desktop-app/lib_rpl`、`desktop-app/lib_webview`、`desktop-app/libprisma`、`desktop-app/lib_translate`、`desktop-app/lib_qr`（12 个子模块，共 989 项）。
- **裁决**：`category: "dependency"`, `risk: "medium"`, `review_depth: "deep"`, `status: "pending"`, `responsibility_ids: []`。
- **分层理由**：此类仓库为 Telegram 专属界面、工具、异步框架与包装层（如 Qt 气泡渲染、贴纸动画播放、WebView 容器包装等），虽属基础设施或辅助库，但可能耦合特定产品交互或协议语义。依据 FMEP-SOURCE-01，必须保持 `pending` 并设为 `deep review`，逐个评估其在 Fabushi 架构下的产品适用性。

### 1.3 低风险通用第三方待核子模块（Low / Pending / Contract）
- **范围**：`nayuki/QR-Code-generator`（1 个子模块，共 52 项）。
- **裁决**：`category: "dependency"`, `risk: "low"`, `review_depth: "contract"`, `status: "pending"`, `responsibility_ids: []`。
- **分层理由**：通用二维码生成算法库，无专有业务语义；但由于 inventory 中未发现独立 `LICENSE` 或 `COPYING` 文件，依照许可证核对规则，类型待核期间保持 `pending`，待后续补充源文件头等证据闭环。

### 1.4 中风险通用安全/密码学第三方库（Medium / Complete / Contract）
- **范围**：`Yubico/libfido2`、`PJK/libcbor`（2 个子模块，共 437 项）。
- **裁决**：`category: "dependency"`, `risk: "medium"`, `review_depth: "contract"`, `status: "complete"`, `responsibility_ids: []`。
- **分层理由**：`libfido2`（硬件认证）与 `libcbor`（二进制反序列化）为通用第三方上游开源库，处理非受信网络输入与加密认证，安全风险敏感，故风险定级为 `medium`。经全仓构建配置审查，Fabushi 不构建、不链接、不分发此两库（构建可达性：否），无运行时分发义务，完成 `contract` 契约闭环。

### 1.5 低风险通用第三方库及构建工具（Low / Complete / Contract）
- **范围**：`ericniebler/range-v3`、`hunspell/hunspell`、`hime-ime/hime`、`flatpak/xdg-desktop-portal`、`KDE/kcoreaddons`、`KDE/kimageformats`、`hamonikr/nimf`、`desktop-app/MicroTeX`、`fcitx/fcitx5-qt`、`lz4/lz4`、`desktop-app/cmark-gfm`、`tzcnt/TooManyCooks`、`gitlab.com/mnauw/cppgir`、`desktop-app/cmake_helpers`、`Cyan4973/xxHash`、`google/cld3`、`Microsoft/GSL`、`martinmoene/expected-lite`、`TartanLlama/expected`、`tsyrogit/zxcvbn-c`、`nnen/doxygen-theme`（21 个子模块，共 7,667 项）。
- **裁决**：`category: "dependency"`, `risk: "low"`, `review_depth: "contract"`, `status: "complete"`, `responsibility_ids: []`。
- **分层理由**：均为通用工具、算法、数学公式渲染或输入法支持库，无任何 Telegram 专有业务语义。经全仓构建配置审查，构建可达性均为否，完成 `contract` 契约闭环。

---

## 2. 挂载方案对比与 FMEP-STATE-01 影响

校验器 `scripts/tdrp-source-review.mjs` 中的 `validateSourceReviewRecords`（第 164–199 行）对 `source_reviews` 记录的完整性与挂载关系执行校验：

```javascript
// scripts/tdrp-source-review.mjs
const product = ['product', 'mixed-ui'].includes(review.category);
if (!pending && (product || review.risk === 'high') && review.review_depth !== 'deep') throw Error('deep review required');
if (!pending && ['generated', 'dependency'].includes(review.category) && review.review_depth === 'disposition') throw Error('contract review required');
if (!uniqueTextArray(review.responsibility_ids) || (!pending && product && !review.responsibility_ids.length)) {
  throw Error('missing or duplicate responsibility links');
}
for (const id of review.responsibility_ids) {
  const related = sourcesByResponsibility.get(id);
  if (!related) throw Error('unmapped responsibility: ' + id);
  if (!related.has(key)) throw Error('responsibility does not cover source identity: ' + id + ' ' + key);
}
```

### 方案对比

| 评估维度 | 方案 1：治理职责行挂载（不推荐） | 方案 2：空责任链接直接契约审查（采纳） |
| --- | --- | --- |
| **挂载实现** | 在 `ledger.rows` 新增 39 行治理职责（如 `TDRP-DEP-*`），并在 `additional_source_identities` 中填入全部 9,486 个 identity。 | 在 `ledger.source_reviews` 中声明 `responsibility_ids: []`，不增加 `ledger.rows`。 |
| **门禁合规 (FMEP-HYGIENE-01)** | 严重破坏超级行门禁：`range-v3` 单行将包含 3,201 项附加源，单行大小突破数十 KB。 | 完全符合规范：`ledger.rows` 零超级行，无须伪造虚构的本地符号。 |
| **产品分母 (FMEP-STATE-01)** | 污染责任分母：分母虚增 39 行非产品职责，稀释真实产品完成度。 | 保持分母客观：`ledger.rows` 保持 187 行不变，完全隔离产品业务与非产品依赖。 |
| **校验器兼容性** | 校验通过，但账本文件体积膨胀。 | 原生兼容：`tdrp-source-review.mjs` 第 188 行仅对 `product` 强制要求非空，`dependency` 允许 `responsibility_ids: []`。 |
| **候选模式行为** | 候选模式与验收模式均加载这 39 行。 | 候选模式（`allowPending=true`）下，8,104 条 complete 计入 `reviewed`，1,382 条 pending 记录计入 `recorded`，全量 9,486 项通过校验。验收模式（`allowPending=false`）下，因 1,382 条待决而安全拒绝，防止未决语义提前关闭。 |

---

## 3. 39 个子模块全量证据表

### 构建可达性审查说明
- **审查范围**：全仓所有清单与构建文件，共 207 处（`Cargo.toml`、`Cargo.lock`、`package.json`、`package-lock.json`、`.gitmodules`、`CMakeLists.txt`、`*CMake*`）。
- **执行命令**：`git grep -i -E '\b<pattern>\b' -- '*Cargo.toml' '*Cargo.lock' '*package.json' '*package-lock.json' '.gitmodules' '*CMake*'`。
- **事实结论**：除 `pulldown-cmark` 确认存在于 `source/mahayana/mahayana-rs/Cargo.lock` 承接 Markdown 解析，以及本地自有 `codegen` crate 仅与子模块同名外，所有 39 个子模块全仓匹配均为 **0 出现**。Fabushi 构建系统完全不引用、不编译、不链接、不分发此 39 个子模块。

### 全量 39 个子模块证据表

| 序号 | 子模块 (Repository) | 对象数 | 分层 (Risk / Status / Depth) | 分层理由（产品语义评估 / 依赖性质） | 构建可达性搜索模式 | 搜索结果与位置 | 许可证证据（Path & Blob SHA） | 仓库内替代实现证据 |
| ---: | --- | ---: | --- | --- | --- | --- | --- | --- |
| 1 | `ericniebler/range-v3` | 3,202 | low / complete / contract | 通用 C++20 ranges 算法库，无产品语义 | `\brange-v3\b\|\brange_v3\b` | 未出现（0 匹配） | `LICENSE.txt` (blob `698193e974ec62c17bbd913769336de5e9e85ce2`, BSL-1.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 2 | `hunspell/hunspell` | 854 | low / complete / contract | 通用拼写检查算法引擎，无产品语义 | `\bhunspell\b` | 未出现（0 匹配） | `COPYING` (blob `1f963da0...`, GPL-2.0), `COPYING.LESSER` (blob `67cd97bb...`, LGPL-2.1), `COPYING.MPL` (blob `7714141d...`, MPL-1.1) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 3 | `hime-ime/hime` | 580 | low / complete / contract | Linux 输入法平台支持库，无产品语义 | `\bhime\b` | 未出现（0 匹配） | `icons/*/COPYING` (GPL-2.0-or-later) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 4 | `desktop-app/lib_ui` | 432 | medium / pending / deep | Telegram 专属 UI 交互、组件绘制与消息气泡排版层，含核心聊天 UI 行为语义 | `\blib_ui\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | Telegram 专属 UI 交互，待评估产品语义 |
| 5 | `flatpak/xdg-desktop-portal` | 417 | low / complete / contract | Linux XDG 桌面门户客户端库，无产品语义 | `\bxdg-desktop-portal\b` | 未出现（0 匹配） | `COPYING` (blob `4362b49151d7b34ef83b3067a8f9c9f877d72a0e`, LGPL-2.1) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 6 | `KDE/kcoreaddons` | 377 | low / complete / contract | KDE 核心工具与类型封装库，无产品语义 | `\bkcoreaddons\b` | 未出现（0 匹配） | `LICENSES/LGPL-2.0-or-later.txt` (blob `5c96471a...`, LGPL-2.0-or-later) 等 | Fabushi 未提供等价实现；产品适用性另行评估 |
| 7 | `KDE/kimageformats` | 375 | low / complete / contract | KDE 图像格式解码插件库，无产品语义 | `\bkimageformats\b` | 未出现（0 匹配） | `LICENSES/LGPL-2.0-or-later.txt` (blob `5c96471a...`, LGPL-2.0-or-later) 等 | Fabushi 未提供等价实现；产品适用性另行评估 |
| 8 | `hamonikr/nimf` | 365 | low / complete / contract | Linux 输入法框架支持库，无产品语义 | `\bnimf\b` | 未出现（0 匹配） | `COPYING` (blob `341c30bda4457610db4829294e5d7e526d4c835b`, LGPL-3.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 9 | `TelegramMessenger/tgcalls` | 310 | high / pending / deep | Telegram 专属音视频通话引擎与信令状态机，含端到端 VoIP 通话关键语义 | `\btgcalls\b` | 未出现（0 匹配） | `LICENSE` (blob `65c5ca88a67c30becee01c5a8816d964b03862f9`, GNU LGPL v3) | 通话信令与状态机，待评估产品语义 |
| 10 | `desktop-app/MicroTeX` | 279 | low / complete / contract | 通用 LaTeX 数学公式渲染引擎，无产品语义 | `\bMicroTeX\b` | 未出现（0 匹配） | `LICENSE` (blob `78a9eca5ca3514f2fd0a2e81018eeb8ae7d981a2`, MIT) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 11 | `Yubico/libfido2` | 260 | medium / complete / contract | 通用 FIDO2/WebAuthn 硬件认证库，安全敏感组件 | `\blibfido2\b` | 未出现（0 匹配） | `LICENSE` (blob `62f584be02dd06d5043d86abff816332e8aee5a5`, BSD-2-Clause) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 12 | `desktop-app/lib_base` | 247 | medium / pending / deep | Telegram 专属基础基础设施库，含跨平台底层抽象与基础语义 | `\blib_base\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 基础框架及工具，待评估产品语义 |
| 13 | `fcitx/fcitx5-qt` | 215 | low / complete / contract | Linux Fcitx5 输入法 Qt 前端集成库，无产品语义 | `\bfcitx5-qt\b` | 未出现（0 匹配） | `LICENSES/LGPL-2.1-or-later.txt` (blob `04bb156e...`, LGPL-2.1-or-later) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 14 | `lz4/lz4` | 187 | low / complete / contract | 通用快速压缩/解压算法库，无产品语义 | `\blz4\b` | 未出现（0 匹配） | `lib/LICENSE` (blob `48849169...`, BSD-2-Clause) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 15 | `PJK/libcbor` | 177 | medium / complete / contract | 通用 CBOR 二进制数据反序列化库，处理非受信数据输入，安全敏感组件 | `\blibcbor\b` | 未出现（0 匹配） | `LICENSE.md` (blob `49e9b5390364c71a2619d8059b7e1511b8df1e84`, MIT) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 16 | `desktop-app/cmark-gfm` | 162 | low / complete / contract | 通用 GitHub Flavored Markdown 解析库，无产品语义 | `\bcmark-gfm\b` | 未出现（0 匹配） | `COPYING` (blob `db88a81b3ec3513cda9a4822565fbc06d958b74a`, BSD-2-Clause) | `pulldown-cmark` (`source/mahayana/mahayana-rs/Cargo.lock`) |
| 17 | `tzcnt/TooManyCooks` | 126 | low / complete / contract | 通用 C++ 并发任务调度算法库，无产品语义 | `\bTooManyCooks\b` | 未出现（0 匹配） | `LICENSE` (blob `36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`, BSL-1.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 18 | `gitlab.com/mnauw/cppgir` | 104 | low / complete / contract | GObject-Introspection C++ 绑定生成器，构建期工具，无产品语义 | `\bcppgir\b` | 未出现（0 匹配） | `LICENSE` (blob `d113221e5489255cc4c27d8d4015f23aed0bf472`, MIT) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 19 | `desktop-app/cmake_helpers` | 102 | low / complete / contract | CMake 构建配置通用辅助脚本，无产品语义 | `\bcmake_helpers\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（CMake 辅助脚本） | Fabushi 未提供等价实现；产品适用性另行评估 |
| 20 | `Cyan4973/xxHash` | 100 | low / complete / contract | 通用非加密哈希算法库，无产品语义 | `\bxxHash\b` | 未出现（0 匹配） | `LICENSE` (blob `e4c5da7234eca7baad9a46f0b144e49ccad2e500`, BSD-2-Clause) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 21 | `google/cld3` | 87 | low / complete / contract | 通用自然语言语种识别神经网络模型库，无产品语义 | `\bcld3\b` | 未出现（0 匹配） | `LICENSE` (blob `c5899b26febde1d80fadd24c32254e70c68544ae`, Apache-2.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 22 | `desktop-app/codegen` | 56 | medium / pending / deep | Telegram 专属代码生成器，负责样式与序列化生成，可能含生成与协议语义 | `desktop-app/codegen` | 未出现（仓内同名 `codegen` 为独立 Rust crate，非此代码） | 无独立 LICENSE 文件（代码生成工具） | Qt 代码生成逻辑，待评估产品语义 |
| 23 | `nayuki/QR-Code-generator` | 52 | low / pending / contract | 通用二维码生成算法库，无独立 LICENSE 文件，类型待核 | `\bqrcodegen\b` | 未出现（0 匹配） | 无独立 LICENSE 文件，类型待核（Readme.markdown 声明待核） | Fabushi 未提供等价实现；产品适用性另行评估 |
| 24 | `desktop-app/lib_crl` | 51 | medium / pending / deep | Telegram 专属协程与异步调度库，含核心并发调度语义 | `\blib_crl\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 异步调度库，待评估产品语义 |
| 25 | `Microsoft/GSL` | 44 | low / complete / contract | 微软 C++ Core Guidelines 支持库，无产品语义 | `\bMicrosoft/GSL\b` | 未出现（0 匹配） | `LICENSE` (blob `aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`, MIT) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 26 | `martinmoene/expected-lite` | 43 | low / complete / contract | C++11 单头文件 std::expected polyfill 库，无产品语义 | `\bexpected-lite\b` | 未出现（0 匹配） | `LICENSE.txt` (blob `36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`, BSL-1.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 27 | `desktop-app/lib_webrtc` | 40 | medium / pending / deep | Telegram 专属 WebRTC 包装适配层，含音视频设备采集与流转关键语义 | `\blib_webrtc\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | WebRTC 包装层，待评估产品语义 |
| 28 | `desktop-app/lib_spellcheck` | 39 | medium / pending / deep | Telegram 专属拼写检查系统对接逻辑，含编辑器输入交互语义 | `\blib_spellcheck\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 拼写检查封装，待评估产品语义 |
| 29 | `desktop-app/lib_lottie` | 33 | medium / pending / deep | Telegram 专属 Lottie/TGS 动画解析与贴纸播放器，含贴纸动画与表情语义 | `\blib_lottie\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 动画渲染层，待评估产品语义 |
| 30 | `desktop-app/lib_rpl` | 33 | medium / pending / deep | Telegram 专属响应式流式编程库，含界面事件响应与绑定语义 | `\blib_rpl\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 响应式编程库，待评估产品语义 |
| 31 | `desktop-app/lib_webview` | 33 | medium / pending / deep | Telegram 专属本地 WebView 容器包装，含 Bot WebApp 唤起与消息桥接语义 | `\blib_webview\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | WebView 容器包装，待评估产品语义 |
| 32 | `desktop-app/lib_storage` | 25 | high / pending / deep | Telegram 专属本地加密数据库存储格式与密钥派生，含本地数据持久化关键产品语义 | `\blib_storage\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 加密存储格式，待评估产品语义 |
| 33 | `TartanLlama/expected` | 24 | low / complete / contract | C++ std::expected 实现库，无产品语义 | `\bTartanLlama/expected\b` | 未出现（0 匹配） | `COPYING` (blob `0e259d42c996742e9e3cba14c677129b2c1b6311`, CC0-1.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 34 | `desktop-app/libprisma` | 18 | medium / pending / deep | Telegram 专属语法高亮与主题调色板解析器，含主题配色语义 | `\blibprisma\b` | 未出现（0 匹配） | `LICENSE` (blob `1941f980b3a4f3e3828705a9db5fe5c053f505ac`, MIT) | 主题解析层，待评估产品语义 |
| 35 | `tsyrogit/zxcvbn-c` | 18 | low / complete / contract | 通用密码强度估算 C 库，无产品语义 | `\bzxcvbn-c\b` | 未出现（0 匹配） | `LICENSE.txt` (blob `5dd0b430b5cd87848bf1c6467c5b4dd370af5175`, MIT) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 36 | `desktop-app/lib_tl` | 6 | high / pending / deep | Telegram 专属 Type Language 编译器与二进制解析，含 MTProto 通信协议核心语义 | `\blib_tl\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | TL 协议 schema 生成，待评估产品语义 |
| 37 | `nnen/doxygen-theme` | 6 | low / complete / contract | Doxygen 文档 HTML CSS 主题，开发期文档展示，无产品语义 | `\bdoxygen-theme\b` | 未出现（0 匹配） | `LICENSE` (blob `cc9abbf534bab2887c1658598278543e672282e9`, GPL-3.0) | Fabushi 未提供等价实现；产品适用性另行评估 |
| 38 | `desktop-app/lib_translate` | 4 | medium / pending / deep | Telegram 专属文本翻译服务接口包装，含消息翻译集成语义 | `\blib_translate\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 翻译提供层，待评估产品语义 |
| 39 | `desktop-app/lib_qr` | 3 | medium / pending / deep | Telegram 专属二维码渲染窗口与桌面端登录扫码组件，含登录流程语义 | `\blib_qr\b` | 未出现（0 匹配） | 无独立 LICENSE 文件（归属 Telegram GPL-3.0 体系） | 二维码展示层，待评估产品语义 |
| **合计** | **39 个子模块** | **9,486** | — | **8,104 complete / 1,382 pending** | — | — | — | **23 个纯通用库 contract 闭环，16 个专有/待核库 pending 待决** |

---

## 4. 许可证合规性与分发义务裁决

依据上述构建可达性审查事实：
1. **构建与链接可达性**：Fabushi 的 Rust 生产构建管线与 Electron/Web 前端应用均不构建、不链接、不打包上述 39 个子模块中的任何源码或编译目标文件（构建可达性：否）。
2. **分发义务**：开源许可证（如 LGPL-2.1、LGPL-3.0、GPL-3.0、BSD、MIT、Apache-2.0 等）的版权声明、反向工程与提供源码等分发附带义务均以**二进制发布或代码再分发**为触发条件。鉴于 Fabushi 既未分发上述任何代码或衍生二进制，也未在运行时链接加载，故**无运行时许可证分发义务**。
3. **保留待核项**：对于无独立 LICENSE 文件或包含专有实现潜力的 16 个子模块（15 个 Telegram 专属子模块及 `nayuki/QR-Code-generator`），由于其处于 `status: "pending"` 状态，许可证最终合规性随其后续产品语义评审结论一并处置。

---

## 5. 证据锚点与结论

所有通用第三方库的 `source_reviews` 的 `evidence` 字段统一引用：  
`docs/specs/fmep-bulk-subtrees-disposition.md#submodule-contract-disposition`。

该规范使 23 个纯通用库（8,104 项对象）在候选模式与契约审查中达成客观、可追溯的 `contract` 闭环；同时将 16 个专有或含潜在产品语义及待核许可证的子模块（1,382 项对象）稳妥置于 `pending` 监控之下，避免虚构完成度或遗漏关键业务语义。

<a id="submodule-contract-disposition"></a>

---

## 6. 本地验证与基线环境对齐说明

在本地复核测试中，对套件运行事实如实记录如下：
1. **批量处置专用测试**：
   - `node --test scripts/tdrp-bulk-disposition.test.mjs`：14 项测试全部通过（pass 14, fail 0）。
   - `python3 scripts/tdrp-bulk-disposition.test.py`：14 项测试全部通过（Ran 14 tests, OK）。
   - `python3 scripts/tdrp-source-review-schema.test.py`：37 项测试全部通过（Ran 37 tests, OK）。
2. **主审查合约测试环境说明 (`scripts/tdrp-source-review.test.mjs`)**：
   - 在 macOS 默认环境下（系统默认临时目录 `TMPDIR=/var/folders/...`，其中 `/var` 是指向 `/private/var` 的软链接），直接执行 `node --test scripts/tdrp-source-review.test.mjs` 结果为 **pass 144 / fail 7**。
   - 7 个失败用例均为候选恢复用例（如 `candidate recovery rejects duplicate member paths and traversal`），失败原因为 `validate-tdrp-source-candidate.mjs` 中的 `noSymlinks` 检查将包含系统默认软链接的临时目录路径拦截（错误信息：`AssertionError: candidate evidence: symlink in evidence path`）。
   - 经在基线锚点 `59de520c4` 上对比复测，结果同样为 pass 144 / fail 7，证实该现象为 macOS 默认环境既有行为，与基线完全一致，非本次改动引入。
   - 若在消除路径软链接的环境中执行（如显式设置 `TMPDIR=/private/tmp`），则 151 项测试全部通过（pass 151, fail 0）。
