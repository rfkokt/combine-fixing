/** Typo severity levels */
export type TypoSeverity = "error" | "warning" | "info";

/** A single detected typo finding */
export interface TypoFinding {
  id: string;
  original: string;
  suggestion: string;
  context: string;
  severity: TypoSeverity;
  position: {
    paragraph: number;
    start: number;
    end: number;
  };
  source: "dictionary" | "rules" | "ai";
  status: "pending" | "accepted" | "rejected" | "ignored";
}

/** Document metadata */
export interface DocumentInfo {
  path: string;
  name: string;
  size: number;
  pageCount?: number;
  wordCount?: number;
  lastModified?: string;
}

/** Spellcheck scan result */
export interface ScanResult {
  document: DocumentInfo;
  findings: TypoFinding[];
  totalWords: number;
  scanDuration: number;
}

/** Merge configuration */
export interface MergeConfig {
  separator: "page-break" | "section-break" | "none";
  preserveStyles: boolean;
}

/** Navigation pages */
export type PageId = "home" | "merge" | "spellcheck" | "settings";
