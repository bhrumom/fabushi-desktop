# FDN-001 G15 Desktop Host resource and privacy evidence

This is a bounded FDN-001 G15 slice for the production Desktop Host-owned
`source/fabushi-network-worker`. It does not promote G03, G05, G09 or G16.

The gate preserves the FDN-001 network-worker budgets without relaxation:

- steady-state RSS must remain below 150 MiB;
- five-minute mean CPU must remain below 1% of one core;
- worker-owned state and per-component evidence/log growth are bounded to 20 MiB
  in the acceptance fixture;
- the measured filesystem must keep at least `max(1 GiB, 5%)` free;
- the worker is placed behind an owner-based output fence so its only permitted
  fixture destinations are loopback services;
- evidence and encrypted worker state are scanned for the real ephemeral
  Headscale pre-auth key and common token/private-key patterns.

## Production ownership

The same exact source HEAD is used for:

- `source/fabushi-network-worker/main.go`, the shipping `tsnet.Server` worker
  with Tailscale support logging disabled and no web client;
- `source/host/src/network_worker_process.rs`, the Host supervisor which clears
  the child environment, passes auth/state secrets only through private stdin,
  redacts Debug output, and content-redacts worker stderr;
- `source/host/tests/network_worker_process_contract.rs`, the focused supervisor
  contract.

The resource measurement itself does not use the fake worker from the focused
Host contract. It starts the shipping Go worker against a real pinned
`docker.io/headscale/headscale:0.29.5` control server over a local TLS endpoint,
with a local static DERP map and `tailscale.com v1.104.1`.

## Actions-only execution

`.github/workflows/fdn-g15-host-resource-privacy.yml` is the only supported
execution entry point. It builds the shipping worker, runs the focused Host
supervisor contract, then runs a five-minute steady-state acceptance probe.
No developer workstation or remote device build is used.

Pull-request success is bounded PR evidence only. Promotion to Desktop
current-main G15 evidence requires a repository-protected merge followed by a
successful run on the resulting exact Desktop main SHA, with run/job/artifact
identity and digest recorded.
