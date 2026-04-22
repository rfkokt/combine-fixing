import { Upload } from "lucide-react";

export function MergePage() {
  return (
    <div className="h-full blueprint-grid overflow-y-auto">
      <div className="relative z-10 p-8 max-w-5xl mx-auto">
        <div className="mb-8">
          <p className="phase-label text-sm mb-2">[01] DOCUMENT MERGE</p>
          <h1 className="text-display text-4xl text-text-primary">
            MERGE DOCUMENTS
          </h1>
        </div>

        {/* Dropzone */}
        <div className="corner-marks corner-marks-cyan border-2 border-dashed border-border-subtle hover:border-accent-cyan/50 transition-colors duration-300 p-16 flex flex-col items-center justify-center gap-4 cursor-pointer group">
          <Upload
            size={48}
            className="text-text-dim group-hover:text-accent-cyan transition-colors duration-300"
          />
          <p className="text-display text-xl text-text-muted group-hover:text-text-primary transition-colors">
            DROP .DOCX FILES HERE
          </p>
          <p className="text-label text-[0.6rem] text-text-dim">
            OR CLICK TO BROWSE
          </p>
        </div>

        {/* Placeholder for document list */}
        <div className="mt-8 border border-border-subtle p-6">
          <p className="text-label text-text-dim text-center">
            NO DOCUMENTS ADDED YET
          </p>
        </div>
      </div>
    </div>
  );
}
