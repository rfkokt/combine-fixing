import { Key } from "lucide-react";

export function SettingsPage() {
  return (
    <div className="h-full blueprint-grid overflow-y-auto">
      <div className="relative z-10 p-8 max-w-3xl mx-auto">
        <div className="mb-8">
          <p className="phase-label text-sm mb-2">[03] CONFIGURATION</p>
          <h1 className="text-display text-4xl text-text-primary">
            SETTINGS
          </h1>
        </div>

        {/* OpenAI API Key */}
        <div className="border border-border-subtle p-6 mb-6">
          <div className="flex items-center gap-3 mb-4">
            <Key size={18} className="text-accent-cyan" />
            <h3 className="text-label text-text-primary">OPENAI API KEY</h3>
          </div>
          <p className="text-body text-sm text-text-muted mb-4">
            Required for AI Deep Scan feature. Your API key is stored locally and never sent to third parties.
          </p>
          <input
            type="password"
            placeholder="sk-..."
            className="w-full bg-transparent border border-border-subtle px-4 py-2.5 text-body text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-cyan transition-colors"
          />
          <button className="btn-primary mt-4">
            SAVE KEY
          </button>
        </div>

        {/* Language Settings */}
        <div className="border border-border-subtle p-6 mb-6">
          <h3 className="text-label text-text-primary mb-4">LANGUAGE</h3>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" defaultChecked className="accent-accent-cyan" />
              <span className="text-body text-sm text-text-muted">Bahasa Indonesia</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input type="checkbox" defaultChecked className="accent-accent-cyan" />
              <span className="text-body text-sm text-text-muted">English</span>
            </label>
          </div>
        </div>

        {/* Custom Dictionary */}
        <div className="border border-border-subtle p-6">
          <h3 className="text-label text-text-primary mb-4">CUSTOM DICTIONARY</h3>
          <p className="text-body text-sm text-text-muted mb-4">
            Words added here won't be flagged as typos.
          </p>
          <textarea
            placeholder="Add words, one per line..."
            rows={6}
            className="w-full bg-transparent border border-border-subtle px-4 py-2.5 text-body text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-cyan transition-colors resize-none selectable"
          />
        </div>
      </div>
    </div>
  );
}
