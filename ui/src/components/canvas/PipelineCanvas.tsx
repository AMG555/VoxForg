import React, { useState, useRef } from 'react';
import { Play, Loader2, Download, Upload, Sparkles } from 'lucide-react';
import { PipelineDefinition, Voice } from '../../types';
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
        position: { x: 50, y: 120 },
      },
      {
        id: 'node-2',
        name: 'Speaker Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 360, y: 120 },
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
        position: { x: 670, y: 120 },
      },
      {
        id: 'node-4',
        name: 'Speech Synthesizer',
        node_type: 'synthesizer',
        params: {},
        position: { x: 980, y: 120 },
      },
      {
        id: 'node-5',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 180 },
        position: { x: 1290, y: 120 },
      },
      {
        id: 'node-6',
        name: 'Master Output',
        node_type: 'output_sink',
        params: {},
        position: { x: 1600, y: 120 },
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
    name: 'Studio Podcast Mastering',
    nodes: [
      {
        id: 'p-1',
        name: 'Podcast Host Script',
        node_type: 'text_input',
        params: {
          text: 'Welcome back to Quantum Wave. Today we dive into neural speech synthesis architectures in high-concurrency environments.',
        },
        position: { x: 50, y: 120 },
      },
      {
        id: 'p-2',
        name: 'Neural Synthesizer',
        node_type: 'synthesizer',
        params: { voice: 'en-US-JennyNeural' },
        position: { x: 360, y: 120 },
      },
      {
        id: 'p-3',
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
        position: { x: 670, y: 120 },
      },
      {
        id: 'p-4',
        name: 'Broadcast Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 980, y: 120 },
      },
    ],
    edges: [
      { id: 'pe1', from_node: 'p-1', to_node: 'p-2' },
      { id: 'pe2', from_node: 'p-2', to_node: 'p-3' },
      { id: 'pe3', from_node: 'p-3', to_node: 'p-4' },
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
        name: 'Station Jingle & Announcement',
        node_type: 'text_input',
        params: {
          text: 'You are listening to 104.7 VoxFM. Up next: non-stop neural audio streams with zero latency.',
        },
        position: { x: 50, y: 120 },
      },
      {
        id: 'r-2',
        name: 'Resonant Synthesizer',
        node_type: 'synthesizer',
        params: { voice: 'en-US-GuyNeural' },
        position: { x: 360, y: 120 },
      },
      {
        id: 'r-3',
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
        position: { x: 670, y: 120 },
      },
      {
        id: 'r-4',
        name: 'Transmitter Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 980, y: 120 },
      },
    ],
    edges: [
      { id: 're1', from_node: 'r-1', to_node: 'r-2' },
      { id: 're2', from_node: 'r-2', to_node: 'r-3' },
      { id: 're3', from_node: 'r-3', to_node: 'r-4' },
    ],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
};

export const PipelineCanvas: React.FC<PipelineCanvasProps> = ({ voices }) => {
  const [pipeline, setPipeline] = useState<PipelineDefinition>(PRESETS.narrative);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>('node-1');
  const [isRunning, setIsRunning] = useState(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
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
      <div className="flex-1 flex flex-col h-full bg-[#0B0E14] relative">
        {/* Canvas Toolbar */}
        <div className="h-12 border-b border-[#242E3D] bg-[#121820]/80 backdrop-blur px-4 flex items-center justify-between z-10 select-none">
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
                  }
                }}
                className="bg-[#0B0E14] text-[#94A3B8] border border-[#242E3D] rounded px-2 py-1 text-[11px] font-mono focus:border-amber-500 focus:outline-none cursor-pointer"
                defaultValue="narrative"
              >
                <option value="narrative">Preset: Narrative Dialogue</option>
                <option value="podcast">Preset: Studio Podcast</option>
                <option value="radio">Preset: Punchy Radio</option>
              </select>
            </div>
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

        {/* Canvas Workspace Area */}
        <div className="flex-1 overflow-x-auto overflow-y-hidden p-8 canvas-grid relative flex items-center">
          <div className="flex items-center space-x-12 min-w-max">
            {pipeline.nodes.map((node, index) => (
              <React.Fragment key={node.id}>
                <NodeCard
                  node={node}
                  isSelected={node.id === selectedNodeId}
                  onSelect={setSelectedNodeId}
                  onDelete={handleDeleteNode}
                />
                {index < pipeline.nodes.length - 1 && (
                  <div className="w-12 h-0.5 bg-[#38BDF8] relative flex items-center justify-center">
                    <div className="w-2 h-2 rounded-full bg-[#38BDF8] animate-ping" />
                  </div>
                )}
              </React.Fragment>
            ))}
          </div>
        </div>

        {/* Real-time Audio Visualizer Bottom Dock */}
        {audioUrl && (
          <div className="absolute bottom-4 left-4 z-20 w-96 shadow-2xl backdrop-blur-md bg-[#0B0E14]/90 rounded-lg">
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
        voices={voices}
        onClose={() => setSelectedNodeId(null)}
        onUpdateParams={handleUpdateParams}
      />
    </div>
  );
};
