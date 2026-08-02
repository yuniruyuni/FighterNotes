import {
  ArrowLeft,
  BarChart3,
  CircleAlert,
  Play,
  Search,
  Zap,
} from "lucide-react";
import type { ReactNode } from "react";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import { AppFooter } from "~/shared/ui/AppFooter.js";

interface WorkspaceSidebarProps {
  filename: string;
  cards: readonly AdviceCard[];
  selected: string;
  onBack(): void;
  onSummary(): void;
  onCard(card: AdviceCard, index: number): void;
  onVideo(): void;
  onDebug(): void;
}

export function WorkspaceSidebar(props: WorkspaceSidebarProps) {
  return (
    <aside className="clips-sidebar">
      <div className="sidebar-header">
        <button type="button" className="btn-back" onClick={props.onBack}>
          <ArrowLeft size={16} aria-hidden="true" />
          <span>解析し直す</span>
        </button>
        <div className="sidebar-filename" title={props.filename}>
          {props.filename}
        </div>
      </div>
      <nav className="workspace-navigation" aria-label="解析結果">
        <div className="clip-list">
          <SidebarItem
            className="summary-item"
            selected={props.selected === "summary"}
            controls="view-summary"
            label="解析サマリー"
            icon={<BarChart3 size={16} aria-hidden="true" />}
            onClick={props.onSummary}
          />
          {props.cards.map((card, index) => (
            <SidebarItem
              key={card.id}
              selected={props.selected === `card-${index}`}
              controls={
                card.evidence.length > 0 ? "view-video" : "view-summary"
              }
              label={card.title}
              detail={`${card.evidence.length} 場面`}
              icon={
                card.id === "big_hits" ? (
                  <Zap size={16} aria-hidden="true" />
                ) : (
                  <CircleAlert size={16} aria-hidden="true" />
                )
              }
              onClick={() => props.onCard(card, index)}
            />
          ))}
          <SidebarItem
            selected={props.selected === "video"}
            controls="view-video"
            label="動画"
            icon={<Play size={16} aria-hidden="true" />}
            onClick={props.onVideo}
          />
        </div>
        <div className="sidebar-debug">
          <SidebarItem
            className="debug-item"
            selected={props.selected === "debug"}
            controls="view-debug"
            label="認識デバッグ"
            icon={<Search size={16} aria-hidden="true" />}
            onClick={props.onDebug}
          />
        </div>
      </nav>
      <div className="sidebar-footer">
        <AppFooter compact />
      </div>
    </aside>
  );
}

function SidebarItem({
  className = "",
  selected,
  controls,
  label,
  detail,
  icon,
  onClick,
}: {
  className?: string;
  selected: boolean;
  controls: string;
  label: string;
  detail?: string;
  icon: ReactNode;
  onClick(): void;
}) {
  return (
    <button
      type="button"
      className={`clip-item ${className} ${selected ? "selected" : ""}`.trim()}
      aria-current={selected ? "page" : undefined}
      aria-controls={controls}
      onClick={onClick}
    >
      <span className="ci-label">
        {icon}
        <span>{label}</span>
      </span>
      {detail && <span className="ci-hp">{detail}</span>}
    </button>
  );
}
