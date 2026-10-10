# TooManyCooks runtime/concurrency authority — deterministic read 7,057–7,154

Authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` → `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` (Boost-1.0).

This batch revalidates 7,057–7,064 and exact-reads all remaining pinned TooManyCooks blobs through 7,154. Combined with durable 7,029–7,056, all **126** component entries are read/decomposed. **Unknown is unchanged.**

## Existing owners and evidence

- `GatewayHostSupervisor` — connection generation, health timeout, abandoned/stale fencing.
- `HostRunnerComposition` + frozen Host→Runner bridge — unique shipping composition and teardown.
- `RoutedProviderCancellation/RoutedProviderTaskRegistry` — cancellation and active-task lifetime.
- `TurnSettlement` — exactly-once terminal state.
- canonical `native/mahayana-messaging/**` — existing Conversation/Message runtime truth.
- `PressureCpuProfiler` — real macOS/Linux/Windows runtime pressure capture.
- GitHub Actions Build/Release owners.

Predecessor exact HEAD `4028b7613fc760d0f6ef7d4884cd620b409a9c0c` is fully green for Rust desktop runtime `38062398837` and Desktop Chat Parity `38062398841`; jobs `114243173201/114243173171/114243173186/114243173229/114243200994` succeeded. Artifacts: `11673552460` sha256 `33cf046454569e14a5c3e2157dac152d9c6826353284227bdbf03d56160fc2b9`; `11673313447` sha256 `59ff2da8f97f4048a16863df579fa38d72ebc167779faafc523475f47e4b560d`; `11673707789` sha256 `169d997c11c40f7074b60099dac91e764cc377472e5a934cde04cc395d545fc1`. After this descendant commit they are predecessor-only.

## Applicability

Executor/priority restoration; coroutine/awaitable/fork/join lifetime; UAF-safe settlement; atomic wait/wake/teardown; foreign callback lifetime/cancellation; queue close/reclamation; CPU capacity/topology/work stealing; and sanitizer/fuzz/coverage/build-option consistency are applicable behavioral/quality responsibilities. Exact C++ APIs, Asio, vcpkg and hwloc are not retained as product owners. Exact hwloc pinning stays open until a real Fabushi worker-pool use case/replacement is proven; it is not fabricated N/A.

## Exact entries

| order | path | blob | cluster | disposition |
| ---: | --- | --- | --- | --- |
| 7057 | `include/tmc/all_headers.hpp` | `9f14633b57c5d4b319d1309b4bc6db643bc1822d` | build-runtime-support | mapped-open |
| 7058 | `include/tmc/asio/README.md` | `da7cabbe8f1269e3cf3e22ab8b197e411c6f39ea` | foreign-callback-lifetime | mapped-open |
| 7059 | `include/tmc/asio/aw_asio.hpp` | `ed2d397faffe2cb597f55af914d5d12e464b8883` | foreign-callback-lifetime | mapped-open |
| 7060 | `include/tmc/asio/ex_asio.hpp` | `b0c2d3ea968be5b5ce3e7047db46f3b522cd2dd6` | foreign-callback-lifetime | mapped-open |
| 7061 | `include/tmc/atomic_condvar.hpp` | `b820b76354f4f59b80e083089d4bf6ddd0e3acaf` | wait-wake-teardown | mapped-open |
| 7062 | `include/tmc/auto_reset_event.hpp` | `c0fb12be4db2b9b1f8d113d0466350f7f00e6e6e` | wait-wake-teardown | mapped-open |
| 7063 | `include/tmc/aw_resume_on.hpp` | `ceeb9d900db9a27405bfd40f3b6b7116f3d0f9cf` | executor-context | mapped-open |
| 7064 | `include/tmc/aw_yield.hpp` | `0b2fd698304520ce3db64949bfc7547a9cd2d844` | executor-context | mapped-open |
| 7065 | `include/tmc/barrier.hpp` | `1826c96987a632c9211eec0414907a9d7a555070` | wait-wake-teardown | mapped-open |
| 7066 | `include/tmc/channel.hpp` | `a2ac2fd162bf18ed9bf6f12c10793259d72891f2` | queue-reclamation | mapped-open |
| 7067 | `include/tmc/current.hpp` | `514036427d81f66d070fbc6d6c187616834bdce1` | executor-context | mapped-open |
| 7068 | `include/tmc/detail/atomic_bitmap.hpp` | `b4f8785a9f21cbedf3fef73f4d846e95001121e0` | build-runtime-support | mapped-open |
| 7069 | `include/tmc/detail/auto_reset_event.ipp` | `6e2bfb710da1bc07990ae0301148baf1daaec0e7` | wait-wake-teardown | mapped-open |
| 7070 | `include/tmc/detail/awaitable_customizer.hpp` | `3f5e5da85e39701481cdc1687800d5dd9b251bc9` | coroutine-lifetime | mapped-open |
| 7071 | `include/tmc/detail/barrier.ipp` | `319fcd08507944cfa08db1f83aa14943bf0c04ff` | wait-wake-teardown | mapped-open |
| 7072 | `include/tmc/detail/bit_manip.hpp` | `088e0025ce2d3e5d015b29f9354a5900f8e99739` | build-runtime-support | mapped-open |
| 7073 | `include/tmc/detail/compat.hpp` | `c633ffb52c5840ff7fc0821f6dd916d1697b96eb` | build-runtime-support | mapped-open |
| 7074 | `include/tmc/detail/concepts_awaitable.hpp` | `64626b77c99ad2a02a18a1e7f5892dd62d49b479` | coroutine-lifetime | mapped-open |
| 7075 | `include/tmc/detail/concepts_work_item.hpp` | `1609b8435b852cc79397ff4f29697dc195854f1b` | build-runtime-support | mapped-open |
| 7076 | `include/tmc/detail/container_cpu_quota.hpp` | `4323224e3197bfc8fb9f2afe15fa5ce9b994e362` | cpu-capacity-topology | mapped-open |
| 7077 | `include/tmc/detail/container_cpu_quota.ipp` | `882ff882ebace12da9b731596985459e120543c7` | cpu-capacity-topology | mapped-open |
| 7078 | `include/tmc/detail/coro_functor.hpp` | `600b20b6d02a44c29b44dcebc52ab66eab553538` | coroutine-lifetime | mapped-open |
| 7079 | `include/tmc/detail/ex_braid.ipp` | `1d6f0533369a00550e5571bd2d3598515d36b373` | executor-context | mapped-open |
| 7080 | `include/tmc/detail/ex_cpu.ipp` | `a945624f882a8bcddc2958e4837c35ee99b0f620` | cpu-capacity-topology | mapped-open |
| 7081 | `include/tmc/detail/ex_cpu_st.ipp` | `31d350df2691fa1939700196688ab9c1cb40611d` | cpu-capacity-topology | mapped-open |
| 7082 | `include/tmc/detail/ex_manual_st.ipp` | `82eb6d0fca7f57329486b4a561b8aa979923dc4a` | build-runtime-support | mapped-open |
| 7083 | `include/tmc/detail/hwloc_forward_defs.hpp` | `3e18e9beac5353df2f1a30a5e6cd1df5dd670849` | cpu-capacity-topology | mapped-open |
| 7084 | `include/tmc/detail/hwloc_forward_defs.ipp` | `a50a6d3a903a79af2ac49bc140f6f6f88d0914a4` | cpu-capacity-topology | mapped-open |
| 7085 | `include/tmc/detail/hwloc_unique_bitmap.hpp` | `cfb50ced9aa41f06c5458e0e3d43e2ab028b832a` | cpu-capacity-topology | mapped-open |
| 7086 | `include/tmc/detail/hwloc_unique_bitmap.ipp` | `ada5a2956b306380dcf5802597abc9737244b4a1` | cpu-capacity-topology | mapped-open |
| 7087 | `include/tmc/detail/impl.hpp` | `7f6e50301758ea2ee1a72566be95b7c617477b54` | build-runtime-support | mapped-open |
| 7088 | `include/tmc/detail/init_params.hpp` | `a5e4ae66eda28e5993836b13ba87a28474f14d28` | build-runtime-support | mapped-open |
| 7089 | `include/tmc/detail/init_params.ipp` | `c4bbb8af0e093dcd0e9570ba4f2f88b78b4875f4` | build-runtime-support | mapped-open |
| 7090 | `include/tmc/detail/manual_reset_event.ipp` | `bca437955dd6ffd3ba29b4d146d866e590db4d69` | wait-wake-teardown | mapped-open |
| 7091 | `include/tmc/detail/matrix.hpp` | `fc1e1c5a4d818f4e8547c03e9f751e8f1927ea9b` | build-runtime-support | mapped-open |
| 7092 | `include/tmc/detail/matrix.ipp` | `619e992c222769d65600cf54b0f118b65672cb70` | build-runtime-support | mapped-open |
| 7093 | `include/tmc/detail/mixins.hpp` | `54ca1d8e271f458df9a74283d49653f008a6e8a5` | executor-context | mapped-open |
| 7094 | `include/tmc/detail/mutex.ipp` | `027f8349c8a0e1bede07620b8724e5639a2369aa` | wait-wake-teardown | mapped-open |
| 7095 | `include/tmc/detail/qu_chase_lev32.hpp` | `ff4e2b42691baf6612c2b37c2f78a3c11fdb5424` | queue-reclamation | mapped-open |
| 7096 | `include/tmc/detail/qu_chase_lev64.hpp` | `92c2532c3c2ffe8e6ce0e3563a73d655b04b7875` | queue-reclamation | mapped-open |
| 7097 | `include/tmc/detail/qu_inbox.hpp` | `be4b4fa4163d9fa0ea0e03efcff34ee760bc98a8` | queue-reclamation | mapped-open |
| 7098 | `include/tmc/detail/qu_mc.hpp` | `c5fb07f81003ff1a77d1b13922d509d7576341cc` | queue-reclamation | mapped-open |
| 7099 | `include/tmc/detail/qu_mpsc_blocking.hpp` | `92e6b3523ef95e1ce6a3aa95a9abcac470d21341` | queue-reclamation | mapped-open |
| 7100 | `include/tmc/detail/qu_storage.hpp` | `16f97cfa87c160c4f1dbe8477049752afac28fa5` | queue-reclamation | mapped-open |
| 7101 | `include/tmc/detail/qu_work_stealing.hpp` | `5129fae7738f2c85794f1cab32fe0157183bea90` | queue-reclamation | mapped-open |
| 7102 | `include/tmc/detail/result_each.hpp` | `845f5c5bb621b546802d81d9900262a92203cb33` | coroutine-lifetime | mapped-open |
| 7103 | `include/tmc/detail/result_each.ipp` | `95c178506cf7c8d1146921b7f43995ff211ebdab` | coroutine-lifetime | mapped-open |
| 7104 | `include/tmc/detail/rw_lock.ipp` | `7f389009b32127cb1e5dde9f780315fe9c1f9fd5` | wait-wake-teardown | mapped-open |
| 7105 | `include/tmc/detail/semaphore.ipp` | `03574a5539ce672fd32c70a77660454b4ae3c1d9` | wait-wake-teardown | mapped-open |
| 7106 | `include/tmc/detail/task_unsafe.hpp` | `5f1e2497ec58a54ef56ead73f59e648fee08fe87` | coroutine-lifetime | mapped-open |
| 7107 | `include/tmc/detail/task_wrapper.hpp` | `bac09d0ddf5da50517bdd610d748178b915099d6` | coroutine-lifetime | mapped-open |
| 7108 | `include/tmc/detail/thread_layout.hpp` | `20fdb7257f31aa72fdc8f92d82681953070395a9` | cpu-capacity-topology | mapped-open |
| 7109 | `include/tmc/detail/thread_layout.ipp` | `4c621292188a71487750d29bc0dbbcb808b4a038` | cpu-capacity-topology | mapped-open |
| 7110 | `include/tmc/detail/thread_locals.hpp` | `3e16fa95bbeb922150d2d5c61754cd78bf37d0a3` | executor-context | mapped-open |
| 7111 | `include/tmc/detail/timer_compat.hpp` | `a929785bceba0b3244bcedc0a9ca5e30dcb23a1e` | build-runtime-support | mapped-open |
| 7112 | `include/tmc/detail/tiny_lock.hpp` | `f894d5321fafbfa0cb877b912dc204e9d2760ee0` | build-runtime-support | mapped-open |
| 7113 | `include/tmc/detail/tiny_opt.hpp` | `63fb3573217b7c9d66615ad5fbc86bc202a44d54` | queue-reclamation | mapped-open |
| 7114 | `include/tmc/detail/tiny_stack.hpp` | `a35d08b3f45dc42a0c0fb9862b8ee25be64ffc7d` | queue-reclamation | mapped-open |
| 7115 | `include/tmc/detail/tiny_vec.hpp` | `47f62376537f79f864f71103f9b990e7c115fdbf` | queue-reclamation | mapped-open |
| 7116 | `include/tmc/detail/topology.ipp` | `ec183c882f6118b149041a2765e0bd28b09de194` | cpu-capacity-topology | mapped-open |
| 7117 | `include/tmc/detail/tsan.hpp` | `0f6c15cc7310fa617c72b9c0a22334269e611434` | build-runtime-support | mapped-open |
| 7118 | `include/tmc/detail/tuple_helpers.hpp` | `b4999c3997d4e1e048a1999ffb77e01c97168f65` | build-runtime-support | mapped-open |
| 7119 | `include/tmc/detail/waiter_list.hpp` | `4e053720612db1606c1e817239e56c72ba15a5d3` | wait-wake-teardown | mapped-open |
| 7120 | `include/tmc/detail/waiter_list.ipp` | `fad3c1c42ef55ddfe512d2b7ca5799fa21aa2f54` | wait-wake-teardown | mapped-open |
| 7121 | `include/tmc/ex_any.hpp` | `79de8b20a078343b7e9f83702bee2269e1111155` | executor-context | mapped-open |
| 7122 | `include/tmc/ex_braid.hpp` | `70460acf743997541df484b2d751e056ada1dcf7` | executor-context | mapped-open |
| 7123 | `include/tmc/ex_cpu.hpp` | `8b7220c19a7f821c03cde14c4a592da2450a1a84` | cpu-capacity-topology | mapped-open |
| 7124 | `include/tmc/ex_cpu_st.hpp` | `95bbb2c7801001c23b40e8812f5d5e31f74c2e7e` | cpu-capacity-topology | mapped-open |
| 7125 | `include/tmc/ex_manual_st.hpp` | `7ef30a401cd36ec199304192af8db54e7c255e80` | build-runtime-support | mapped-open |
| 7126 | `include/tmc/external.hpp` | `71c0c6304b21fb9b900158ebef60b05ef883d0b9` | executor-context | mapped-open |
| 7127 | `include/tmc/fork_group.hpp` | `2afa4217f618a27af2898473b9a2c28b5b722779` | coroutine-lifetime | mapped-open |
| 7128 | `include/tmc/latch.hpp` | `3d6e630d7eed8fd8f1b4ac61ac73624e205d3321` | wait-wake-teardown | mapped-open |
| 7129 | `include/tmc/manual_reset_event.hpp` | `561d681654ce3bb2cdd0ca869f0b1ef5ccd2fa43` | wait-wake-teardown | mapped-open |
| 7130 | `include/tmc/mutex.hpp` | `5a6a027fa6d4dbe0e40d9fd27f702247399e220b` | wait-wake-teardown | mapped-open |
| 7131 | `include/tmc/mux_many.hpp` | `52236f8260b2117588fb41b85dcc3475405a736c` | coroutine-lifetime | mapped-open |
| 7132 | `include/tmc/mux_tuple.hpp` | `f324a461a3cff87044836730115c47f18a352ce0` | coroutine-lifetime | mapped-open |
| 7133 | `include/tmc/qu_mpsc_bounded.hpp` | `025a84db6c1cd83d3dd9ffce2e4a8202cb336813` | queue-reclamation | mapped-open |
| 7134 | `include/tmc/qu_mpsc_unbounded.hpp` | `38e6799379a5b8a4c9229db105b85766185aef6f` | queue-reclamation | mapped-open |
| 7135 | `include/tmc/qu_spsc_bounded.hpp` | `0d8fdba3e49130383c7249f2a6296461190099c7` | queue-reclamation | mapped-open |
| 7136 | `include/tmc/qu_spsc_unbounded.hpp` | `e20352c80a94a97195fb16f30fed968f8f948b5a` | queue-reclamation | mapped-open |
| 7137 | `include/tmc/rw_lock.hpp` | `e6208ca7976f9ea3ff908b0c12ec224545f73596` | wait-wake-teardown | mapped-open |
| 7138 | `include/tmc/select.hpp` | `d9f237a9cd41ac2f3587226b62fe8c78c147a167` | coroutine-lifetime | mapped-open |
| 7139 | `include/tmc/semaphore.hpp` | `ed4de0d317595ad4f934229cfd67cc03287ef6e6` | wait-wake-teardown | mapped-open |
| 7140 | `include/tmc/spawn.hpp` | `ecc5260f4ce7982ae246b3f72e75eab5d1e0b137` | coroutine-lifetime | mapped-open |
| 7141 | `include/tmc/spawn_func.hpp` | `3d217e5018075e5702fa42552dee03ec4ca08ba5` | coroutine-lifetime | mapped-open |
| 7142 | `include/tmc/spawn_group.hpp` | `c2cbbb1e752e42f095e5087d1ba323080f85047e` | coroutine-lifetime | mapped-open |
| 7143 | `include/tmc/spawn_many.hpp` | `6def2c1457487ab771599f151a1b9d442a02072b` | coroutine-lifetime | mapped-open |
| 7144 | `include/tmc/spawn_tuple.hpp` | `d66230df0d8415e54d02f7db384dfc4b89de40e5` | coroutine-lifetime | mapped-open |
| 7145 | `include/tmc/sync.hpp` | `3a2cd7b1bfefb635c084800da7ca23d5eb885b18` | coroutine-lifetime | mapped-open |
| 7146 | `include/tmc/task.hpp` | `a4d9b29a0aa6c218a655aea67e8e708476493e49` | coroutine-lifetime | mapped-open |
| 7147 | `include/tmc/topology.hpp` | `2ede246c582be9671ba03510bcc14cfc947900fa` | cpu-capacity-topology | mapped-open |
| 7148 | `include/tmc/traits.hpp` | `348a8ae3f5732d29e63102e9993b468e92028c82` | build-runtime-support | mapped-open |
| 7149 | `include/tmc/utils.hpp` | `d7247518e12ba55cdcd5bc03abf3a8b2ef0e4090` | build-runtime-support | mapped-open |
| 7150 | `include/tmc/version.hpp` | `a79b5717a33b70ce38f7d5ece8148901c009ea1d` | build-runtime-support | mapped-open |
| 7151 | `include/tmc/work_item.hpp` | `39f61111c52ff53a50fff44f6844ce2834a6cfcb` | build-runtime-support | mapped-open |
| 7152 | `ports/toomanycooks/portfile.cmake` | `f7ed3fe176beb068b8981db95ece336e168beb71` | build-runtime-support | mapped-open |
| 7153 | `ports/toomanycooks/usage` | `4a6a201328aae508ee3c4aced8cb07de5d32eaa8` | build-runtime-support | mapped-open |
| 7154 | `ports/toomanycooks/vcpkg.json` | `dbe8209a6312d83cc3e6118ceb497641f08f066e` | build-runtime-support | mapped-open |

## Accounting

- recursive total: **16,125**
- read-through: **7,154**
- unread: **8,971**
- unknown: **15,845**
- omitted: **0**
- next: **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3`

No row in this batch is VERIFIED merely because it was read.