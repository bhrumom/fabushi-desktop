#!/usr/bin/env bash
set -euo pipefail

if [[ "${GITHUB_ACTIONS:-}" != "true" ]]; then
  echo "fdn-g15-host-resource-privacy is GitHub Actions only" >&2
  exit 64
fi

: "${EXPECTED_SHA:?EXPECTED_SHA is required}"
: "${RUNNER_TEMP:?RUNNER_TEMP is required}"

actual_sha="$(git rev-parse HEAD)"
test "$actual_sha" = "$EXPECTED_SHA"
dirty="$(git status --porcelain)"
if [[ -n "$dirty" ]]; then
  echo "G15 Host probe requires a clean source tree; found:" >&2
  printf '%s\n' "$dirty" >&2
  exit 1
fi

evidence_dir="${1:-evidence}"
mkdir -p "$evidence_dir"
work_dir="/tmp/fdn-g15-host-resource-${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-1}"
container="fdn-g15-host-headscale-${GITHUB_RUN_ID:-0}-${GITHUB_RUN_ATTEMPT:-1}"
probe_user="fdng15host"
worker_binary="$RUNNER_TEMP/fabushi-network-worker"
worker_exec="/tmp/fdn-g15-fabushi-network-worker-${GITHUB_RUN_ID:-0}-${GITHUB_RUN_ATTEMPT:-1}"
worker_log="$evidence_dir/host-network-worker.log"
worker_protocol="$evidence_dir/host-network-worker-protocol.jsonl"
worker_pid=""
sudo_pid=""
uid_value=""
auth_key=""
fifo_fd_open=0
metadata_sink_pid=""
metadata_sink_port="8793"

cleanup() {
  set +e
  if [[ "$fifo_fd_open" -eq 1 ]]; then
    printf '%s\n' '{"type":"shutdown","requestId":"shutdown-g15-host"}' >&3 2>/dev/null || true
    exec 3>&- || true
  fi
  if [[ -n "$worker_pid" ]]; then
    sudo kill "$worker_pid" 2>/dev/null || true
  fi
  if [[ -n "$sudo_pid" ]]; then
    wait "$sudo_pid" 2>/dev/null || true
  fi
  docker rm -f "$container" >/dev/null 2>&1 || true
  if [[ -n "$metadata_sink_pid" ]]; then
    kill "$metadata_sink_pid" 2>/dev/null || true
    wait "$metadata_sink_pid" 2>/dev/null || true
  fi
  if [[ -n "$uid_value" ]]; then
    sudo iptables -t nat -D OUTPUT -m owner --uid-owner "$uid_value" -d 169.254.169.254/32 -p tcp --dport 80 -m comment --comment fdn-g15-host-cloud-metadata-sink -j DNAT --to-destination "127.0.0.1:$metadata_sink_port" 2>/dev/null || true
    sudo iptables -D OUTPUT -m owner --uid-owner "$uid_value" '!' -d 127.0.0.0/8 -m comment --comment fdn-g15-host-worker-diag -j LOG --log-prefix "FDN-G15-HOST4 " 2>/dev/null || true
    sudo ip6tables -D OUTPUT -m owner --uid-owner "$uid_value" '!' -d ::1/128 -m comment --comment fdn-g15-host-worker-diag -j LOG --log-prefix "FDN-G15-HOST6 " 2>/dev/null || true
    sudo iptables -D OUTPUT -m owner --uid-owner "$uid_value" '!' -d 127.0.0.0/8 -m comment --comment fdn-g15-host-worker -j REJECT 2>/dev/null || true
    sudo ip6tables -D OUTPUT -m owner --uid-owner "$uid_value" '!' -d ::1/128 -m comment --comment fdn-g15-host-worker -j REJECT 2>/dev/null || true
  fi
  if id "$probe_user" >/dev/null 2>&1; then
    sudo userdel "$probe_user" >/dev/null 2>&1 || true
  fi
  sudo rm -f "$worker_exec" >/dev/null 2>&1 || true
  sudo rm -f /usr/local/share/ca-certificates/fabushi-fdn-g15-host.crt >/dev/null 2>&1 || true
  sudo update-ca-certificates --fresh >/dev/null 2>&1 || true
  sudo sed -i '/[[:space:]]fdn-g15-host\.test$/d' /etc/hosts 2>/dev/null || true
  sudo rm -rf "$work_dir"
}
trap cleanup EXIT

rm -rf "$work_dir"
mkdir -p "$work_dir/headscale-state" "$work_dir/worker-state" "$work_dir/home" "$work_dir/tls"
chmod 0755 "$work_dir"
chmod 0700 "$work_dir/worker-state" "$work_dir/home"

openssl req -x509 -newkey rsa:2048 -nodes -days 1 \
  -subj "/CN=Fabushi FDN G15 Host Test CA" \
  -keyout "$work_dir/tls/ca.key" -out "$work_dir/tls/ca.crt" >/dev/null 2>&1
cat > "$work_dir/tls/server.cnf" <<'EOF'
[req]
distinguished_name = dn
req_extensions = ext
prompt = no
[dn]
CN = fdn-g15-host.test
[ext]
subjectAltName = @san
[san]
DNS.1 = fdn-g15-host.test
IP.1 = 127.0.0.1
EOF
openssl req -newkey rsa:2048 -nodes \
  -keyout "$work_dir/tls/server.key" -out "$work_dir/tls/server.csr" \
  -config "$work_dir/tls/server.cnf" >/dev/null 2>&1
openssl x509 -req -days 1 \
  -in "$work_dir/tls/server.csr" \
  -CA "$work_dir/tls/ca.crt" -CAkey "$work_dir/tls/ca.key" -CAcreateserial \
  -out "$work_dir/tls/server.crt" \
  -extensions ext -extfile "$work_dir/tls/server.cnf" >/dev/null 2>&1
chmod 0644 "$work_dir/tls/server.crt" "$work_dir/tls/server.key" "$work_dir/tls/ca.crt"
sudo cp "$work_dir/tls/ca.crt" /usr/local/share/ca-certificates/fabushi-fdn-g15-host.crt
sudo update-ca-certificates >/dev/null
echo "127.0.0.1 fdn-g15-host.test" | sudo tee -a /etc/hosts >/dev/null

cat > "$work_dir/policy.hujson" <<'EOF'
{
  "groups": {},
  "tagOwners": {},
  "hosts": {},
  "acls": [],
  "nodeAttrs": [
    {
      "target": ["*"],
      "attr": ["disable-captive-portal-detection"]
    }
  ]
}
EOF

cat > "$work_dir/derp.yaml" <<'EOF'
regions:
  902:
    regionid: 902
    regioncode: fabushi-g15-host-local
    regionname: Fabushi G15 Host local fixture
    nodes:
      - name: 902a
        regionid: 902
        hostname: fdn-g15-host.test
        ipv4: 127.0.0.1
        stunport: 0
        stunonly: false
        derpport: 443
EOF

cat > "$work_dir/headscale.yaml" <<'EOF'
server_url: https://fdn-g15-host.test
listen_addr: 127.0.0.1:443
metrics_listen_addr: ""
grpc_listen_addr: 127.0.0.1:50443
grpc_allow_insecure: false
trusted_proxies: []
noise:
  private_key_path: /var/lib/headscale/noise_private.key
prefixes:
  v4: 100.64.0.0/10
  v6: fd7a:115c:a1e0::/48
  allocation: random
derp:
  server:
    enabled: true
    region_id: 902
    region_code: "fabushi-g15-host"
    region_name: "Fabushi G15 Host isolated Actions"
    verify_clients: true
    stun_listen_addr: "127.0.0.1:3478"
    private_key_path: /var/lib/headscale/derp_server_private.key
    automatically_add_embedded_derp_region: false
  urls: []
  paths:
    - /etc/headscale/derp.yaml
  auto_update_enabled: false
  update_frequency: 24h
disable_check_updates: true
node:
  expiry: 1h
  ephemeral:
    inactivity_timeout: 30m
database:
  type: sqlite
  debug: false
  gorm:
    prepare_stmt: true
    parameterized_queries: true
    skip_err_record_not_found: true
    slow_threshold: 1000
  sqlite:
    path: /var/lib/headscale/db.sqlite
    write_ahead_log: true
    wal_autocheckpoint: 1000
log:
  level: warn
  format: json
policy:
  mode: file
  path: /etc/headscale/policy.hujson
dns:
  magic_dns: false
  base_domain: ""
  override_local_dns: false
  nameservers:
    global: []
    split: {}
  search_domains: []
  extra_records: []
unix_socket: /var/run/headscale/headscale.sock
unix_socket_permission: "0770"
tls_cert_path: /etc/headscale/tls.crt
tls_key_path: /etc/headscale/tls.key
EOF

docker run -d --name "$container" --network host \
  -v "$work_dir/headscale.yaml:/etc/headscale/config.yaml:ro" \
  -v "$work_dir/policy.hujson:/etc/headscale/policy.hujson:ro" \
  -v "$work_dir/derp.yaml:/etc/headscale/derp.yaml:ro" \
  -v "$work_dir/tls/server.crt:/etc/headscale/tls.crt:ro" \
  -v "$work_dir/tls/server.key:/etc/headscale/tls.key:ro" \
  -v "$work_dir/headscale-state:/var/lib/headscale" \
  docker.io/headscale/headscale:0.29.5 serve >/dev/null

ready=0
for _ in $(seq 1 60); do
  if docker exec "$container" headscale users list >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 1
done
if [[ "$ready" -ne 1 ]]; then
  echo "Headscale Host fixture did not become ready" >&2
  docker ps -a --filter "name=^/$container$" >&2 || true
  docker logs "$container" >&2 || true
  docker inspect "$container" --format '{{json .State}}' >&2 || true
  exit 1
fi

docker exec "$container" headscale users create g15-host >/dev/null
user_id="$(docker exec "$container" headscale users list --name g15-host --output json | python3 -c 'import json,sys; rows=json.load(sys.stdin); assert len(rows)==1; print(rows[0]["id"])')"
[[ "$user_id" =~ ^[0-9]+$ ]]
auth_key="$(docker exec "$container" headscale preauthkeys create --user "$user_id" | tr -d '\r' | tail -n1)"
test "${#auth_key}" -ge 20

if ! id "$probe_user" >/dev/null 2>&1; then
  sudo useradd --system --no-create-home --shell /usr/sbin/nologin "$probe_user"
fi
uid_value="$(id -u "$probe_user")"
sudo install -o "$probe_user" -g "$probe_user" -m 0755 "$worker_binary" "$worker_exec"
sudo chown -R "$probe_user":"$probe_user" "$work_dir/worker-state" "$work_dir/home"
# The runner shell owns both FIFO opens: it connects the child's stdin before
# sudo changes uid and keeps fd 3 for the private initialize/status stream.
# Keep the FIFO runner-only so auth material is never exposed via filesystem ACLs.
mkfifo -m 0600 "$work_dir/worker.stdin"

# GitHub-hosted Ubuntu advertises Azure-style DMI, so pinned Tailscale v1.104.1
# performs its normal cloud metadata probe before deciding this fixture is not a
# supported cloud VM. The shipping Desktop worker is not a cloud VM. Redirect
# only this probe user's metadata request into a loopback 404 sink so G15 keeps
# zero undeclared egress without weakening the production worker.
python3 - "$metadata_sink_port" <<'PY' >/dev/null 2>&1 &
from http.server import BaseHTTPRequestHandler, HTTPServer
import sys

class Handler(BaseHTTPRequestHandler):
    def _not_cloud(self):
        self.send_response(404)
        self.end_headers()
    do_GET = _not_cloud
    do_PUT = _not_cloud
    def log_message(self, *_):
        pass

HTTPServer(("127.0.0.1", int(sys.argv[1])), Handler).serve_forever()
PY
metadata_sink_pid="$!"
for _ in $(seq 1 20); do
  if curl --silent --output /dev/null --max-time 1 -X PUT "http://127.0.0.1:$metadata_sink_port/latest/api/token"; then
    break
  fi
  sleep 0.1
done
kill -0 "$metadata_sink_pid"
sudo iptables -t nat -I OUTPUT 1 -m owner --uid-owner "$uid_value" \
  -d 169.254.169.254/32 -p tcp --dport 80 \
  -m comment --comment fdn-g15-host-cloud-metadata-sink \
  -j DNAT --to-destination "127.0.0.1:$metadata_sink_port"

sudo iptables -I OUTPUT 1 -m owner --uid-owner "$uid_value" '!' -d 127.0.0.0/8 -m comment --comment fdn-g15-host-worker -j REJECT
sudo ip6tables -I OUTPUT 1 -m owner --uid-owner "$uid_value" '!' -d ::1/128 -m comment --comment fdn-g15-host-worker -j REJECT
# Log only packet headers immediately before the zero-tolerance reject rule so
# an Actions-only fixture failure identifies the protocol/destination. No
# payload is logged and production networking remains unchanged.
sudo iptables -I OUTPUT 1 -m owner --uid-owner "$uid_value" '!' -d 127.0.0.0/8 -m comment --comment fdn-g15-host-worker-diag -j LOG --log-prefix "FDN-G15-HOST4 "
sudo ip6tables -I OUTPUT 1 -m owner --uid-owner "$uid_value" '!' -d ::1/128 -m comment --comment fdn-g15-host-worker-diag -j LOG --log-prefix "FDN-G15-HOST6 "

# G15 isolates undeclared telemetry/privacy egress from G05 NAT-traversal
# behavior. Tailscale's portmapper legitimately probes LAN gateways
# (UPnP/NAT-PMP/PCP), so disable it only in this loopback-only Actions fixture.
# Headscale v0.29.5 merges custom regions while leaving default Tailscale
# regions enabled. G15 measures privacy egress, not G05 public DERP routing,
# so pin this Actions-only worker to the real embedded loopback DERP.
sudo -u "$probe_user" env -i \
  TS_DISABLE_PORTMAPPER=true \
  TS_DEBUG_USE_DERP_ADDR=fdn-g15-host.test \
  PATH=/usr/bin:/bin \
  HOME="$work_dir/home" \
  "$worker_exec" <"$work_dir/worker.stdin" >"$worker_protocol" 2>"$worker_log" &
sudo_pid="$!"
exec 3>"$work_dir/worker.stdin"
fifo_fd_open=1

init_json="$(python3 - "$auth_key" "$work_dir/worker-state" <<'PY'
import json, sys
print(json.dumps({
  "type": "initialize",
  "protocolVersion": 1,
  "hostname": "g15-host-worker",
  "controlUrl": "https://fdn-g15-host.test",
  "authKey": sys.argv[1],
  "stateDir": sys.argv[2],
  "stateKeyB64": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
}, separators=(",", ":")))
PY
)"
printf '%s\n' "$init_json" >&3
unset init_json

worker_pid=""
ready_worker=0
for _ in $(seq 1 90); do
  worker_pid="$(pgrep -u "$uid_value" -f "$worker_exec" | head -n1 || true)"
  if [[ -n "$worker_pid" ]] && grep -q '"type":"ready"' "$worker_protocol" 2>/dev/null; then
    ready_worker=1
    break
  fi
  if ! kill -0 "$sudo_pid" 2>/dev/null; then
    cat "$worker_log" >&2 || true
    cat "$worker_protocol" >&2 || true
    exit 1
  fi
  sleep 1
done
test "$ready_worker" -eq 1
test -n "$worker_pid"

running=0
for attempt in $(seq 1 60); do
  request_id="status-g15-host-$attempt"
  printf '{"type":"status","requestId":"%s"}\n' "$request_id" >&3
  for _ in $(seq 1 10); do
    if grep -Fq "\"requestId\":\"$request_id\"" "$worker_protocol"; then
      if grep -F "\"requestId\":\"$request_id\"" "$worker_protocol" | tail -n1 | grep -Fq '"backendState":"Running"'; then
        running=1
      fi
      break
    fi
    sleep 1
  done
  [[ "$running" -eq 1 ]] && break
  sleep 1
done
if [[ "$running" -ne 1 ]]; then
  echo "Host-owned network worker did not reach Running" >&2
  cat "$worker_log" >&2 || true
  tail -n 20 "$worker_protocol" >&2 || true
  exit 1
fi

sleep 20

clock_ticks="$(getconf CLK_TCK)"
start_ticks="$(awk '{print $14 + $15}' "/proc/$worker_pid/stat")"
start_ns="$(date +%s%N)"
max_rss_kib=0
max_state_bytes=0
max_log_bytes=0
max_protocol_bytes=0
min_available_bytes=9223372036854775807
total_bytes=0

for _ in $(seq 1 60); do
  sleep 5
  sudo kill -0 "$worker_pid"
  rss_kib="$(awk '/^VmRSS:/ {print $2}' "/proc/$worker_pid/status")"
  state_bytes="$(sudo du -sb "$work_dir/worker-state" | awk '{print $1}')"
  log_bytes="$(stat -c %s "$worker_log")"
  protocol_bytes="$(stat -c %s "$worker_protocol")"
  read -r fs_total fs_available < <(df -B1 --output=size,avail "$work_dir/worker-state" | tail -n1)
  (( rss_kib > max_rss_kib )) && max_rss_kib="$rss_kib"
  (( state_bytes > max_state_bytes )) && max_state_bytes="$state_bytes"
  (( log_bytes > max_log_bytes )) && max_log_bytes="$log_bytes"
  (( protocol_bytes > max_protocol_bytes )) && max_protocol_bytes="$protocol_bytes"
  (( fs_available < min_available_bytes )) && min_available_bytes="$fs_available"
  total_bytes="$fs_total"
done

end_ns="$(date +%s%N)"
end_ticks="$(awk '{print $14 + $15}' "/proc/$worker_pid/stat")"
elapsed_ns=$((end_ns - start_ns))
delta_ticks=$((end_ticks - start_ticks))
ipv4_blocked="$(sudo iptables -nvx -L OUTPUT | awk '/fdn-g15-host-worker/ {sum += $1} END {print sum+0}')"
ipv6_blocked="$(sudo ip6tables -nvx -L OUTPUT | awk '/fdn-g15-host-worker/ {sum += $1} END {print sum+0}')"
blocked_packets=$((ipv4_blocked + ipv6_blocked))
if (( blocked_packets != 0 )); then
  echo "Host G15 unexpected worker-owned packet headers:" >&2
  sudo dmesg | grep -E 'FDN-G15-HOST[46] ' >&2 || true
fi

state_limit_bytes=$((20 * 1024 * 1024))
log_limit_bytes=$((20 * 1024 * 1024))
rss_limit_bytes=$((150 * 1024 * 1024))
max_rss_bytes=$((max_rss_kib * 1024))
reserve_five_percent=$(((total_bytes + 19) / 20))
reserve_bytes=$((1024 * 1024 * 1024))
(( reserve_five_percent > reserve_bytes )) && reserve_bytes="$reserve_five_percent"

export FDN_G15_HOST_EXPECTED_SHA="$EXPECTED_SHA"
export FDN_G15_HOST_ELAPSED_NS="$elapsed_ns"
export FDN_G15_HOST_DELTA_TICKS="$delta_ticks"
export FDN_G15_HOST_CLOCK_TICKS="$clock_ticks"
export FDN_G15_HOST_MAX_RSS_BYTES="$max_rss_bytes"
export FDN_G15_HOST_RSS_LIMIT_BYTES="$rss_limit_bytes"
export FDN_G15_HOST_MAX_STATE_BYTES="$max_state_bytes"
export FDN_G15_HOST_STATE_LIMIT_BYTES="$state_limit_bytes"
export FDN_G15_HOST_MAX_LOG_BYTES="$max_log_bytes"
export FDN_G15_HOST_MAX_PROTOCOL_BYTES="$max_protocol_bytes"
export FDN_G15_HOST_LOG_LIMIT_BYTES="$log_limit_bytes"
export FDN_G15_HOST_TOTAL_BYTES="$total_bytes"
export FDN_G15_HOST_MIN_AVAILABLE_BYTES="$min_available_bytes"
export FDN_G15_HOST_RESERVE_BYTES="$reserve_bytes"
export FDN_G15_HOST_BLOCKED_PACKETS="$blocked_packets"
export FDN_G15_HOST_RUN_ID="${GITHUB_RUN_ID:-}"

python3 - "$evidence_dir/fdn-g15-host-resource-privacy.json" <<'PY'
import json, os, sys
elapsed_ns=int(os.environ["FDN_G15_HOST_ELAPSED_NS"])
ticks=int(os.environ["FDN_G15_HOST_DELTA_TICKS"])
hz=int(os.environ["FDN_G15_HOST_CLOCK_TICKS"])
elapsed_s=elapsed_ns/1_000_000_000
cpu=(ticks/hz)/elapsed_s*100
report={
 "desktopSha":os.environ["FDN_G15_HOST_EXPECTED_SHA"],
 "runId":os.environ.get("FDN_G15_HOST_RUN_ID"),
 "environment":"github-hosted-ubuntu-24.04",
 "runtime":{
   "worker":"shipping source/fabushi-network-worker supervised contract owner",
   "tailscale":"v1.104.1",
   "coordinator":"docker.io/headscale/headscale:0.29.5",
   "coordinatorMode":"real local TLS Headscale; no mock transport",
   "steadyStateSeconds":round(elapsed_s,3)
 },
 "budgets":{
   "rssBytes":{"maxObserved":int(os.environ["FDN_G15_HOST_MAX_RSS_BYTES"]),"limit":int(os.environ["FDN_G15_HOST_RSS_LIMIT_BYTES"])},
   "cpuSingleCorePercent":{"fiveMinuteAverage":round(cpu,6),"limitExclusive":1.0},
   "workerStateBytes":{"maxObserved":int(os.environ["FDN_G15_HOST_MAX_STATE_BYTES"]),"limit":int(os.environ["FDN_G15_HOST_STATE_LIMIT_BYTES"])},
   "workerLogBytes":{"maxObserved":int(os.environ["FDN_G15_HOST_MAX_LOG_BYTES"]),"protocolMaxObserved":int(os.environ["FDN_G15_HOST_MAX_PROTOCOL_BYTES"]),"limit":int(os.environ["FDN_G15_HOST_LOG_LIMIT_BYTES"])},
   "diskReserveBytes":{"filesystemTotal":int(os.environ["FDN_G15_HOST_TOTAL_BYTES"]),"minimumAvailableObserved":int(os.environ["FDN_G15_HOST_MIN_AVAILABLE_BYTES"]),"requiredReserve":int(os.environ["FDN_G15_HOST_RESERVE_BYTES"])}
 },
 "egress":{
   "fixtureApprovedOrigins":["https://fdn-g15-host.test:443","127.0.0.1 embedded DERP/STUN"],
   "unexpectedWorkerOwnedPacketsRejected":int(os.environ["FDN_G15_HOST_BLOCKED_PACKETS"])
 }
}
report["budgets"]["rssBytes"]["pass"]=report["budgets"]["rssBytes"]["maxObserved"] < report["budgets"]["rssBytes"]["limit"]
report["budgets"]["cpuSingleCorePercent"]["pass"]=cpu < 1.0
report["budgets"]["workerStateBytes"]["pass"]=report["budgets"]["workerStateBytes"]["maxObserved"] <= report["budgets"]["workerStateBytes"]["limit"]
report["budgets"]["workerLogBytes"]["pass"]=max(report["budgets"]["workerLogBytes"]["maxObserved"],report["budgets"]["workerLogBytes"]["protocolMaxObserved"]) <= report["budgets"]["workerLogBytes"]["limit"]
report["budgets"]["diskReserveBytes"]["pass"]=report["budgets"]["diskReserveBytes"]["minimumAvailableObserved"] >= report["budgets"]["diskReserveBytes"]["requiredReserve"]
report["egress"]["pass"]=report["egress"]["unexpectedWorkerOwnedPacketsRejected"] == 0
passes=[v["pass"] for v in report["budgets"].values()]+[report["egress"]["pass"]]
report["status"]="PASS" if all(passes) else "FAIL"
with open(sys.argv[1],"w",encoding="utf-8") as f: json.dump(report,f,indent=2,sort_keys=True)
print(json.dumps(report,sort_keys=True))
if report["status"]!="PASS": raise SystemExit(1)
PY

if grep -R -aF -- "$auth_key" "$evidence_dir" "$work_dir/worker-state" >/dev/null 2>&1; then
  echo "G15 Host privacy failure: auth key leaked into evidence/state" >&2
  exit 1
fi
if grep -R -aE 'hskey-auth-|tskey-auth-|Authorization:[[:space:]]*Bearer|-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----' "$evidence_dir" >/dev/null 2>&1; then
  echo "G15 Host privacy failure: credential-like material in evidence" >&2
  exit 1
fi

grep -F 'envknob.SetNoLogsNoSupport()' source/fabushi-network-worker/main.go >/dev/null
grep -F '.env_clear()' source/host/src/network_worker_process.rs >/dev/null
grep -F '.field("auth_key", &"<redacted>")' source/host/src/network_worker_process.rs >/dev/null
grep -F '.field("state_key_b64", &"<redacted>")' source/host/src/network_worker_process.rs >/dev/null
grep -F '"PATH",' source/host/src/network_worker_process.rs >/dev/null
if grep -nE '"(HTTP_PROXY|HTTPS_PROXY|ALL_PROXY|NO_PROXY|FABUSHI_[A-Z0-9_]*TOKEN|FABUSHI_[A-Z0-9_]*SECRET)"' source/host/src/network_worker_process.rs >/dev/null; then
  echo "G15 Host privacy failure: forbidden environment inheritance" >&2
  exit 1
fi

sha256sum "$evidence_dir/fdn-g15-host-resource-privacy.json" "$worker_log" "$worker_protocol" > "$evidence_dir/SHA256SUMS"
