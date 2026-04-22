import { create } from 'zustand';

export type TypoSeverity = 'error' | 'warning' | 'info';
export type TypoStatus = 'pending' | 'accepted' | 'rejected' | 'ignored';

export interface TypoPosition {
  paragraph: number;
  start: number;
  end: number;
}

export interface TypoFinding {
  id: string;
  original: string;
  suggestion: string;
  context: string;
  severity: TypoSeverity;
  position: TypoPosition;
  source: 'dictionary' | 'rules' | 'ai';
  status: TypoStatus;
}

export interface DocumentInfo {
  path: string;
  name: string;
  size: number;
  word_count?: number;
  paragraph_count?: number;
}

export interface ScanResult {
  document: DocumentInfo;
  findings: TypoFinding[];
  total_words: number;
  scan_duration_ms: number;
}

interface SpellcheckState {
  currentDocument: DocumentInfo | null;
  findings: TypoFinding[];
  isScanning: boolean;
  setScanning: (status: boolean) => void;
  setScanResult: (result: ScanResult) => void;
  setCurrentDocument: (doc: DocumentInfo) => void;
  updateFindingStatus: (id: string, status: TypoStatus) => void;
  clear: () => void;
}

export const useSpellcheckStore = create<SpellcheckState>((set) => ({
  currentDocument: null,
  findings: [],
  isScanning: false,
  setScanning: (status) => set({ isScanning: status }),
  setScanResult: (result) => set({ 
    currentDocument: result.document,
    findings: result.findings 
  }),
  setCurrentDocument: (doc) => set({ currentDocument: doc }),
  updateFindingStatus: (id, status) => set((state) => ({
    findings: state.findings.map(f => f.id === id ? { ...f, status } : f)
  })),
  clear: () => set({ currentDocument: null, findings: [], isScanning: false })
}));
