import { useState, useEffect } from 'react';
import { PRESET_PROVIDERS } from '../lib/providers';

const PROVIDERS = PRESET_PROVIDERS;

export function SettingsPage() {
  const [saved, setSaved] = useState(false);
  const [keys, setKeys] = useState<Record<string, string>>({});

  useEffect(() => {
    const loadedKeys: Record<string, string> = {};
    PROVIDERS.forEach(p => {
      // Migrate old key if exists
      let key = localStorage.getItem(`ai_api_key_${p.id}`);
      if (!key && localStorage.getItem('openai_api_key') && (p.id === 'groq' || p.id === localStorage.getItem('ai_provider'))) {
          key = localStorage.getItem('openai_api_key');
          if (key) localStorage.setItem(`ai_api_key_${p.id}`, key);
      }
      loadedKeys[p.id] = key || '';
    });
    setKeys(loadedKeys);
  }, []);

  const handleKeyChange = (id: string, value: string) => {
    setKeys(prev => ({ ...prev, [id]: value }));
    localStorage.setItem(`ai_api_key_${id}`, value);
  };

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="h-full blueprint-grid overflow-y-auto">
      <div className="relative z-10 p-8 max-w-3xl mx-auto">
        <div className="mb-8">
          <p className="phase-label text-sm mb-2">[03] CONFIGURATION</p>
          <h1 className="text-display text-4xl text-text-primary">SETTINGS</h1>
        </div>

        {/* Language Settings */}
        <div className="border border-border-subtle p-6 mb-6">
          <h3 className="text-label text-text-primary mb-4">LANGUAGE</h3>
          <p className="text-body text-sm text-text-muted mb-4">
            Pilih bahasa yang digunakan untuk spell checking.
          </p>
          <div className="flex gap-4">
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                defaultChecked
                className="accent-accent-cyan"
              />
              <span className="text-body text-sm text-text-muted">
                Bahasa Indonesia
              </span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                defaultChecked
                className="accent-accent-cyan"
              />
              <span className="text-body text-sm text-text-muted">English</span>
            </label>
          </div>
        </div>

        {/* Custom Dictionary */}
        <div className="border border-border-subtle p-6 mb-6">
          <h3 className="text-label text-text-primary mb-4">
            CUSTOM DICTIONARY
          </h3>
          <p className="text-body text-sm text-text-muted mb-4">
            Words added here won't be flagged as typos.
          </p>
          <textarea
            placeholder="Add words, one per line..."
            rows={6}
            className="w-full bg-transparent border border-border-subtle px-4 py-2.5 text-body text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-cyan transition-colors resize-none selectable"
          />
          <div className="flex items-center gap-3 mt-4">
            <button onClick={handleSave} className="btn-primary">
              SAVE DICTIONARY
            </button>
            {saved && (
              <span className="text-xs text-accent-lime">Saved ✓</span>
            )}
          </div>
        </div>

        {/* API Keys */}
        <div className="border border-border-subtle p-6">
          <h3 className="text-label text-text-primary mb-4">API KEYS</h3>
          <p className="text-body text-sm text-text-muted mb-6">
            Masukkan API Key untuk masing-masing provider. Kunci akan disimpan secara lokal di browser kamu.
          </p>
          
          <div className="space-y-4">
            {PROVIDERS.map(provider => (
              <div key={provider.id} className="flex flex-col gap-1">
                <label className="text-xs text-text-primary">{provider.name}</label>
                <input
                  type="password"
                  value={keys[provider.id] || ''}
                  onChange={(e) => handleKeyChange(provider.id, e.target.value)}
                  placeholder={`API Key untuk ${provider.name}...`}
                  className="w-full bg-transparent border border-border-subtle px-4 py-2 text-body text-sm text-text-primary placeholder:text-text-dim focus:outline-none focus:border-accent-cyan transition-colors selectable"
                />
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
