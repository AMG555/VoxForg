import React from 'react';
import { Cpu, Server, Zap, CheckCircle2, Activity } from 'lucide-react';
import { HardwareInfo } from '../../types';

interface HardwareInspectorProps {
  hardware: HardwareInfo | null;
  onNavigateTab?: (tab: 'canvas' | 'voices' | 'catalog' | 'hardware' | 'qa') => void;
}

interface EngineDescriptor {
  name: string;
  category: string;
  description: string;
  statusBadge: {
    label: string;
    bg: string;
    text: string;
    border: string;
  };
  action?: {
    label: string;
    tab: 'catalog' | 'voices';
  };
}

const ENGINE_DESCRIPTORS: Record<string, EngineDescriptor> = {
  'edge-tts': {
    name: 'Microsoft Edge Cloud Relay',
    category: 'Cloud WebSocket Relay',
    description: 'High-fidelity zero-cost neural speech relay with real-time streaming chunking.',
    statusBadge: {
      label: 'ONLINE & READY',
      bg: 'bg-emerald-500/10',
      text: 'text-emerald-400',
      border: 'border-emerald-500/30',
    },
  },
  'piper-tts': {
    name: 'Piper Local Neural ONNX Engine',
    category: 'Offline On-Device Neural',
    description: 'Ultra-fast native ONNX runtime running locally without cloud dependency. Requires model weights.',
    statusBadge: {
      label: 'WEIGHTS CONFIGURED',
      bg: 'bg-amber-500/10',
      text: 'text-amber-400',
      border: 'border-amber-500/30',
    },
    action: {
      label: 'Manage in Catalog',
      tab: 'catalog',
    },
  },
  'openai-router': {
    name: 'OpenRouter & OpenAI Cloud Gateway',
    category: 'Upstream Model Router',
    description: 'Dynamic load-balancer and fall-through proxy for OpenRouter and OpenAI neural voices.',
    statusBadge: {
      label: 'ROUTER READY',
      bg: 'bg-sky-500/10',
      text: 'text-sky-400',
      border: 'border-sky-500/30',
    },
    action: {
      label: 'Configure Key',
      tab: 'voices',
    },
  },
  'qwen3-tts': {
    name: 'Qwen3 Zero-Shot Voice Cloner',
    category: 'Acoustic Conditioning Transformer',
    description: 'Speaker embedding extraction and zero-shot voice cloning transformer pipeline.',
    statusBadge: {
      label: 'CLONING READY',
      bg: 'bg-purple-500/10',
      text: 'text-purple-400',
      border: 'border-purple-500/30',
    },
  },
  'mock-tts': {
    name: 'In-Memory Synthesis Benchmark Harness',
    category: 'Mathematical Test Harness',
    description: 'Deterministic sine wave harmonic generator for unit testing, buffer verification, and latency benchmarks.',
    statusBadge: {
      label: 'BENCHMARK HARNESS',
      bg: 'bg-slate-500/10',
      text: 'text-slate-400',
      border: 'border-slate-500/30',
    },
  },
};

export const HardwareInspector: React.FC<HardwareInspectorProps> = ({ hardware, onNavigateTab }) => {
  const registeredEngines = hardware?.registered_engines || [
    'edge-tts',
    'piper-tts',
    'openai-router',
    'qwen3-tts',
    'mock-tts',
  ];

  return (
    <div className="flex-1 p-8 overflow-y-auto bg-[#0B0E14] space-y-6">
      <div className="max-w-4xl space-y-6">
        <div>
          <span className="text-[11px] font-mono uppercase text-amber-500 tracking-wider">
            System Telemetry & Engines
          </span>
          <h2 className="text-xl font-bold text-white mt-0.5">Hardware Capability Profile</h2>
          <p className="text-xs text-[#94A3B8] font-mono mt-1">
            Automated host probe detecting SIMD vector extensions, accelerators, and active synthesis engines.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Server className="w-4 h-4 text-sky-400" />
              <span className="text-xs font-mono uppercase">Assigned Tier</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {hardware?.hardware_tier || 'TIER_3_PRO'}
            </p>
            <p className="text-[11px] text-[#64748B]">Optimal local and cloud engines unlocked</p>
          </div>

          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Cpu className="w-4 h-4 text-emerald-400" />
              <span className="text-xs font-mono uppercase">Platform & Kernel</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {hardware ? `${hardware.os} (${hardware.arch})` : 'Windows (x86_64)'}
            </p>
            <p className="text-[11px] text-[#64748B]">Native kernel SIMD vector extensions</p>
          </div>

          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Zap className="w-4 h-4 text-amber-400" />
              <span className="text-xs font-mono uppercase">Registered Engines</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {registeredEngines.length} Online
            </p>
            <p className="text-[11px] text-[#64748B]">Multi-engine dispatch active</p>
          </div>
        </div>

        <div className="bg-[#121820] border border-[#242E3D] rounded-lg overflow-hidden">
          <div className="px-4 py-3 border-b border-[#242E3D] bg-[#0B0E14]/40 flex items-center justify-between">
            <h3 className="text-xs font-semibold text-white uppercase tracking-wider flex items-center space-x-2">
              <Activity className="w-4 h-4 text-amber-500" />
              <span>Registered Speech Synthesis Engines</span>
            </h3>
            <span className="text-[10px] font-mono text-[#64748B]">
              Real-time diagnostic telemetry
            </span>
          </div>

          <div className="divide-y divide-[#242E3D]">
            {registeredEngines.map((engineId) => {
              const desc = ENGINE_DESCRIPTORS[engineId] || {
                name: engineId,
                category: 'Synthesis Engine',
                description: 'Speech generation engine registered in kernel.',
                statusBadge: {
                  label: 'ONLINE',
                  bg: 'bg-emerald-500/10',
                  text: 'text-emerald-400',
                  border: 'border-emerald-500/30',
                },
              };

              return (
                <div key={engineId} className="px-4 py-3.5 flex flex-col md:flex-row md:items-center justify-between gap-3 hover:bg-[#1A222D]/40 transition-colors">
                  <div className="flex items-start space-x-3 min-w-0">
                    <CheckCircle2 className="w-4 h-4 text-emerald-400 flex-shrink-0 mt-0.5" />
                    <div>
                      <div className="flex items-center space-x-2">
                        <span className="text-xs font-mono font-bold text-white">{desc.name}</span>
                        <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-[#0B0E14] text-[#94A3B8] border border-[#242E3D]">
                          {engineId}
                        </span>
                      </div>
                      <p className="text-[11px] text-[#94A3B8] mt-1 leading-relaxed">
                        {desc.description}
                      </p>
                    </div>
                  </div>

                  <div className="flex items-center space-x-2 flex-shrink-0 self-start md:self-auto">
                    {desc.action && onNavigateTab && (
                      <button
                        onClick={() => onNavigateTab(desc.action!.tab)}
                        className="text-[10px] font-mono px-2 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-amber-400 border border-amber-500/30 hover:border-amber-400 transition-colors"
                      >
                        {desc.action.label}
                      </button>
                    )}
                    <span
                      className={`text-[10px] font-mono font-bold px-2 py-0.5 rounded border ${desc.statusBadge.bg} ${desc.statusBadge.text} ${desc.statusBadge.border}`}
                    >
                      {desc.statusBadge.label}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
