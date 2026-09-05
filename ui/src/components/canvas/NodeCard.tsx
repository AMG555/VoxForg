import React from 'react';
import {
  FileText,
  Users,
  UserCheck,
  Cpu,
  Sliders,
  Layers,
  Save,
  Trash2,
} from 'lucide-react';
import { PipelineNode } from '../../types';

interface NodeCardProps {
  node: PipelineNode;
  isSelected: boolean;
  onSelect: (id: string) => void;
  onDelete: (id: string) => void;
}

const nodeIcons: Record<string, React.ReactNode> = {
  text_input: <FileText className="w-4 h-4 text-sky-400" />,
  speaker_parser: <Users className="w-4 h-4 text-purple-400" />,
  voice_assigner: <UserCheck className="w-4 h-4 text-amber-400" />,
  synthesizer: <Cpu className="w-4 h-4 text-emerald-400" />,
  audio_filter: <Sliders className="w-4 h-4 text-pink-400" />,
  audio_merge: <Layers className="w-4 h-4 text-indigo-400" />,
  output_sink: <Save className="w-4 h-4 text-rose-400" />,
};

const nodeCategoryLabels: Record<string, string> = {
  text_input: 'Ingestion',
  speaker_parser: 'Analysis',
  voice_assigner: 'Routing',
  synthesizer: 'Synthesis',
  audio_filter: 'DSP Filter',
  audio_merge: 'Mastering',
  output_sink: 'Output Sink',
};

export const NodeCard: React.FC<NodeCardProps> = ({
  node,
  isSelected,
  onSelect,
  onDelete,
}) => {
  return (
    <div
      onClick={() => onSelect(node.id)}
      className={`relative w-64 rounded-lg bg-[#121820] border transition-all cursor-pointer shadow-lg select-none ${
        isSelected
          ? 'border-amber-500 ring-1 ring-amber-500/50 shadow-amber-500/10'
          : 'border-[#242E3D] hover:border-[#3B485C]'
      }`}
    >
      {/* Input port connector */}
      <div className="absolute -left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 rounded-full bg-[#1A222D] border-2 border-[#38BDF8] flex items-center justify-center shadow">
        <div className="w-1.5 h-1.5 rounded-full bg-[#38BDF8]" />
      </div>

      {/* Output port connector */}
      <div className="absolute -right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 rounded-full bg-[#1A222D] border-2 border-emerald-400 flex items-center justify-center shadow">
        <div className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
      </div>

      {/* Card Header */}
      <div className="flex items-center justify-between p-3 border-b border-[#242E3D] bg-[#0B0E14]/40 rounded-t-lg">
        <div className="flex items-center space-x-2">
          <div className="p-1 rounded bg-[#1A222D] border border-[#242E3D]">
            {nodeIcons[node.node_type] || <Cpu className="w-4 h-4 text-gray-400" />}
          </div>
          <div>
            <span className="text-[10px] uppercase font-mono tracking-wider text-[#94A3B8]">
              {nodeCategoryLabels[node.node_type] || 'Node'}
            </span>
            <h4 className="text-xs font-semibold text-white truncate max-w-[130px]">
              {node.name}
            </h4>
          </div>
        </div>

        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete(node.id);
          }}
          className="text-[#64748B] hover:text-rose-400 p-1 rounded transition-colors"
        >
          <Trash2 className="w-3.5 h-3.5" />
        </button>
      </div>

      {/* Card Content Snippet */}
      <div className="p-3 text-xs font-mono text-[#94A3B8] space-y-1">
        {node.node_type === 'text_input' && (
          <p className="line-clamp-2 text-[#F0F4F8] font-sans text-xs">
            "{node.params.text || 'No text provided'}"
          </p>
        )}
        {node.node_type === 'voice_assigner' && (
          <div className="text-[11px]">
            <span>Default: </span>
            <span className="text-amber-400">{node.params.default_voice || 'en-US-AriaNeural'}</span>
          </div>
        )}
        {node.node_type === 'audio_merge' && (
          <div className="text-[11px]">
            <span>Pause Gap: </span>
            <span className="text-sky-400">{node.params.pause_ms || 150}ms</span>
          </div>
        )}
        {node.node_type === 'audio_filter' && (
          <div className="text-[11px]">
            <span>Peak Limit: </span>
            <span className="text-emerald-400">{((node.params.normalize_peak || 0.95) * 100).toFixed(0)}%</span>
          </div>
        )}
        {node.node_type === 'synthesizer' && (
          <div className="text-[11px] text-emerald-400">
            <span>Hardware Fast Path Active</span>
          </div>
        )}
        {node.node_type === 'output_sink' && (
          <div className="text-[11px] text-rose-300">
            <span>Container: </span>
            <span>WAV / 24kHz</span>
          </div>
        )}
      </div>
    </div>
  );
};
