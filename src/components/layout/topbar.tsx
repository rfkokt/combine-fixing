import { useAppStore } from "@/stores/app-store";
import { FileText } from "lucide-react";

export function Topbar() {
  const { currentPage } = useAppStore();

  const pageLabels: Record<string, string> = {
    home: "DASHBOARD",
    merge: "DOCUMENT MERGE",
    spellcheck: "TYPO SCANNER",
    settings: "CONFIGURATION",
  };

  return (
    <header className="h-12 border-b border-border-subtle bg-surface-nav backdrop-blur-md flex items-center justify-between px-6">
      {/* Current page indicator */}
      <div className="flex items-center gap-3">
        <div className="w-1 h-4 bg-accent-cyan" />
        <span className="text-label text-text-muted">
          {pageLabels[currentPage] || "DOCFIXER"}
        </span>
      </div>

      {/* Right side actions */}
      <div className="flex items-center gap-4">
        <button className="btn-primary text-[0.65rem] py-1.5 px-4 flex items-center gap-2">
          <FileText size={14} />
          OPEN FILE
        </button>
      </div>
    </header>
  );
}
