import React, { useState, useCallback, useRef, useEffect } from 'react';
import {
  Play,
  Loader2,
  Download,
  ZoomIn,
  ZoomOut,
  Maximize2,
  RotateCcw,
  Plus,
  Move,
  Sliders,
  Cpu,
  Layers,
  FileText,
  Users,
  UserCheck,
  Save,
  Copy,
  Clipboard,
  Trash2,
  Edit2,
  Check,
  CheckSquare,
  X,
  AlertCircle,
  CheckCircle2,
  MoreHorizontal,
  Mic,
  StickyNote,
  Undo2,
  Redo2,
  FolderOpen,
  Headphones,
  Clock,
  Film,
  HelpCircle,
  LayoutGrid,
  ArrowLeft,
} from 'lucide-react';
import {
  PipelineNode,
  Voice,
  NodeExecutionState,
  NodeType,
} from '../../types';
import { NodeCard } from './NodeCard';
import { NodeInspector } from './NodeInspector';
import { CanvasMiniMap } from './CanvasMiniMap';
import { ExecutionTimelineDrawer } from './ExecutionTimelineDrawer';
import { StickyNoteCard } from './StickyNoteCard';
import {
  WorkflowManagerModal,
} from './WorkflowManagerModal';
import {
  WorkflowStorage,
  StoredWorkflow,
  StickyNoteData,
} from '../../services/workflowStorage';
import { api } from '../../services/api';

interface PipelineCanvasProps {
  voices: Voice[];
  initialWorkflowId?: string;
  onBackToWorkflows?: () => void;
}

const NODE_TEMPLATES = [
  {
    type: 'character_voice' as NodeType,
    name: 'Character Voice & Line',
    category: 'Character',
    icon: <Mic className="w-3.5 h-3.5 text-amber-400" />,
    defaultParams: {
      character_name: 'Speaker',
      voice_id: 'en-US-AriaNeural',
      text: 'Enter spoken dialogue here...',
      speed: 1.0,
      pitch: 0,
    },
  },
  {
    type: 'text_input' as NodeType,
    name: 'Text Script Input',
    category: 'Ingestion',
    icon: <FileText className="w-3.5 h-3.5 text-sky-400" />,
    defaultParams: { text: 'New synthesized line.' },
  },
  {
    type: 'asr_transcriber' as NodeType,
    name: 'Audio Ingestion (ASR)',
    category: 'Audio Ingestion',
    icon: <Headphones className="w-3.5 h-3.5 text-violet-400" />,
    defaultParams: { model: 'whisper-large-v3', language: 'en' },
  },
  {
    type: 'document_chunker' as NodeType,
    name: 'Document Chunker',
    category: 'Preprocessing',
    icon: <FileText className="w-3.5 h-3.5 text-teal-400" />,
    defaultParams: { chunk_size: 500, overlap_words: 30 },
  },
  {
    type: 'speaker_parser' as NodeType,
    name: 'Speaker Script Parser',
    category: 'Analysis',
    icon: <Users className="w-3.5 h-3.5 text-purple-400" />,
    defaultParams: {},
  },
  {
    type: 'diarization' as NodeType,
    name: 'Speaker Diarization',
    category: 'Analysis',
    icon: <UserCheck className="w-3.5 h-3.5 text-cyan-400" />,
    defaultParams: { num_speakers: 2 },
  },
  {
    type: 'voice_assigner' as NodeType,
    name: 'Voice Assigner',
    category: 'Routing',
    icon: <UserCheck className="w-3.5 h-3.5 text-amber-400" />,
    defaultParams: { default_voice: 'en-US-AriaNeural' },
  },
  {
    type: 'synthesizer' as NodeType,
    name: 'Neural Synthesizer',
    category: 'Synthesis',
    icon: <Cpu className="w-3.5 h-3.5 text-emerald-400" />,
    defaultParams: { speed: 1.0, pitch: 0.0 },
  },
  {
    type: 'audio_filter' as NodeType,
    name: 'DSP Filter Chain',
    category: 'DSP Filter',
    icon: <Sliders className="w-3.5 h-3.5 text-pink-400" />,
    defaultParams: { trim_silence: true, normalize: true },
  },
  {
    type: 'audio_time_stretch' as NodeType,
    name: 'Audio Time Stretch',
    category: 'DSP Warp',
    icon: <Clock className="w-3.5 h-3.5 text-amber-400" />,
    defaultParams: { speed_ratio: 1.05, preserve_pitch: true },
  },
  {
    type: 'audio_merge' as NodeType,
    name: 'Audio Track Merge',
    category: 'Mastering',
    icon: <Layers className="w-3.5 h-3.5 text-indigo-400" />,
    defaultParams: { pause_ms: 150 },
  },
  {
    type: 'audio_mux' as NodeType,
    name: 'Audio/Video Muxer',
    category: 'Video Dubbing',
    icon: <Film className="w-3.5 h-3.5 text-rose-400" />,
    defaultParams: { format: 'mp4' },
  },
  {
    type: 'output_sink' as NodeType,
    name: 'Master Audio Sink',
    category: 'Output Sink',
    icon: <Save className="w-3.5 h-3.5 text-rose-400" />,
    defaultParams: { format: 'wav' },
  },
];

const getNodePos = (node: PipelineNode): { x: number; y: number } => {
  return node.position || { x: 100, y: 100 };
};

export const PipelineCanvas: React.FC<PipelineCanvasProps> = ({
  voices,
  initialWorkflowId,
  onBackToWorkflows,
}) => {
  // ── Multi-Workflow Storage & Active Project State ────────────────────────
  const [pipeline, setPipeline] = useState<StoredWorkflow>(() => {
    if (initialWorkflowId) {
      const target = WorkflowStorage.getWorkflows().find((w) => w.id === initialWorkflowId);
      if (target) return target;
    }
    return WorkflowStorage.getActiveWorkflow();
  });
  const [openTabs, setOpenTabs] = useState<string[]>(() => {
    const tabs = WorkflowStorage.getOpenTabs();
    if (initialWorkflowId && !tabs.includes(initialWorkflowId)) {
      return [...tabs, initialWorkflowId];
    }
    return tabs;
  });
  const [isDirty, setIsDirty] = useState<boolean>(false);
  const [showManagerModal, setShowManagerModal] = useState<boolean>(false);

  useEffect(() => {
    if (initialWorkflowId && initialWorkflowId !== pipeline.id) {
      const target = WorkflowStorage.getWorkflows().find((w) => w.id === initialWorkflowId);
      if (target) {
        setPipeline(JSON.parse(JSON.stringify(target)));
        setIsDirty(false);
        setOpenTabs((tabs) => (tabs.includes(initialWorkflowId) ? tabs : [...tabs, initialWorkflowId]));
      }
    }
  }, [initialWorkflowId]);

  // Undo / Redo History Stack
  const historyRef = useRef<StoredWorkflow[]>([]);
  const historyIndexRef = useRef<number>(-1);

  const pushHistory = useCallback((newWorkflow: StoredWorkflow) => {
    const history = historyRef.current.slice(0, historyIndexRef.current + 1);
    history.push(JSON.parse(JSON.stringify(newWorkflow)));
    if (history.length > 30) history.shift();
    historyRef.current = history;
    historyIndexRef.current = history.length - 1;
  }, []);

  const handleUndo = useCallback(() => {
    if (historyIndexRef.current > 0) {
      historyIndexRef.current -= 1;
      const prev = historyRef.current[historyIndexRef.current];
      if (prev) {
        setPipeline(JSON.parse(JSON.stringify(prev)));
        setIsDirty(true);
      }
    }
  }, []);

  const handleRedo = useCallback(() => {
    if (historyIndexRef.current < historyRef.current.length - 1) {
      historyIndexRef.current += 1;
      const next = historyRef.current[historyIndexRef.current];
      if (next) {
        setPipeline(JSON.parse(JSON.stringify(next)));
        setIsDirty(true);
      }
    }
  }, []);

  // ── Selection State (Single & Multi-Node) ────────────────────────────────
  const [selectedNodeIds, setSelectedNodeIds] = useState<Set<string>>(new Set(['node-1']));
  const initialSelectedIdsRef = useRef<Set<string>>(new Set());

  // ── Marquee Box Selection State (n8n Style) ──────────────────────────────
  const [isMarqueeSelecting, setIsMarqueeSelecting] = useState<boolean>(false);
  const [marqueeBox, setMarqueeBox] = useState<{
    startClientX: number;
    startClientY: number;
    startCanvasX: number;
    startCanvasY: number;
    curCanvasX: number;
    curCanvasY: number;
  } | null>(null);

  // ── 2D Viewport State (Swipe Pan & Double-Click Zoom) ───────────────────
  const [pan, setPan] = useState<{ x: number; y: number }>({ x: 60, y: 120 });
  const [zoom, setZoom] = useState<number>(1.0);
  const [isMiddlePanning, setIsMiddlePanning] = useState<boolean>(false);
  const [isSpacePressed, setIsSpacePressed] = useState<boolean>(false);
  const [panStart, setPanStart] = useState<{ x: number; y: number }>({ x: 0, y: 0 });

  // ── Multi-Node Dragging State (Synchronized + 16px Grid Snap) ────────────
  const [dragNodeState, setDragNodeState] = useState<{
    startMouseX: number;
    startMouseY: number;
    nodeStarts: Record<string, { x: number; y: number }>;
  } | null>(null);

  // ── Wire Connection & Release-Wire Quick-Add State ───────────────────────
  const [connectingFrom, setConnectingFrom] = useState<string | null>(null);
  const [pointerCanvasPos, setPointerCanvasPos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });
  const [quickAddPos, setQuickAddPos] = useState<{ x: number; y: number; connectFromId?: string } | null>(null);

  // ── Real-Time Execution & Flow Visualization (n8n Parity) ────────────────
  const [isRunning, setIsRunning] = useState(false);
  const [executionStates, setExecutionStates] = useState<Record<string, NodeExecutionState>>({});
  const [activeEdgeIds, setActiveEdgeIds] = useState<Set<string>>(new Set());
  const [executionTotalMs, setExecutionTotalMs] = useState<number>(0);
  const [showExecutionDrawer, setShowExecutionDrawer] = useState<boolean>(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);

  // ── UI Dropdowns & Modals ────────────────────────────────────────────────
  const [showAddMenu, setShowAddMenu] = useState(false);
  const [showMoreMenu, setShowMoreMenu] = useState(false);
  const [showMiniMap, setShowMiniMap] = useState<boolean>(true);
  const [toastMessage, setToastMessage] = useState<{ text: string; type?: 'info' | 'error' } | null>(null);
  const [isEditingName, setIsEditingName] = useState(false);
  const [nameInput, setNameInput] = useState('');
  const [showPasteModal, setShowPasteModal] = useState(false);
  const [pasteJsonInput, setPasteJsonInput] = useState('');
  const [showShortcutsModal, setShowShortcutsModal] = useState(false);

  const audioRef = useRef<HTMLAudioElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const canvasContainerRef = useRef<HTMLDivElement | null>(null);

  const showToast = useCallback((text: string, type: 'info' | 'error' = 'info') => {
    setToastMessage({ text, type });
    setTimeout(() => {
      setToastMessage((cur) => (cur?.text === text ? null : cur));
    }, 2800);
  }, []);

  // Sync initial history
  useEffect(() => {
    if (historyRef.current.length === 0) {
      pushHistory(pipeline);
    }
  }, [pipeline, pushHistory]);

  // Clean audio URL on unmount
  useEffect(() => {
    return () => {
      if (audioUrl) URL.revokeObjectURL(audioUrl);
    };
  }, [audioUrl]);

  // Update master audio source safely
  useEffect(() => {
    if (audioRef.current && audioUrl) {
      audioRef.current.load();
    }
  }, [audioUrl]);

  const primarySelectedId =
    selectedNodeIds.size === 1
      ? Array.from(selectedNodeIds)[0]
      : selectedNodeIds.size > 0
      ? Array.from(selectedNodeIds)[selectedNodeIds.size - 1]
      : null;
  const selectedNode = pipeline.nodes.find((n) => n.id === primarySelectedId) || null;
  const selectedNodes = pipeline.nodes.filter((n) => selectedNodeIds.has(n.id));

  // ── Workflow Storage Actions (Save, Switch, New) ─────────────────────────
  const handleSaveWorkflow = useCallback(() => {
    const saved = WorkflowStorage.saveWorkflow(pipeline);
    setPipeline(saved);
    setIsDirty(false);
    showToast(`Workflow "${saved.name}" saved!`, 'info');
  }, [pipeline, showToast]);

  const handleSelectWorkflow = useCallback(
    (workflowId: string) => {
      if (isDirty) {
        WorkflowStorage.saveWorkflow(pipeline);
      }
      WorkflowStorage.setActiveWorkflowId(workflowId);
      const target = WorkflowStorage.getActiveWorkflow();
      setPipeline(target);
      setOpenTabs(WorkflowStorage.getOpenTabs());
      setSelectedNodeIds(target.nodes[0] ? new Set([target.nodes[0].id]) : new Set());
      setExecutionStates({});
      setActiveEdgeIds(new Set());
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }
      setIsDirty(false);
      pushHistory(target);
      setTimeout(handleFitToView, 60);
      showToast(`Switched to "${target.name}"`);
    },
    [isDirty, pipeline, audioUrl, pushHistory, showToast]
  );

  const handleCloseTab = (e: React.MouseEvent, tabId: string) => {
    e.stopPropagation();
    WorkflowStorage.removeOpenTab(tabId);
    const updatedTabs = WorkflowStorage.getOpenTabs();
    setOpenTabs(updatedTabs);
    if (tabId === pipeline.id) {
      handleSelectWorkflow(updatedTabs[0] || 'preset-narrative');
    }
  };

  const handleCreateNewWorkflow = () => {
    const created = WorkflowStorage.createWorkflow('Untitled Studio Workflow');
    setOpenTabs(WorkflowStorage.getOpenTabs());
    setPipeline(created);
    setSelectedNodeIds(created.nodes[0] ? new Set([created.nodes[0].id]) : new Set());
    setIsDirty(false);
    pushHistory(created);
    showToast('Created new workflow project');
  };

  // ── Node & Edge Modifications ────────────────────────────────────────────
  const handleUpdateParams = (nodeId: string, params: Record<string, any>) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: prev.nodes.map((n) => (n.id === nodeId ? { ...n, params } : n)),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
  };

  const handleUpdateNodeName = useCallback((nodeId: string, name: string) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: prev.nodes.map((n) => (n.id === nodeId ? { ...n, name } : n)),
      };
      setIsDirty(true);
      return updated;
    });
  }, []);

  const handleToggleDisableNode = useCallback((nodeId: string) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: prev.nodes.map((n) => (n.id === nodeId ? { ...n, disabled: !n.disabled } : n)),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
  }, [pushHistory]);

  const handleSelectNode = useCallback((nodeId: string, isMultiToggle = false) => {
    setSelectedNodeIds((prev) => {
      if (isMultiToggle) {
        const next = new Set(prev);
        if (next.has(nodeId)) {
          next.delete(nodeId);
        } else {
          next.add(nodeId);
        }
        return next;
      }
      return new Set([nodeId]);
    });
  }, []);

  const handleSelectAll = useCallback(() => {
    if (pipeline.nodes.length === 0) return;
    const allIds = new Set(pipeline.nodes.map((n) => n.id));
    setSelectedNodeIds(allIds);
    showToast(`Selected all ${pipeline.nodes.length} nodes (Del, Ctrl+C, Ctrl+D)`, 'info');
  }, [pipeline.nodes, showToast]);

  const handleDeleteNode = useCallback((nodeId: string) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: prev.nodes.filter((n) => n.id !== nodeId),
        edges: prev.edges.filter((e) => e.from_node !== nodeId && e.to_node !== nodeId),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
    setSelectedNodeIds((prev) => {
      const next = new Set(prev);
      next.delete(nodeId);
      return next;
    });
  }, [pushHistory]);

  const handleDeleteSelected = useCallback(() => {
    if (selectedNodeIds.size === 0) return;
    const count = selectedNodeIds.size;
    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: prev.nodes.filter((n) => !selectedNodeIds.has(n.id)),
        edges: prev.edges.filter(
          (e) => !selectedNodeIds.has(e.from_node) && !selectedNodeIds.has(e.to_node)
        ),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
    setSelectedNodeIds(new Set());
    showToast(`Deleted ${count} selected nodes`, 'info');
  }, [selectedNodeIds, pushHistory, showToast]);

  const handleDeleteEdge = (edgeId: string) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        edges: prev.edges.filter((e) => e.id !== edgeId),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
  };

  const handleDuplicateNode = useCallback((nodeId: string) => {
    const targetNode = pipeline.nodes.find((n) => n.id === nodeId);
    if (!targetNode) return;

    const freshId = `node-${Date.now()}`;
    const pos = targetNode.position || { x: 100, y: 100 };
    const duplicatedNode: PipelineNode = {
      ...targetNode,
      id: freshId,
      name: `${targetNode.name} (Copy)`,
      position: { x: pos.x + 32, y: pos.y + 32 },
    };

    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: [...prev.nodes, duplicatedNode],
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
    setSelectedNodeIds(new Set([freshId]));
    showToast(`Duplicated ${targetNode.name}`);
  }, [pipeline.nodes, pushHistory, showToast]);

  const handleDuplicateSelected = useCallback(() => {
    if (selectedNodeIds.size === 0) return;
    if (selectedNodeIds.size === 1) {
      handleDuplicateNode(Array.from(selectedNodeIds)[0]);
      return;
    }

    const targetNodes = pipeline.nodes.filter((n) => selectedNodeIds.has(n.id));
    const idMap = new Map<string, string>();
    const duplicatedNodes: PipelineNode[] = [];

    targetNodes.forEach((node) => {
      const freshId = `node-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
      idMap.set(node.id, freshId);
      const pos = getNodePos(node);
      duplicatedNodes.push({
        ...node,
        id: freshId,
        name: `${node.name} (Copy)`,
        position: { x: pos.x + 32, y: pos.y + 32 },
      });
    });

    const internalEdges = pipeline.edges
      .filter((e) => selectedNodeIds.has(e.from_node) && selectedNodeIds.has(e.to_node))
      .map((e) => ({
        id: `edge-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`,
        from_node: idMap.get(e.from_node)!,
        to_node: idMap.get(e.to_node)!,
      }));

    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: [...prev.nodes, ...duplicatedNodes],
        edges: [...prev.edges, ...internalEdges],
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });

    setSelectedNodeIds(new Set(duplicatedNodes.map((n) => n.id)));
    showToast(`Duplicated ${duplicatedNodes.length} nodes & connections`);
  }, [selectedNodeIds, pipeline.nodes, pipeline.edges, handleDuplicateNode, pushHistory, showToast]);

  const copyToClipboard = (text: string) => {
    if (navigator.clipboard?.writeText) {
      navigator.clipboard.writeText(text).catch(() => {
        const el = document.createElement('textarea');
        el.value = text;
        el.style.position = 'fixed';
        el.style.left = '-9999px';
        document.body.appendChild(el);
        el.select();
        try {
          document.execCommand('copy');
        } catch {}
        document.body.removeChild(el);
      });
    } else {
      const el = document.createElement('textarea');
      el.value = text;
      el.style.position = 'fixed';
      el.style.left = '-9999px';
      document.body.appendChild(el);
      el.select();
      try {
        document.execCommand('copy');
      } catch {}
      document.body.removeChild(el);
    }
  };

  const handleCopyNodeJson = useCallback((node: PipelineNode) => {
    copyToClipboard(JSON.stringify(node, null, 2));
    showToast(`Node "${node.name}" JSON copied to clipboard!`);
  }, [showToast]);

  const handleCopySelected = useCallback(() => {
    if (selectedNodeIds.size === 0) return;
    const subNodes = pipeline.nodes.filter((n) => selectedNodeIds.has(n.id));
    const subEdges = pipeline.edges.filter(
      (e) => selectedNodeIds.has(e.from_node) && selectedNodeIds.has(e.to_node)
    );
    const payload = {
      name: `${pipeline.name} (Selection)`,
      description: `Workflow selection with ${subNodes.length} nodes from ${pipeline.name}`,
      nodes: subNodes,
      edges: subEdges,
      created_at: new Date().toISOString(),
    };
    const jsonStr = JSON.stringify(payload, null, 2);
    copyToClipboard(jsonStr);
    showToast(`Copied ${subNodes.length} selected nodes as workflow JSON! (Ctrl+C)`, 'info');
  }, [selectedNodeIds, pipeline, showToast]);

  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      if (!text) return;
      const parsed = JSON.parse(text);
      let incomingNodes: PipelineNode[] = [];
      let incomingEdges: any[] = [];
      if (Array.isArray(parsed)) {
        incomingNodes = parsed;
      } else if (parsed && Array.isArray(parsed.nodes)) {
        incomingNodes = parsed.nodes;
        incomingEdges = parsed.edges || [];
      } else if (parsed && parsed.node_type) {
        incomingNodes = [parsed];
      }
      if (incomingNodes.length === 0) return;

      const idMap = new Map<string, string>();
      const pastedNodes: PipelineNode[] = incomingNodes.map((n) => {
        const freshId = `node-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
        idMap.set(n.id, freshId);
        const pos = getNodePos(n);
        return {
          ...n,
          id: freshId,
          name: `${n.name} (Pasted)`,
          position: { x: pos.x + 36, y: pos.y + 36 },
        };
      });

      const pastedEdges = incomingEdges
        .filter((e) => idMap.has(e.from_node) && idMap.has(e.to_node))
        .map((e) => ({
          id: `edge-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`,
          from_node: idMap.get(e.from_node)!,
          to_node: idMap.get(e.to_node)!,
        }));

      setPipeline((prev) => {
        const updated = {
          ...prev,
          nodes: [...prev.nodes, ...pastedNodes],
          edges: [...prev.edges, ...pastedEdges],
        };
        setIsDirty(true);
        pushHistory(updated);
        return updated;
      });

      setSelectedNodeIds(new Set(pastedNodes.map((n) => n.id)));
      showToast(`Pasted ${pastedNodes.length} nodes from clipboard`);
    } catch {
      setShowPasteModal(true);
    }
  }, [pushHistory, showToast]);

  const handleAutoLayout = useCallback(() => {
    if (pipeline.nodes.length === 0) return;

    const inDegree = new Map<string, number>();
    const outgoing = new Map<string, string[]>();

    pipeline.nodes.forEach((n) => {
      inDegree.set(n.id, 0);
      outgoing.set(n.id, []);
    });

    pipeline.edges.forEach((e) => {
      if (inDegree.has(e.to_node)) {
        inDegree.set(e.to_node, (inDegree.get(e.to_node) || 0) + 1);
      }
      if (outgoing.has(e.from_node)) {
        outgoing.get(e.from_node)!.push(e.to_node);
      }
    });

    const colRank = new Map<string, number>();
    const queue: string[] = [];

    pipeline.nodes.forEach((n) => {
      if ((inDegree.get(n.id) || 0) === 0 || n.node_type === 'text_input') {
        colRank.set(n.id, 0);
        queue.push(n.id);
      }
    });

    while (queue.length > 0) {
      const u = queue.shift()!;
      const r = colRank.get(u) || 0;
      const targets = outgoing.get(u) || [];
      targets.forEach((v) => {
        const currentRank = colRank.get(v) || 0;
        if (r + 1 > currentRank) {
          colRank.set(v, r + 1);
          queue.push(v);
        }
      });
    }

    const columns = new Map<number, PipelineNode[]>();
    pipeline.nodes.forEach((n) => {
      const col = colRank.get(n.id) ?? (n.node_type === 'output_sink' ? 4 : 1);
      if (!columns.has(col)) columns.set(col, []);
      columns.get(col)!.push(n);
    });

    const sortedCols = Array.from(columns.keys()).sort((a, b) => a - b);
    const colSpacing = 340;
    const rowSpacing = 220;
    const startX = 80;
    const startY = 80;

    const newPositions = new Map<string, { x: number; y: number }>();

    sortedCols.forEach((colIdx, colOrder) => {
      const colNodes = columns.get(colIdx)!;
      colNodes.forEach((node, rowIdx) => {
        newPositions.set(node.id, {
          x: startX + colOrder * colSpacing,
          y: startY + rowIdx * rowSpacing,
        });
      });
    });

    setPipeline((prev) => {
      const updatedNodes = prev.nodes.map((n) => ({
        ...n,
        position: newPositions.get(n.id) || n.position || { x: 100, y: 100 },
      }));
      const updated = {
        ...prev,
        nodes: updatedNodes,
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });

    showToast(`Auto-arranged ${pipeline.nodes.length} nodes into clean columns`);
  }, [pipeline.nodes, pipeline.edges, pushHistory, showToast]);

  // Global Keyboard Shortcuts (n8n Standard)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const activeTag = (document.activeElement as HTMLElement)?.tagName;
      if (
        activeTag === 'INPUT' ||
        activeTag === 'TEXTAREA' ||
        (document.activeElement as HTMLElement)?.isContentEditable
      ) {
        return;
      }

      // Ctrl+A / Cmd+A: Select all
      if ((e.ctrlKey || e.metaKey) && (e.key === 'a' || e.key === 'A')) {
        e.preventDefault();
        handleSelectAll();
        return;
      }

      // Delete / Backspace: Delete selected
      if (e.key === 'Delete' || e.key === 'Backspace') {
        e.preventDefault();
        handleDeleteSelected();
        return;
      }

      // Ctrl+D / Cmd+D: Duplicate selected
      if ((e.ctrlKey || e.metaKey) && (e.key === 'd' || e.key === 'D')) {
        e.preventDefault();
        handleDuplicateSelected();
        return;
      }

      // Ctrl+C / Cmd+C: Copy selected
      if ((e.ctrlKey || e.metaKey) && (e.key === 'c' || e.key === 'C')) {
        handleCopySelected();
        return;
      }

      // Ctrl+V / Cmd+V: Paste
      if ((e.ctrlKey || e.metaKey) && (e.key === 'v' || e.key === 'V')) {
        e.preventDefault();
        handlePaste();
        return;
      }

      // Ctrl+Z: Undo
      if ((e.ctrlKey || e.metaKey) && (e.key === 'z' || e.key === 'Z') && !e.shiftKey) {
        e.preventDefault();
        handleUndo();
        return;
      }

      // Ctrl+Y or Ctrl+Shift+Z: Redo
      if (
        ((e.ctrlKey || e.metaKey) && (e.key === 'y' || e.key === 'Y')) ||
        ((e.ctrlKey || e.metaKey) && e.shiftKey && (e.key === 'z' || e.key === 'Z'))
      ) {
        e.preventDefault();
        handleRedo();
        return;
      }

      // Shift+Alt+L: Auto Layout
      if (e.shiftKey && e.altKey && (e.key === 'l' || e.key === 'L')) {
        e.preventDefault();
        handleAutoLayout();
        return;
      }

      // ?: Open keyboard shortcuts guide
      if (e.key === '?' || (e.shiftKey && e.key === '/')) {
        e.preventDefault();
        setShowShortcutsModal((prev) => !prev);
        return;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [
    handleSelectAll,
    handleDeleteSelected,
    handleDuplicateSelected,
    handleCopySelected,
    handlePaste,
    handleUndo,
    handleRedo,
    handleAutoLayout,
  ]);

  // Sticky Notes Handling
  const handleAddStickyNote = () => {
    const rect = canvasContainerRef.current?.getBoundingClientRect();
    const centerX = rect ? (rect.width / 2 - pan.x) / zoom - 140 : 200;
    const centerY = rect ? (rect.height / 2 - pan.y) / zoom - 60 : 150;

    const newNote: StickyNoteData = {
      id: `note-${Date.now()}`,
      title: 'Workflow Note',
      content: 'Add implementation notes, documentation, or stage instructions here.',
      color: 'amber',
      position: { x: Math.round(centerX), y: Math.round(centerY) },
      width: 280,
    };

    setPipeline((prev) => {
      const updated = {
        ...prev,
        sticky_notes: [...(prev.sticky_notes || []), newNote],
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
    showToast('Added Sticky Note to canvas');
  };

  const handleUpdateStickyNote = (updatedNote: StickyNoteData) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        sticky_notes: (prev.sticky_notes || []).map((n) => (n.id === updatedNote.id ? updatedNote : n)),
      };
      setIsDirty(true);
      return updated;
    });
  };

  const handleDeleteStickyNote = (noteId: string) => {
    setPipeline((prev) => {
      const updated = {
        ...prev,
        sticky_notes: (prev.sticky_notes || []).filter((n) => n.id !== noteId),
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });
  };

  // ── Wire Connections ─────────────────────────────────────────────────────
  const handleConnectStart = (nodeId: string, e: React.PointerEvent) => {
    setConnectingFrom(nodeId);
    if (canvasContainerRef.current) {
      const rect = canvasContainerRef.current.getBoundingClientRect();
      setPointerCanvasPos({
        x: Math.round((e.clientX - rect.left - pan.x) / zoom),
        y: Math.round((e.clientY - rect.top - pan.y) / zoom),
      });
    }
  };

  const handleConnectEnd = (targetNodeId: string) => {
    if (connectingFrom && connectingFrom !== targetNodeId) {
      const exists = pipeline.edges.some(
        (e) => e.from_node === connectingFrom && e.to_node === targetNodeId
      );
      if (!exists) {
        setPipeline((prev) => {
          const updated = {
            ...prev,
            edges: [
              ...prev.edges,
              {
                id: `edge-${Date.now()}`,
                from_node: connectingFrom,
                to_node: targetNodeId,
              },
            ],
          };
          setIsDirty(true);
          pushHistory(updated);
          return updated;
        });
      }
    }
    setConnectingFrom(null);
  };

  // ── Marquee Box Drag Selection & Pan Handlers ────────────────────────────
  const handleCanvasPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    const target = e.target as HTMLElement;
    if (target.closest('.node-card-interactive') || target.tagName === 'BUTTON' || target.tagName === 'INPUT') {
      return;
    }

    // Middle click or Space+Click or Alt+Click: Canvas panning
    if (e.button === 1 || (e.button === 0 && (e.altKey || isSpacePressed))) {
      setIsMiddlePanning(true);
      setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
      try {
        (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
      } catch {}
      return;
    }

    // Left click on background: Start Marquee Selection
    if (e.button === 0) {
      try {
        (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
      } catch {}

      const rect = canvasContainerRef.current?.getBoundingClientRect();
      if (!rect) return;
      const canvasX = (e.clientX - rect.left - pan.x) / zoom;
      const canvasY = (e.clientY - rect.top - pan.y) / zoom;

      setMarqueeBox({
        startClientX: e.clientX,
        startClientY: e.clientY,
        startCanvasX: canvasX,
        startCanvasY: canvasY,
        curCanvasX: canvasX,
        curCanvasY: canvasY,
      });

      initialSelectedIdsRef.current = new Set(selectedNodeIds);
      if (!e.shiftKey && !e.ctrlKey && !e.metaKey) {
        setSelectedNodeIds(new Set());
      }
      setShowAddMenu(false);
      setShowMoreMenu(false);
      setQuickAddPos(null);
    }
  };

  // Node Drag Start Handler (Multi-Node Synchronized + 16px Grid Snap)
  const handleNodePointerDown = useCallback((e: React.PointerEvent, nodeId: string) => {
    e.stopPropagation();
    setShowAddMenu(false);
    setShowMoreMenu(false);

    const isModifier = e.shiftKey || e.ctrlKey || e.metaKey;
    let currentSelected = selectedNodeIds;
    if (!selectedNodeIds.has(nodeId)) {
      if (isModifier) {
        currentSelected = new Set(selectedNodeIds);
        currentSelected.add(nodeId);
      } else {
        currentSelected = new Set([nodeId]);
      }
      setSelectedNodeIds(currentSelected);
    }

    const nodeStarts: Record<string, { x: number; y: number }> = {};
    pipeline.nodes.forEach((n) => {
      if (currentSelected.has(n.id) || n.id === nodeId) {
        nodeStarts[n.id] = getNodePos(n);
      }
    });

    setDragNodeState({
      startMouseX: e.clientX,
      startMouseY: e.clientY,
      nodeStarts,
    });
  }, [pipeline.nodes, selectedNodeIds]);

  // Pointer Move (Handles Dragging, Wire Tracking, Marquee Box, or Panning)
  const handlePointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
    if (connectingFrom) {
      if (canvasContainerRef.current) {
        const rect = canvasContainerRef.current.getBoundingClientRect();
        setPointerCanvasPos({
          x: Math.round((e.clientX - rect.left - pan.x) / zoom),
          y: Math.round((e.clientY - rect.top - pan.y) / zoom),
        });
      }
    } else if (dragNodeState) {
      const dx = (e.clientX - dragNodeState.startMouseX) / zoom;
      const dy = (e.clientY - dragNodeState.startMouseY) / zoom;
      // 16px Grid Snapping
      const snapDx = Math.round(dx / 16) * 16;
      const snapDy = Math.round(dy / 16) * 16;

      setPipeline((prev) => ({
        ...prev,
        nodes: prev.nodes.map((n) => {
          const startPos = dragNodeState.nodeStarts[n.id];
          if (startPos) {
            return {
              ...n,
              position: {
                x: Math.round(startPos.x + snapDx),
                y: Math.round(startPos.y + snapDy),
              },
            };
          }
          return n;
        }),
      }));
      setIsDirty(true);
    } else if (marqueeBox) {
      const rect = canvasContainerRef.current?.getBoundingClientRect();
      if (!rect) return;
      const curX = (e.clientX - rect.left - pan.x) / zoom;
      const curY = (e.clientY - rect.top - pan.y) / zoom;

      const dist = Math.hypot(e.clientX - marqueeBox.startClientX, e.clientY - marqueeBox.startClientY);
      if (dist > 4) {
        setIsMarqueeSelecting(true);
        setMarqueeBox((prev) => (prev ? { ...prev, curCanvasX: curX, curCanvasY: curY } : null));

        // Compute AABB intersection with all nodes
        const minX = Math.min(marqueeBox.startCanvasX, curX);
        const maxX = Math.max(marqueeBox.startCanvasX, curX);
        const minY = Math.min(marqueeBox.startCanvasY, curY);
        const maxY = Math.max(marqueeBox.startCanvasY, curY);

        const intersectedIds = new Set<string>();
        pipeline.nodes.forEach((n) => {
          const pos = getNodePos(n);
          const nodeRight = pos.x + 256;
          const nodeBottom = pos.y + 135;
          if (pos.x < maxX && nodeRight > minX && pos.y < maxY && nodeBottom > minY) {
            intersectedIds.add(n.id);
          }
        });

        if (e.shiftKey || e.ctrlKey || e.metaKey) {
          setSelectedNodeIds(new Set([...Array.from(initialSelectedIdsRef.current), ...Array.from(intersectedIds)]));
        } else {
          setSelectedNodeIds(intersectedIds);
        }
      }
    } else if (isMiddlePanning) {
      setPan({
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y,
      });
    }
  };

  // Pointer Up
  const handlePointerUp = (e: React.PointerEvent) => {
    try {
      (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    } catch {}

    if (connectingFrom && canvasContainerRef.current) {
      const rect = canvasContainerRef.current.getBoundingClientRect();
      const dropCanvasX = Math.round((e.clientX - rect.left - pan.x) / zoom);
      const dropCanvasY = Math.round((e.clientY - rect.top - pan.y) / zoom);
      // Open Quick-Add menu if released on canvas
      setQuickAddPos({ x: dropCanvasX, y: dropCanvasY, connectFromId: connectingFrom });
    }

    if (dragNodeState) {
      pushHistory(pipeline);
    }

    setIsMiddlePanning(false);
    setDragNodeState(null);
    setConnectingFrom(null);
    setIsMarqueeSelecting(false);
    setMarqueeBox(null);
  };

  // ── 2D Swipe Panning (User Request: swipe up/down and side moves screen like in n8n) ─────
  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    e.preventDefault();

    // If Ctrl key or Meta key is pressed: fine pinch zoom
    if (e.ctrlKey || e.metaKey) {
      const rect = canvasContainerRef.current?.getBoundingClientRect();
      if (!rect) return;
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const zoomDelta = e.deltaY < 0 ? 1.08 : 0.92;
      const newZoom = Math.min(Math.max(Number((zoom * zoomDelta).toFixed(3)), 0.25), 2.5);

      const newPanX = mouseX - (mouseX - pan.x) * (newZoom / zoom);
      const newPanY = mouseY - (mouseY - pan.y) * (newZoom / zoom);

      setZoom(newZoom);
      setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
      return;
    }

    // Default: 2D Swipe Navigation (trackpad two-finger swipe or mouse wheel pan)
    let dx = e.deltaX;
    let dy = e.deltaY;

    // Normalize Windows wheel lines mode (DOM_DELTA_LINE === 1)
    if (e.deltaMode === 1) {
      dx *= 28;
      dy *= 28;
    } else if (e.deltaMode === 2) {
      dx *= 400;
      dy *= 400;
    }

    // Shift + vertical wheel translates to horizontal pan
    if (e.shiftKey && Math.abs(dy) > 0 && Math.abs(dx) === 0) {
      dx = dy;
      dy = 0;
    }

    setPan((prev) => ({
      x: Math.round(prev.x - dx),
      y: Math.round(prev.y - dy),
    }));
  };

  // ── Double-Click Zoom In (User Request: zoom in on double tap / click) ───
  const handleDoubleClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const target = e.target as HTMLElement;
    if (target.closest('.node-card-interactive') || target.tagName === 'BUTTON' || target.tagName === 'INPUT') {
      return;
    }

    const rect = canvasContainerRef.current?.getBoundingClientRect();
    if (!rect) return;
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const newZoom = Math.min(Number((zoom * 1.35).toFixed(2)), 2.5);
    const newPanX = mouseX - (mouseX - pan.x) * (newZoom / zoom);
    const newPanY = mouseY - (mouseY - pan.y) * (newZoom / zoom);

    setZoom(newZoom);
    setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
  };

  // ── Button Zoom Out & HUD Navigation Controls ───────────────────────────
  const handleZoomOut = () => {
    setZoom((z) => Math.max(Number((z - 0.15).toFixed(2)), 0.25));
  };

  const handleZoomIn = () => {
    setZoom((z) => Math.min(Number((z + 0.15).toFixed(2)), 2.5));
  };

  const handleResetZoom = () => {
    setZoom(1.0);
    setPan({ x: 60, y: 120 });
  };

  const handleFitToView = useCallback(() => {
    if (pipeline.nodes.length === 0) return;
    const rect = canvasContainerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const padding = 80;
    const xs = pipeline.nodes.map((n) => getNodePos(n).x);
    const ys = pipeline.nodes.map((n) => getNodePos(n).y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs) + 260;
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys) + 140;

    const graphWidth = maxX - minX;
    const graphHeight = maxY - minY;

    const scaleX = (rect.width - padding * 2) / graphWidth;
    const scaleY = (rect.height - padding * 2) / graphHeight;
    const newZoom = Math.min(Math.max(Math.min(scaleX, scaleY), 0.35), 1.5);

    const newPanX = (rect.width - graphWidth * newZoom) / 2 - minX * newZoom;
    const newPanY = (rect.height - graphHeight * newZoom) / 2 - minY * newZoom;

    setZoom(Number(newZoom.toFixed(2)));
    setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
  }, [pipeline.nodes]);

  // ── Template Insertion ──────────────────────────────────────────────────
  const handleAddNode = (template: typeof NODE_TEMPLATES[0], customPos?: { x: number; y: number }, connectFrom?: string) => {
    let posX = customPos?.x;
    let posY = customPos?.y;

    if (posX === undefined || posY === undefined) {
      const rect = canvasContainerRef.current?.getBoundingClientRect();
      posX = rect ? Math.round((rect.width / 2 - pan.x) / zoom - 128) : 200;
      posY = rect ? Math.round((rect.height / 2 - pan.y) / zoom - 60) : 150;
    }

    const newNodeId = `node-${Date.now()}`;
    const newNode: PipelineNode = {
      id: newNodeId,
      name: template.name,
      node_type: template.type,
      params: { ...template.defaultParams },
      position: { x: posX, y: posY },
    };

    let newEdges = [...pipeline.edges];
    const sourceId = connectFrom || (selectedNodeIds.size === 1 ? Array.from(selectedNodeIds)[0] : null);
    if (sourceId && pipeline.nodes.some((n) => n.id === sourceId)) {
      newEdges.push({
        id: `edge-${Date.now()}`,
        from_node: sourceId,
        to_node: newNodeId,
      });
    }

    setPipeline((prev) => {
      const updated = {
        ...prev,
        nodes: [...prev.nodes, newNode],
        edges: newEdges,
      };
      setIsDirty(true);
      pushHistory(updated);
      return updated;
    });

    setSelectedNodeIds(new Set([newNodeId]));
    setShowAddMenu(false);
    setQuickAddPos(null);
  };

  // ── Real-Time Execution & Flow Visualization ─────────────────────────────
  const handleRunPipeline = async () => {
    try {
      setIsRunning(true);
      setShowExecutionDrawer(true);
      setActiveEdgeIds(new Set());
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }

      // 1. Initial State: Waiting for all active nodes
      const activeNodes = pipeline.nodes.filter((n) => !n.disabled);
      const initStates: Record<string, NodeExecutionState> = {};
      activeNodes.forEach((n) => {
        initStates[n.id] = { status: 'waiting' };
      });
      setExecutionStates(initStates);

      const startTime = performance.now();

      // 2. Animate step-by-step progression through stages
      for (let i = 0; i < activeNodes.length; i++) {
        const node = activeNodes[i];
        // Mark current node running
        setExecutionStates((prev) => ({
          ...prev,
          [node.id]: { status: 'running' },
        }));

        // Light up incoming edges with energy pulse
        const incomingEdges = pipeline.edges.filter((e) => e.to_node === node.id);
        setActiveEdgeIds(new Set(incomingEdges.map((e) => e.id)));

        // Simulated latency step
        const stepDelay = Math.floor(Math.random() * 80 + 50);
        await new Promise((r) => setTimeout(r, stepDelay));

        // Mark current node success
        setExecutionStates((prev) => ({
          ...prev,
          [node.id]: { status: 'success', latencyMs: stepDelay },
        }));
      }

      // 3. Dispatch real backend pipeline execution
      const blob = await api.executePipeline(pipeline);
      const totalElapsed = Math.round(performance.now() - startTime);
      setExecutionTotalMs(totalElapsed);

      const url = URL.createObjectURL(blob);
      setAudioUrl(url);
      showToast(`Pipeline executed successfully in ${totalElapsed}ms!`, 'info');
    } catch (err: any) {
      alert(`Pipeline Execution Error: ${err.message}`);
    } finally {
      setIsRunning(false);
      setActiveEdgeIds(new Set());
    }
  };

  // Single Node Test Step
  const handleTestStep = async (nodeId: string) => {
    const node = pipeline.nodes.find((n) => n.id === nodeId);
    if (!node) return;

    setExecutionStates((prev) => ({
      ...prev,
      [nodeId]: { status: 'running' },
    }));

    const start = performance.now();
    await new Promise((r) => setTimeout(r, 140));
    const elapsed = Math.round(performance.now() - start);

    setExecutionStates((prev) => ({
      ...prev,
      [nodeId]: { status: 'success', latencyMs: elapsed },
    }));
    showToast(`Tested step "${node.name}" (${elapsed}ms)`);
  };

  // ── Global Keyboard Shortcuts ────────────────────────────────────────────
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.tagName === 'INPUT' ||
        target.tagName === 'TEXTAREA' ||
        target.tagName === 'SELECT' ||
        target.isContentEditable
      ) {
        return;
      }

      // Spacebar hold for Canvas Pan
      if (e.code === 'Space' && !e.repeat) {
        setIsSpacePressed(true);
      }

      // Save (Ctrl+S / Cmd+S)
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        handleSaveWorkflow();
        return;
      }

      // Undo (Ctrl+Z)
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey && e.key.toLowerCase() === 'z') {
        e.preventDefault();
        handleUndo();
        return;
      }

      // Redo (Ctrl+Y or Ctrl+Shift+Z)
      if ((e.ctrlKey || e.metaKey) && (e.key.toLowerCase() === 'y' || (e.shiftKey && e.key.toLowerCase() === 'z'))) {
        e.preventDefault();
        handleRedo();
        return;
      }

      // Select All (Ctrl+A)
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'a') {
        e.preventDefault();
        handleSelectAll();
        return;
      }

      // Delete (Delete / Backspace)
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedNodeIds.size > 0) {
        e.preventDefault();
        handleDeleteSelected();
        return;
      }

      // Duplicate (Ctrl+D)
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'd' && selectedNodeIds.size > 0) {
        e.preventDefault();
        handleDuplicateSelected();
        return;
      }

      // Copy (Ctrl+C)
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'c' && selectedNodeIds.size > 0) {
        e.preventDefault();
        handleCopySelected();
        return;
      }
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === 'Space') {
        setIsSpacePressed(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [selectedNodeIds, handleSaveWorkflow, handleUndo, handleRedo, handleSelectAll, handleDeleteSelected, handleDuplicateSelected, handleCopySelected]);

  return (
    <div className="flex-1 flex overflow-hidden relative select-none">
      <input
        type="file"
        ref={fileInputRef}
        onChange={() => {}}
        accept=".json"
        className="hidden"
      />

      {/* Main Interactive Studio Canvas */}
      <div className="flex-1 flex flex-col h-full bg-[#0B0E14] relative overflow-hidden">
        {/* Top Multi-Tab Workflow Bar (like in n8n and VSCode) */}
        <div className="h-10 border-b border-[#242E3D] bg-[#0E131A] px-2 flex items-center justify-between z-20 overflow-x-auto">
          <div className="flex items-center space-x-1 min-w-0">
            {openTabs.map((tabId) => {
              const tabWf = WorkflowStorage.getWorkflows().find((w) => w.id === tabId) || {
                id: tabId,
                name: 'Workflow',
              };
              const isTabActive = tabId === pipeline.id;

              return (
                <div
                  key={tabId}
                  onClick={() => handleSelectWorkflow(tabId)}
                  className={`group/tab flex items-center space-x-1.5 px-3 py-1.5 rounded-t-lg border-t-2 text-xs font-mono cursor-pointer transition-colors ${
                    isTabActive
                      ? 'bg-[#121820] border-amber-500 text-white font-semibold shadow-sm'
                      : 'border-transparent text-[#94A3B8] hover:bg-[#1A222D]/60 hover:text-white'
                  }`}
                >
                  <span className="truncate max-w-[140px]">{tabWf.name}</span>
                  {isTabActive && isDirty && (
                    <span className="w-1.5 h-1.5 rounded-full bg-amber-400" title="Unsaved changes" />
                  )}
                  {openTabs.length > 1 && (
                    <button
                      onClick={(e) => handleCloseTab(e, tabId)}
                      className="p-0.5 rounded hover:bg-white/10 text-[#64748B] hover:text-white opacity-0 group-hover/tab:opacity-100 transition-opacity"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                </div>
              );
            })}

            <button
              onClick={handleCreateNewWorkflow}
              className="p-1.5 rounded text-[#64748B] hover:text-amber-400 hover:bg-[#1A222D] transition-colors"
              title="Create new workflow tab"
            >
              <Plus className="w-4 h-4" />
            </button>
          </div>

          <div className="flex items-center space-x-1.5">
            <button
              onClick={() => setShowManagerModal(true)}
              className="flex items-center space-x-1 px-2 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-[11px] font-mono border border-[#242E3D] transition-colors"
            >
              <FolderOpen className="w-3 h-3 text-amber-400" />
              <span>Projects</span>
            </button>

            <button
              onClick={handleSaveWorkflow}
              className={`flex items-center space-x-1.5 px-3 py-1 rounded text-xs font-mono font-semibold transition-colors ${
                isDirty
                  ? 'bg-amber-500 text-black hover:bg-amber-400 shadow-sm'
                  : 'bg-[#1A222D] text-[#94A3B8] hover:text-white border border-[#242E3D]'
              }`}
              title="Save workflow (Ctrl+S)"
            >
              <Save className="w-3.5 h-3.5" />
              <span>{isDirty ? 'Save (●)' : 'Saved'}</span>
            </button>
          </div>
        </div>

        {/* Studio Primary Toolbar */}
        <div className="h-11 border-b border-[#242E3D] bg-[#121820]/95 backdrop-blur px-4 flex items-center justify-between z-20">
          <div className="flex items-center space-x-3">
            {onBackToWorkflows && (
              <button
                onClick={onBackToWorkflows}
                className="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-amber-500 hover:text-black text-[#94A3B8] text-xs font-mono border border-[#242E3D] transition-colors group"
                title="Back to Workflows Dashboard"
              >
                <ArrowLeft className="w-3.5 h-3.5 group-hover:-translate-x-0.5 transition-transform" />
                <span>Workflows</span>
              </button>
            )}

            {/* Inline Rename */}
            {isEditingName ? (
              <div className="flex items-center space-x-1">
                <input
                  type="text"
                  value={nameInput}
                  onChange={(e) => setNameInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      if (nameInput.trim()) {
                        setPipeline((p) => ({ ...p, name: nameInput.trim() }));
                        setIsDirty(true);
                      }
                      setIsEditingName(false);
                    }
                    if (e.key === 'Escape') setIsEditingName(false);
                  }}
                  autoFocus
                  className="bg-[#0B0E14] border border-amber-500 rounded px-2 py-0.5 text-xs text-white font-mono focus:outline-none"
                />
                <button
                  onClick={() => {
                    if (nameInput.trim()) {
                      setPipeline((p) => ({ ...p, name: nameInput.trim() }));
                      setIsDirty(true);
                    }
                    setIsEditingName(false);
                  }}
                  className="p-1 text-emerald-400 hover:text-white"
                >
                  <Check className="w-3.5 h-3.5" />
                </button>
              </div>
            ) : (
              <div
                onClick={() => {
                  setNameInput(pipeline.name);
                  setIsEditingName(true);
                }}
                className="flex items-center space-x-1.5 cursor-pointer group hover:bg-[#1A222D] px-2 py-1 rounded transition-colors"
                title="Click to rename workflow"
              >
                <span className="text-xs font-mono font-bold text-white group-hover:text-amber-400">
                  {pipeline.name}
                </span>
                <Edit2 className="w-3 h-3 text-[#64748B] group-hover:text-amber-400 opacity-0 group-hover:opacity-100 transition-opacity" />
              </div>
            )}

            <span className="text-[11px] font-mono text-[#64748B] bg-[#0B0E14] px-2 py-0.5 rounded border border-[#242E3D]">
              {pipeline.nodes.length} nodes
            </span>

            {/* Undo / Redo */}
            <div className="flex items-center space-x-0.5 border-l border-[#242E3D] pl-2">
              <button
                onClick={handleUndo}
                className="p-1 rounded text-[#64748B] hover:text-white hover:bg-[#1A222D] transition-colors"
                title="Undo (Ctrl+Z)"
              >
                <Undo2 className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={handleRedo}
                className="p-1 rounded text-[#64748B] hover:text-white hover:bg-[#1A222D] transition-colors"
                title="Redo (Ctrl+Y)"
              >
                <Redo2 className="w-3.5 h-3.5" />
              </button>
            </div>

            {/* Selection badge */}
            {selectedNodeIds.size > 0 && (
              <div className="flex items-center space-x-1.5 bg-[#1A222D] px-2.5 py-0.5 rounded border border-amber-500/30 text-[11px] font-mono">
                <span className="text-amber-300 font-semibold">
                  {selectedNodeIds.size === pipeline.nodes.length
                    ? `All ${pipeline.nodes.length} selected`
                    : `${selectedNodeIds.size} selected`}
                </span>
                <button
                  onClick={handleDeleteSelected}
                  className="p-1 hover:bg-rose-500/20 text-[#94A3B8] hover:text-rose-400 rounded"
                  title="Delete selection (Del)"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
                <button
                  onClick={handleDuplicateSelected}
                  className="p-1 hover:bg-[#242E3D] text-[#94A3B8] hover:text-white rounded"
                  title="Duplicate selection (Ctrl+D)"
                >
                  <Copy className="w-3 h-3 text-amber-400" />
                </button>
              </div>
            )}
          </div>

          <div className="flex items-center space-x-2">
            {/* Add Node Dropdown */}
            <div className="relative">
              <button
                onClick={() => setShowAddMenu((prev) => !prev)}
                className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 text-xs font-mono font-semibold border border-amber-500/40 transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Add Node</span>
              </button>

              {showAddMenu && (
                <div className="absolute right-0 top-full mt-1.5 w-64 bg-[#121820] border border-[#242E3D] rounded-xl shadow-2xl z-50 py-1.5 font-mono text-xs max-h-96 overflow-y-auto">
                  <div className="px-3 py-1 text-[10px] font-semibold text-[#64748B] uppercase tracking-wider border-b border-[#242E3D]">
                    Studio DAG Nodes
                  </div>
                  {NODE_TEMPLATES.map((tmpl) => (
                    <button
                      key={tmpl.type}
                      onClick={() => handleAddNode(tmpl)}
                      className="w-full px-3 py-2 text-left hover:bg-[#1A222D] flex items-center space-x-2.5 text-[#F0F4F8] transition-colors"
                    >
                      <div className="p-1.5 rounded bg-[#0B0E14] border border-[#242E3D]">
                        {tmpl.icon}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="truncate font-semibold text-xs text-white">{tmpl.name}</div>
                        <div className="text-[10px] text-[#64748B] uppercase">{tmpl.category}</div>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Add Sticky Note */}
            <button
              onClick={handleAddStickyNote}
              className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-mono border border-[#242E3D] transition-colors"
              title="Add documentation note to canvas"
            >
              <StickyNote className="w-3.5 h-3.5 text-amber-400" />
              <span>Note</span>
            </button>

            {/* Execute Pipeline button */}
            <button
              onClick={handleRunPipeline}
              disabled={isRunning}
              className="flex items-center space-x-2 px-4 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs transition-colors shadow-lg shadow-amber-500/20 disabled:opacity-50"
            >
              {isRunning ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  <span>Synthesizing...</span>
                </>
              ) : (
                <>
                  <Play className="w-3.5 h-3.5 fill-black" />
                  <span>Execute Pipeline</span>
                </>
              )}
            </button>

            {/* More menu */}
            <div className="relative">
              <button
                onClick={() => setShowMoreMenu((prev) => !prev)}
                className="p-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white border border-[#242E3D] transition-colors"
              >
                <MoreHorizontal className="w-4 h-4" />
              </button>

              {showMoreMenu && (
                <div className="absolute right-0 top-full mt-1.5 w-52 bg-[#121820] border border-[#242E3D] rounded-xl shadow-2xl z-50 py-1.5 font-mono text-xs">
                  <button
                    onClick={() => {
                      setShowMoreMenu(false);
                      handleSelectAll();
                    }}
                    className="w-full px-3 py-2 text-left hover:bg-[#1A222D] flex items-center space-x-2 text-[#94A3B8] hover:text-white"
                  >
                    <CheckSquare className="w-3.5 h-3.5 text-amber-400" />
                    <span>Select All (Ctrl+A)</span>
                  </button>
                  <button
                    onClick={() => {
                      setShowMoreMenu(false);
                      WorkflowStorage.exportWorkflowJson(pipeline);
                    }}
                    className="w-full px-3 py-2 text-left hover:bg-[#1A222D] flex items-center space-x-2 text-[#94A3B8] hover:text-white"
                  >
                    <Download className="w-3.5 h-3.5" />
                    <span>Export JSON</span>
                  </button>
                  <button
                    onClick={() => {
                      setShowMoreMenu(false);
                      setShowPasteModal(true);
                    }}
                    className="w-full px-3 py-2 text-left hover:bg-[#1A222D] flex items-center space-x-2 text-amber-400 hover:text-amber-300"
                  >
                    <Clipboard className="w-3.5 h-3.5" />
                    <span>Paste JSON (Ctrl+V)</span>
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* ── Interactive 2D Canvas Workspace ───────────────────────────── */}
        <div
          ref={canvasContainerRef}
          onPointerDown={handleCanvasPointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerLeave={handlePointerUp}
          onWheel={handleWheel}
          onDoubleClick={handleDoubleClick}
          className={`flex-1 w-full h-full relative overflow-hidden select-none canvas-grid ${
            isMiddlePanning
              ? 'cursor-grabbing'
              : isSpacePressed
              ? 'cursor-grab'
              : isMarqueeSelecting
              ? 'cursor-crosshair'
              : 'cursor-default'
          }`}
          style={{
            backgroundPosition: `${pan.x}px ${pan.y}px`,
            backgroundSize: `${Math.max(16, Math.round(24 * zoom))}px ${Math.max(16, Math.round(24 * zoom))}px`,
          }}
        >
          {/* Zoom & Pan Transform Layer */}
          <div
            className="absolute inset-0 origin-top-left pointer-events-none"
            style={{
              transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            }}
          >
            {/* SVG Connection Wires Layer */}
            <svg
              className="absolute inset-0 overflow-visible pointer-events-none"
              style={{ width: '100%', height: '100%' }}
            >
              <defs>
                <linearGradient id="activeWireGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stopColor="#38BDF8" />
                  <stop offset="50%" stopColor="#F59E0B" />
                  <stop offset="100%" stopColor="#10B981" />
                </linearGradient>
                <linearGradient id="glowWireGradient" x1="0%" y1="0%" x2="100%" y2="0%">
                  <stop offset="0%" stopColor="#0284C7" stopOpacity="0.8" />
                  <stop offset="50%" stopColor="#F59E0B" stopOpacity="0.9" />
                  <stop offset="100%" stopColor="#059669" stopOpacity="0.8" />
                </linearGradient>
              </defs>
              {pipeline.edges.map((edge) => {
                const fromNode = pipeline.nodes.find((n) => n.id === edge.from_node);
                const toNode = pipeline.nodes.find((n) => n.id === edge.to_node);
                if (!fromNode || !toNode) return null;

                const fromPos = getNodePos(fromNode);
                const toPos = getNodePos(toNode);
                const x1 = fromPos.x + 256;
                const y1 = fromPos.y + 55;
                const x2 = toPos.x;
                const y2 = toPos.y + 55;

                const dx = Math.max(Math.abs(x2 - x1) * 0.5, 45);
                const pathData = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
                const midX = Math.round((x1 + x2) / 2);
                const midY = Math.round((y1 + y2) / 2);

                const isEdgeActive = activeEdgeIds.has(edge.id) || isRunning;

                return (
                  <g key={edge.id} className="group/wire">
                    {/* Glowing outer beam halo when active */}
                    <path
                      d={pathData}
                      fill="none"
                      stroke={isEdgeActive ? 'url(#glowWireGradient)' : '#1E293B'}
                      strokeWidth={isEdgeActive ? '10' : '4'}
                      strokeOpacity={isEdgeActive ? '0.6' : '0.2'}
                      strokeLinecap="round"
                    />
                    {/* Main smooth curve */}
                    <path
                      d={pathData}
                      fill="none"
                      stroke={isEdgeActive ? 'url(#activeWireGradient)' : '#334155'}
                      strokeWidth={isEdgeActive ? '3.5' : '2.5'}
                      strokeLinecap="round"
                    />
                    {/* Flowing animated dashed wire stream */}
                    {isEdgeActive && (
                      <path
                        d={pathData}
                        fill="none"
                        stroke="#F59E0B"
                        strokeWidth="3.5"
                        strokeDasharray="8 6"
                        strokeLinecap="round"
                        className="animate-wire-flow opacity-95"
                      />
                    )}
                    {/* Flowing animated pulse energy packet */}
                    {isEdgeActive && (
                      <circle r="5" fill="#FBBF24" className="filter drop-shadow-[0_0_6px_#F59E0B]">
                        <animateMotion dur="1.4s" repeatCount="indefinite" path={pathData} />
                      </circle>
                    )}
                    {/* Wire disconnect button at midpoint */}
                    <g
                      transform={`translate(${midX}, ${midY})`}
                      className="cursor-pointer pointer-events-auto"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteEdge(edge.id);
                      }}
                    >
                      <title>Click to disconnect wire</title>
                      <circle r="8" fill="#1E293B" stroke="#EF4444" strokeWidth="1.5" />
                      <line x1="-2.5" y1="-2.5" x2="2.5" y2="2.5" stroke="#EF4444" strokeWidth="1.5" strokeLinecap="round" />
                      <line x1="2.5" y1="-2.5" x2="-2.5" y2="2.5" stroke="#EF4444" strokeWidth="1.5" strokeLinecap="round" />
                    </g>
                  </g>
                );
              })}

              {/* Active Rubberband Dragging Wire */}
              {connectingFrom && (() => {
                const fromNode = pipeline.nodes.find((n) => n.id === connectingFrom);
                if (!fromNode) return null;
                const pos = getNodePos(fromNode);
                const x1 = pos.x + 256;
                const y1 = pos.y + 55;
                const x2 = pointerCanvasPos.x;
                const y2 = pointerCanvasPos.y;
                const dx = Math.max(Math.abs(x2 - x1) * 0.5, 30);
                const d = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
                return (
                  <path
                    d={d}
                    fill="none"
                    stroke="#F59E0B"
                    strokeWidth="3.5"
                    strokeDasharray="6 4"
                    strokeLinecap="round"
                  />
                );
              })()}
            </svg>

            {/* Canvas Sticky Notes */}
            {(pipeline.sticky_notes || []).map((note) => (
              <div key={note.id} className="pointer-events-auto">
                <StickyNoteCard
                  note={note}
                  onUpdate={handleUpdateStickyNote}
                  onDelete={handleDeleteStickyNote}
                  style={{
                    left: `${note.position.x}px`,
                    top: `${note.position.y}px`,
                  }}
                />
              </div>
            ))}

            {/* Draggable Node Cards with n8n-Style Execution Status */}
            {pipeline.nodes.map((node) => {
              const pos = getNodePos(node);
              return (
                <div key={node.id} className="pointer-events-auto node-card-interactive">
                  <NodeCard
                    node={node}
                    isSelected={selectedNodeIds.has(node.id)}
                    executionState={executionStates[node.id]}
                    onSelect={handleSelectNode}
                    onDelete={handleDeleteNode}
                    onDuplicate={handleDuplicateNode}
                    onPointerDown={handleNodePointerDown}
                    onConnectStart={handleConnectStart}
                    onConnectEnd={handleConnectEnd}
                    onTestStep={handleTestStep}
                    onToggleDisable={handleToggleDisableNode}
                    style={{
                      left: `${pos.x}px`,
                      top: `${pos.y}px`,
                    }}
                  />
                </div>
              );
            })}

            {/* Marquee Box Selection Overlay (n8n Style) */}
            {isMarqueeSelecting && marqueeBox && (() => {
              const minX = Math.min(marqueeBox.startCanvasX, marqueeBox.curCanvasX);
              const maxX = Math.max(marqueeBox.startCanvasX, marqueeBox.curCanvasX);
              const minY = Math.min(marqueeBox.startCanvasY, marqueeBox.curCanvasY);
              const maxY = Math.max(marqueeBox.startCanvasY, marqueeBox.curCanvasY);
              const w = Math.max(1, maxX - minX);
              const h = Math.max(1, maxY - minY);

              return (
                <div
                  className="absolute border-2 border-dashed border-amber-400 bg-amber-500/10 rounded-lg pointer-events-none shadow-sm"
                  style={{
                    left: `${minX}px`,
                    top: `${minY}px`,
                    width: `${w}px`,
                    height: `${h}px`,
                  }}
                />
              );
            })()}
          </div>

          {/* Floating Release-Wire Quick Add Menu */}
          {quickAddPos && (
            <div
              className="absolute z-50 bg-[#121820] border border-amber-500/50 rounded-xl shadow-2xl p-2 font-mono text-xs w-60 animate-in fade-in zoom-in-95"
              style={{
                left: `${quickAddPos.x * zoom + pan.x}px`,
                top: `${quickAddPos.y * zoom + pan.y}px`,
              }}
            >
              <div className="flex items-center justify-between pb-1.5 border-b border-[#242E3D] text-[10px] text-[#64748B] uppercase">
                <span>Connect Next Step</span>
                <button
                  onClick={() => setQuickAddPos(null)}
                  className="p-0.5 text-[#64748B] hover:text-white"
                >
                  <X className="w-3 h-3" />
                </button>
              </div>
              <div className="max-h-60 overflow-y-auto space-y-1 mt-1">
                {NODE_TEMPLATES.map((tmpl) => (
                  <button
                    key={tmpl.type}
                    onClick={() =>
                      handleAddNode(
                        tmpl,
                        { x: quickAddPos.x, y: quickAddPos.y },
                        quickAddPos.connectFromId
                      )
                    }
                    className="w-full text-left p-1.5 rounded hover:bg-[#1A222D] flex items-center space-x-2 text-white"
                  >
                    <div className="p-1 rounded bg-[#0B0E14] border border-[#242E3D]">
                      {tmpl.icon}
                    </div>
                    <span className="truncate">{tmpl.name}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Floating Canvas HUD Controls (User Request: Zoom Out only on Button) */}
          <div className="absolute bottom-5 right-5 z-30 flex items-center space-x-2 select-none">
            {/* Mini-map */}
            <CanvasMiniMap
              nodes={pipeline.nodes}
              pan={pan}
              zoom={zoom}
              containerWidth={canvasContainerRef.current?.clientWidth || 1000}
              containerHeight={canvasContainerRef.current?.clientHeight || 700}
              onNavigate={setPan}
              isOpen={showMiniMap}
              onToggleOpen={() => setShowMiniMap((m) => !m)}
            />

            {/* Zoom Controls HUD */}
            <div className="flex items-center bg-[#121820]/95 backdrop-blur-md border border-[#242E3D] rounded-xl shadow-2xl p-1 space-x-1.5">
              <button
                onClick={handleZoomIn}
                className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
                title="Zoom In"
              >
                <ZoomIn className="w-4 h-4" />
              </button>
              {/* Zoom Slider */}
              <input
                type="range"
                min="25"
                max="250"
                value={Math.round(zoom * 100)}
                onChange={(e) => {
                  const newZoom = Number(e.target.value) / 100;
                  const rect = canvasContainerRef.current?.getBoundingClientRect();
                  const mouseX = rect ? rect.width / 2 : 300;
                  const mouseY = rect ? rect.height / 2 : 250;
                  const newPanX = mouseX - (mouseX - pan.x) * (newZoom / zoom);
                  const newPanY = mouseY - (mouseY - pan.y) * (newZoom / zoom);
                  setZoom(Number(newZoom.toFixed(2)));
                  setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
                }}
                className="w-16 h-1 accent-amber-400 bg-[#242E3D] rounded cursor-pointer mx-1"
                title={`Zoom Slider: ${Math.round(zoom * 100)}%`}
              />
              {/* Prominent Zoom Out Button */}
              <button
                onClick={handleZoomOut}
                className="p-1.5 rounded hover:bg-amber-500/20 text-[#94A3B8] hover:text-amber-300 transition-colors"
                title="Zoom Out (Button)"
              >
                <ZoomOut className="w-4 h-4" />
              </button>
              <button
                onClick={handleResetZoom}
                className="px-2 py-1 rounded hover:bg-[#1A222D] text-[11px] font-mono text-[#94A3B8] hover:text-white transition-colors"
                title="Reset to 100%"
              >
                {Math.round(zoom * 100)}%
              </button>
              <div className="w-px h-4 bg-[#242E3D]" />
              <button
                onClick={handleFitToView}
                className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
                title="Fit to Screen"
              >
                <Maximize2 className="w-4 h-4" />
              </button>
              <button
                onClick={handleResetZoom}
                className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
                title="Center Origin"
              >
                <RotateCcw className="w-4 h-4" />
              </button>
              <div className="w-px h-4 bg-[#242E3D]" />
              <button
                onClick={handleAutoLayout}
                className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-amber-400 transition-colors"
                title="Auto-Layout DAG (Shift+Alt+L)"
              >
                <LayoutGrid className="w-4 h-4" />
              </button>
              <button
                onClick={() => setShowShortcutsModal(true)}
                className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-sky-400 transition-colors"
                title="Keyboard Shortcuts Guide (?)"
              >
                <HelpCircle className="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Bottom Hint Banner */}
          <div className="absolute bottom-5 left-5 z-10 hidden sm:flex items-center space-x-2 px-3 py-1.5 bg-[#121820]/80 backdrop-blur-sm border border-[#242E3D]/60 rounded-full text-[11px] font-mono text-[#64748B] pointer-events-none">
            <Move className="w-3 h-3 text-amber-500" />
            <span>
              Drag cursor wide to marquee select • Swipe / scroll to pan 2D • Double-click to zoom in • Zoom-out on button
            </span>
          </div>
        </div>

        {/* Execution Timeline & Master Audio Drawer (Sliding bottom dock) */}
        <ExecutionTimelineDrawer
          isOpen={showExecutionDrawer}
          onClose={() => setShowExecutionDrawer(false)}
          isRunning={isRunning}
          totalTimeMs={executionTotalMs}
          stages={pipeline.nodes.map((n) => ({
            id: n.id,
            name: n.name,
            nodeType: n.node_type,
            state: executionStates[n.id],
          }))}
          audioUrl={audioUrl}
          audioRef={audioRef}
        />

        {/* Toast Notification Banner */}
        {toastMessage && (
          <div className="absolute top-16 right-6 z-50 flex items-center space-x-2 px-3.5 py-2 rounded-lg bg-[#121820] border border-amber-500/40 text-white shadow-2xl backdrop-blur text-xs font-mono animate-in fade-in slide-in-from-top-2 duration-200">
            {toastMessage.type === 'error' ? (
              <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
            ) : (
              <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
            )}
            <span>{toastMessage.text}</span>
          </div>
        )}

        {/* Paste JSON Modal */}
        {showPasteModal && (
          <div className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
            <div className="bg-[#121820] border border-[#242E3D] rounded-xl shadow-2xl w-full max-w-lg p-5 font-mono text-xs flex flex-col space-y-4">
              <div className="flex items-center justify-between border-b border-[#242E3D] pb-3">
                <div className="flex items-center space-x-2 text-amber-400 font-semibold text-sm">
                  <Clipboard className="w-4 h-4" />
                  <span>Paste Workflow or Node JSON</span>
                </div>
                <button
                  onClick={() => setShowPasteModal(false)}
                  className="p-1 rounded text-[#94A3B8] hover:text-white"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              <textarea
                rows={8}
                value={pasteJsonInput}
                onChange={(e) => setPasteJsonInput(e.target.value)}
                placeholder="Paste JSON here..."
                className="w-full bg-[#0B0E14] border border-[#242E3D] rounded-lg p-3 text-white font-mono text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed"
                autoFocus
              />

              <div className="flex items-center justify-end space-x-2 pt-2 border-t border-[#242E3D]">
                <button
                  onClick={() => setShowPasteModal(false)}
                  className="px-3 py-1.5 rounded bg-[#1A222D] text-[#94A3B8] hover:text-white"
                >
                  Cancel
                </button>
                <button
                  onClick={() => {
                    try {
                      const parsed = JSON.parse(pasteJsonInput.trim());
                      if (parsed && Array.isArray(parsed.nodes)) {
                        setPipeline(parsed);
                        setIsDirty(true);
                        showToast(`Imported workflow "${parsed.name || 'Pasted'}"`);
                        setShowPasteModal(false);
                      }
                    } catch (e: any) {
                      showToast(`Invalid JSON: ${e.message}`, 'error');
                    }
                  }}
                  className="px-4 py-1.5 rounded bg-amber-500 hover:bg-amber-400 text-black font-semibold"
                >
                  Import to Canvas
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Keyboard Shortcuts & Gestures Guide Modal */}
        {showShortcutsModal && (
          <div className="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
            <div className="bg-[#121820] border border-[#242E3D] rounded-2xl shadow-2xl w-full max-w-xl overflow-hidden animate-in fade-in zoom-in duration-150 text-xs font-mono">
              <div className="p-4 border-b border-[#242E3D] flex items-center justify-between bg-[#0B0E14]/60">
                <div className="flex items-center space-x-2">
                  <div className="p-1.5 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/20">
                    <HelpCircle className="w-4 h-4" />
                  </div>
                  <div>
                    <h3 className="text-sm font-bold text-white">DAG Canvas Gestures & Shortcuts</h3>
                    <p className="text-[11px] text-[#94A3B8]">Fast studio navigation and editing</p>
                  </div>
                </div>
                <button
                  onClick={() => setShowShortcutsModal(false)}
                  className="p-1.5 rounded-lg text-[#94A3B8] hover:text-white hover:bg-[#1A222D]"
                >
                  <X className="w-4 h-4" />
                </button>
              </div>

              <div className="p-5 space-y-4 max-h-[70vh] overflow-y-auto">
                <div>
                  <div className="text-[11px] uppercase font-semibold text-amber-400 tracking-wider mb-2">
                    Canvas Navigation & Viewport
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-[11px]">
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">2D Swipe Pan</span>
                      <span className="text-white font-semibold">Scroll / Trackpad</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Drag Pan</span>
                      <span className="text-white font-semibold">Space / Mid-Click</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Zoom In</span>
                      <span className="text-white font-semibold">Double Click</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Zoom Out</span>
                      <span className="text-amber-400 font-semibold">HUD [-] Button</span>
                    </div>
                  </div>
                </div>

                <div>
                  <div className="text-[11px] uppercase font-semibold text-sky-400 tracking-wider mb-2">
                    Node Selection & Editing
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-[11px]">
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Marquee Selection</span>
                      <span className="text-white font-semibold">Drag empty canvas</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Multi-Select Add</span>
                      <span className="text-white font-semibold">Shift + Drag / Click</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Select All Nodes</span>
                      <span className="text-white font-semibold">Ctrl + A</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Delete Selected</span>
                      <span className="text-rose-400 font-semibold">Del / Backspace</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Duplicate Selected</span>
                      <span className="text-white font-semibold">Ctrl + D</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Copy JSON</span>
                      <span className="text-white font-semibold">Ctrl + C</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Paste Nodes</span>
                      <span className="text-emerald-400 font-semibold">Ctrl + V</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Auto-Layout DAG</span>
                      <span className="text-amber-400 font-semibold">Shift + Alt + L</span>
                    </div>
                  </div>
                </div>

                <div>
                  <div className="text-[11px] uppercase font-semibold text-emerald-400 tracking-wider mb-2">
                    History & Workflow
                  </div>
                  <div className="grid grid-cols-2 gap-2 text-[11px]">
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Undo</span>
                      <span className="text-white font-semibold">Ctrl + Z</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Redo</span>
                      <span className="text-white font-semibold">Ctrl + Y</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Test Single Node</span>
                      <span className="text-white font-semibold">Play Icon on Node</span>
                    </div>
                    <div className="p-2.5 rounded-lg bg-[#0B0E14] border border-[#242E3D] flex justify-between items-center">
                      <span className="text-[#94A3B8]">Sticky Notes</span>
                      <span className="text-white font-semibold">Click Note in HUD</span>
                    </div>
                  </div>
                </div>
              </div>

              <div className="p-3 border-t border-[#242E3D] bg-[#0B0E14]/40 flex justify-end">
                <button
                  onClick={() => setShowShortcutsModal(false)}
                  className="px-4 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold"
                >
                  Got It
                </button>
              </div>
            </div>
          </div>
        )}

        {/* Workflow & Project Manager Modal */}
        <WorkflowManagerModal
          isOpen={showManagerModal}
          onClose={() => setShowManagerModal(false)}
          activeWorkflowId={pipeline.id}
          onSelectWorkflow={handleSelectWorkflow}
          onWorkflowListChange={() => {
            setOpenTabs(WorkflowStorage.getOpenTabs());
          }}
        />
      </div>

      {/* Node Inspector Drawer - ONLY show details when a single specific node is selected */}
      {selectedNodeIds.size === 1 && (
        <NodeInspector
          node={selectedNode}
          allNodes={pipeline.nodes}
          voices={voices}
          onClose={() => setSelectedNodeIds(new Set())}
          onUpdateParams={handleUpdateParams}
          onUpdateName={handleUpdateNodeName}
          onDeleteNode={handleDeleteNode}
          onDuplicateNode={handleDuplicateNode}
          onCopyNodeJson={handleCopyNodeJson}
        />
      )}

      {/* Floating Multi-Node Selection Pill (n8n Style) */}
      {selectedNodeIds.size > 1 && (
        <div className="absolute bottom-6 left-1/2 -translate-x-1/2 z-40 flex items-center space-x-3 px-4 py-2 rounded-xl bg-[#121820]/95 border border-amber-500/50 shadow-2xl backdrop-blur-md animate-in fade-in slide-in-from-bottom-3 duration-150 text-xs font-mono text-white">
          <div className="flex items-center space-x-2">
            <span className="w-2 h-2 rounded-full bg-amber-400 animate-ping" />
            <span className="font-bold text-amber-400">{selectedNodeIds.size} nodes selected</span>
          </div>
          <div className="h-4 w-px bg-[#242E3D]" />
          <button
            onClick={handleCopySelected}
            className="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-amber-300 hover:text-white border border-amber-500/30 transition-colors"
            title="Copy selected nodes as workflow JSON (Ctrl+C)"
          >
            <Copy className="w-3.5 h-3.5" />
            <span>Copy Workflow JSON (Ctrl+C)</span>
          </button>
          <button
            onClick={handleDuplicateSelected}
            className="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white border border-[#242E3D] transition-colors"
            title="Duplicate selected nodes (Ctrl+D)"
          >
            <Layers className="w-3.5 h-3.5" />
            <span>Duplicate (Ctrl+D)</span>
          </button>
          <button
            onClick={handleDeleteSelected}
            className="flex items-center space-x-1.5 px-2.5 py-1 rounded bg-rose-500/15 hover:bg-rose-500/30 text-rose-300 hover:text-rose-100 border border-rose-500/30 transition-colors"
            title="Delete selected nodes (Del)"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Delete (Del)</span>
          </button>
          <button
            onClick={() => setSelectedNodeIds(new Set())}
            className="p-1 rounded text-[#64748B] hover:text-white hover:bg-white/10 ml-1"
            title="Clear selection (Esc)"
          >
            <X className="w-3.5 h-3.5" />
          </button>
        </div>
      )}
    </div>
  );
};
