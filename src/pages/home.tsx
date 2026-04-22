import { useAppStore } from "@/stores/app-store";
import { GitMerge, SpellCheck, FileText, ArrowRight } from "lucide-react";

interface ActionCardProps {
  phase: string;
  title: string;
  subtitle: string;
  icon: React.ReactNode;
  onClick: () => void;
}

function ActionCard({ phase, title, subtitle, icon, onClick }: ActionCardProps) {
  return (
    <button
      onClick={onClick}
      className="lift-card corner-marks corner-marks-cyan p-8 text-left cursor-pointer group w-full"
    >
      <span className="phase-label text-sm">{phase}</span>
      <div className="mt-4 flex items-center gap-4">
        <div className="text-accent-cyan">{icon}</div>
        <div>
          <h2 className="text-display text-3xl text-text-primary group-hover:text-accent-cyan transition-colors duration-300">
            {title}
          </h2>
          <p className="text-label text-[0.65rem] text-text-muted mt-1">
            {subtitle}
          </p>
        </div>
      </div>
      <div className="mt-6 flex items-center gap-2 text-text-dim group-hover:text-accent-cyan transition-all duration-300">
        <span className="text-label text-[0.6rem]">GET STARTED</span>
        <ArrowRight size={12} className="transform group-hover:translate-x-1 transition-transform duration-300" />
      </div>
    </button>
  );
}

export function HomePage() {
  const { setCurrentPage } = useAppStore();

  return (
    <div className="h-full blueprint-grid overflow-y-auto">
      <div className="relative z-10 p-8 max-w-5xl mx-auto">
        {/* Hero */}
        <div className="mb-12">
          <p className="phase-label text-sm mb-2">[SYSTEM READY]</p>
          <h1 className="text-display text-6xl text-text-primary mb-3">
            DOCFIXER
          </h1>
          <p className="text-body text-text-muted max-w-lg">
            Desktop document processor untuk menggabungkan dokumen dan memperbaiki kesalahan penulisan secara otomatis.
          </p>
        </div>

        {/* Action Cards */}
        <div className="grid grid-cols-2 gap-6 mb-12">
          <ActionCard
            phase="[01]"
            title="MERGE DOCS"
            subtitle="COMBINE & REORDER"
            icon={<GitMerge size={32} />}
            onClick={() => setCurrentPage("merge")}
          />
          <ActionCard
            phase="[02]"
            title="CHECK TYPOS"
            subtitle="SCAN & CORRECT"
            icon={<SpellCheck size={32} />}
            onClick={() => setCurrentPage("spellcheck")}
          />
        </div>

        {/* Recent Activity */}
        <div>
          <h3 className="text-label text-text-dim mb-4 flex items-center gap-2">
            <FileText size={14} />
            RECENT ACTIVITY
          </h3>
          <div className="border border-border-subtle">
            {[1, 2, 3].map((i) => (
              <div
                key={i}
                className="flex items-center justify-between px-5 py-3 border-b border-border-subtle last:border-b-0 hover:bg-surface-hover transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="status-dot" />
                  <span className="text-body text-sm text-text-muted">
                    No recent documents
                  </span>
                </div>
                <span className="text-label text-[0.6rem] text-text-dim">—</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
