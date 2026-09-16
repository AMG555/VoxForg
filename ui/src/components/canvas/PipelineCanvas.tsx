import React, { useState, useRef, useCallback } from 'react';
import {
  Play,
  Loader2,
  Download,
  Upload,
  Sparkles,
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
} from 'lucide-react';
import { PipelineDefinition, PipelineNode, Voice } from '../../types';
import { NodeCard } from './NodeCard';
import { NodeInspector } from './NodeInspector';
import { api } from '../../services/api';
import { AudioVisualizer } from '../common/AudioVisualizer';

interface PipelineCanvasProps {
  voices: Voice[];
}

const PRESETS: Record<string, PipelineDefinition> = {
  narrative: {
    id: 'preset-narrative',
    name: 'Narrative Dialogue Pipeline',
    nodes: [
      {
        id: 'node-1',
        name: 'Input Script',
        node_type: 'text_input',
        params: {
          text: "Captain: We have cleared asteroid belt alpha.\nNav: Coordinates aligned, sir.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'node-2',
        name: 'Speaker Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'node-3',
        name: 'Voice Allocation',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-AriaNeural',
          speaker_map: {
            Captain: 'en-US-GuyNeural',
            Nav: 'en-US-AriaNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'node-4',
        name: 'Speech Synthesizer',
        node_type: 'synthesizer',
        params: {},
        position: { x: 1020, y: 120 },
      },
      {
        id: 'node-5',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 180 },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'node-6',
        name: 'Master Output',
        node_type: 'output_sink',
        params: {},
        position: { x: 1660, y: 120 },
      },
    ],
    edges: [
      { id: 'e1', from_node: 'node-1', to_node: 'node-2' },
      { id: 'e2', from_node: 'node-2', to_node: 'node-3' },
      { id: 'e3', from_node: 'node-3', to_node: 'node-4' },
      { id: 'e4', from_node: 'node-4', to_node: 'node-5' },
      { id: 'e5', from_node: 'node-5', to_node: 'node-6' },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  podcast: {
    id: 'preset-podcast',
    name: 'Multi-Voice Studio Podcast',
    nodes: [
      {
        id: 'p-1',
        name: 'Podcast Interview Script',
        node_type: 'text_input',
        params: {
          text: "Host: Welcome back to Quantum Wave! Today we dive into multi-voice neural speech synthesis in high-concurrency environments.\nGuest: Thanks for having me, Alex! Combining distinct voices on a single directed canvas is a massive leap for studio production.\nHost: Absolutely. With inter-speaker crossfades and dynamic audio mastering, creators can build full podcast episodes in seconds.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'p-2',
        name: 'Speaker Script Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'p-3',
        name: 'Podcast Voice Assigner',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-JennyNeural',
          speaker_map: {
            Host: 'en-US-JennyNeural',
            Guest: 'en-US-GuyNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'p-4',
        name: 'Neural Synthesizer',
        node_type: 'synthesizer',
        params: { speed: 1.0, pitch: 0.0 },
        position: { x: 1020, y: 120 },
      },
      {
        id: 'p-5',
        name: 'Studio DSP Mastering',
        node_type: 'audio_filter',
        params: {
          trim_silence: true,
          silence_threshold_db: -42.0,
          silence_pad_ms: 40,
          enable_eq: true,
          eq_low_gain_db: 1.5,
          eq_mid_gain_db: -0.5,
          eq_high_gain_db: 2.0,
          enable_compressor: true,
          compressor_threshold_db: -18.0,
          compressor_ratio: 2.5,
          enable_limiter: true,
          limiter_ceiling_db: -0.8,
          normalize: true,
        },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'p-6',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 220 },
        position: { x: 1660, y: 120 },
      },
      {
        id: 'p-7',
        name: 'Broadcast Master Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 1980, y: 120 },
      },
    ],
    edges: [
      { id: 'pe1', from_node: 'p-1', to_node: 'p-2' },
      { id: 'pe2', from_node: 'p-2', to_node: 'p-3' },
      { id: 'pe3', from_node: 'p-3', to_node: 'p-4' },
      { id: 'pe4', from_node: 'p-4', to_node: 'p-5' },
      { id: 'pe5', from_node: 'p-5', to_node: 'p-6' },
      { id: 'pe6', from_node: 'p-6', to_node: 'p-7' },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  communications: {
    id: 'preset-comms',
    name: 'Tactical Communications Studio',
    nodes: [
      {
        id: 'c-1',
        name: 'Comms Dispatch Script',
        node_type: 'text_input',
        params: {
          text: "Dispatch: Control to Falcon-1, radar contact lost at vector zero-four-zero. Confirm visual status.\nPilot: Falcon-1 copy. Heavy static layer at ten thousand feet, switching auxiliary frequency now.\nDispatch: Roger Falcon-1. Maintain designated corridor and report telemetry link.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'c-2',
        name: 'Speaker Script Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'c-3',
        name: 'Tactical Voice Allocation',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-AriaNeural',
          speaker_map: {
            Dispatch: 'en-US-AriaNeural',
            Pilot: 'en-US-GuyNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'c-4',
        name: 'Comms Synthesizer',
        node_type: 'synthesizer',
        params: { speed: 1.05, pitch: 0.0 },
        position: { x: 1020, y: 120 },
      },
      {
        id: 'c-5',
        name: 'Radio Bandpass DSP',
        node_type: 'audio_filter',
        params: {
          trim_silence: true,
          silence_threshold_db: -36.0,
          enable_eq: true,
          eq_low_gain_db: -3.0,
          eq_mid_gain_db: 3.5,
          eq_high_gain_db: 2.0,
          enable_compressor: true,
          compressor_threshold_db: -14.0,
          compressor_ratio: 4.0,
          enable_limiter: true,
          limiter_ceiling_db: -0.3,
          normalize: true,
        },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'c-6',
        name: 'Radio Audio Merge',
        node_type: 'audio_merge',
        params: { pause_ms: 140 },
        position: { x: 1660, y: 120 },
      },
      {
        id: 'c-7',
        name: 'Mission Comms Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 1980, y: 120 },
      },
    ],
    edges: [
      { id: 'ce1', from_node: 'c-1', to_node: 'c-2' },
      { id: 'ce2', from_node: 'c-2', to_node: 'c-3' },
      { id: 'ce3', from_node: 'c-3', to_node: 'c-4' },
      { id: 'ce4', from_node: 'c-4', to_node: 'c-5' },
      { id: 'ce5', from_node: 'c-5', to_node: 'c-6' },
      { id: 'ce6', from_node: 'c-6', to_node: 'c-7' },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  radio: {
    id: 'preset-radio',
    name: 'Punchy Radio Broadcaster',
    nodes: [
      {
        id: 'r-1',
        name: 'Station Jingle & Dialogue',
        node_type: 'text_input',
        params: {
          text: "DJ: You are listening to 104.7 VoxFM! Taking our first live caller on the neural hotline.\nCaller: Hey DJ! The audio quality on this station is crystal clear, loving the mix!\nDJ: Appreciate the love, caller! Cranking up non-stop neural audio with zero latency.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'r-2',
        name: 'Speaker Script Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'r-3',
        name: 'Station Voice Assigner',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-GuyNeural',
          speaker_map: {
            DJ: 'en-US-GuyNeural',
            Caller: 'en-US-JennyNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'r-4',
        name: 'Resonant Synthesizer',
        node_type: 'synthesizer',
        params: { voice: 'en-US-GuyNeural' },
        position: { x: 1020, y: 120 },
      },
      {
        id: 'r-5',
        name: 'Aggressive Radio DSP',
        node_type: 'audio_filter',
        params: {
          trim_silence: true,
          silence_threshold_db: -36.0,
          enable_eq: true,
          eq_low_gain_db: 3.5,
          eq_mid_gain_db: 2.0,
          eq_high_gain_db: 3.0,
          enable_compressor: true,
          compressor_threshold_db: -12.0,
          compressor_ratio: 4.0,
          enable_limiter: true,
          limiter_ceiling_db: -0.3,
          normalize: true,
        },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'r-6',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 100 },
        position: { x: 1660, y: 120 },
      },
      {
        id: 'r-7',
        name: 'Transmitter Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 1980, y: 120 },
      },
    ],
    edges: [
      { id: 're1', from_node: 'r-1', to_node: 'r-2' },
      { id: 're2', from_node: 'r-2', to_node: 'r-3' },
      { id: 're3', from_node: 'r-3', to_node: 'r-4' },
      { id: 're4', from_node: 'r-4', to_node: 'r-5' },
      { id: 're5', from_node: 'r-5', to_node: 'r-6' },
      { id: 're6', from_node: 'r-6', to_node: 'r-7' },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
};

const NODE_TEMPLATES = [
  {
    type: 'text_input',
    name: 'Text Script Input',
    category: 'Ingestion',
    icon: <FileText className="w-3.5 h-3.5 text-sky-400" />,
    defaultParams: { text: 'New synthesized line.' },
  },
  {
    type: 'speaker_parser',
    name: 'Speaker Script Parser',
    category: 'Analysis',
    icon: <Users className="w-3.5 h-3.5 text-purple-400" />,
    defaultParams: {},
  },
  {
    type: 'voice_assigner',
    name: 'Voice Assigner',
    category: 'Routing',
    icon: <UserCheck className="w-3.5 h-3.5 text-amber-400" />,
    defaultParams: { default_voice: 'en-US-AriaNeural' },
  },
  {
    type: 'synthesizer',
    name: 'Neural Synthesizer',
    category: 'Synthesis',
    icon: <Cpu className="w-3.5 h-3.5 text-emerald-400" />,
    defaultParams: { speed: 1.0, pitch: 0.0 },
  },
  {
    type: 'audio_filter',
    name: 'DSP Filter Chain',
    category: 'DSP Filter',
    icon: <Sliders className="w-3.5 h-3.5 text-pink-400" />,
    defaultParams: { trim_silence: true, normalize: true },
  },
  {
    type: 'audio_merge',
    name: 'Audio Track Merge',
    category: 'Mastering',
    icon: <Layers className="w-3.5 h-3.5 text-indigo-400" />,
    defaultParams: { pause_ms: 150 },
  },
  {
    type: 'output_sink',
    name: 'Master Audio Sink',
    category: 'Output Sink',
    icon: <Save className="w-3.5 h-3.5 text-rose-400" />,
    defaultParams: { format: 'wav' },
  },
];

const getNodePos = (node: PipelineNode): { x: number; y: number } => {
  return node.position || { x: 100, y: 100 };
};

export const PipelineCanvas: React.FC<PipelineCanvasProps> = ({ voices }) => {
  const [pipeline, setPipeline] = useState<PipelineDefinition>(PRESETS.narrative);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>('node-1');
  const [isRunning, setIsRunning] = useState(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [showAddMenu, setShowAddMenu] = useState(false);

  // 2D Canvas Viewport State (Pan & Zoom)
  const [pan, setPan] = useState<{ x: number; y: number }>({ x: 60, y: 120 });
  const [zoom, setZoom] = useState<number>(1.0);
  const [isPanning, setIsPanning] = useState<boolean>(false);
  const [panStart, setPanStart] = useState<{ x: number; y: number }>({ x: 0, y: 0 });

  // Node Dragging State
  const [dragNodeState, setDragNodeState] = useState<{
    nodeId: string;
    startMouseX: number;
    startMouseY: number;
    startNodeX: number;
    startNodeY: number;
  } | null>(null);

  // Wire Connection Dragging State
  const [connectingFrom, setConnectingFrom] = useState<string | null>(null);
  const [pointerCanvasPos, setPointerCanvasPos] = useState<{ x: number; y: number }>({ x: 0, y: 0 });

  const canvasContainerRef = useRef<HTMLDivElement | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  React.useEffect(() => {
    return () => {
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
      }
    };
  }, [audioUrl]);

  const selectedNode = pipeline.nodes.find((n) => n.id === selectedNodeId) || null;

  const handleUpdateParams = (nodeId: string, params: Record<string, any>) => {
    setPipeline((prev) => ({
      ...prev,
      nodes: prev.nodes.map((n) => (n.id === nodeId ? { ...n, params } : n)),
    }));
  };

  const handleDeleteNode = (nodeId: string) => {
    setPipeline((prev) => ({
      ...prev,
      nodes: prev.nodes.filter((n) => n.id !== nodeId),
      edges: prev.edges.filter((e) => e.from_node !== nodeId && e.to_node !== nodeId),
    }));
    if (selectedNodeId === nodeId) {
      setSelectedNodeId(null);
    }
  };

  const handleDeleteEdge = (edgeId: string) => {
    setPipeline((prev) => ({
      ...prev,
      edges: prev.edges.filter((e) => e.id !== edgeId),
    }));
  };

  const handleAutoLayout = () => {
    const spacingX = 320;
    const startX = 60;
    const startY = 120;
    setPipeline((prev) => ({
      ...prev,
      nodes: prev.nodes.map((n, idx) => ({
        ...n,
        position: { x: startX + idx * spacingX, y: startY },
      })),
    }));
    setTimeout(handleFitToView, 50);
  };

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
        setPipeline((prev) => ({
          ...prev,
          edges: [
            ...prev.edges,
            {
              id: `edge-${Date.now()}`,
              from_node: connectingFrom,
              to_node: targetNodeId,
            },
          ],
        }));
      }
    }
    setConnectingFrom(null);
  };

  // Canvas Panning Handlers
  const handleCanvasPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    // Only pan on background click (button 0 = left, button 1 = middle)
    if (e.button !== 0 && e.button !== 1) return;
    const target = e.target as HTMLElement;
    // Don't pan if clicking directly on a node card or interactive element
    if (target.closest('.node-card-interactive') || target.tagName === 'BUTTON' || target.tagName === 'INPUT') {
      return;
    }

    setIsPanning(true);
    setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
    setSelectedNodeId(null);
    setShowAddMenu(false);
  };

  // Node Drag Start Handler
  const handleNodePointerDown = useCallback((e: React.PointerEvent, nodeId: string) => {
    e.stopPropagation();
    setSelectedNodeId(nodeId);
    setShowAddMenu(false);

    const node = pipeline.nodes.find((n) => n.id === nodeId);
    if (!node) return;

    const pos = getNodePos(node);
    setDragNodeState({
      nodeId,
      startMouseX: e.clientX,
      startMouseY: e.clientY,
      startNodeX: pos.x,
      startNodeY: pos.y,
    });
  }, [pipeline.nodes]);

  // Unified Pointer Move
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

      setPipeline((prev) => ({
        ...prev,
        nodes: prev.nodes.map((n) =>
          n.id === dragNodeState.nodeId
            ? {
                ...n,
                position: {
                  x: Math.round(dragNodeState.startNodeX + dx),
                  y: Math.round(dragNodeState.startNodeY + dy),
                },
              }
            : n
        ),
      }));
    } else if (isPanning) {
      setPan({
        x: e.clientX - panStart.x,
        y: e.clientY - panStart.y,
      });
    }
  };

  // Pointer Up / Cancel
  const handlePointerUp = () => {
    setIsPanning(false);
    setDragNodeState(null);
    setConnectingFrom(null);
  };

  // Smooth Wheel Zoom Centered at Cursor
  const handleWheel = (e: React.WheelEvent<HTMLDivElement>) => {
    e.preventDefault();
    const rect = canvasContainerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    const zoomDelta = e.deltaY < 0 ? 1.12 : 0.89;
    const newZoom = Math.min(Math.max(Number((zoom * zoomDelta).toFixed(3)), 0.25), 2.5);

    // Zoom anchored to mouse position:
    const newPanX = mouseX - (mouseX - pan.x) * (newZoom / zoom);
    const newPanY = mouseY - (mouseY - pan.y) * (newZoom / zoom);

    setZoom(newZoom);
    setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
  };

  // Reset Zoom to 100%
  const handleResetZoom = () => {
    setZoom(1.0);
    setPan({ x: 60, y: 120 });
  };

  // Fit All Nodes in Viewport
  const handleFitToView = () => {
    if (pipeline.nodes.length === 0) return;
    const rect = canvasContainerRef.current?.getBoundingClientRect();
    if (!rect) return;

    const padding = 80;
    const xs = pipeline.nodes.map((n) => getNodePos(n).x);
    const ys = pipeline.nodes.map((n) => getNodePos(n).y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs) + 260; // card width is 256
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys) + 150; // card height ~140

    const graphWidth = maxX - minX;
    const graphHeight = maxY - minY;

    const scaleX = (rect.width - padding * 2) / graphWidth;
    const scaleY = (rect.height - padding * 2) / graphHeight;
    const newZoom = Math.min(Math.max(Math.min(scaleX, scaleY), 0.35), 1.5);

    const newPanX = (rect.width - graphWidth * newZoom) / 2 - minX * newZoom;
    const newPanY = (rect.height - graphHeight * newZoom) / 2 - minY * newZoom;

    setZoom(Number(newZoom.toFixed(2)));
    setPan({ x: Math.round(newPanX), y: Math.round(newPanY) });
  };

  // Add Node from Template
  const handleAddNode = (template: typeof NODE_TEMPLATES[0]) => {
    const rect = canvasContainerRef.current?.getBoundingClientRect();
    const centerX = rect ? (rect.width / 2 - pan.x) / zoom - 128 : 200;
    const centerY = rect ? (rect.height / 2 - pan.y) / zoom - 60 : 150;

    const newNodeId = `node-${Date.now()}`;
    const newNode: PipelineNode = {
      id: newNodeId,
      name: template.name,
      node_type: template.type as any,
      params: { ...template.defaultParams },
      position: { x: Math.round(centerX), y: Math.round(centerY) },
    };

    // Auto connect from currently selected node if present
    let newEdges = [...pipeline.edges];
    if (selectedNodeId && pipeline.nodes.some((n) => n.id === selectedNodeId)) {
      newEdges.push({
        id: `edge-${Date.now()}`,
        from_node: selectedNodeId,
        to_node: newNodeId,
      });
    }

    setPipeline((prev) => ({
      ...prev,
      nodes: [...prev.nodes, newNode],
      edges: newEdges,
    }));
    setSelectedNodeId(newNodeId);
    setShowAddMenu(false);
  };

  const handleRunPipeline = async () => {
    try {
      setIsRunning(true);
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }
      const blob = await api.executePipeline(pipeline);
      const url = URL.createObjectURL(blob);
      setAudioUrl(url);
    } catch (err: any) {
      alert(`Pipeline Execution Error: ${err.message}`);
    } finally {
      setIsRunning(false);
    }
  };

  const handleExportJson = () => {
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(pipeline, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `${pipeline.name.toLowerCase().replace(/\s+/g, '-')}.voxforg.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  };

  const handleImportJson = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (event) => {
      try {
        const content = event.target?.result as string;
        const imported = JSON.parse(content);
        if (Array.isArray(imported.nodes) && Array.isArray(imported.edges)) {
          setPipeline(imported);
          setSelectedNodeId(imported.nodes[0]?.id || null);
          setTimeout(handleFitToView, 50);
        } else {
          alert('Invalid pipeline JSON: Missing nodes or edges array');
        }
      } catch (err: any) {
        alert(`Failed to parse pipeline file: ${err.message}`);
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  return (
    <div className="flex-1 flex overflow-hidden relative">
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleImportJson}
        accept=".json"
        className="hidden"
      />

      {/* Central Interactive Node Canvas */}
      <div className="flex-1 flex flex-col h-full bg-[#0B0E14] relative overflow-hidden">
        {/* Canvas Toolbar */}
        <div className="h-12 border-b border-[#242E3D] bg-[#121820]/90 backdrop-blur px-4 flex items-center justify-between z-20 select-none">
          <div className="flex items-center space-x-3">
            <span className="text-xs font-mono font-bold text-white">{pipeline.name}</span>
            <span className="text-[11px] font-mono text-[#64748B]">
              ({pipeline.nodes.length} nodes, {pipeline.edges.length} edges)
            </span>

            {/* Presets dropdown */}
            <div className="flex items-center space-x-1 pl-3 border-l border-[#242E3D]">
              <Sparkles className="w-3.5 h-3.5 text-amber-500" />
              <select
                onChange={(e) => {
                  const preset = PRESETS[e.target.value];
                  if (preset) {
                    setPipeline(preset);
                    setSelectedNodeId(preset.nodes[0]?.id || null);
                    setTimeout(handleFitToView, 50);
                  }
                }}
                className="bg-[#0B0E14] text-[#94A3B8] border border-[#242E3D] rounded px-2 py-1 text-[11px] font-mono focus:border-amber-500 focus:outline-none cursor-pointer"
                defaultValue="narrative"
              >
                <option value="narrative">Preset: Narrative Dialogue</option>
                <option value="podcast">Preset: Multi-Voice Podcast</option>
                <option value="communications">Preset: Tactical Comms</option>
                <option value="radio">Preset: Punchy Radio</option>
              </select>
            </div>

            {/* Add Node Dropdown */}
            <div className="relative pl-2">
              <button
                onClick={() => setShowAddMenu((prev) => !prev)}
                className="flex items-center space-x-1 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-amber-400 hover:text-amber-300 text-xs font-mono border border-amber-500/30 hover:border-amber-500 transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span>Add Node</span>
              </button>

              {showAddMenu && (
                <div className="absolute left-2 top-full mt-1 w-56 bg-[#121820] border border-[#242E3D] rounded-lg shadow-2xl z-50 py-1 font-mono text-xs">
                  <div className="px-3 py-1.5 text-[10px] font-semibold text-[#64748B] uppercase tracking-wider border-b border-[#242E3D]">
                    Available DAG Nodes
                  </div>
                  {NODE_TEMPLATES.map((tmpl) => (
                    <button
                      key={tmpl.type}
                      onClick={() => handleAddNode(tmpl)}
                      className="w-full px-3 py-2 text-left hover:bg-[#1A222D] flex items-center space-x-2 text-[#F0F4F8] transition-colors"
                    >
                      <div className="p-1 rounded bg-[#0B0E14] border border-[#242E3D]">
                        {tmpl.icon}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="truncate font-semibold text-xs">{tmpl.name}</div>
                        <div className="text-[10px] text-[#64748B] uppercase">{tmpl.category}</div>
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </div>

            {/* Auto-Align studio nodes button */}
            <button
              onClick={handleAutoLayout}
              className="flex items-center space-x-1 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-mono border border-[#242E3D] transition-colors ml-2"
              title="Auto-arrange studio nodes sequentially"
            >
              <Move className="w-3.5 h-3.5 text-amber-500" />
              <span>Auto-Align</span>
            </button>
          </div>

          <div className="flex items-center space-x-2">
            <button
              onClick={handleExportJson}
              className="flex items-center space-x-1 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-mono border border-[#242E3D] transition-colors"
              title="Export DAG to JSON"
            >
              <Download className="w-3.5 h-3.5" />
              <span>Export</span>
            </button>

            <button
              onClick={() => fileInputRef.current?.click()}
              className="flex items-center space-x-1 px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-mono border border-[#242E3D] transition-colors"
              title="Import DAG from JSON"
            >
              <Upload className="w-3.5 h-3.5" />
              <span>Import</span>
            </button>

            {audioUrl && (
              <audio
                ref={audioRef}
                controls
                src={audioUrl}
                onPlay={() => setIsPlaying(true)}
                onPause={() => setIsPlaying(false)}
                onEnded={() => setIsPlaying(false)}
                className="h-7 w-60 rounded bg-[#1A222D]"
              />
            )}

            <button
              onClick={handleRunPipeline}
              disabled={isRunning}
              className="flex items-center space-x-2 px-3.5 py-1.5 rounded bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs transition-colors shadow-lg shadow-amber-500/20 disabled:opacity-50"
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
          </div>
        </div>

        {/* Interactive Infinite Canvas Workspace */}
        <div
          ref={canvasContainerRef}
          onPointerDown={handleCanvasPointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onPointerLeave={handlePointerUp}
          onWheel={handleWheel}
          className={`flex-1 w-full h-full relative overflow-hidden select-none canvas-grid ${
            isPanning ? 'cursor-grabbing' : 'cursor-default'
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
              {pipeline.edges.map((edge) => {
                const fromNode = pipeline.nodes.find((n) => n.id === edge.from_node);
                const toNode = pipeline.nodes.find((n) => n.id === edge.to_node);
                if (!fromNode || !toNode) return null;

                const fromPos = getNodePos(fromNode);
                const toPos = getNodePos(toNode);
                // Card width is 256px (w-64); connector ports are centered on right and left edges
                const x1 = fromPos.x + 256;
                const y1 = fromPos.y + 55;
                const x2 = toPos.x;
                const y2 = toPos.y + 55;

                const dx = Math.max(Math.abs(x2 - x1) * 0.5, 45);
                const pathData = `M ${x1} ${y1} C ${x1 + dx} ${y1}, ${x2 - dx} ${y2}, ${x2} ${y2}`;
                const midX = Math.round((x1 + x2) / 2);
                const midY = Math.round((y1 + y2) / 2);

                return (
                  <g key={edge.id} className="group/wire">
                    {/* Ambient outer glow halo */}
                    <path
                      d={pathData}
                      fill="none"
                      stroke="#0284C7"
                      strokeWidth="8"
                      strokeOpacity="0.3"
                      strokeLinecap="round"
                    />
                    {/* Main smooth cyan bezier curve */}
                    <path
                      d={pathData}
                      fill="none"
                      stroke="#38BDF8"
                      strokeWidth="3"
                      strokeLinecap="round"
                    />
                    {/* Flowing animated pulse marker */}
                    <circle r="4" fill="#F59E0B">
                      <animateMotion dur="2.4s" repeatCount="indefinite" path={pathData} />
                    </circle>
                    {/* Edge delete button at midpoint */}
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

            {/* Draggable Node Cards */}
            {pipeline.nodes.map((node) => {
              const pos = getNodePos(node);
              return (
                <div
                  key={node.id}
                  className="pointer-events-auto node-card-interactive"
                >
                  <NodeCard
                    node={node}
                    isSelected={node.id === selectedNodeId}
                    onSelect={setSelectedNodeId}
                    onDelete={handleDeleteNode}
                    onPointerDown={handleNodePointerDown}
                    onConnectStart={handleConnectStart}
                    onConnectEnd={handleConnectEnd}
                    style={{
                      left: `${pos.x}px`,
                      top: `${pos.y}px`,
                    }}
                  />
                </div>
              );
            })}
          </div>

          {/* Floating Zoom & Pan HUD Controls */}
          <div className="absolute bottom-5 right-5 z-20 flex items-center bg-[#121820]/90 backdrop-blur-md border border-[#242E3D] rounded-lg shadow-2xl p-1 space-x-1 select-none">
            <button
              onClick={() => setZoom((z) => Math.min(Number((z + 0.15).toFixed(2)), 2.5))}
              className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
              title="Zoom In"
            >
              <ZoomIn className="w-4 h-4" />
            </button>
            <button
              onClick={() => setZoom((z) => Math.max(Number((z - 0.15).toFixed(2)), 0.25))}
              className="p-1.5 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
              title="Zoom Out"
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
          </div>

          {/* Bottom Hint Banner */}
          <div className="absolute bottom-5 left-5 z-10 hidden sm:flex items-center space-x-2 px-3 py-1.5 bg-[#121820]/75 backdrop-blur-sm border border-[#242E3D]/60 rounded-full text-[11px] font-mono text-[#64748B] pointer-events-none">
            <Move className="w-3 h-3 text-amber-500" />
            <span>Drag canvas to pan • Scroll to zoom • Drag cards to position</span>
          </div>
        </div>

        {/* Real-time Audio Visualizer Bottom Dock */}
        {audioUrl && (
          <div className="absolute bottom-16 left-5 z-30 w-96 shadow-2xl backdrop-blur-md bg-[#0B0E14]/90 rounded-lg border border-[#242E3D]">
            <AudioVisualizer
              audioElement={audioRef.current}
              isPlaying={isPlaying}
            />
          </div>
        )}
      </div>

      {/* Node Inspector Drawer */}
      <NodeInspector
        node={selectedNode}
        allNodes={pipeline.nodes}
        voices={voices}
        onClose={() => setSelectedNodeId(null)}
        onUpdateParams={handleUpdateParams}
      />
    </div>
  );
};
