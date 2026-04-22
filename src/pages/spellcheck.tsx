import { Upload, Zap } from "lucide-react";

export function SpellcheckPage() {
  return (
    <div className="h-full flex">
      {/* Document Viewer (Left) */}
      <div className="flex-1 blueprint-grid overflow-y-auto">
        <div className="relative z-10 p-8">
          <div className="mb-8">
            <p className="phase-label text-sm mb-2">[02] TYPO SCANNER</p>
            <h1 className="text-display text-4xl text-text-primary">
              SPELLCHECK
            </h1>
          </div>

          {/* Dropzone when no file loaded */}
          <div className="corner-marks corner-marks-cyan border-2 border-dashed border-border-subtle hover:border-accent-cyan/50 transition-colors duration-300 p-16 flex flex-col items-center justify-center gap-4 cursor-pointer group">
            <Upload
              size={48}
              className="text-text-dim group-hover:text-accent-cyan transition-colors duration-300"
            />
            <p className="text-display text-xl text-text-muted group-hover:text-text-primary transition-colors">
              DROP A DOCUMENT TO SCAN
            </p>
            <p className="text-label text-[0.6rem] text-text-dim">
              .DOCX OR .XLSX FILES SUPPORTED
            </p>
          </div>

          {/* Quick/Deep scan buttons */}
          <div className="mt-8 flex gap-4">
            <button className="btn-primary flex items-center gap-2" disabled>
              <Zap size={14} />
              QUICK SCAN
            </button>
            <button className="btn-ghost flex items-center gap-2" disabled>
              <Zap size={14} />
              AI DEEP SCAN
            </button>
          </div>
        </div>
      </div>

      {/* Review Panel (Right) */}
      <div className="w-[360px] h-full border-l border-border-subtle bg-bg-secondary flex flex-col">
        <div className="px-5 py-4 border-b border-border-subtle flex items-center justify-between">
          <span className="text-label text-text-muted">SCAN RESULTS</span>
          <span className="text-label text-[0.6rem] text-text-dim border border-border-subtle px-2 py-0.5">
            0 ISSUES
          </span>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <p className="text-label text-text-dim text-[0.65rem]">
            NO SCAN RESULTS YET
          </p>
        </div>
      </div>
    </div>
  );
}
