import React, { useState } from 'react';
import {
  ChevronUp,
  ChevronDown,
  Play,
  CheckCircle2,
  Clock,
  Download,
  Volume2,
  Square,
  Sparkles,
  X,
} from 'lucide-react';
import { NodeExecutionState } from '../../types';
import { AudioVisualizer } from '../common/AudioVisualizer';

interface ExecutionTimelineDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  isRunning: boolean;
  totalTimeMs: number;
  stages: Array<{
    id: string;
    name: string;
    nodeType: string;
    state?: NodeExecutionState;
  }>;
  audioUrl: string | null;
  audioRef: React.RefObject<HTMLAudioElement | null>;
}

export const ExecutionTimelineDrawer: React.FC<ExecutionTimelineDrawerProps> = ({
  isOpen,
  onClose,
  isRunning,
  totalTimeMs,
  stages,
  audioUrl,
  audioRef,
}) => {
  const [isExpanded, setIsExpanded] = useState(true);
  const [isPlaying, setIsPlaying] = useState(false);

  if (!isOpen && !audioUrl && !isRunning) return null;

  return (
    <div
      className={`absolute bottom-0 left-0 right-0 z-40 bg-[#121820]/95 backdrop-blur-xl border-t border-[#242E3D] shadow-2xl transition-all duration-300 font-mono text-xs select-none ${
        isExpanded ? 'h-56' : 'h-10'
      }`}
    >
      {/* Header bar */}
      <div
        onClick={() => setIsExpanded((prev) => !prev)}
        className="h-10 px-5 border-b border-[#242E3D]/80 flex items-center justify-between cursor-pointer hover:bg-[#1A222D]/50 transition-colors"
      >
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-1.5 text-amber-400 font-bold text-xs uppercase tracking-wider">
            <Sparkles className="w-3.5 h-3.5" />
            <span>Execution Telemetry & Master Flow</span>
          </div>

          <div className="flex items-center space-x-2 text-[11px] text-[#94A3B8]">
            <span>•</span>
            <span className="flex items-center space-x-1">
              <Clock className="w-3 h-3 text-sky-400" />
              <span>{totalTimeMs}ms total runtime</span>
            </span>
            <span>•</span>
            <span className="flex items-center space-x-1 text-emerald-400">
              <CheckCircle2 className="w-3 h-3" />
              <span>
                {stages.filter((s) => s.state?.status === 'success').length} / {stages.length} nodes
              </span>
            </span>
          </div>
        </div>

        <div className="flex items-center space-x-2">
          {audioUrl && (
            <a
              href={audioUrl}
              download="voxforg-master-output.wav"
              onClick={(e) => e.stopPropagation()}
              className="flex items-center space-x-1 px-2 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-[11px] transition-colors"
            >
              <Download className="w-3 h-3 text-amber-400" />
              <span>Download Master WAV</span>
            </a>
          )}

          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsExpanded((prev) => !prev);
            }}
            className="p-1 rounded text-[#94A3B8] hover:text-white"
            title={isExpanded ? 'Collapse Drawer' : 'Expand Drawer'}
          >
            {isExpanded ? <ChevronDown className="w-4 h-4" /> : <ChevronUp className="w-4 h-4" />}
          </button>

          <button
            onClick={(e) => {
              e.stopPropagation();
              onClose();
            }}
            className="p-1 rounded text-[#94A3B8] hover:text-rose-400"
            title="Close Drawer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Expanded body */}
      {isExpanded && (
        <div className="p-4 h-[calc(100%-2.5rem)] flex items-stretch gap-5 overflow-hidden">
          {/* Left: Step-by-step Gantt Timeline */}
          <div className="flex-1 overflow-y-auto space-y-2 pr-2">
            <div className="text-[10px] uppercase font-semibold text-[#64748B] tracking-wider mb-1">
              Stage Progression & Latencies
            </div>

            {stages.map((stage, idx) => {
              const status = stage.state?.status || 'idle';
              const latency = stage.state?.latencyMs || 0;
              const maxLatency = Math.max(...stages.map((s) => s.state?.latencyMs || 1), 100);
              const barWidth = Math.max(8, Math.min(100, (latency / maxLatency) * 100));

              return (
                <div
                  key={stage.id}
                  className="flex items-center space-x-3 text-xs p-1.5 rounded-lg bg-[#0B0E14] border border-[#242E3D]/70"
                >
                  <span className="w-5 text-[10px] text-[#64748B] font-mono">#{idx + 1}</span>

                  <div className="w-44 truncate">
                    <span className="text-white font-medium">{stage.name}</span>
                    <span className="text-[10px] text-[#64748B] block truncate uppercase">
                      {stage.nodeType.replace('_', ' ')}
                    </span>
                  </div>

                  {/* Latency Progress Bar */}
                  <div className="flex-1 bg-[#1A222D] h-3.5 rounded-full overflow-hidden relative">
                    <div
                      className={`h-full rounded-full transition-all duration-300 ${
                        status === 'running'
                          ? 'bg-amber-400 animate-pulse w-full'
                          : status === 'success'
                          ? 'bg-emerald-500'
                          : status === 'error'
                          ? 'bg-rose-500'
                          : 'bg-[#242E3D]'
                      }`}
                      style={{ width: status === 'running' ? '100%' : `${barWidth}%` }}
                    />
                  </div>

                  {/* Status pill */}
                  <div className="w-20 text-right">
                    {status === 'running' && (
                      <span className="text-amber-400 text-[11px] animate-pulse">Running...</span>
                    )}
                    {status === 'success' && (
                      <span className="text-emerald-400 font-bold">{latency}ms</span>
                    )}
                    {status === 'error' && (
                      <span className="text-rose-400 font-bold">Failed</span>
                    )}
                    {status === 'idle' && (
                      <span className="text-[#64748B] text-[10px]">Waiting</span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Right: Master Output Audio Telemetry */}
          {audioUrl && (
            <div className="w-96 flex flex-col space-y-2.5 border-l border-[#242E3D] pl-5">
              <div className="flex items-center justify-between">
                <span className="text-[10px] uppercase font-semibold text-amber-400 tracking-wider flex items-center gap-1.5">
                  <Volume2 className="w-3.5 h-3.5" />
                  <span>Master Broadcast Audio</span>
                </span>
                <span className="text-[10px] text-[#64748B]">RIFF WAV 24kHz</span>
              </div>

              <div className="flex items-center space-x-2">
                <button
                  onClick={() => {
                    if (!audioRef.current) return;
                    if (isPlaying) {
                      audioRef.current.pause();
                      setIsPlaying(false);
                    } else {
                      audioRef.current.currentTime = 0;
                      audioRef.current.play().catch(console.error);
                      setIsPlaying(true);
                    }
                  }}
                  className="px-3 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs flex items-center space-x-1.5 transition-colors shadow-sm shrink-0"
                >
                  {isPlaying ? (
                    <>
                      <Square className="w-3.5 h-3.5 fill-black" />
                      <span>Pause</span>
                    </>
                  ) : (
                    <>
                      <Play className="w-3.5 h-3.5 fill-black" />
                      <span>Play Master</span>
                    </>
                  )}
                </button>

                <audio
                  ref={audioRef as any}
                  controls
                  src={audioUrl}
                  onPlay={() => setIsPlaying(true)}
                  onPause={() => setIsPlaying(false)}
                  onEnded={() => setIsPlaying(false)}
                  className="flex-1 h-8 rounded bg-[#0B0E14]"
                />
              </div>

              {/* Real-time Spectrum dock */}
              <div className="flex-1 min-h-0">
                <AudioVisualizer
                  audioElement={audioRef.current}
                  isPlaying={isPlaying}
                />
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
