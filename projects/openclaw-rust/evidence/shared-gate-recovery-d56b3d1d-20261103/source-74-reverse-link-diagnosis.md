# H74 共享来源门失败：反向责任链接只读因果报告

## 结论与边界

**本次是来源账本集成缺项（IMPL_BUG / metadata integration），不是已证明的产品运行时缺陷，也不是应被放宽的 validator 断言。** `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` 已进入唯一 ledger 与四条 RTM 行，但其五个固定 U 来源行都未加入该责任的反向 ID，且 `inventory/index.json` 没有既有合同要求的 `pending_mapping_additions`。`--before-expansion` 因而正确拒绝了这组未声明的增补。

最小修复：index 顶层声明五个 tuple 和旧 canonical digest；旧 rows/counter 不动。既有 Actions expand→shard→strict gate 后 CAS 发布派生提交。无需改 schema、validator、tests、锁、RTM 或产品代码。

只读 Git/API/source/std 与日志；未运行项目 helper/validation，未改 repo、push/dispatch 或读 peer review 报告。此为静态诊断；model/effort 继承。

## 1. 冻结身份与保留证据

- Target H：`74f11d3d31c6245ac5886dc0181460b2d4349744`；tree：`aca91e1cfbedeecb039d442fb3528caac89873a3`。
- Evidence 后继：`b15e58957a597997363e1273f2f32a222365c310`。已只读比较 H→后继，只有两个 evidence 文件变化；此处涉及的 lock/index/shards/ledger/RTM/scripts/workflows 均相同。
- 固定 U：`openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`；tree：`90c8aab39972da11cfad220361781d49e1bc6700`。lock、index、ledger、RTM、manifest 一致。`live_candidate` 是历史 discovery，不是本次修复输入。
- Run `41580539540` attempt `1`，job `125734077461`，原始日志 `/tmp/ar-74-source-evidence/source-original-decoded.log`，28,920 bytes，sha256 `31f25cb8af36bfb77ebfbd4179ea5728774379f2cf70f7dddebf2e1f2fc9dd1c`。
- 日志 163–179：`test_source_review.py` 的 **33 tests OK**；295–325：`test_inventory_materialization.py` 的 **21 tests OK**；326–338：pre-expansion 在 `source_review.py:94` 抛出 `Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001`，exit 1。前两组正面证据保留；它们是 synthetic/shared-contract 控制，不是这五条真实声明已经闭合的证据。
- workflow 使用 `set -euo pipefail`；失败在 `expand_inventory.py` 之前，未走到真实候选 expansion/shard write、post-materialization strict check 或 authority artifact 上传。不能把后续未执行项记为 pass。

已读权威：H `AGENTS.md:371–395`，主 Spec §17.2（L690–696），SOT 当前固定身份段，module-map M23，P0 task，MSU spec L7–13、MAC-v1。§17.2 要求单一 inventory/ledger/RTM，阅读、映射、审查事实分立，禁止手改零计数。

## 2. 现有责任与精确缺项

H `parity-ledger.json:36810,36859–36944`：责任为 `business-runtime`、unit `OCRM-UNIT-MEDIA-008`，existing owner `source/host/src/extensions/session/agent_db.rs`。状态保持 `read=partial`、`mapping=mapped`、`implementation=implementing`、`wiring=wired`、`verification=not-run`、`acceptance=pending`，`reviewer=null`。这正符合 pending 增补的三个状态前提；不能为修门伪升为 reviewed/verified/accepted。

RTM `OCRM-R23/R24/AC23/AC24` 都已有该责任，均 `acceptance_status=in-progress`、`not_applicable_reason=null`，保留 MSU-01…10 和两个 blocker：`BL-MEDIA-FINAL-CUSTODY-FULL-EQUIVALENCE-001`、`BL-MEDIA-SETTLEMENT-EXACT-H-EVIDENCE-001`。因此不是 RTM 漏绑或应删除新责任。

下表 blob 均与固定 U GitHub 返回的 `sha`、ledger source_ref 和原始 JSONL 行一致；source_ref 全文范围也与返回文件行数一致。JSONL 行号是对应 gzip 解压后的 1-based 行号。

| 来源 path | blob_sha | source_ref 范围 | shard / JSONL 行 | 原有 mapping |
| --- | --- | --- | --- | --- |
| `src/gateway/agent-turn/agent-run-final-media.ts` | `f4bf5b4308cb2a7160ad1db0142160d14fb71551` | 1–243 | 0036 / 96 | unclassified，IDs=[] |
| `src/gateway/agent-turn/agent-run-media.ts` | `27a3a65974fad078872b257ee28cafccdad4691f` | 1–127 | 0036 / 98 | mapped，已有 `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` |
| `src/infra/sqlite-worker-error.ts` | `b13f8354e7244d630ee274b3b0debf9a3f5bb8c6` | 1–135 | 0043 / 463 | unclassified，IDs=[] |
| `src/gateway/managed-image-attachments.custody.ts` | `1975104ea6cdcfea25d3bc4362edbaea5e673adf` | 1–167 | 0036 / 956 | unclassified，IDs=[] |
| `src/media/store.read-scope.ts` | `73515a9ed6a2dc9f1a0adddc638d542ad1323ee5` | 1–269 | 0044 / 914 | unclassified，IDs=[] |

只读 ledger↔shards 关联的缺边恰为以上五条。path/blob 均已存在，第二项原责任必须保留，所以仅新增 **4 个 mapped paths**，来源分母不变。

五行现存 `read_status=unread`、`read_ranges=[]`、`review_status=pending`、`category/risk=unknown`、`required_review_depth=deep`。本次读取源文件不能追溯改写 H 的阅读事实，也不授权变更这些字段。

### 固定 U 源码语义核对

五文件均由 GitHub `fetch_file` 指定 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` 获取，返回 `sha` 如表。精确源行：final-media L209/231/234–235 保留 committed/unknown；agent-run-media L104–122 由 dispatch authority 调 finalizer；sqlite-worker-error L81–135 识别 canonical unknown；custody L72/90 保留 uncertain insert/cleanup 字节；read-scope L111–121、132–178 保护 delegated settlement。引用有真实源行为依据；未宣称目标完整等价。

## 3. 因果链与已存在的 pending 合同

1. `verify_audit_integrity.py:42–97` 先核对 lock/index/ledger/RTM、schema、shard/canonical digest、旧分母和 row→ledger/review。
2. `source_review.py:51–70` 的 `pending_mapping_additions(index, prior_digest)`：字段不存在即空集合。若存在，外层键必须恰为 `from_canonical_inventory_sha256` 与非空 `entries`；每 entry 键必须恰为 `responsibility_id/path/blob_sha`，值是非空字符串、tuple 不重复，digest 必须绑定当前未改变的 prior manifest。
3. `source_review.py:73–103` 从每条 ledger source_ref 查找同 path/blob 行。缺反向 ID 只在 `before_expansion=True`、ledger `mapping=mapped`、`verification=not-run`、`acceptance=pending` 且 exact tuple 已声明时允许。最后要求 pending 集合恰等于真实缺边，并保持**当前已物化** mapped path 集合等于 linked path 集合。
4. 本 index 无 pending declaration，所以第一条 final-media source_ref 进入 L94 失败。其 blob/path 存在且正确，状态前提也正确，缺的是声明或已物化边。
5. `expand_inventory.py:35–88` 先保护旧历史的 hash/membership/order/source identity/size；L106–160 以 ledger 为责任侧 authority，按 exact `(path,blob_sha)` **并集**补齐 reverse IDs，只有原 `unclassified` 才改 `mapped`。它不会从责任的 `read=partial` 推断 source read。
6. `shard_inventory.py:90–100` 先确认声明中的每条边已出现在候选相同 blob 行；L105–134 写 deterministic shards/manifest/dependency declarations；L135–163 移除已消费 pending 并从真实 rows 投影计数；随后 `verify` 比较完整重构。
7. source workflow L158–166 顺序是 contracts → precheck → expand → shard → `validate_p0 --negative-probes` → strict integrity。不能把 precheck 移到 expand 后，让 generator 静默填平不合法旧历史。

既有未知/deep/pending 合同无需改动。有效 assertions 包括：identity drift、旧 history digest、精确 pending 集、unused/duplicate pending、丢失旧反向边、mapping 分母、正常 committed gate 不接受 pending、review/acceptance 不走 pending；全部应保留。

## 4. root 最小修复：既有 index 声明

建议的手写变更路径仅 `projects/openclaw-rust/inventory/index.json`。在顶层增加以下字段，其他字段保持原值。它不是 canonical inventory 行或第二套账本；不得放进 `source-inventory.schema.json` 所约束的候选 payload。

```json
"pending_mapping_additions": {
  "from_canonical_inventory_sha256": "8994988c264876be709930e2f87c6b60ae124a6077a833ccd324dc1117d26d93",
  "entries": [
    {"responsibility_id":"OCRM-RESP-MEDIA-FINAL-CUSTODY-001","path":"src/gateway/agent-turn/agent-run-final-media.ts","blob_sha":"f4bf5b4308cb2a7160ad1db0142160d14fb71551"},
    {"responsibility_id":"OCRM-RESP-MEDIA-FINAL-CUSTODY-001","path":"src/gateway/agent-turn/agent-run-media.ts","blob_sha":"27a3a65974fad078872b257ee28cafccdad4691f"},
    {"responsibility_id":"OCRM-RESP-MEDIA-FINAL-CUSTODY-001","path":"src/gateway/managed-image-attachments.custody.ts","blob_sha":"1975104ea6cdcfea25d3bc4362edbaea5e673adf"},
    {"responsibility_id":"OCRM-RESP-MEDIA-FINAL-CUSTODY-001","path":"src/infra/sqlite-worker-error.ts","blob_sha":"b13f8354e7244d630ee274b3b0debf9a3f5bb8c6"},
    {"responsibility_id":"OCRM-RESP-MEDIA-FINAL-CUSTODY-001","path":"src/media/store.read-scope.ts","blob_sha":"73515a9ed6a2dc9f1a0adddc638d542ad1323ee5"}
  ]
}
```

落地时重核 prior digest。并发物化若改变 manifest，重读真实缺边，不能只换锚点。b15 此处与 H74 相同。

声明态保持 `mapped_paths=884`、`unclassified=61246`、`unread=62067`，因为这些仍描述旧 shard 事实。若只加声明却先把 counter 改为 888/61242，precheck 会正确报告 coverage/index drift。若先在 shard 补边又保留旧 pending，precheck 会正确拒绝 unused additions；这是两种阶段误用。

## 5. Actions 物化必须一起发布的字段/身份

| 项目 | 旧值 → 此五边物化后的值 |
| --- | --- |
| 五行 `responsibility_ids` | 并集增加 `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`；保留第二行原 ID |
| 四行 `responsibility_status` | `unclassified → mapped`；已有 mapped 行保持 mapped |
| ledger `file_coverage.mapped_paths` | 884 → 888 |
| ledger `file_coverage.unclassified_paths` | 61246 → 61242 |
| index `inventory.unclassified` | 61246 → 61242 |
| index `pending_mapping_additions` | 完成 exact materialization 后删除整个字段 |
| index `inventory.unread` | 62067 保持；本增补没有阅读状态变化 |
| tracked/blob/symlink/gitlink | 62130 / 62128 / 2 / 0 保持 |
| applicability/disposition/unreviewed-applicable counters | 现值均 0 保持；0 不代表这些维度已闭合 |
| 8 completion flags、lock accepted/closure、index authority/decomposition/semantic-read | 现有 false 全保留 |

所有 exact source identity、size、read/review/risk/category/dependency/evidence 等其余字段不变。现有 read-observations 的 63 full/28 partial 已反映在所查旧行，五个媒体路径不在该集合。本 mapping-only 增补的 unread 预期仍为 62067；最终由 Actions 按完整 rows 投影。

需同一派生提交的最小数据文件集合是 index、ledger、manifest、`part-0036.jsonl.gz`、`part-0043.jsonl.gz`、`part-0044.jsonl.gz`。三 shard 的 count、first_path、last_path 不变，compressed/uncompressed sha256 与总 canonical digest 必须按 Actions 输出更新：

| shard | 旧 compressed_sha256 | 旧 uncompressed_sha256 |
| --- | --- | --- |
| 0036 | `836261857bf189bcb4584d552f0cb4f67fffb9b5f36ff9aabea067bc1240c480` | `8dd58c1716949ef1a254bef832256a2e200dc6dec800934d63524758e4d175c1` |
| 0043 | `2b142113fa92ec877cd1f1f01c2e3879f24244adf39faa2a93d54bd401b7e85b` | `40a0bcfebcae84bf67e1f8c5811714878f6aecffacfe114e35b369e83122efa6` |
| 0044 | `c679e7f417903b4f84268789014f396a26fb798946221955bfb97c30ee3b3edf` | `80f291a85dd3387589d8b323b9b9e373f2c16d5a1e6299c6cc67690fefebbe55` |

旧 canonical SHA256 为上述 `8994988c…`。新 SHA 只能引用真实派生 bytes；本报告不计算/伪造候选 hash。Actions 同时更新 `upstream-recursive-inventory.json`、其 `.sha256` 与 `authority-summary.json.canonical_inventory_sha256`，其 `canonical_file_records=62130`。writer 会重写 dependency declarations，但五路径都不是声明输入，故该文件无预期语义 delta；workflow stage 的 RTM 也无本修复必需 delta。若出现其他差异，按真实额外输入解释，不能盲目覆盖。

`ocrm-p0-authority.yml:176–252` 已有 source job success 后下载同 exact subject artifact、核对 G01/G02 receipt 与 hash、重新物化、branch HEAD CAS、单提交非 force 发布。root 只采用完整派生集合；不要拆开推送 shard 与 manifest/counter。若 CAS 发现分支已动，保留产物并重新核对新头，不覆盖它。派生 H 是新的 subject，需要自己的严格来源证据。

## 6. 验证合同误用分类及恢复范围

- **本日志：IMPL_BUG / source declaration integration。** 没有证据支持把真实缺边称为 TEST_FLAW。修复 index 声明/物化，不删 source_refs、不删责任、不改计数凑零、不放宽断言。
- **阶段误用：** `rust-desktop-runtime.yml:2925–2931` 调用正常严格 `verify_audit_integrity.py`。声明态 H 即使通过 source precheck，严格 gate 仍会按 `verify_audit_integrity.py:99` 拒绝 `Unmaterialized pending mapping additions`。这正是合同，不是 validator bug。要宣称 committed audit 成功必须针对物化后的 H；若把 precheck 当 strict acceptance，是 evidence/phase TEST_FLAW，修正 subject/阶段而保留 assertion。
- **结果误用：** 33/21 synthetic positives 保留，不证明真实 tuple 已物化或产品验收；不得将 H74 结果重标为新 H。
- **保留现有控制：** `test_source_review.py:249–325` 已覆盖 pending 限定、状态/identity/digest、旧边、重复/unused；`test_inventory_materialization.py:308–347` 已覆盖 real pipeline 消费与拒绝未物化 delta。不改 tests。
- 后续 expansion/strict source/Runtime/真实服务设备/acceptance 仍须自己的证据。保留原失败与有效 positives，只安排受影响或尚缺的 exact-H 检查。

## 附录：H74 原始 JSONL 行（逐字摘录）

下列为 H74 对应 gzip 解压后的逐字 JSONL，非拟议行；b15 相同。

### part-0036.jsonl.gz / line 96

```json
{"applicability_status":"unknown","blob_sha":"f4bf5b4308cb2a7160ad1db0142160d14fb71551","category":"unknown","content_identity":null,"dependency_edges":[],"disposition":null,"disposition_status":"pending","entry_kind":"blob","evidence_refs":[],"git_type":"blob","language_kind":"unclassified","license_provenance":null,"mode":"100644","module_id":null,"path":"src/gateway/agent-turn/agent-run-final-media.ts","rationale":null,"read_ranges":[],"read_status":"unread","required_review_depth":"deep","responsibility_ids":[],"responsibility_status":"unclassified","review_inputs":[],"review_status":"pending","reviewed_dependency_edges":[],"reviewer":null,"risk":"unknown","sha":"f4bf5b4308cb2a7160ad1db0142160d14fb71551","size_bytes":9259,"source_commit":"1440bb1771372334bea23d7642bb52deb7db1845","source_repository":"openclaw/openclaw","source_tree":"90c8aab39972da11cfad220361781d49e1bc6700"}
```

### part-0036.jsonl.gz / line 98

```json
{"applicability_status":"unknown","blob_sha":"27a3a65974fad078872b257ee28cafccdad4691f","category":"unknown","content_identity":null,"dependency_edges":[],"disposition":null,"disposition_status":"pending","entry_kind":"blob","evidence_refs":[],"git_type":"blob","language_kind":"unclassified","license_provenance":null,"mode":"100644","module_id":null,"path":"src/gateway/agent-turn/agent-run-media.ts","rationale":null,"read_ranges":[],"read_status":"unread","required_review_depth":"deep","responsibility_ids":["OCRM-RESP-MEDIA-STORE-READ-SCOPE-001"],"responsibility_status":"mapped","review_inputs":[],"review_status":"pending","reviewed_dependency_edges":[],"reviewer":null,"risk":"unknown","sha":"27a3a65974fad078872b257ee28cafccdad4691f","size_bytes":5421,"source_commit":"1440bb1771372334bea23d7642bb52deb7db1845","source_repository":"openclaw/openclaw","source_tree":"90c8aab39972da11cfad220361781d49e1bc6700"}
```

### part-0043.jsonl.gz / line 463

```json
{"applicability_status":"unknown","blob_sha":"b13f8354e7244d630ee274b3b0debf9a3f5bb8c6","category":"unknown","content_identity":null,"dependency_edges":[],"disposition":null,"disposition_status":"pending","entry_kind":"blob","evidence_refs":[],"git_type":"blob","language_kind":"unclassified","license_provenance":null,"mode":"100644","module_id":null,"path":"src/infra/sqlite-worker-error.ts","rationale":null,"read_ranges":[],"read_status":"unread","required_review_depth":"deep","responsibility_ids":[],"responsibility_status":"unclassified","review_inputs":[],"review_status":"pending","reviewed_dependency_edges":[],"reviewer":null,"risk":"unknown","sha":"b13f8354e7244d630ee274b3b0debf9a3f5bb8c6","size_bytes":4739,"source_commit":"1440bb1771372334bea23d7642bb52deb7db1845","source_repository":"openclaw/openclaw","source_tree":"90c8aab39972da11cfad220361781d49e1bc6700"}
```

### part-0036.jsonl.gz / line 956

```json
{"applicability_status":"unknown","blob_sha":"1975104ea6cdcfea25d3bc4362edbaea5e673adf","category":"unknown","content_identity":null,"dependency_edges":[],"disposition":null,"disposition_status":"pending","entry_kind":"blob","evidence_refs":[],"git_type":"blob","language_kind":"unclassified","license_provenance":null,"mode":"100644","module_id":null,"path":"src/gateway/managed-image-attachments.custody.ts","rationale":null,"read_ranges":[],"read_status":"unread","required_review_depth":"deep","responsibility_ids":[],"responsibility_status":"unclassified","review_inputs":[],"review_status":"pending","reviewed_dependency_edges":[],"reviewer":null,"risk":"unknown","sha":"1975104ea6cdcfea25d3bc4362edbaea5e673adf","size_bytes":5658,"source_commit":"1440bb1771372334bea23d7642bb52deb7db1845","source_repository":"openclaw/openclaw","source_tree":"90c8aab39972da11cfad220361781d49e1bc6700"}
```

### part-0044.jsonl.gz / line 914

```json
{"applicability_status":"unknown","blob_sha":"73515a9ed6a2dc9f1a0adddc638d542ad1323ee5","category":"unknown","content_identity":null,"dependency_edges":[],"disposition":null,"disposition_status":"pending","entry_kind":"blob","evidence_refs":[],"git_type":"blob","language_kind":"unclassified","license_provenance":null,"mode":"100644","module_id":null,"path":"src/media/store.read-scope.ts","rationale":null,"read_ranges":[],"read_status":"unread","required_review_depth":"deep","responsibility_ids":[],"responsibility_status":"unclassified","review_inputs":[],"review_status":"pending","reviewed_dependency_edges":[],"reviewer":null,"risk":"unknown","sha":"73515a9ed6a2dc9f1a0adddc638d542ad1323ee5","size_bytes":8935,"source_commit":"1440bb1771372334bea23d7642bb52deb7db1845","source_repository":"openclaw/openclaw","source_tree":"90c8aab39972da11cfad220361781d49e1bc6700"}
```
