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
  Copy,
  Mic,
  Play,
  Loader2,
  CheckCircle2,
  AlertCircle,
  EyeOff,
  Eye,
  Headphones,
  Clock,
  Film,
} from 'lucide-react';
import { PipelineNode, NodeExecutionState } from '../../types';

interface NodeCardProps {
  node: PipelineNode;
  isSelected: boolean;
  executionState?: NodeExecutionState;
  onSelect: (id: string, isMulti?: boolean) => void;
  onDelete: (id: string) => void;
  onDuplicate?: (id: string) => void;
  onPointerDown?: (e: React.PointerEvent, id: string) => void;
  onConnectStart?: (nodeId: string, e: React.PointerEvent) => void;
  onConnectEnd?: (nodeId: string) => void;
  onTestStep?: (id: string) => void;
  onToggleDisable?: (id: string) => void;
  style?: React.CSSProperties;
}

const nodeIcons: Record<string, React.ReactNode> = {
  character_voice: <Mic className="w-4 h-4 text-amber-400" />,
  text_input: <FileText className="w-4 h-4 text-sky-400" />,
  speaker_parser: <Users className="w-4 h-4 text-purple-400" />,
  voice_assigner: <UserCheck className="w-4 h-4 text-amber-400" />,
  synthesizer: <Cpu className="w-4 h-4 text-emerald-400" />,
  audio_filter: <Sliders className="w-4 h-4 text-pink-400" />,
  audio_merge: <Layers className="w-4 h-4 text-indigo-400" />,
  output_sink: <Save className="w-4 h-4 text-rose-400" />,
  asr_transcriber: <Headphones className="w-4 h-4 text-violet-400" />,
  diarization: <UserCheck className="w-4 h-4 text-cyan-400" />,
  document_chunker: <FileText className="w-4 h-4 text-teal-400" />,
  audio_time_stretch: <Clock className="w-4 h-4 text-amber-400" />,
  audio_mux: <Film className="w-4 h-4 text-rose-400" />,
};

const nodeCategoryLabels: Record<string, string> = {
  character_voice: 'Character',
  text_input: 'Ingestion',
  speaker_parser: 'Analysis',
  voice_assigner: 'Routing',
  synthesizer: 'Synthesis',
  audio_filter: 'DSP Filter',
  audio_merge: 'Mastering',
  output_sink: 'Output Sink',
  asr_transcriber: 'Audio Ingestion',
  diarization: 'Audio Analysis',
  document_chunker: 'Preprocessing',
  audio_time_stretch: 'DSP Warp',
  audio_mux: 'Video Dubbing',
};

export const NodeCard: React.FC<NodeCardProps> = ({
  node,
  isSelected,
  executionState,
  onSelect,
  onDelete,
  onDuplicate,
  onPointerDown,
  onConnectStart,
  onConnectEnd,
  onTestStep,
  onToggleDisable,
  style,
}) => {
  const isRunning = executionState?.status === 'running';
  const isSuccess = executionState?.status === 'success';
  const isError = executionState?.status === 'error';
  const isDisabled = !!node.disabled;

  let borderStyle = 'border-[#242E3D] hover:border-[#3B485C]';
  let shadowStyle = 'shadow-xl';

  if (isRunning) {
    borderStyle = 'border-amber-400 ring-2 ring-amber-400/60 animate-pulse';
    shadowStyle = 'shadow-2xl shadow-amber-500/30';
  } else if (isSuccess) {
    borderStyle = 'border-emerald-500/90 ring-1 ring-emerald-500/40';
    shadowStyle = 'shadow-xl shadow-emerald-500/10';
  } else if (isError) {
    borderStyle = 'border-rose-500 ring-2 ring-rose-500/40';
    shadowStyle = 'shadow-xl shadow-rose-500/20';
  } else if (isSelected) {
    borderStyle = 'border-amber-500 ring-2 ring-amber-500/60';
    shadowStyle = 'shadow-2xl shadow-amber-500/20';
  }

  return (
    <div
      style={style}
      onPointerDown={(e) => onPointerDown?.(e, node.id)}
      onClick={(e) => {
        e.stopPropagation();
        onSelect(node.id, e.shiftKey || e.ctrlKey || e.metaKey);
      }}
      className={`absolute w-64 rounded-xl bg-[#121820] border cursor-grab active:cursor-grabbing select-none transition-all duration-150 ${borderStyle} ${shadowStyle} ${
        isDisabled ? 'opacity-40 grayscale-[40%]' : ''
      } ${isSelected ? 'z-30' : isRunning ? 'z-40' : 'z-20'} group/card`}
    >
      {/* Input port connector (left) */}
      <div
        onPointerUp={(e) => {
          e.stopPropagation();
          onConnectEnd?.(node.id);
        }}
        className="absolute -left-3 top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-[#121820] border-2 border-[#38BDF8] hover:border-amber-400 hover:scale-125 flex items-center justify-center shadow-lg transition-transform cursor-pointer z-30 group/inport"
        title="Input port (drop wire here)"
      >
        <div className="w-2 h-2 rounded-full bg-[#38BDF8] group-hover/inport:bg-amber-400 transition-colors" />
      </div>

      {/* Output port connector (right) */}
      <div
        onPointerDown={(e) => {
          e.stopPropagation();
          onConnectStart?.(node.id, e);
        }}
        className="absolute -right-3 top-1/2 -translate-y-1/2 w-5 h-5 rounded-full bg-[#121820] border-2 border-emerald-400 hover:border-amber-400 hover:scale-125 flex items-center justify-center shadow-lg transition-transform cursor-crosshair z-30 group/outport"
        title="Output port (drag to connect next node)"
      >
        <div className="w-2 h-2 rounded-full bg-emerald-400 group-hover/outport:bg-amber-400 transition-colors" />
      </div>

      {/* Card Header */}
      <div className="flex items-center justify-between p-3 border-b border-[#242E3D] bg-[#0B0E14]/40 rounded-t-xl">
        <div className="flex items-center space-x-2 min-w-0">
          <div className="p-1 rounded bg-[#1A222D] border border-[#242E3D] shrink-0">
            {nodeIcons[node.node_type] || <Cpu className="w-4 h-4 text-gray-400" />}
          </div>
          <div className="min-w-0">
            <span className="text-[10px] uppercase font-mono tracking-wider text-[#94A3B8] block truncate">
              {nodeCategoryLabels[node.node_type] || 'Node'}
            </span>
            <h4
              className={`text-xs font-semibold text-white truncate max-w-[125px] ${
                isDisabled ? 'line-through text-[#64748B]' : ''
              }`}
            >
              {node.name}
            </h4>
          </div>
        </div>

        {/* Quick action controls */}
        <div className="flex items-center space-x-0.5">
          {/* Test step play button on hover */}
          {onTestStep && !isDisabled && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onTestStep(node.id);
              }}
              disabled={isRunning}
              className="p-1 rounded text-[#94A3B8] hover:text-amber-400 hover:bg-[#1A222D] opacity-0 group-hover/card:opacity-100 transition-all"
              title="Test this step (single-node run)"
            >
              <Play className="w-3.5 h-3.5" />
            </button>
          )}

          {/* Disable/Enable Toggle */}
          {onToggleDisable && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onToggleDisable(node.id);
              }}
              className={`p-1 rounded hover:bg-[#1A222D] transition-colors ${
                isDisabled ? 'text-amber-400' : 'text-[#64748B] hover:text-white opacity-0 group-hover/card:opacity-100'
              }`}
              title={isDisabled ? 'Enable node' : 'Disable node (bypass)'}
            >
              {isDisabled ? <EyeOff className="w-3.5 h-3.5" /> : <Eye className="w-3.5 h-3.5" />}
            </button>
          )}

          {onDuplicate && (
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                onDuplicate(node.id);
              }}
              className="text-[#64748B] hover:text-amber-400 p-1 rounded hover:bg-[#1A222D] transition-colors"
              title="Duplicate node (Ctrl+D)"
            >
              <Copy className="w-3.5 h-3.5" />
            </button>
          )}

          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onDelete(node.id);
            }}
            className="text-[#64748B] hover:text-rose-400 p-1 rounded hover:bg-[#1A222D] transition-colors"
            title="Delete node (Del)"
          >
            <Trash2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Execution status indicator badge */}
      {isRunning && (
        <div className="bg-amber-500/15 border-b border-amber-500/30 px-3 py-1 flex items-center justify-between text-[10px] font-mono text-amber-300">
          <div className="flex items-center space-x-1.5">
            <Loader2 className="w-3 h-3 animate-spin text-amber-400" />
            <span>Processing...</span>
          </div>
          <span className="text-[9px] uppercase tracking-wider font-semibold">Active</span>
        </div>
      )}

      {isSuccess && (
        <div className="bg-emerald-500/10 border-b border-emerald-500/25 px-3 py-1 flex items-center justify-between text-[10px] font-mono text-emerald-400">
          <div className="flex items-center space-x-1.5">
            <CheckCircle2 className="w-3 h-3 text-emerald-400" />
            <span>Success</span>
          </div>
          <span>{executionState?.latencyMs ? `${executionState.latencyMs}ms` : 'Done'}</span>
        </div>
      )}

      {isError && (
        <div className="bg-rose-500/15 border-b border-rose-500/30 px-3 py-1 flex items-center justify-between text-[10px] font-mono text-rose-300">
          <div className="flex items-center space-x-1.5">
            <AlertCircle className="w-3 h-3 text-rose-400" />
            <span className="truncate max-w-[150px]">{executionState?.message || 'Execution Error'}</span>
          </div>
        </div>
      )}

      {/* Card Content Snippet */}
      <div className="p-3 text-xs font-mono text-[#94A3B8] space-y-1">
        {node.node_type === 'character_voice' && (
          <div className="space-y-1">
            <div className="flex items-center justify-between text-[11px]">
              <span className="font-semibold text-amber-400 truncate max-w-[120px]">
                {node.params.character_name || 'Character'}
              </span>
              <span className="text-[10px] text-sky-400 bg-sky-950/50 px-1.5 py-0.5 rounded border border-sky-800/50 truncate max-w-[90px]">
                {node.params.voice_id ? node.params.voice_id.split('-').slice(-1)[0] : 'Voice'}
              </span>
            </div>
            <p className="line-clamp-2 text-[#F0F4F8] font-sans text-xs italic">
              "{node.params.text || 'No dialogue entered'}"
            </p>
          </div>
        )}
        {node.node_type === 'text_input' && (
          <p className="line-clamp-2 text-[#F0F4F8] font-sans text-xs">
            "{node.params.text || 'No text provided'}"
          </p>
        )}
        {node.node_type === 'voice_assigner' && (
          <div className="text-[11px]">
            <span>Default: </span>
            <span className="text-amber-400 truncate block">{node.params.default_voice || 'en-US-AriaNeural'}</span>
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
          <div className="text-[11px] text-emerald-400 flex items-center justify-between">
            <span>Neural Fast Path</span>
            <span className="text-[10px] text-[#64748B]">{(node.params.speed || 1.0).toFixed(1)}x</span>
          </div>
        )}
        {node.node_type === 'output_sink' && (
          <div className="text-[11px] text-rose-300">
            <span>Container: </span>
            <span>WAV / 24kHz</span>
          </div>
        )}
        {node.node_type === 'asr_transcriber' && (
          <div className="text-[11px] text-violet-300">
            <span>Engine: </span>
            <span>Whisper Large-v3</span>
          </div>
        )}
        {node.node_type === 'diarization' && (
          <div className="text-[11px] text-cyan-300">
            <span>Diarizer: </span>
            <span>PyAnnote Neural</span>
          </div>
        )}
        {node.node_type === 'document_chunker' && (
          <div className="text-[11px] text-teal-300">
            <span>Max Chunk: </span>
            <span>{node.params.chunk_size || 500} words</span>
          </div>
        )}
        {node.node_type === 'audio_time_stretch' && (
          <div className="text-[11px] text-amber-300">
            <span>Ratio: </span>
            <span>{(node.params.speed_ratio || 1.0).toFixed(2)}x (Preserve Pitch)</span>
          </div>
        )}
        {node.node_type === 'audio_mux' && (
          <div className="text-[11px] text-rose-300">
            <span>Muxer: </span>
            <span>FFmpeg Stream Copy</span>
          </div>
        )}
      </div>
    </div>
  );
};
