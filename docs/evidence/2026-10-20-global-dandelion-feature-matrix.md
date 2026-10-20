# Global Dandelion 功能对应与验收矩阵

日期：2026-10-20

## 基线与证据规则

| 项目 | 精确值 |
| --- | --- |
| 源仓库 | `https://github.com/bhrumom/global-dandelion-rs` |
| 源分支 | `main` |
| 源提交 | `1b90698cbccbcf49662b8c56d96f145fe223a9b4` |
| 源 native base | `aa6436315cde7b66868b0592d1c67b4590ebd1f9` |
| 桌面宿主提交 | `afb489baef4f501a0b9a6b8ff8cefef7f90470b5` |
| Host 协议 | `fabushi.miniapp.tools.v1` |
| 生产小程序版本 | `0.3.1` |
| 生产包摘要 | `5c41325a52ceb52ba40ad9615d973c38e1d9873484698960b8bc76675f4df492` |
| 源功能机器清单 | `global-dandelion-rs/docs/migration/parity.json`，37 行 |
| 生产包能力声明 | `requestedCapabilities: []`；能力由当前 Host 运行时发现、授权和平台实现决定 |

状态含义：`已实现` 只表示有真实代码路径和相应证据；`已接入待验收` 表示入口和 Host/原生路径存在，但缺少当前提交上的真实引擎或安装后证据；`受限` 表示源功能在当前平台/安全边界中有明确差异；`未接入` 表示不能通过小程序使用，不能当作通过。

## 逐项对应表

| # | 源功能 / 符号 | 小程序入口 | Host 接口 | 原生实现 / 平台 | 当前证据与结论 |
|---:|---|---|---|---|---|
| 1 | `time-metadata`：`get_current_local_time`、`sync_network_time` | 平台面板（能力存在时） | `platform.time` | Rust `platform`；Linux 原生时钟/对等端比较 | Host 路由已存在；未有当前 0.3.0 真实对等端证据，已接入待验收 |
| 2 | `host-clock-mutation`：`sync_network_time` | 无入口 | 无；明确拒绝 | 源版本已移除主机时钟自动修改 | 设计排除，非遗漏；受限/移除 |
| 3 | `disk-partitions`：`DiskPartition`、`linux_get_disk_partitions` | 平台挂载面板 | `platform.mounts` | Linux `inventory` mountinfo 解析 | Host 路由存在；仅 Linux，未有安装包真实挂载证据 |
| 4 | `disk-space`：`DiskUsage`、`linux_get_disk_usage` | 平台面板 | `platform.inspect` / `platform.mounts` | Linux `statvfs` 与保留空间检查 | 共享 Host contract 有负面校验；真实平台验收缺失 |
| 5 | `network-interfaces`：`linux_get_net_interfaces`、`find_specific_interfaces` | 平台面板 | `platform.inspect` | Linux 被动接口/地址/计数器盘点 | Linux-only；未有当前安装运行证据 |
| 6 | `connections`：`linux_get_net_connections`、`analyze_network_connections` | 平台面板 | `platform.inspect` | Linux 被动 TCP/TCP6 摘要 | Linux-only；未有当前安装运行证据 |
| 7 | `structured-logging`：`log_message`、`HeadlessLogPrinter` | 任务错误/诊断面板 | `diagnostics.*` 及任务错误 | Rust stderr 结构化日志，Host 筛选 | CI CLI/契约覆盖；当前小程序真实诊断输出未单独验收 |
| 8 | `progress-ui`：`create_progress_bar`、`StatusDisplayThread` | 任务状态、取消按钮 | `transfer.status`、`transfer.cancel` | Host task ID 状态机；UI 替代 TUI | 桌面 Host 契约覆盖；当前仅离线 Wasm 进度在安装验收通过 |
| 9 | `live-speed-stats`：速度统计符号集 | 任务进度详情 | `transfer.status` | Rust telemetry/TUI | 由状态结果承载；未有真实传输速度证据 |
| 10 | `day-night-pacing`：`_apply_speed_level`、`TimeBasedPacerThread` | 配置/任务状态 | `config.check`、`transfer.send` | Rust schedule/pacing | CLI 实现存在；Host 配置与真实运行待验收 |
| 11 | `constant-rate`：`send_chunk_in_batches` | 发送任务控制 | `transfer.send` | Rust token bucket | 参数路径存在；无授权真实目标证据 |
| 12 | `surface-scan`：`ThrottledSurfaceScanThread` | 扫描面板（read） | `scan.read` | Rust scanner + `ReadVerifier` | 入口和路径存在；未有当前包的真实 Host 扫描证据 |
| 13 | `metadata-scan`：`MetadataScanThread` | 扫描面板（metadata） | `scan.metadata` | Rust metadata scanner | Host 契约/选择限制存在；真实安装包扫描待验收 |
| 14 | `hotspot`：`HotspotDataBroadcaster` | 热点规划/启停/状态 | `hotspot.plan/apply/status/stop` | Linux Host + `ProcessRunner`，需显式授权 | 路由和回滚 journal 存在；无授权硬件证据，受限 |
| 15 | `release-version`：`_parse_version` | 更新信息 | `update.check` | Rust updater 版本解析 | 路由存在；未有当前生产包更新检查证据 |
| 16 | `updater`：`AutoUpdateThread` | 更新检查/暂存 | `update.check`、`update.stage` | 签名 updater；Linux systemd opt-in | Host 入口存在；签名暂存未在桌面安装验收执行 |
| 17 | `rename-effect`：`RenamerThread`、`restore_interrupted_files` | 无独立面板 | 无独立接口 | Linux legacy link capability | 源侧实现，不是当前小程序独立入口；受限 |
| 18 | `loopback-send`：`LoopbackSenderThread` | 发送任务 | `transfer.send` | Rust transport | 真实发送目标未配置/未验收 |
| 19 | `loopback-receive`：`LoopbackReceiverThread` | 接收任务 | `transfer.receive` | Rust transport；Host 持有 mTLS 文件 | 接口存在；未使用真实授权测试凭据验收 |
| 20 | `private-vps`：`VpsSenderThread` | 发送目标池 | `transfer.send` | mTLS transport | 目标池由 Host 注入；无授权 VPS 证据 |
| 21 | `lan`：`ForcedNetworkScanThread` | 发送目标池 | `transfer.send`、`platform.inspect` | mTLS + 明确配置路由 | 无授权局域网目标证据 |
| 22 | `startup-drive-scan`：`StartupDriveScanner` | 挂载/扫描状态 | `platform.mounts`、`scan.*` | Linux mount policy + service | Host 仅显式选择；无自动启动扫描安装验收 |
| 23 | `external-drive-monitor`：`ExternalDriveMonitorThread` | 平台变化观察 | `platform.watch` | Linux mount diff observer/service | 接口存在；无真实热插拔证据 |
| 24 | `source-scan`：`FileScannerThread` | 文件/目录选择后扫描 | `scan.metadata`、`scan.read` | Host 受控句柄 + Rust scanner | 选择边界和失效句柄契约有覆盖；真实引擎待验收 |
| 25 | `chunk-framing`：`FileScannerThread.process_file` | 发送/离线处理结果 | `transfer.send`；离线 Wasm 内部 codec | Rust packet / GDAN v2；Wasm encode/verify | macOS 安装验收已证明本地 packet round-trip，不等同网络发送 |
| 26 | `wan-workers`：`ChunkSenderWorker` | 发送进度 | `transfer.send`、`transfer.status` | mTLS transport worker | 无授权 WAN 目标证据 |
| 27 | `worker-explosion`：`WorkerManagerThread` | 无入口 | 无 | 源侧明确移除，由 Supervisor 限制并发 | 设计排除，非遗漏 |
| 28 | `shutdown`：`graceful_shutdown` | 取消/关闭窗口 | `transfer.cancel`；生命周期 `cancelAll` | Host lifecycle cancellation | 关闭窗口/账号切换取消路径有契约；真实长任务待验收 |
| 29 | `state`：`load_state`、`save_state` | 状态检查/导入 | `state.inspect`、`state.import` | Host-owned state directory + Rust state | 接口和大小限制存在；当前安装包重启恢复仅证明小程序设置数据，未证明 Rust 状态 |
| 30 | `country-metadata`：`load_country_database` | 兼容数据查询 | `compat.countryLookup` | Rust compat country database | 路由存在；真实数据查询待验收 |
| 31 | `random-public-targets`：`generate_ip_from_cidr`、`prepare_target_ip_pool` | 目标池/发送配置 | `transfer.send` | Host 注入授权 TargetPool + Rust selector | 不允许隐式公网；无目标授权证据 |
| 32 | `target-health`：`check_internet_connectivity` | 状态/诊断 | `diagnostics.status`、`transfer.status` | peer receipt/socket meter | 路由存在；无真实目标证据 |
| 33 | `neighbour-discovery`：`scan_network_devices` | 平台/目标观察 | `platform.inspect` | 只读平台 inventory | Linux-only；无真实网络环境证据 |
| 34 | `target-policy`：`prepare_target_ip_pool` | 目标策略配置 | `transfer.send` | Host 授权 exact-address capability | 参数校验存在；无授权目标验收 |
| 35 | `batch-send`：`send_chunk_in_batches` | 发送任务 | `transfer.send`、`transfer.status` | bounded authenticated peer fanout | 无真实目标和内容校验证据 |
| 36 | `wire-budget`：`ChunkSenderWorker` | 任务状态/流控 | `transfer.send`、`transfer.status` | Rust core + Supervisor | Host 长任务限制存在；真实传输待验收 |
| 37 | `orchestration`：`main` | 全部面板的任务入口 | 能力发现 + 各工具路由 | Rust CLI orchestration，Host argv 数组执行 | Host 路由/权限/账号代际校验已实现；完整端到端证据缺失 |

## 结论与缺口

1. 当前生产 0.3.1 已证明的是：离线 WebAssembly packet 编解码、真实安装包加载、冷启动/网络禁用启动、重装保留小程序设置，以及 Host 工具协议与隔离窗口契约。
2. 当前还不能证明“源版本全部功能已通过小程序”：网络发送/接收、Rust 状态恢复、Linux 挂载观察、热点硬件回滚、签名更新暂存、真实诊断/兼容数据和目标健康度均缺少对应的安装后真实证据。
3. 源实现的原生能力是 Linux-first；macOS/Windows 当前 Host 不能把 Linux CLI 伪装成可用能力。应继续保持能力发现按平台真实返回，并为 macOS/Windows 分别实现共享核心或原生适配后再验收。
4. 本矩阵区分了模拟契约、Rust/CLI CI、安装包验证和真实操作验收；没有证据的行不标记为通过。

## 当前交付证据索引

- 源版本构建与准备包：Actions `39375352233`，源提交 `1b90698cbccbcf49662b8c56d96f145fe223a9b4`。
- 生产签名发布：backend Actions `39375612808`，发布 `v0.3.1`，包摘要见上表。
- macOS 当前源提交安装验收：Desktop Actions `39372081156`，提交 `afb489baef4f501a0b9a6b8ff8cefef7f90470b5`。
- macOS 安装验收 JSON：`artifacts/macos-run-39372081156-miniapp/run-r1x5T8/acceptance.json`。
- macOS 安装包：`artifacts/macos-run-39372081156-macos/fabushi-1.2.77-macos-arm64.dmg`。
- Linux、Windows：当前没有等价的签名安装包及实际操作验收证据，保留为未完成。
- Android/iOS：共享包/协议/桥接兼容检查已存在；本轮未执行真机或模拟器设备验收。
