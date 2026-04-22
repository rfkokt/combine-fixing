import { useState, useEffect } from "react";
import { Upload, Trash2, GripVertical, FileText, CheckCircle, FileOutput, ChevronDown } from "lucide-react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DndContext, closestCenter, KeyboardSensor, PointerSensor, useSensor, useSensors, DragEndEvent } from "@dnd-kit/core";
import { arrayMove, SortableContext, sortableKeyboardCoordinates, useSortable, verticalListSortingStrategy } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { toast } from "sonner";

interface DocumentPreview {
  path: string;
  name: string;
  size: number;
  preview_text: string;
  word_count: number;
  paragraph_count: number;
}

interface MergeResult {
  output_path: string;
  total_paragraphs: number;
  total_words: number;
  documents_merged: number;
}

interface MergeProgress {
  current: number;
  total: number;
  percentage: number;
  status: string;
  message: string;
}

const SEPARATORS = [
  { id: "page_break", label: "PAGE BREAK", desc: "Add page break between documents" },
  { id: "section_break", label: "SECTION BREAK", desc: "Add section break with new page settings" },
  { id: "double_page_break", label: "DOUBLE PAGE BREAK", desc: "Add two page breaks for clear separation" },
  { id: "none", label: "NO SEPARATOR", desc: "Documents will be merged directly" },
] as const;

interface SortableDocumentProps {
  doc: DocumentPreview;
  index: number;
  onRemove: (path: string) => void;
  onPreview: (path: string) => void;
}

function SortableDocument({ doc, index, onRemove, onPreview }: SortableDocumentProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: doc.path });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.5 : 1,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`flex items-center gap-4 p-4 bg-surface-card border border-border-subtle corner-marks group transition-all duration-200 hover:border-border-hover ${isDragging ? 'z-50 shadow-lg' : ''}`}
    >
      <div
        {...attributes}
        {...listeners}
        className="cursor-grab active:cursor-grabbing p-1 hover:bg-surface-hover"
      >
        <GripVertical size={18} className="text-text-dim group-hover:text-text-muted" />
      </div>

      <div className="flex items-center justify-center w-8 h-8 bg-bg-primary border border-border-subtle">
        <span className="text-label text-xs text-text-dim">{String(index + 1).padStart(2, '0')}</span>
      </div>

      <div className="flex-1 min-w-0 cursor-pointer" onClick={() => onPreview(doc.path)}>
        <p className="text-label text-sm text-text-primary truncate">{doc.name}</p>
        <p className="text-[0.65rem] text-text-dim mt-0.5">
          {(doc.size / 1024).toFixed(1)} KB • {doc.word_count.toLocaleString()} words
        </p>
        {doc.preview_text && (
          <p className="text-[0.6rem] text-text-dim mt-1 truncate opacity-60">
            {doc.preview_text.split('\n')[0]}
          </p>
        )}
      </div>

      <button
        onClick={() => onRemove(doc.path)}
        className="p-2 text-text-dim hover:text-red-400 hover:bg-red-500/10 transition-colors"
      >
        <Trash2 size={16} />
      </button>
    </div>
  );
}

export function MergePage() {
  const [documents, setDocuments] = useState<DocumentPreview[]>([]);
  const [isMerging, setIsMerging] = useState(false);
  const [mergeProgress, setMergeProgress] = useState<MergeProgress | null>(null);
  const [selectedSeparator, setSelectedSeparator] = useState<string>("page_break");
  const [showSeparatorMenu, setShowSeparatorMenu] = useState(false);
  const [previewDoc, setPreviewDoc] = useState<DocumentPreview | null>(null);
  const [mergeResult, setMergeResult] = useState<MergeResult | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  useEffect(() => {
    const unlistenRef = { current: (() => {}) };
    listen<MergeProgress>("merge-progress", (event) => {
      setMergeProgress(event.payload);
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    });
    return () => unlistenRef.current();
  }, []);

  const handleSelectFiles = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Word Documents',
          extensions: ['docx']
        }]
      });

      if (selected && Array.isArray(selected)) {
        const newDocs = await Promise.all(
          selected.map(async (path) => {
            try {
              const preview = await invoke<DocumentPreview>('get_document_preview', { filePath: path });
              return preview;
            } catch (e) {
              console.error('Failed to load preview:', e);
              return null;
            }
          })
        );

        const validDocs = newDocs.filter(Boolean) as DocumentPreview[];
        setDocuments(prev => {
          const existing = new Set(prev.map(d => d.path));
          const unique = validDocs.filter(d => !existing.has(d.path));
          return [...prev, ...unique];
        });
        toast.success(`Added ${validDocs.length} document(s)`);
      }
    } catch (error) {
      toast.error('Failed to select files');
      console.error(error);
    }
  };

  const handleRemoveDocument = (path: string) => {
    setDocuments(prev => prev.filter(d => d.path !== path));
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (over && active.id !== over.id) {
      setDocuments((items) => {
        const oldIndex = items.findIndex(d => d.path === active.id);
        const newIndex = items.findIndex(d => d.path === over.id);
        return arrayMove(items, oldIndex, newIndex);
      });
    }
  };

  const handlePreview = async (path: string) => {
    try {
      const preview = await invoke<DocumentPreview>('get_document_preview', { filePath: path });
      setPreviewDoc(preview);
    } catch (error) {
      toast.error('Failed to load preview');
    }
  };

  const handleMerge = async () => {
    if (documents.length === 0) {
      toast.error('No documents to merge');
      return;
    }

    try {
      const outputPath = await save({
        defaultPath: 'merged_document.docx',
        filters: [{ name: 'Word Document', extensions: ['docx'] }]
      });

      if (!outputPath) return;

      setIsMerging(true);
      setMergeResult(null);
      setMergeProgress({ current: 0, total: documents.length, percentage: 0, status: 'preparing', message: 'Preparing to merge...' });

      const inputPaths = documents.map(d => d.path);
      const result = await invoke<MergeResult>('merge_documents', {
        inputPaths,
        outputPath,
        separator: selectedSeparator
      });

      setMergeResult(result);
      setMergeProgress({ current: documents.length, total: documents.length, percentage: 100, status: 'done', message: 'Merge complete!' });
      toast.success(`Merged ${result.documents_merged} documents successfully!`);
    } catch (error) {
      console.error('Merge error:', error);
      toast.error(typeof error === 'string' ? error : 'Failed to merge documents');
      setMergeProgress(prev => prev ? { ...prev, status: 'error', message: String(error) } : null);
    } finally {
      setIsMerging(false);
    }
  };

  const selectedSep = SEPARATORS.find(s => s.id === selectedSeparator) || SEPARATORS[0];

  return (
    <div className="h-full flex">
      <div className="flex-1 blueprint-grid overflow-y-auto relative">
        <div className="relative z-10 p-8">
          <div className="mb-8">
            <p className="phase-label text-sm mb-2">[01] DOCUMENT MERGE</p>
            <h1 className="text-display text-4xl text-text-primary">MERGE DOCUMENTS</h1>
          </div>

          {/* Dropzone */}
          {!documents.length && (
            <div
              onClick={handleSelectFiles}
              className="corner-marks corner-marks-cyan border-2 border-dashed border-border-subtle hover:border-accent-cyan/50 transition-colors duration-300 p-16 flex flex-col items-center justify-center gap-4 cursor-pointer group bg-surface-card"
            >
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
          )}

          {/* Document List */}
          {documents.length > 0 && (
            <div className="space-y-6">
              {/* Add More Button */}
              <button
                onClick={handleSelectFiles}
                className="w-full p-4 border border-dashed border-border-subtle hover:border-accent-cyan/50 transition-colors duration-300 flex items-center justify-center gap-3 group"
              >
                <Upload size={20} className="text-text-dim group-hover:text-accent-cyan transition-colors" />
                <span className="text-label text-sm text-text-muted group-hover:text-text-primary transition-colors">
                  ADD MORE DOCUMENTS
                </span>
              </button>

              {/* Sortable List */}
              <div className="space-y-2">
                <p className="text-label text-xs text-text-dim mb-3">
                  DRAG TO REORDER ({documents.length} DOCUMENT(S))
                </p>
                <DndContext
                  sensors={sensors}
                  collisionDetection={closestCenter}
                  onDragEnd={handleDragEnd}
                >
                  <SortableContext
                    items={documents.map(d => d.path)}
                    strategy={verticalListSortingStrategy}
                  >
                    {documents.map((doc, index) => (
                      <SortableDocument
                        key={doc.path}
                        doc={doc}
                        index={index}
                        onRemove={handleRemoveDocument}
                        onPreview={handlePreview}
                      />
                    ))}
                  </SortableContext>
                </DndContext>
              </div>

              {/* Merge Configuration */}
              <div className="border border-border-subtle p-6 corner-marks">
                <p className="text-label text-xs text-text-dim mb-4">SEPARATOR TYPE</p>
                <div className="relative">
                  <button
                    onClick={() => setShowSeparatorMenu(!showSeparatorMenu)}
                    className="w-full flex items-center justify-between p-3 bg-bg-primary border border-border-subtle hover:border-border-hover transition-colors"
                  >
                    <div className="flex items-center gap-3">
                      <span className="text-label text-sm text-accent-cyan">{selectedSep.label}</span>
                      <span className="text-[0.65rem] text-text-dim">{selectedSep.desc}</span>
                    </div>
                    <ChevronDown size={16} className="text-text-dim" />
                  </button>

                  {showSeparatorMenu && (
                    <div className="absolute top-full left-0 right-0 mt-1 bg-bg-primary border border-border-subtle z-50">
                      {SEPARATORS.map(sep => (
                        <button
                          key={sep.id}
                          onClick={() => {
                            setSelectedSeparator(sep.id);
                            setShowSeparatorMenu(false);
                          }}
                          className={`w-full p-3 text-left hover:bg-surface-hover transition-colors flex items-center gap-3 ${selectedSeparator === sep.id ? 'bg-surface-card' : ''}`}
                        >
                          <span className={`text-label text-xs ${selectedSeparator === sep.id ? 'text-accent-cyan' : 'text-text-muted'}`}>
                            {sep.label}
                          </span>
                          <span className="text-[0.6rem] text-text-dim">{sep.desc}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Progress */}
              {mergeProgress && (
                <div className="border border-border-subtle bg-bg-primary/50 p-5 corner-marks">
                  <div className="flex items-center justify-between mb-3">
                    <span className="text-label text-xs text-text-muted">
                      {mergeProgress.status === 'done' ? 'MERGE COMPLETE' :
                       mergeProgress.status === 'error' ? 'MERGE FAILED' :
                       'MERGING...'}
                    </span>
                    <span className="text-label text-accent-cyan text-sm">
                      {mergeProgress.percentage}%
                    </span>
                  </div>

                  <div className="w-full h-2 bg-bg-primary border border-border-subtle overflow-hidden mb-3">
                    <div
                      className={`h-full transition-all duration-500 ease-out ${mergeProgress.status === 'done' ? 'bg-accent-lime' : mergeProgress.status === 'error' ? 'bg-red-400' : 'bg-accent-cyan'}`}
                      style={{ width: `${mergeProgress.percentage}%` }}
                    />
                  </div>

                  <p className="text-[0.65rem] text-text-dim">{mergeProgress.message}</p>
                </div>
              )}

              {/* Result */}
              {mergeResult && (
                <div className="border border-accent-lime/30 bg-accent-lime/5 p-5 corner-marks">
                  <div className="flex items-center gap-2 mb-3">
                    <CheckCircle size={18} className="text-accent-lime" />
                    <span className="text-label text-accent-lime">MERGE SUCCESSFUL</span>
                  </div>
                  <div className="grid grid-cols-3 gap-3">
                    <div className="bg-bg-primary p-3 border border-border-subtle">
                      <p className="text-[0.6rem] text-text-dim">DOCUMENTS</p>
                      <p className="text-display text-xl text-accent-cyan">{mergeResult.documents_merged}</p>
                    </div>
                    <div className="bg-bg-primary p-3 border border-border-subtle">
                      <p className="text-[0.6rem] text-text-dim">PARAGRAPHS</p>
                      <p className="text-display text-xl text-accent-lime">{mergeResult.total_paragraphs.toLocaleString()}</p>
                    </div>
                    <div className="bg-bg-primary p-3 border border-border-subtle">
                      <p className="text-[0.6rem] text-text-dim">WORDS</p>
                      <p className="text-display text-xl text-text-primary">{mergeResult.total_words.toLocaleString()}</p>
                    </div>
                  </div>
                </div>
              )}

              {/* Merge Button */}
              <button
                onClick={handleMerge}
                disabled={isMerging || documents.length === 0}
                className={`w-full py-4 flex items-center justify-center gap-3 font-mono text-xs font-medium tracking-widest uppercase transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed
                  ${isMerging ? 'bg-accent-cyan/50 text-bg-primary' : 'bg-accent-cyan text-bg-primary hover:bg-white'}`}
              >
                {isMerging ? (
                  <>
                    <div className="w-4 h-4 border-2 border-bg-primary/30 border-t-bg-primary animate-spin" />
                    PROCESSING...
                  </>
                ) : (
                  <>
                    <FileOutput size={18} />
                    MERGE {documents.length} DOCUMENT{documents.length > 1 ? 'S' : ''}
                  </>
                )}
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Preview Panel */}
      {previewDoc && (
        <div className="w-[400px] h-full border-l border-border-subtle bg-bg-secondary flex flex-col">
          <div className="px-5 py-4 border-b border-border-subtle bg-bg-primary/50 flex items-center gap-2">
            <FileText size={14} className="text-text-muted" />
            <span className="text-label text-text-muted">DOCUMENT PREVIEW</span>
            <button
              onClick={() => setPreviewDoc(null)}
              className="ml-auto text-text-dim hover:text-text-primary"
            >
              ×
            </button>
          </div>

          <div className="flex-1 overflow-y-auto p-5">
            <div className="mb-4">
              <h3 className="text-label text-sm text-text-primary mb-1">{previewDoc.name}</h3>
              <p className="text-[0.65rem] text-text-dim">
                {(previewDoc.size / 1024).toFixed(1)} KB • {previewDoc.word_count.toLocaleString()} words • {previewDoc.paragraph_count} paragraphs
              </p>
            </div>

            <div className="border-t border-border-subtle pt-4">
              <p className="text-label text-[0.6rem] text-text-dim mb-2">FIRST 5 PARAGRAPHS</p>
              <div className="space-y-3">
                {previewDoc.preview_text.split('\n').filter(p => p.trim()).map((line, i) => (
                  <p key={i} className="text-body text-sm text-text-muted leading-relaxed">
                    {line}
                  </p>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}