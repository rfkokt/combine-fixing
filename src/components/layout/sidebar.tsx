import { useAppStore } from "@/stores/app-store";
import type { PageId } from "@/lib/types";
import {
  GitMerge,
  SpellCheck,
  Settings,
  Scan,
} from "lucide-react";

interface NavItem {
  id: PageId;
  label: string;
  icon: React.ReactNode;
  phase: string;
}

const navItems: NavItem[] = [
  {
    id: "home",
    label: "DASHBOARD",
    icon: <Scan size={18} />,
    phase: "[00]",
  },
  {
    id: "merge",
    label: "MERGE",
    icon: <GitMerge size={18} />,
    phase: "[01]",
  },
  {
    id: "spellcheck",
    label: "SPELLCHECK",
    icon: <SpellCheck size={18} />,
    phase: "[02]",
  },
  {
    id: "settings",
    label: "SETTINGS",
    icon: <Settings size={18} />,
    phase: "[03]",
  },
];

export function Sidebar() {
  const { currentPage, setCurrentPage } = useAppStore();

  return (
    <aside className="w-[240px] h-full border-r border-border-subtle bg-bg-primary flex flex-col">
      {/* Logo */}
      <div className="px-5 py-6 border-b border-border-subtle">
        <h1 className="text-display text-2xl tracking-widest text-text-primary">
          DOCFIXER
        </h1>
        <p className="text-label text-[0.6rem] text-text-dim mt-1">
          DOCUMENT PROCESSOR v0.1
        </p>
      </div>

      {/* Navigation */}
      <nav className="flex-1 py-4 px-3">
        {navItems.map((item) => {
          const isActive = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setCurrentPage(item.id)}
              className={`
                w-full flex items-center gap-3 px-3 py-3 mb-1
                transition-all duration-300 cursor-pointer
                ${
                  isActive
                    ? "bg-surface-hover border-l-2 border-accent-cyan text-accent-cyan"
                    : "border-l-2 border-transparent text-text-muted hover:text-text-primary hover:bg-surface-hover"
                }
              `}
            >
              <span className="phase-label text-[0.6rem] opacity-50">
                {item.phase}
              </span>
              {item.icon}
              <span className="text-label text-[0.65rem]">{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* Status bar at bottom */}
      <div className="px-5 py-4 border-t border-border-subtle">
        <div className="flex items-center gap-2">
          <div className="status-dot" />
          <span className="text-label text-[0.6rem] text-text-dim">
            READY
          </span>
        </div>
      </div>
    </aside>
  );
}
