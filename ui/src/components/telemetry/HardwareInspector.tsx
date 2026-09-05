import React from 'react';
import { Cpu, Server, Zap, CheckCircle } from 'lucide-react';
import { HardwareInfo } from '../../types';

interface HardwareInspectorProps {
  hardware: HardwareInfo | null;
}

export const HardwareInspector: React.FC<HardwareInspectorProps> = ({ hardware }) => {
  return (
    <div className="flex-1 p-8 overflow-y-auto bg-[#0B0E14] space-y-6">
      <div className="max-w-4xl space-y-6">
        <div>
          <span className="text-[11px] font-mono uppercase text-amber-500 tracking-wider">
            System Telemetry & Engines
          </span>
          <h2 className="text-xl font-bold text-white mt-0.5">Hardware Capability Profile</h2>
          <p className="text-xs text-[#94A3B8] font-mono mt-1">
            Automated host probe detecting SIMD vector extensions and compute accelerators.
          </p>
        </div>

        <div className="grid grid-cols-3 gap-4">
          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Server className="w-4 h-4 text-sky-400" />
              <span className="text-xs font-mono uppercase">Assigned Tier</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {hardware?.hardware_tier || 'TIER_2_STANDARD'}
            </p>
            <p className="text-[11px] text-[#64748B]">Optimal local and cloud engines unlocked</p>
          </div>

          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Cpu className="w-4 h-4 text-emerald-400" />
              <span className="text-xs font-mono uppercase">Platform</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {hardware ? `${hardware.os} (${hardware.arch})` : 'x86_64'}
            </p>
            <p className="text-[11px] text-[#64748B]">Native kernel SIMD runtime</p>
          </div>

          <div className="bg-[#121820] border border-[#242E3D] p-4 rounded-lg space-y-2">
            <div className="flex items-center space-x-2 text-[#94A3B8]">
              <Zap className="w-4 h-4 text-amber-400" />
              <span className="text-xs font-mono uppercase">Active Engines</span>
            </div>
            <p className="text-lg font-bold font-mono text-white">
              {hardware?.registered_engines.length || 2} Online
            </p>
            <p className="text-[11px] text-[#64748B]">Ready for low-latency synthesis</p>
          </div>
        </div>

        <div className="bg-[#121820] border border-[#242E3D] rounded-lg overflow-hidden">
          <div className="px-4 py-3 border-b border-[#242E3D] bg-[#0B0E14]/40">
            <h3 className="text-xs font-semibold text-white uppercase tracking-wider">
              Registered Engines
            </h3>
          </div>
          <div className="divide-y divide-[#242E3D]">
            {(hardware?.registered_engines || ['edge-tts', 'mock-tts']).map((engine) => (
              <div key={engine} className="px-4 py-3 flex items-center justify-between">
                <div className="flex items-center space-x-3">
                  <CheckCircle className="w-4 h-4 text-emerald-400" />
                  <div>
                    <span className="text-xs font-mono font-bold text-white">{engine}</span>
                    <p className="text-[11px] text-[#64748B]">
                      {engine === 'edge-tts'
                        ? 'Microsoft Edge Cloud Relay (Zero-cost neural voices)'
                        : 'Deterministic In-Memory TTS Engine (Test & Benchmark)'}
                    </p>
                  </div>
                </div>
                <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                  HEALTHY
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
