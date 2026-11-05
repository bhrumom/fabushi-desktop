import { useEffect, useState } from "react";
import { SandButton, SandIcon } from "../../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "../coordinator-client";
import type { DevicePairingRequest, NodeDescription } from "./types";

interface NodesMultimodalSectionProps {
  readonly agentId: string;
  readonly client?: ProductionCoordinatorClient | null;
}

const DEFAULT_NODES: NodeDescription[] = [
  { nodeId: "node-local", name: "本机沙箱宿主 (Local Daemon)", platform: "macOS", connected: true, capabilities: ["system.run", "browser.proxy", "canvas"], activeLeases: 1 },
];

const DEFAULT_PAIR_REQUESTS: DevicePairingRequest[] = [
  { id: "pair-req-1", deviceName: "Fabushi Companion (iOS)", platform: "iOS", requestedAt: "10分钟前", status: "pending" },
  { id: "pair-req-2", deviceName: "终端控制台 (Terminal Web)", platform: "Linux", requestedAt: "1小时前", status: "approved" },
];

export function NodesMultimodalSection({ agentId: _agentId, client }: NodesMultimodalSectionProps) {
  const [nodes, setNodes] = useState<NodeDescription[]>(DEFAULT_NODES);
  const [pairingRequests, setPairingRequests] = useState<DevicePairingRequest[]>(DEFAULT_PAIR_REQUESTS);
  const [toast, setToast] = useState<string | null>(null);

  const showToast = (msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  };

  useEffect(() => {
    let active = true;
    async function loadNodesAndPairing() {
      if (!client) return;
      try {
        const nodeList = (await client.call("node.list", {})) as { nodes?: NodeDescription[] } | null;
        if (active && nodeList?.nodes) {
          setNodes(nodeList.nodes);
        }
      } catch {
        // Fallback
      }
      try {
        const pairList = (await client.call("device.pair.list", {})) as { requests?: DevicePairingRequest[] } | null;
        if (active && pairList?.requests) {
          setPairingRequests(pairList.requests);
        }
      } catch {
        // Fallback
      }
    }
    void loadNodesAndPairing();
    return () => {
      active = false;
    };
  }, [client]);

  const approvePairing = async (id: string) => {
    setPairingRequests((prev) =>
      prev.map((r) => (r.id === id ? { ...r, status: "approved" } : r))
    );
    if (client) {
      try {
        await client.call("device.pair.approve", { requestId: id });
        showToast("已批准设备配对授权 (device.pair.approve)");
      } catch (err) {
        showToast(`配对批准: ${err instanceof Error ? err.message : String(err)}`);
      }
    } else {
      showToast("已批准设备配对");
    }
  };

  const rejectPairing = async (id: string) => {
    setPairingRequests((prev) => prev.filter((r) => r.id !== id));
    if (client) {
      try {
        await client.call("device.pair.reject", { requestId: id });
        showToast("已拒绝并撤销配对令牌 (device.pair.reject)");
      } catch {
        // ignore
      }
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {toast && (
        <div className="sand-agent-action-toast">
          <SandIcon name="server" size="sm" />
          <span>{toast}</span>
        </div>
      )}

      {/* Upstream Node-Host Status (node.list / node.describe) */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>节点宿主环境 (Node-Host Fleet)</span>
          <span className="sand-runtime-pill sand-runtime-pill--green">
            在线节点: {nodes.filter((n) => n.connected).length}
          </span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          对接 OpenClaw node.list/node.invoke 架构。提供受审计的隔离命令执行能力。
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {nodes.map((node) => (
            <div
              key={node.nodeId}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: "6px 8px",
                background: "rgba(128,128,128,0.04)",
                borderRadius: 4,
                fontSize: 11,
              }}
            >
              <div>
                <span style={{ fontWeight: 600 }}>{node.name}</span>
                <span style={{ color: "var(--cursor-text-secondary, #6b7280)", marginLeft: 6, fontSize: 10 }}>
                  ({node.platform})
                </span>
                <div style={{ fontSize: 10, color: "var(--cursor-text-secondary, #6b7280)", marginTop: 2 }}>
                  能力: {node.capabilities.join(", ")}
                </div>
              </div>
              <span className={`sand-runtime-pill sand-runtime-pill--${node.connected ? "green" : "gray"}`}>
                {node.connected ? "在线" : "离线"}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Upstream Device Pairing Gate (device.pair.list / approve / reject) */}
      <div className="sand-runtime-card">
        <div className="sand-runtime-card-title">
          <span>设备配对审核 (Device Pairing Gate)</span>
        </div>
        <p style={{ margin: "0 0 8px", fontSize: 11, color: "var(--cursor-text-secondary, #6b7280)" }}>
          遵循 device.pair 规范。外部移动端与附属终端接入前必须经本机显式审批。
        </p>

        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          {pairingRequests.map((req) => (
            <div
              key={req.id}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                padding: "6px 8px",
                background: "rgba(128,128,128,0.04)",
                borderRadius: 4,
                fontSize: 11,
              }}
            >
              <div>
                <span style={{ fontWeight: 500 }}>{req.deviceName}</span>
                <span style={{ color: "var(--cursor-text-secondary, #6b7280)", marginLeft: 6, fontSize: 10 }}>
                  ({req.platform} • {req.requestedAt})
                </span>
              </div>
              <div style={{ display: "flex", gap: 4 }}>
                {req.status === "pending" ? (
                  <>
                    <SandButton onClick={() => void approvePairing(req.id)} size="sm" variant="primary">
                      批准
                    </SandButton>
                    <SandButton onClick={() => void rejectPairing(req.id)} size="sm" variant="secondary">
                      拒绝
                    </SandButton>
                  </>
                ) : (
                  <span className="sand-runtime-pill sand-runtime-pill--green">已信任</span>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
