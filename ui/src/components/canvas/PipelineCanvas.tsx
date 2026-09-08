import React, { useState } from 'react';
import { Play, Loader2 } from 'lucide-react';
import { PipelineDefinition, Voice } from '../../types';
import { NodeCard } from './NodeCard';
import { NodeInspector } from './NodeInspector';
import { api } from '../../services/api';

interface PipelineCanvasProps {
  voices: Voice[];
}

export const PipelineCanvas: React.FC<PipelineCanvasProps> = ({ voices }) => {
  const [pipeline, setPipeline] = useState<PipelineDefinition>({
    id: 'demo-pipeline-1',
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
  });

  const [selectedNodeId, setSelectedNodeId] = useState<string | null>('node-1');
  const [isRunning, setIsRunning] = useState(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);

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

  return (
    <div className="flex-1 flex overflow-hidden relative">
      {/* Central Interactive Node Canvas */}
      <div className="flex-1 flex flex-col h-full bg-[#0B0E14] relative">
        {/* Canvas Toolbar */}
        <div className="h-12 border-b border-[#242E3D] bg-[#121820]/60 backdrop-blur px-4 flex items-center justify-between z-10 select-none">
          <div className="flex items-center space-x-3">
            <span className="text-xs font-mono font-bold text-white">{pipeline.name}</span>
            <span className="text-[11px] font-mono text-[#64748B]">
              ({pipeline.nodes.length} nodes, {pipeline.edges.length} connections)
            </span>
          </div>

          <div className="flex items-center space-x-3">
            {audioUrl && (
              <audio controls src={audioUrl} className="h-7 w-60 rounded bg-[#1A222D]" />
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
