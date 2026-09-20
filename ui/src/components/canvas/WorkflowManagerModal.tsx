import React, { useState } from 'react';
import {
  X,
  Search,
  Plus,
  Copy,
  Trash2,
  Edit2,
  Check,
  Download,
  Upload,
  Calendar,
  Layers,
  Sparkles,
  ArrowRight,
  FolderOpen,
} from 'lucide-react';
import { StoredWorkflow, WorkflowStorage } from '../../services/workflowStorage';

interface WorkflowManagerModalProps {
  isOpen: boolean;
  onClose: () => void;
  activeWorkflowId: string;
  onSelectWorkflow: (id: string) => void;
  onWorkflowListChange: () => void;
}

export const WorkflowManagerModal: React.FC<WorkflowManagerModalProps> = ({
  isOpen,
  onClose,
  activeWorkflowId,
  onSelectWorkflow,
  onWorkflowListChange,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string>('All');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editNameInput, setEditNameInput] = useState('');

  if (!isOpen) return null;

  const workflows = WorkflowStorage.getWorkflows();

  const categories = ['All', 'Dialogue', 'Podcast', 'Radio', 'General'];

  const filteredWorkflows = workflows.filter((w) => {
    const matchesSearch =
      w.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (w.tags && w.tags.some((t) => t.toLowerCase().includes(searchQuery.toLowerCase()))) ||
      (w.description && w.description.toLowerCase().includes(searchQuery.toLowerCase()));

    const matchesCategory = selectedCategory === 'All' || w.category === selectedCategory;

    return matchesSearch && matchesCategory;
  });

  const handleCreateNew = () => {
    const newWorkflow = WorkflowStorage.createWorkflow('Untitled Studio Workflow');
    onWorkflowListChange();
    onSelectWorkflow(newWorkflow.id);
    onClose();
  };

  const handleDuplicate = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    const duplicated = WorkflowStorage.duplicateWorkflow(id);
    onWorkflowListChange();
    onSelectWorkflow(duplicated.id);
  };

  const handleDelete = (id: string, name: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (workflows.length <= 1) {
      alert('Cannot delete the last remaining workflow.');
      return;
    }
    if (confirm(`Delete workflow "${name}" permanently?`)) {
      WorkflowStorage.deleteWorkflow(id);
      onWorkflowListChange();
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
      onWorkflowListChange();
    }
    setEditingId(null);
  };

  const handleExport = (w: StoredWorkflow, e: React.MouseEvent) => {
    e.stopPropagation();
    WorkflowStorage.exportWorkflowJson(w);
  };

  const handleImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    if (file.size > 10 * 1024 * 1024) {
      alert('Workflow file exceeds 10MB limit');
      if (e.target) e.target.value = '';
      return;
    }

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const text = event.target?.result as string;
        const parsed = JSON.parse(text);
        if (Array.isArray(parsed.nodes) && Array.isArray(parsed.edges)) {
          if (parsed.nodes.length > 500) {
            alert('Workflow exceeds maximum allowed 500 nodes limit');
            return;
          }
          if (parsed.edges.length > 2000) {
            alert('Workflow exceeds maximum allowed 2000 edges limit');
            return;
          }
          const freshId = `wf-${Date.now()}`;
          const imported: StoredWorkflow = {
            ...parsed,
            id: freshId,
            name: parsed.name ? `${parsed.name.slice(0, 100)} (Imported)` : 'Imported Workflow',
            created_at: new Date().toISOString(),
            updated_at: new Date().toISOString(),
          };
          const list = WorkflowStorage.getWorkflows();
          list.unshift(imported);
          WorkflowStorage.saveWorkflows(list);
          onWorkflowListChange();
          onSelectWorkflow(freshId);
          onClose();
        } else {
          alert('Invalid pipeline JSON: Missing nodes or edges array');
        }
      } catch (err: any) {
        alert(`Failed to import workflow: ${err.message}`);
      }
    };
    reader.readAsText(file);
    if (e.target) e.target.value = '';
  };

  return (
    <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
      <div className="bg-[#121820] border border-[#242E3D] rounded-2xl shadow-2xl w-full max-w-4xl max-h-[85vh] flex flex-col overflow-hidden animate-in fade-in zoom-in-95 duration-200">
        {/* Header */}
        <div className="px-6 py-4 border-b border-[#242E3D] bg-[#0B0E14]/70 flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className="p-2 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-400">
              <FolderOpen className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-base font-bold text-white tracking-wide">
                Workflow Project Studio
              </h2>
              <p className="text-xs font-mono text-[#94A3B8] mt-0.5">
                Organize, switch, and manage separate production DAG projects
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-2">
            <label className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-mono border border-[#242E3D] cursor-pointer transition-colors">
              <Upload className="w-3.5 h-3.5" />
              <span>Import JSON</span>
              <input
                type="file"
                accept=".json"
                onChange={handleImport}
                className="hidden"
              />
            </label>

            <button
              onClick={handleCreateNew}
              className="flex items-center space-x-1.5 px-3.5 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black text-xs font-semibold font-mono transition-colors shadow-sm"
            >
              <Plus className="w-4 h-4" />
              <span>New Workflow</span>
            </button>

            <button
              onClick={onClose}
              className="p-1.5 rounded-lg text-[#64748B] hover:text-white hover:bg-[#1A222D] transition-colors"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Toolbar Filter */}
        <div className="px-6 py-3 border-b border-[#242E3D] bg-[#121820] flex items-center justify-between gap-4">
          <div className="relative flex-1 max-w-sm">
            <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-[#64748B]" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search workflows, tags, categories..."
              className="w-full bg-[#0B0E14] border border-[#242E3D] rounded-lg pl-8 pr-3 py-1.5 text-xs text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none font-mono"
            />
          </div>

          <div className="flex items-center space-x-1.5 overflow-x-auto">
            {categories.map((cat) => (
              <button
                key={cat}
                onClick={() => setSelectedCategory(cat)}
                className={`px-2.5 py-1 rounded-md text-[11px] font-mono transition-colors ${
                  selectedCategory === cat
                    ? 'bg-amber-500/15 border border-amber-500 text-amber-400 font-semibold'
                    : 'bg-[#0B0E14] border border-[#242E3D] text-[#94A3B8] hover:text-white hover:border-[#38485C]'
                }`}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        {/* Workflow Grid */}
        <div className="p-6 overflow-y-auto flex-1 grid grid-cols-1 md:grid-cols-2 gap-4">
          {filteredWorkflows.map((w) => {
            const isActive = w.id === activeWorkflowId;
            const isEditingThis = editingId === w.id;

            return (
              <div
                key={w.id}
                onClick={() => {
                  onSelectWorkflow(w.id);
                  onClose();
                }}
                className={`group relative rounded-xl border p-4 bg-[#0B0E14] hover:bg-[#121820] cursor-pointer transition-all ${
                  isActive
                    ? 'border-amber-500 ring-1 ring-amber-500/40 bg-amber-500/[0.03]'
                    : 'border-[#242E3D] hover:border-[#3B485C]'
                }`}
              >
                {/* Active Indicator Pin */}
                {isActive && (
                  <div className="absolute top-3 right-3 flex items-center space-x-1 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/30 text-amber-400 text-[10px] font-mono font-semibold">
                    <Check className="w-3 h-3" />
                    <span>Active on Canvas</span>
                  </div>
                )}

                {/* Workflow Title */}
                <div className="pr-24">
                  {isEditingThis ? (
                    <div
                      className="flex items-center space-x-1.5"
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
                        className="bg-[#121820] border border-amber-500 rounded px-2 py-0.5 text-xs text-white font-mono focus:outline-none"
                      />
                      <button
                        onClick={(e) => saveEditing(w.id, e)}
                        className="p-1 text-emerald-400 hover:text-white"
                      >
                        <Check className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          setEditingId(null);
                        }}
                        className="p-1 text-[#64748B] hover:text-white"
                      >
                        <X className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  ) : (
                    <div className="flex items-center space-x-2">
                      <h3 className="font-semibold text-white text-sm group-hover:text-amber-400 transition-colors truncate">
                        {w.name}
                      </h3>
                      <button
                        onClick={(e) => startEditing(w, e)}
                        className="opacity-0 group-hover:opacity-100 p-1 rounded text-[#64748B] hover:text-amber-400 transition-opacity"
                        title="Rename"
                      >
                        <Edit2 className="w-3 h-3" />
                      </button>
                    </div>
                  )}

                  <div className="flex items-center space-x-2 mt-1.5 text-[11px] font-mono text-[#64748B]">
                    <span className="px-1.5 py-0.5 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
                      {w.category || 'General'}
                    </span>
                    <span>•</span>
                    <span className="flex items-center space-x-1">
                      <Layers className="w-3 h-3" />
                      <span>{w.nodes?.length || 0} nodes</span>
                    </span>
                    <span>•</span>
                    <span>{w.edges?.length || 0} wires</span>
                  </div>
                </div>

                {/* Tags */}
                {w.tags && w.tags.length > 0 && (
                  <div className="flex items-center flex-wrap gap-1 mt-3">
                    {w.tags.map((t) => (
                      <span
                        key={t}
                        className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#1A222D]/60 text-[#94A3B8]"
                      >
                        #{t}
                      </span>
                    ))}
                  </div>
                )}

                {/* Footer Controls */}
                <div className="mt-4 pt-3 border-t border-[#242E3D]/70 flex items-center justify-between text-xs font-mono text-[#64748B]">
                  <div className="flex items-center space-x-1">
                    <Calendar className="w-3 h-3" />
                    <span>
                      {new Date(w.updated_at || w.created_at).toLocaleDateString()}
                    </span>
                  </div>

                  <div className="flex items-center space-x-1">
                    <button
                      onClick={(e) => handleDuplicate(w.id, e)}
                      className="p-1 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-amber-400 transition-colors"
                      title="Duplicate workflow"
                    >
                      <Copy className="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={(e) => handleExport(w, e)}
                      className="p-1 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
                      title="Export JSON"
                    >
                      <Download className="w-3.5 h-3.5" />
                    </button>
                    {workflows.length > 1 && (
                      <button
                        onClick={(e) => handleDelete(w.id, w.name, e)}
                        className="p-1 rounded hover:bg-rose-500/10 text-[#64748B] hover:text-rose-400 transition-colors"
                        title="Delete workflow"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                    <div className="w-px h-3 bg-[#242E3D] mx-1" />
                    <button
                      onClick={() => {
                        onSelectWorkflow(w.id);
                        onClose();
                      }}
                      className="flex items-center space-x-1 px-2 py-1 rounded bg-[#1A222D] group-hover:bg-amber-500 group-hover:text-black text-white text-[11px] font-semibold transition-colors"
                    >
                      <span>Open</span>
                      <ArrowRight className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        {/* Footer Note */}
        <div className="px-6 py-3 border-t border-[#242E3D] bg-[#0B0E14]/40 flex items-center justify-between text-xs font-mono text-[#64748B]">
          <div className="flex items-center space-x-2">
            <Sparkles className="w-3.5 h-3.5 text-amber-400" />
            <span>Workflows auto-persist in browser localStorage</span>
          </div>
          <span>{workflows.length} total projects</span>
        </div>
      </div>
    </div>
  );
};
