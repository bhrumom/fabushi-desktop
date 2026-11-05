import { useState } from "react";
import { SandIcon } from "../recovered/ui/sand-kit-primitives";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import type { RuntimeDetailSectionId } from "./agent-runtime/types";
import { RuntimeSubagentsSection } from "./agent-runtime/RuntimeSubagentsSection";
import { SecurityPolicySection } from "./agent-runtime/SecurityPolicySection";
import { ExternalChannelsSection } from "./agent-runtime/ExternalChannelsSection";
import { NodesMultimodalSection } from "./agent-runtime/NodesMultimodalSection";
import "./agent-runtime/agent-runtime.css";

export type { RuntimeDetailSectionId };

export interface FabushiAgentRuntimeDetailSectionsProps {
  readonly agentId: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly initialSection?: RuntimeDetailSectionId;
  readonly onClose?: () => void;
}

const SECTIONS: Array<{ id: RuntimeDetailSectionId; label: string; icon: "sliders" | "shield" | "chat-bubbles" | "server" }> = [
  { id: "runtime", label: "运行时与调度", icon: "sliders" },
  { id: "security", label: "安全与审计", icon: "shield" },
  { id: "channels", label: "外部渠道", icon: "chat-bubbles" },
  { id: "nodes", label: "节点与多模态", icon: "server" },
];

export function FabushiAgentRuntimeDetailSections({
  agentId,
  client = null,
  initialSection = "runtime",
  onClose: _onClose,
}: FabushiAgentRuntimeDetailSectionsProps) {
  const [activeSection, setActiveSection] = useState<RuntimeDetailSectionId>(initialSection);

  return (
    <div className="sand-agent-runtime-pane" data-testid="fabushi-agent-runtime-pane">
      {/* Canonical Sub-Navigation */}
      <nav className="sand-runtime-section-nav" aria-label="Runtime details tabs">
        {SECTIONS.map((sec) => (
          <button
            key={sec.id}
            type="button"
            className="sand-runtime-nav-btn"
            data-active={activeSection === sec.id}
            data-testid={`runtime-nav-${sec.id}`}
            onClick={() => setActiveSection(sec.id)}
          >
            <SandIcon name={sec.icon} size="sm" />
            <span>{sec.label}</span>
          </button>
        ))}
      </nav>

      {/* Section Scroll Container */}
      <div className="sand-runtime-section-scroll">
        {activeSection === "runtime" && <RuntimeSubagentsSection agentId={agentId} client={client} />}
        {activeSection === "security" && <SecurityPolicySection agentId={agentId} client={client} />}
        {activeSection === "channels" && <ExternalChannelsSection agentId={agentId} client={client} />}
        {activeSection === "nodes" && <NodesMultimodalSection agentId={agentId} client={client} />}
      </div>
    </div>
  );
}
