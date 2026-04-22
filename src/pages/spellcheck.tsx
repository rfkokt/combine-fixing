import { Upload, Zap, FileText, Activity, Settings, CheckCircle, AlertTriangle, Wrench, Brain, Sparkles, Search } from "lucide-react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSpellcheckStore, ScanResult, DocumentInfo, TypoFinding } from "../stores/spellcheck-store";
import { PRESET_PROVIDERS } from "../lib/providers";
import { toast } from "sonner";
import { useEffect, useState, useRef, useCallback } from "react";

interface AiFixResult {
  message: string;
  findings: TypoFinding[];
}

interface AiProgress {
  current: number;
  total: number;
  percentage: number;
  status: string;
  fixedCount: number;
  message: string;
}

const FIX_MODES = [
  {
    id: 'quick',
    label: 'QUICK FIX',
    desc: 'Offline only — regex fixes',
    icon: Wrench,
    color: 'text-accent-lime',
    borderColor: 'border-accent-lime',
    needsApi: false,
  },
  {
    id: 'smart',
    label: 'SMART FIX',
    desc: 'Hybrid — AI only for flagged',
    icon: Brain,
    color: 'text-accent-cyan',
    borderColor: 'border-accent-cyan',
    needsApi: true,
  },
  {
    id: 'deep',
    label: 'DEEP FIX',
    desc: 'Full AI — check everything',
    icon: Sparkles,
    color: 'text-purple-400',
    borderColor: 'border-purple-400',
    needsApi: true,
  },
] as const;

export function SpellcheckPage() {
  const { currentDocument, findings, isScanning, setScanning, setScanResult, setCurrentDocument, clear } = useSpellcheckStore();
  
  // AI Config State
  const [providerId, setProviderId] = useState(() => localStorage.getItem('ai_provider') || 'groq');
  const [baseUrl, setBaseUrl] = useState(() => localStorage.getItem('ai_base_url') || 'https://api.groq.com/openai/v1');
  const [model, setModel] = useState(() => localStorage.getItem('ai_model') || 'llama-3.3-70b-versatile');
  const [fixMode, setFixMode] = useState<string>(() => localStorage.getItem('fix_mode') || 'smart');

  // AI Progress State
  const [aiProgress, setAiProgress] = useState<AiProgress | null>(null);
  const [fixedFilePath, setFixedFilePath] = useState<string | null>(null);
  const [fixedFindings, setFixedFindings] = useState<TypoFinding[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const unlistenRef = useRef<(() => void) | null>(null);

  const handleConfigChange = (key: string, value: string) => {
    localStorage.setItem(key, value);
    if (key === 'ai_provider') setProviderId(value);
    if (key === 'ai_base_url') setBaseUrl(value);
    if (key === 'ai_model') setModel(value);
  };

  const handleProviderSelect = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedId = e.target.value;
    handleConfigChange('ai_provider', selectedId);
    
    const provider = PRESET_PROVIDERS.find(p => p.id === selectedId);
    if (provider && provider.id !== 'custom') {
      handleConfigChange('ai_base_url', provider.baseUrl);
      handleConfigChange('ai_model', provider.models[0]);
    }
  };

  const handleFixModeChange = (mode: string) => {
    setFixMode(mode);
    localStorage.setItem('fix_mode', mode);
  };

  const selectedProvider = PRESET_PROVIDERS.find(p => p.id === providerId) || PRESET_PROVIDERS[3];
  const selectedMode = FIX_MODES.find(m => m.id === fixMode) || FIX_MODES[1];

  // Listen for AI progress events
  useEffect(() => {
    let cancelled = false;
    listen<AiProgress>("ai-progress", (event) => {
      if (!cancelled) {
        setAiProgress(event.payload);
      }
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });
    return () => {
      cancelled = true;
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, []);

  // Initialize engine on mount
  useEffect(() => {
    invoke('init_spellcheck').then(() => {
      console.log('Spellcheck engine initialized');
    }).catch(console.error);
  }, []);

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Documents',
          extensions: ['docx', 'xlsx']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        loadDocument(selected);
      }
    } catch (error) {
      toast.error('Failed to select file');
      console.error(error);
    }
  };

  const loadDocument = async (filePath: string) => {
    setScanning(true);
    setAiProgress(null);
    setFixedFilePath(null);
    setFixedFindings([]);
    setSearchQuery("");
    try {
      const doc: DocumentInfo = await invoke('load_document', { filePath });
      setCurrentDocument(doc);
    } catch (error) {
      console.error('Load error:', error);
      toast.error(typeof error === 'string' ? error : 'Failed to load document');
    } finally {
      setScanning(false);
    }
  };

  const runOfflineScan = async () => {
    if (!currentDocument) return;
    
    setScanning(true);
    toast.info('Running offline typo scan...', { id: 'scan-toast' });
    
    try {
      const result: ScanResult = await invoke('scan_document', { filePath: currentDocument.path });
      setScanResult(result);
      toast.success(`Scan complete! Found ${result.findings.length} potential issues.`, { id: 'scan-toast' });
    } catch (error) {
      console.error('Scan error:', error);
      toast.error(typeof error === 'string' ? error : 'Failed to scan document', { id: 'scan-toast' });
    } finally {
      setScanning(false);
    }
  };

  const handleAIFix = async () => {
    if (!currentDocument) return;

    try {
      const outputPath = currentDocument.path.replace(/\.docx$/i, '_fixed.docx');
      setFixedFilePath(null); // clear previous
      setFixedFindings([]); // clear previous

      setScanning(true);
      setAiProgress({ current: 0, total: 0, percentage: 0, status: 'starting', fixedCount: 0, message: 'Initializing...' });
      
      const modeLabel = selectedMode.label;
      toast.info(`${modeLabel}: Processing your document...`, { id: 'ai-toast', duration: 300000 });

      // Fetch API key for the selected provider only
      const currentApiKey = localStorage.getItem(`ai_api_key_${providerId}`) || '';
      
      if (selectedMode.needsApi && !currentApiKey) {
        toast.error(`Missing API Key for ${selectedProvider.name}. Please set it in Settings.`, { id: 'ai-toast' });
        setScanning(false);
        setAiProgress(null);
        return;
      }

      const result: AiFixResult = await invoke('fix_document_with_ai', {
        inputPath: currentDocument.path,
        outputPath: outputPath,
        apiKey: currentApiKey,
        baseUrl: baseUrl,
        model: model,
        fixMode: fixMode
      });

      setFixedFindings(result.findings || []);

      const fileName = outputPath.split(/[/\\]/).pop() || '_fixed.docx';
      setFixedFilePath(outputPath);
      toast.success(`Done! Saved as ${fileName}`, { id: 'ai-toast' });
      
      // Update aiProgress status one final time so it shows done
      setAiProgress(prev => prev ? { ...prev, status: 'done', percentage: 100 } : null);
    } catch (error) {
      console.error('Fix error:', error);
      toast.error(typeof error === 'string' ? error : 'Failed to fix document', { id: 'ai-toast' });
      setAiProgress(prev => prev ? { ...prev, status: 'error', message: typeof error === 'string' ? error : 'Failed to fix document' } : null);
    } finally {
      setScanning(false);
    }
  };

  const getProgressIcon = useCallback(() => {
    if (!aiProgress) return null;
    switch (aiProgress.status) {
      case 'done': return <CheckCircle className="text-accent-lime" size={16} />;
      case 'error': return <AlertTriangle className="text-red-400" size={16} />;
      default: return <Activity className="text-accent-cyan animate-pulse" size={16} />;
    }
  }, [aiProgress]);

  const getProgressBarColor = useCallback(() => {
    if (!aiProgress) return 'bg-accent-cyan';
    switch (aiProgress.status) {
      case 'done': return 'bg-accent-lime';
      case 'error': return 'bg-red-400';
      default: return 'bg-accent-cyan';
    }
  }, [aiProgress]);

  return (
    <div className="h-full flex">
      {/* Document Viewer (Left) */}
      <div className="flex-1 blueprint-grid overflow-y-auto relative">
        {isScanning && !aiProgress && (
          <div className="absolute inset-0 bg-bg-primary/80 z-50 flex items-center justify-center backdrop-blur-sm">
             <div className="text-center">
               <Activity className="w-12 h-12 text-accent-cyan mx-auto mb-4 animate-pulse" />
               <p className="text-display text-accent-cyan text-xl">SCANNING DOCUMENT...</p>
             </div>
             <div className="scan-line" />
          </div>
        )}
        
        <div className="relative z-10 p-8">
          <div className="mb-8">
            <p className="phase-label text-sm mb-2">[02] TYPO SCANNER</p>
            <h1 className="text-display text-4xl text-text-primary">
              SPELLCHECK
            </h1>
          </div>

          {!currentDocument ? (
            <div 
              onClick={handleSelectFile}
              className="corner-marks corner-marks-cyan border-2 border-dashed border-border-subtle hover:border-accent-cyan/50 transition-colors duration-300 p-16 flex flex-col items-center justify-center gap-4 cursor-pointer group bg-surface-card"
            >
              <Upload
                size={48}
                className="text-text-dim group-hover:text-accent-cyan transition-colors duration-300"
              />
              <p className="text-display text-xl text-text-muted group-hover:text-text-primary transition-colors">
                DROP A DOCUMENT TO SCAN
              </p>
              <p className="text-label text-[0.6rem] text-text-dim">
                .DOCX OR .XLSX FILES SUPPORTED (CLICK TO SELECT)
              </p>
            </div>
          ) : (
            <div className="corner-marks border border-border-subtle bg-surface-card p-6">
              <div className="flex items-center gap-4 mb-6 pb-4 border-b border-border-subtle">
                <FileText className="text-accent-cyan" size={32} />
                <div>
                  <h2 className="text-display text-xl text-text-primary">{currentDocument.name}</h2>
                  <p className="text-label text-xs text-text-muted mt-1">
                    {(currentDocument.size / 1024).toFixed(2)} KB • {currentDocument.word_count || 0} WORDS
                  </p>
                </div>
                <button onClick={() => {
                  clear();
                  setAiProgress(null);
                  setFixedFilePath(null);
                  setFixedFindings([]);
                  setSearchQuery("");
                }} className="ml-auto btn-ghost text-xs py-1 px-3">
                  CLOSE
                </button>
              </div>
              
              <div className="space-y-6">
                 <div className="bg-bg-primary/50 border border-border-subtle p-6 flex flex-col items-center justify-center min-h-[200px]">
                    <p className="text-display text-2xl text-text-muted mb-2">DOCUMENT LOADED</p>
                    <p className="text-label text-text-dim text-center max-w-sm mb-6">
                      Choose a fix mode on the right panel, then click the fix button to start.
                    </p>
                    
                    {findings.length === 0 ? (
                      <button 
                        onClick={runOfflineScan}
                        disabled={isScanning}
                        className="btn-ghost flex items-center justify-center gap-2 py-3 px-6"
                      >
                        <FileText size={16} />
                        RUN OFFLINE SCAN (OPTIONAL)
                      </button>
                    ) : (
                      <p className="text-label text-accent-cyan">Offline scan completed. Found {findings.length} issues.</p>
                    )}
                 </div>

                 {/* === AI Progress Panel === */}
                 {aiProgress && (
                   <div className="border border-border-subtle bg-bg-primary/80 p-5 space-y-4 relative overflow-hidden">
                     {(aiProgress.status === 'processing' || aiProgress.status === 'pre-filtering' || aiProgress.status === 'auto-fixing') && (
                       <div className="absolute inset-0 pointer-events-none">
                         <div 
                           className="absolute left-0 right-0 h-px bg-accent-cyan/30"
                           style={{ animation: 'scan-sweep 2s ease-in-out infinite' }}
                         />
                       </div>
                     )}
                     
                     {/* Header */}
                     <div className="flex items-center justify-between">
                       <div className="flex items-center gap-2">
                         {getProgressIcon()}
                         <span className="text-label text-text-muted">
                           {aiProgress.status === 'done' ? 'PROCESSING COMPLETE' : 
                            aiProgress.status === 'error' ? 'PROCESSING ERROR' :
                            aiProgress.status === 'exporting' ? 'EXPORTING DOCUMENT...' :
                            aiProgress.status === 'extracting' ? 'EXTRACTING TEXT...' :
                            aiProgress.status === 'auto-fixing' ? 'STAGE 1: AUTO-FIX' :
                            aiProgress.status === 'pre-filtering' ? 'STAGE 2: PRE-FILTER' :
                            aiProgress.status === 'pre-filtered' ? 'STAGE 2: PRE-FILTER DONE' :
                            aiProgress.status === 'processing' ? 'STAGE 3: AI PROCESSING' :
                            'INITIALIZING...'}
                         </span>
                       </div>
                       {aiProgress.total > 0 && (
                         <span className="text-label text-accent-cyan text-sm">
                           {aiProgress.current}/{aiProgress.total}
                         </span>
                       )}
                     </div>

                     {/* Progress Bar */}
                     {aiProgress.total > 0 && (
                       <div className="space-y-2">
                         <div className="w-full h-2 bg-bg-primary border border-border-subtle overflow-hidden">
                           <div 
                             className={`h-full transition-all duration-500 ease-out ${getProgressBarColor()}`}
                             style={{ width: `${aiProgress.percentage}%` }}
                           />
                         </div>
                         <div className="flex items-center justify-between">
                           <span className="text-[0.65rem] text-text-dim">
                             {aiProgress.percentage}%
                           </span>
                           <span className="text-[0.65rem] text-accent-lime">
                             {aiProgress.fixedCount} FIXES
                           </span>
                         </div>
                       </div>
                     )}

                     {/* Status Message */}
                     <p className="text-xs text-text-dim leading-relaxed">
                       {aiProgress.message}
                     </p>

                     {/* Actions when done or error */}
                     {aiProgress.status === 'done' && (
                       <div className="space-y-2 mt-4">
                         <button
                           onClick={async () => {
                             if (!fixedFilePath) return;
                             try {
                               const savePath = await save({
                                 defaultPath: fixedFilePath,
                                 filters: [{ name: 'Word Document', extensions: ['docx'] }]
                               });
                               if (savePath) {
                                 await invoke('save_file_copy', { sourcePath: fixedFilePath, destinationPath: savePath });
                                 toast.success('File successfully downloaded!', { id: 'download-toast' });
                                 setAiProgress(null);
                               }
                             } catch (err) {
                               console.error('Download error:', err);
                               toast.error('Failed to save the file.', { id: 'download-toast' });
                             }
                           }}
                           className="w-full py-3 bg-accent-cyan text-bg-primary font-bold hover:bg-white text-xs tracking-widest transition-colors corner-marks flex items-center justify-center gap-2"
                         >
                           <FileText size={16} />
                           DOWNLOAD FIXED DOCUMENT
                         </button>
                         <button
                           onClick={() => setAiProgress(null)}
                           className="w-full py-2 border border-border-subtle hover:bg-surface-hover text-text-muted hover:text-text-primary text-xs tracking-widest transition-colors corner-marks"
                         >
                           DISMISS
                         </button>
                       </div>
                     )}

                     {aiProgress.status === 'error' && (
                       <button
                         onClick={() => setAiProgress(null)}
                         className="w-full mt-4 py-2 border border-border-subtle hover:bg-surface-hover text-text-muted hover:text-text-primary text-xs tracking-widest transition-colors corner-marks"
                       >
                         CLOSE PANEL
                       </button>
                     )}
                   </div>
                 )}

                 {/* === Changes List === */}
                 {fixedFindings.length > 0 && aiProgress?.status === 'done' && (() => {
                   // Filter out items where only whitespace changed (often confusing for users)
                   const visibleFindings = fixedFindings.filter(f => {
                     if (f.source === 'ai') return true;
                     const origClean = f.original.replace(/\s+/g, '').toLowerCase();
                     const suggClean = f.suggestion.replace(/\s+/g, '').toLowerCase();
                     return origClean !== suggClean;
                   });
                   
                   const filteredFindings = visibleFindings.filter(f => 
                     f.original.toLowerCase().includes(searchQuery.toLowerCase()) || 
                     f.suggestion.toLowerCase().includes(searchQuery.toLowerCase())
                   );

                   return (
                   <div className="mt-6 border border-border-subtle bg-bg-primary/50 p-5">
                     <div className="flex flex-wrap gap-4 items-center justify-between mb-4">
                       <h3 className="text-display text-lg text-accent-cyan flex items-center gap-2">
                         <CheckCircle size={18} />
                         CHANGES APPLIED ({filteredFindings.length}/{visibleFindings.length})
                       </h3>
                       <div className="relative">
                         <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-dim" />
                         <input 
                           type="text" 
                           value={searchQuery}
                           onChange={(e) => setSearchQuery(e.target.value)}
                           placeholder="Search changes..." 
                           className="bg-bg-primary border border-border-subtle pl-8 pr-3 py-1.5 text-xs text-text-primary focus:outline-none focus:border-accent-cyan transition-colors corner-marks w-64"
                         />
                       </div>
                     </div>
                     <div className="max-h-[350px] overflow-y-auto space-y-3 pr-2 scrollbar-blueprint">
                       {filteredFindings.map((f, i) => (
                         <div key={i} className="bg-bg-primary p-4 border border-border-subtle corner-marks">
                           <div className="flex items-center gap-2 mb-3">
                             <span className={`text-[0.6rem] px-2 py-0.5 border ${f.source === 'ai' ? 'bg-accent-cyan/10 text-accent-cyan border-accent-cyan/30' : 'bg-accent-lime/10 text-accent-lime border-accent-lime/30'}`}>
                               {f.source === 'ai' ? 'AI REVISION' : 'AUTO FIX'}
                             </span>
                             <span className="text-xs text-text-muted">Paragraf {f.position.paragraph}</span>
                           </div>
                           <div className="grid grid-cols-2 gap-4 text-sm">
                             <div className="p-3 bg-red-500/5 border border-red-500/20 text-red-400/80 line-through text-xs leading-relaxed max-h-32 overflow-y-auto whitespace-pre-wrap">
                               {f.original}
                             </div>
                             <div className="p-3 bg-accent-lime/5 border border-accent-lime/20 text-accent-lime text-xs leading-relaxed max-h-32 overflow-y-auto whitespace-pre-wrap">
                               {f.suggestion}
                             </div>
                           </div>
                         </div>
                       ))}
                       {filteredFindings.length === 0 && (
                         <div className="text-center text-text-dim text-xs py-8">
                           {visibleFindings.length === 0 
                             ? "Only whitespace/invisible fixes were applied." 
                             : `No changes match your search "${searchQuery}"`}
                         </div>
                       )}
                     </div>
                   </div>
                   );
                 })()}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Review Panel (Right) - AI Settings & Summary */}
      <div className="w-[360px] h-full border-l border-border-subtle bg-bg-secondary flex flex-col">
        <div className="px-5 py-4 border-b border-border-subtle bg-bg-primary/50 flex items-center gap-2">
          <Settings size={14} className="text-text-muted" />
          <span className="text-label text-text-muted">CONFIGURATION</span>
        </div>
        
        <div className="flex-1 overflow-y-auto p-5 space-y-6">
          {/* Fix Mode Selector */}
          <div className="space-y-3">
            <p className="text-label text-text-dim">FIX MODE</p>
            <div className="space-y-2">
              {FIX_MODES.map((mode) => {
                const Icon = mode.icon;
                const isActive = fixMode === mode.id;
                return (
                  <button
                    key={mode.id}
                    onClick={() => handleFixModeChange(mode.id)}
                    disabled={isScanning}
                    className={`w-full flex items-center gap-3 p-3 border transition-all duration-300 text-left group
                      ${isActive 
                        ? `${mode.borderColor} bg-surface-hover` 
                        : 'border-border-subtle bg-transparent hover:border-border-hover hover:bg-surface-card'
                      }`}
                  >
                    <Icon 
                      size={18} 
                      className={`${isActive ? mode.color : 'text-text-dim group-hover:text-text-muted'} transition-colors`} 
                    />
                    <div className="flex-1 min-w-0">
                      <p className={`text-label text-xs ${isActive ? mode.color : 'text-text-primary'}`}>
                        {mode.label}
                      </p>
                      <p className="text-[0.6rem] text-text-dim mt-0.5 leading-tight">
                        {mode.desc}
                      </p>
                    </div>
                    {isActive && (
                      <div className={`w-2 h-2 rounded-full ${mode.color.replace('text-', 'bg-')}`} />
                    )}
                  </button>
                );
              })}
            </div>
          </div>

          {/* API Config - only show for modes that need API */}
          {selectedMode.needsApi && (
            <div className="space-y-4 pt-4 border-t border-border-subtle">
              <p className="text-label text-text-dim">AI PROVIDER</p>
              <div className="space-y-3">
                <select 
                  value={providerId}
                  onChange={handleProviderSelect}
                  className="w-full bg-bg-primary border border-border-subtle p-2 text-sm text-text-primary focus:outline-none focus:border-accent-cyan transition-colors cursor-pointer"
                >
                  {PRESET_PROVIDERS.map(p => (
                    <option key={p.id} value={p.id}>{p.name}</option>
                  ))}
                </select>

                {selectedProvider.id === 'custom' && (
                  <div className="space-y-1">
                    <label className="text-[0.6rem] text-text-dim block">BASE URL</label>
                    <input 
                      type="text" 
                      value={baseUrl}
                      onChange={(e) => handleConfigChange('ai_base_url', e.target.value)}
                      placeholder="https://api.openai.com/v1"
                      className="w-full bg-bg-primary border border-border-subtle p-2 text-sm text-text-primary focus:outline-none focus:border-accent-cyan transition-colors"
                    />
                  </div>
                )}

                <div className="space-y-1">
                  <label className="text-[0.6rem] text-text-dim block">MODEL</label>
                  {selectedProvider.id !== 'custom' ? (
                    <select 
                      value={model}
                      onChange={(e) => handleConfigChange('ai_model', e.target.value)}
                      className="w-full bg-bg-primary border border-border-subtle p-2 text-sm text-text-primary focus:outline-none focus:border-accent-cyan transition-colors cursor-pointer"
                    >
                      {selectedProvider.models.map(m => (
                        <option key={m} value={m}>{m}</option>
                      ))}
                    </select>
                  ) : (
                    <input 
                      type="text" 
                      value={model}
                      onChange={(e) => handleConfigChange('ai_model', e.target.value)}
                      placeholder="custom-model-name"
                      className="w-full bg-bg-primary border border-border-subtle p-2 text-sm text-text-primary focus:outline-none focus:border-accent-cyan transition-colors"
                    />
                  )}
                </div>

                <p className="text-[0.6rem] text-text-dim leading-relaxed">
                  API Key untuk {selectedProvider.name} bisa dikonfigurasi di halaman <span className="text-accent-cyan">SETTINGS</span>.
                </p>
              </div>
            </div>
          )}

          {/* Document Summary + Action Button */}
          {currentDocument && (
            <div className="pt-4 border-t border-border-subtle space-y-4">
              <p className="text-label text-text-dim">DOCUMENT SUMMARY</p>
              
              <div className="grid grid-cols-2 gap-2">
                <div className="bg-bg-primary border border-border-subtle p-3 corner-marks">
                  <p className="text-[0.6rem] text-text-dim mb-1">WORDS</p>
                  <p className="text-display text-xl text-accent-cyan">{currentDocument.word_count}</p>
                </div>
                <div className="bg-bg-primary border border-border-subtle p-3 corner-marks">
                  <p className="text-[0.6rem] text-text-dim mb-1">PARAGRAPHS</p>
                  <p className="text-display text-xl text-accent-lime">{currentDocument.paragraph_count}</p>
                </div>
                {findings.length > 0 && (
                  <div className="bg-bg-primary border border-border-subtle p-3 col-span-2 corner-marks">
                    <p className="text-[0.6rem] text-text-dim mb-1">OFFLINE SCAN ISSUES</p>
                    <p className="text-display text-xl text-red-400">{findings.length}</p>
                  </div>
                )}
              </div>

              {/* Mode description card */}
              <div className={`bg-bg-primary/50 border p-3 text-[0.65rem] text-text-dim leading-relaxed ${selectedMode.borderColor.replace('border-', 'border-').replace('border-', 'border-')}`}>
                {fixMode === 'quick' && (
                  <>
                    <span className="text-accent-lime font-bold">QUICK FIX</span> — Fixes double spaces and whitespace issues using regex. No AI needed, instant results. No tokens consumed.
                  </>
                )}
                {fixMode === 'smart' && (
                  <>
                    <span className="text-accent-cyan font-bold">SMART FIX</span> — Auto-fixes simple issues first, then uses dictionary to identify paragraphs with spelling errors. Only those paragraphs are sent to AI. Saves ~85-90% tokens.
                  </>
                )}
                {fixMode === 'deep' && (
                  <>
                    <span className="text-purple-400 font-bold">DEEP FIX</span> — Sends all paragraphs to AI for comprehensive analysis. Best accuracy but uses the most tokens.
                  </>
                )}
              </div>

              <button 
                onClick={handleAIFix}
                disabled={isScanning}
                className={`w-full flex items-center justify-center gap-2 py-4 font-mono text-xs font-medium tracking-widest uppercase transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed
                  ${fixMode === 'quick' 
                    ? 'bg-accent-lime text-bg-primary hover:bg-white' 
                    : fixMode === 'deep'
                    ? 'bg-purple-500 text-white hover:bg-purple-400'
                    : 'bg-accent-cyan text-bg-primary hover:bg-white'
                  }`}
              >
                {isScanning ? (
                  <>
                    <Activity size={16} className="animate-pulse" />
                    PROCESSING...
                  </>
                ) : (
                  <>
                    <Zap size={16} />
                    {selectedMode.label}
                  </>
                )}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
