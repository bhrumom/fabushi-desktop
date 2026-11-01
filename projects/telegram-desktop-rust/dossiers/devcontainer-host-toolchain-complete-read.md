# Fixed-U .devcontainer.json complete read

- Task round: 151
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `787470e7a9700839e31385c1c90a29c0c9492e03`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries
- Source: `.devcontainer.json@16a1bcc1192bed87c5211da3dc63b0b0ed99a93e`
- Tree index: 102 zero-based; mode 100644
- Source identity: 989 Git bytes / 989 ASCII characters / 30 newlines / trailing newline / 31 split lines
- Responsibility: `TDRP-R9-REPOSITORY-DEVELOPMENT-CONTAINER-HOST-TOOLCHAIN-001`

## Complete source contract

The file is a VS Code Dev Containers host configuration. It names the environment `CentOS`, uses local image `tdesktop:centos_env`, configures VS Code/CMake integration, requests `SYS_PTRACE`, disables the container seccomp profile with `seccomp=unconfined`, bind-mounts the local repository to `/usr/src/tdesktop`, and makes that path the container workspace.

Exact editor/tool settings are: Microsoft C/C++ IntelliSense disabled; CMake generator `Ninja Multi-Config`; build directory `${workspaceFolder}/out`; copied compile database `${workspaceFolder}/compile_commands.json`; extensions for cpptools, clangd, Qt, Python, Docker and GitLens.

## Direct consumer and toolchain lineage

The direct consumer is VS Code Dev Containers conventional repository configuration discovery / "Reopen in Container". Fixed-U `docs/building-linux.md@fbdf25355734f35617849224da106f1b9fafb3c9` documents that exact workflow and the same image/workspace path.

The image is not created by this JSON. Fixed-U `Telegram/build/prepare/linux.sh@e4d2f920e4becdff6292d296efe6f21455a19fcb` runs Poetry and pipes `gen_dockerfile` into BuildKit to build `tdesktop:centos_env`. The generator is declared by `Telegram/build/docker/centos_env/pyproject.toml@8f43692fb217f075ccc588c3b677177e42b5688f`, implemented by `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d`, and renders `Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`. That Dockerfile ends with `WORKDIR /usr/src/tdesktop`, `USER user`, a volume at the same path and a build-script default command.

## Security / input-output / failure-recovery

Inputs are the host workspace, the prebuilt local image and host support for the declared Dev Containers/mount/capability settings. Output is a development workspace/tooling environment plus repository-side `out` and compile-command paths when later configured/built; this source itself creates no build artifact.

`SYS_PTRACE` and `seccomp=unconfined` are explicit privilege broadening for this development environment. The bind mount exposes the local workspace into the container. These are not product/runtime permissions and must not be copied into shipping sandbox or deployment policy without an independent owner/spec.

Missing VS Code/Dev Containers, Docker/image preparation failure, absent `tdesktop:centos_env`, unsupported bind/mount semantics, denied ptrace/seccomp options, missing tooling/extensions or workspace-path mismatch is non-passing environment availability. The JSON contains no retry, fallback image, hidden network fetch, secret mount, API credential, durable state database or background recovery owner. Recovery is explicit host/image/config correction and a later fresh qualification.

## Existing-owner-first disposition

Existing Fabushi owners are repository development/process governance, build/toolchain provenance, repository security governance and shared quality/test-evidence. Rejected owners are product Settings/Permissions, Computer, Agent/Coordinator/Host/Runner, platform-core authorization/service, shipping deployment/packaging runtime, Telegram-specific product runtime and any duplicate toolchain/security registry. No platform-core production delta, product UI, service, database or ADR is introduced.

Eleven Stage-A cases are authored and not run: six contracts, one integration, three fault-recovery and one security case. `validation_status=not-run`, `accepted=false`.
