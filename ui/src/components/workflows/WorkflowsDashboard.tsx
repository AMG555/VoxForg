import React, { useState } from 'react';
import {
  Search,
  Plus,
  Copy,
  Trash2,
  Edit2,
  Check,
  Download,
  Upload,
  Layers,
  Sparkles,
  ArrowRight,
  FolderOpen,
  Mic,
  Clock,
  Tag,
  FileText,
  Radio,
  Film,
  X,
  Play,
} from 'lucide-react';
import { StoredWorkflow, WorkflowStorage } from '../../services/workflowStorage';

interface WorkflowsDashboardProps {
  onOpenWorkflow: (id: string) => void;
  onCreateWorkflow: () => void;
}

export const WorkflowsDashboard: React.FC<WorkflowsDashboardProps> = ({
  onOpenWorkflow,
  onCreateWorkflow,
}) => {
  const [workflows, setWorkflows] = useState<StoredWorkflow[]>(() => WorkflowStorage.getWorkflows());
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('All');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editNameInput, setEditNameInput] = useState('');
  const [showImportModal, setShowImportModal] = useState(false);
  const [importJsonInput, setImportJsonInput] = useState('');

  const refreshList = () => {
    setWorkflows(WorkflowStorage.getWorkflows());
  };

  const categories = ['All', 'Dialogue', 'Podcast', 'Radio', 'Video Dubbing', 'General'];

  const filteredWorkflows = workflows.filter((w) => {
    const matchesSearch =
      w.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (w.tags && w.tags.some((t) => t.toLowerCase().includes(searchQuery.toLowerCase()))) ||
      (w.description && w.description.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesCategory = selectedCategory === 'All' || w.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  const handleDuplicate = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const duplicated = WorkflowStorage.duplicateWorkflow(id);
    refreshList();
    onOpenWorkflow(duplicated.id);
  };

  const handleDelete = (id: string, name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (workflows.length <= 1) {
      alert('Cannot delete the last remaining workflow.');
      return;
    }
    if (confirm(`Delete workflow "${name}" permanently?`)) {
      WorkflowStorage.deleteWorkflow(id);
      refreshList();
    }
  };

  const startEditing = (w: StoredWorkflow, e: React.MouseEvent) => {
    e.stopPropagation();
    setEditingId(w.id);
    setEditNameInput(w.name);
  };

  const saveEditing = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (editNameInput.trim()) {
      WorkflowStorage.renameWorkflow(id, editNameInput.trim());
      refreshList();
    }
    setEditingId(null);
  };

  const handleExport = (w: StoredWorkflow, e: React.MouseEvent) => {
    e.stopPropagation();
    WorkflowStorage.exportWorkflowJson(w);
  };

  const handleImportJson = () => {
    try {
      const parsed = JSON.parse(importJsonInput.trim());
      if (parsed && Array.isArray(parsed.nodes)) {
        const freshId = `wf-${Date.now()}`;
        const imported: StoredWorkflow = {
          ...parsed,
          id: freshId,
          name: parsed.name ? `${parsed.name} (Imported)` : 'Imported Workflow',
          category: parsed.category || 'General',
          tags: parsed.tags || ['imported'],
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        };
        const list = WorkflowStorage.getWorkflows();
        list.unshift(imported);
        WorkflowStorage.saveWorkflows(list);
        WorkflowStorage.setActiveWorkflowId(freshId);
        refreshList();
        setShowImportModal(false);
        setImportJsonInput('');
        onOpenWorkflow(freshId);
      } else {
        alert('Invalid workflow JSON structure. Must contain a "nodes" array.');
      }
    } catch (e: any) {
      alert(`Failed to parse JSON: ${e.message}`);
    }
  };

  const TEMPLATES = [
    {
      id: 'template-dialogue',
      title: 'Narrative Dialogue Pipeline',
      description: 'Script ingestion, automatic multi-speaker parser, character voice allocation, neural synthesis, and master crossfade.',
      category: 'Dialogue',
      nodeCount: 6,
      icon: <Mic className="w-5 h-5 text-amber-400" />,
      action: () => {
        const found = workflows.find((w) => w.id === 'preset-narrative');
        if (found) {
          onOpenWorkflow(found.id);
        } else {
          onCreateWorkflow();
        }
      },
    },
    {
      id: 'template-podcast',
      title: 'Multi-Voice Studio Podcast',
      description: 'Host & guest conversational turn-taking, inter-speaker pauses, and high-fidelity DSP mastering filter.',
      category: 'Podcast',
      nodeCount: 6,
      icon: <Radio className="w-5 h-5 text-sky-400" />,
      action: () => {
        const found = workflows.find((w) => w.id === 'preset-podcast');
        if (found) {
          onOpenWorkflow(found.id);
        } else {
          onCreateWorkflow();
        }
      },
    },
    {
      id: 'template-video',
      title: 'Video Dubbing & Audio Mux',
      description: 'Whisper ASR transcription, diarization, neural translation, time-stretching, and FFmpeg video muxing.',
      category: 'Video Dubbing',
      nodeCount: 7,
      icon: <Film className="w-5 h-5 text-rose-400" />,
      action: () => {
        const found = workflows.find((w) => w.id === 'preset-dubbing');
        if (found) {
          onOpenWorkflow(found.id);
        } else {
          onCreateWorkflow();
        }
      },
    },
  ];

  return (
    <div className="flex-1 flex flex-col h-full bg-[#0B0E14] overflow-y-auto select-none">
      {/* Top Banner Header */}
      <div className="border-b border-[#242E3D] bg-[#0E131A] px-8 py-6">
        <div className="max-w-7xl mx-auto flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <div className="flex items-center space-x-3">
              <h1 className="text-xl font-bold text-white tracking-tight">Workflows</h1>
              <span className="px-2.5 py-0.5 rounded-full text-xs font-mono bg-amber-500/10 text-amber-400 border border-amber-500/25">
                {workflows.length} Total
              </span>
            </div>
            <p className="text-xs text-[#94A3B8] mt-1 font-mono">
              Build, automate, and orchestrate multi-engine neural speech and audio pipelines
            </p>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={() => setShowImportModal(true)}
              className="flex items-center space-x-2 px-3.5 py-2 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-white text-xs font-mono border border-[#242E3D] transition-colors"
              title="Import workflow from JSON"
            >
              <Upload className="w-3.5 h-3.5 text-[#94A3B8]" />
              <span>Import JSON</span>
            </button>

            <button
              onClick={onCreateWorkflow}
              className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs font-mono shadow-lg shadow-amber-500/20 transition-all hover:scale-[1.02] active:scale-[0.98]"
              title="Create brand new workflow canvas"
            >
              <Plus className="w-4 h-4 text-black stroke-[2.5]" />
              <span>Add workflow</span>
            </button>
          </div>
        </div>
      </div>

      <div className="max-w-7xl mx-auto w-full px-8 py-8 space-y-8 flex-1">
        {/* Quick-Start Templates Section (n8n Style) */}
        <div>
          <div className="flex items-center space-x-2 mb-4">
            <Sparkles className="w-4 h-4 text-amber-400" />
            <h2 className="text-sm font-semibold uppercase tracking-wider text-white font-mono">
              Start with a Template
            </h2>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            {TEMPLATES.map((tmpl) => (
              <div
                key={tmpl.id}
                onClick={tmpl.action}
                className="group p-4 rounded-xl bg-[#121820] border border-[#242E3D] hover:border-amber-500/60 hover:shadow-xl hover:shadow-amber-500/5 transition-all cursor-pointer flex flex-col justify-between"
              >
                <div className="space-y-2.5">
                  <div className="flex items-center justify-between">
                    <div className="p-2 rounded-lg bg-[#1A222D] border border-[#242E3D]">
                      {tmpl.icon}
                    </div>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
                      {tmpl.nodeCount} nodes
                    </span>
                  </div>
                  <h3 className="text-sm font-semibold text-white group-hover:text-amber-400 transition-colors">
                    {tmpl.title}
                  </h3>
                  <p className="text-xs text-[#94A3B8] line-clamp-2 leading-relaxed">
                    {tmpl.description}
                  </p>
                </div>

                <div className="pt-4 mt-2 flex items-center justify-between text-xs font-mono text-amber-400 group-hover:translate-x-1 transition-transform">
                  <span>Open Pipeline</span>
                  <ArrowRight className="w-3.5 h-3.5" />
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Search & Category Filter Bar */}
        <div className="space-y-4">
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
            <div className="relative flex-1 max-w-md">
              <Search className="w-4 h-4 text-[#64748B] absolute left-3.5 top-1/2 -translate-y-1/2 pointer-events-none" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search workflows by name or tag..."
                className="w-full bg-[#121820] border border-[#242E3D] rounded-xl pl-9 pr-4 py-2 text-xs text-white placeholder-[#64748B] font-mono focus:border-amber-500 focus:outline-none transition-colors"
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-[#64748B] hover:text-white"
                >
                  <X className="w-3.5 h-3.5" />
                </button>
              )}
            </div>

            <div className="flex items-center space-x-1 overflow-x-auto pb-1 sm:pb-0">
              {categories.map((cat) => (
                <button
                  key={cat}
                  onClick={() => setSelectedCategory(cat)}
                  className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-colors ${
                    selectedCategory === cat
                      ? 'bg-amber-500 text-black font-semibold shadow-sm'
                      : 'bg-[#121820] text-[#94A3B8] hover:text-white hover:bg-[#1A222D] border border-[#242E3D]'
                  }`}
                >
                  {cat}
                </button>
              ))}
            </div>
          </div>

          {/* Workflows List Table / Card Grid */}
          <div className="rounded-xl border border-[#242E3D] bg-[#121820] overflow-hidden shadow-xl">
            <div className="grid grid-cols-12 gap-4 px-5 py-3 border-b border-[#242E3D] bg-[#0E131A] text-[11px] font-mono uppercase tracking-wider text-[#64748B]">
              <div className="col-span-5 sm:col-span-4">Workflow Name</div>
              <div className="col-span-3 sm:col-span-2 hidden sm:block">Category</div>
              <div className="col-span-3 sm:col-span-2">Nodes & Structure</div>
              <div className="col-span-2 hidden md:block">Last Updated</div>
              <div className="col-span-4 sm:col-span-2 text-right">Actions</div>
            </div>

            <div className="divide-y divide-[#242E3D]/50">
              {filteredWorkflows.length === 0 ? (
                <div className="p-12 text-center text-xs font-mono text-[#64748B]">
                  No workflows match your search or filter.
                </div>
              ) : (
                filteredWorkflows.map((w) => (
                  <div
                    key={w.id}
                    onClick={() => onOpenWorkflow(w.id)}
                    className="group grid grid-cols-12 gap-4 px-5 py-3.5 items-center hover:bg-[#1A222D]/60 transition-colors cursor-pointer"
                  >
                    {/* Name & description */}
                    <div className="col-span-5 sm:col-span-4 flex items-center space-x-3 min-w-0">
                      <div className="p-2 rounded-lg bg-[#1A222D] border border-[#242E3D] group-hover:border-amber-500/40 transition-colors shrink-0">
                        <FolderOpen className="w-4 h-4 text-amber-400" />
                      </div>

                      <div className="min-w-0 flex-1">
                        {editingId === w.id ? (
                          <div
                            className="flex items-center space-x-1"
                            onClick={(e) => e.stopPropagation()}
                          >
                            <input
                              type="text"
                              value={editNameInput}
                              onChange={(e) => setEditNameInput(e.target.value)}
                              onKeyDown={(e) => {
                                if (e.key === 'Enter') saveEditing(w.id, e as any);
                                if (e.key === 'Escape') setEditingId(null);
                              }}
                              autoFocus
                              className="bg-[#0B0E14] border border-amber-500 rounded px-2 py-0.5 text-xs text-white font-mono focus:outline-none w-full"
                            />
                            <button
                              onClick={(e) => saveEditing(w.id, e)}
                              className="p-1 text-emerald-400 hover:text-white"
                            >
                              <Check className="w-3.5 h-3.5" />
                            </button>
                          </div>
                        ) : (
                          <div className="flex items-center space-x-1.5">
                            <span className="text-xs font-mono font-semibold text-white group-hover:text-amber-400 transition-colors truncate">
                              {w.name}
                            </span>
                            <button
                              onClick={(e) => startEditing(w, e)}
                              className="p-0.5 rounded text-[#64748B] hover:text-white opacity-0 group-hover:opacity-100 transition-opacity"
                              title="Rename workflow"
                            >
                              <Edit2 className="w-3 h-3" />
                            </button>
                          </div>
                        )}
                        <p className="text-[11px] text-[#64748B] truncate mt-0.5 font-mono">
                          {w.description || `${w.nodes.length} stage audio processing network`}
                        </p>
                      </div>
                    </div>

                    {/* Category & tags */}
                    <div className="col-span-3 sm:col-span-2 hidden sm:flex items-center space-x-1.5 flex-wrap">
                      <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-sky-950/60 text-sky-400 border border-sky-800/40">
                        {w.category || 'General'}
                      </span>
                    </div>

                    {/* Nodes & structure */}
                    <div className="col-span-3 sm:col-span-2 flex items-center space-x-2 text-xs font-mono text-[#94A3B8]">
                      <span className="px-2 py-0.5 rounded bg-[#1A222D] text-amber-300 font-semibold border border-[#242E3D]">
                        {w.nodes.length} nodes
                      </span>
                      <span className="text-[#64748B] text-[11px]">
                        {w.edges.length} wires
                      </span>
                    </div>

                    {/* Last updated */}
                    <div className="col-span-2 hidden md:flex items-center space-x-1 text-xs font-mono text-[#64748B]">
                      <Clock className="w-3 h-3" />
                      <span>{new Date(w.updated_at || w.created_at).toLocaleDateString()}</span>
                    </div>

                    {/* Actions */}
                    <div className="col-span-4 sm:col-span-2 flex items-center justify-end space-x-1">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onOpenWorkflow(w.id);
                        }}
                        className="px-2.5 py-1 rounded bg-[#1A222D] hover:bg-amber-500 hover:text-black text-[#94A3B8] text-[11px] font-mono transition-colors"
                        title="Open DAG Studio"
                      >
                        Open
                      </button>

                      <button
                        onClick={(e) => handleDuplicate(w.id, e)}
                        className="p-1.5 rounded hover:bg-[#1A222D] text-[#64748B] hover:text-amber-400 transition-colors"
                        title="Duplicate workflow"
                      >
                        <Copy className="w-3.5 h-3.5" />
                      </button>

                      <button
                        onClick={(e) => handleExport(w, e)}
                        className="p-1.5 rounded hover:bg-[#1A222D] text-[#64748B] hover:text-sky-400 transition-colors"
                        title="Export JSON"
                      >
                        <Download className="w-3.5 h-3.5" />
                      </button>

                      <button
                        onClick={(e) => handleDelete(w.id, w.name, e)}
                        className="p-1.5 rounded hover:bg-[#1A222D] text-[#64748B] hover:text-rose-400 transition-colors"
                        title="Delete workflow"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Import JSON Modal */}
      {showImportModal && (
        <div className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="bg-[#121820] border border-[#242E3D] rounded-xl shadow-2xl w-full max-w-lg p-5 font-mono text-xs flex flex-col space-y-4 animate-in fade-in zoom-in duration-150">
            <div className="flex items-center justify-between border-b border-[#242E3D] pb-3">
              <div className="flex items-center space-x-2 text-amber-400 font-semibold text-sm">
                <Upload className="w-4 h-4" />
                <span>Import Workflow JSON</span>
              </div>
              <button
                onClick={() => setShowImportModal(false)}
                className="p-1 rounded text-[#94A3B8] hover:text-white"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <textarea
              rows={9}
              value={importJsonInput}
              onChange={(e) => setImportJsonInput(e.target.value)}
              placeholder="Paste exported workflow JSON here..."
              className="w-full bg-[#0B0E14] border border-[#242E3D] rounded-lg p-3 text-white font-mono text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed"
              autoFocus
            />

            <div className="flex items-center justify-end space-x-2 pt-2 border-t border-[#242E3D]">
              <button
                onClick={() => setShowImportModal(false)}
                className="px-3 py-1.5 rounded bg-[#1A222D] text-[#94A3B8] hover:text-white"
              >
                Cancel
              </button>
              <button
                onClick={handleImportJson}
                className="px-4 py-1.5 rounded bg-amber-500 hover:bg-amber-400 text-black font-semibold"
              >
                Import & Open
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
