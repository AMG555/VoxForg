import React from 'react';
import { Activity, Cpu, GitFork, Mic, Volume2 } from 'lucide-react';
import { HardwareInfo } from '../types';

interface NavigationProps {
  activeTab: 'canvas' | 'voices' | 'hardware';
  setActiveTab: (tab: 'canvas' | 'voices' | 'hardware') => void;
  hardware: HardwareInfo | null;
}

export const Navigation: React.FC<NavigationProps> = ({ activeTab, setActiveTab, hardware }) => {
  return (
    <header className="h-14 border-b border-[#242E3D] bg-[#121820] flex items-center justify-between px-4 select-none">
      <div className="flex items-center space-x-6">
        <div className="flex items-center space-x-2">
          <div className="w-8 h-8 rounded bg-amber-500/10 border border-amber-500/30 flex items-center justify-center text-amber-500 font-bold">
            <Volume2 className="w-4 h-4" />
          </div>
          <span className="font-bold tracking-wider text-sm font-mono text-white">VOXFORG</span>
          <span className="text-[10px] uppercase font-mono px-1.5 py-0.5 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
            v0.1.0
          </span>
        </div>

        <nav className="flex space-x-1">
          <button
            onClick={() => setActiveTab('canvas')}
            className={`flex items-center space-x-2 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
              activeTab === 'canvas'
                ? 'bg-[#1A222D] text-white border border-[#3B485C]'
                : 'text-[#94A3B8] hover:text-white hover:bg-[#1A222D]/50'
            }`}
          >
            <GitFork className="w-3.5 h-3.5" />
            <span>Pipeline Canvas</span>
          </button>

          <button
            onClick={() => setActiveTab('voices')}
            className={`flex items-center space-x-2 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
              activeTab === 'voices'
                ? 'bg-[#1A222D] text-white border border-[#3B485C]'
                : 'text-[#94A3B8] hover:text-white hover:bg-[#1A222D]/50'
            }`}
          >
            <Mic className="w-3.5 h-3.5" />
            <span>Voice Lab</span>
          </button>

          <button
            onClick={() => setActiveTab('hardware')}
            className={`flex items-center space-x-2 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
              activeTab === 'hardware'
                ? 'bg-[#1A222D] text-white border border-[#3B485C]'
                : 'text-[#94A3B8] hover:text-white hover:bg-[#1A222D]/50'
            }`}
          >
            <Cpu className="w-3.5 h-3.5" />
            <span>Hardware & Engines</span>
          </button>
        </nav>
      </div>

      <div className="flex items-center space-x-4">
        {hardware && (
          <div className="flex items-center space-x-2 text-xs font-mono text-[#94A3B8] bg-[#0B0E14] px-3 py-1 rounded border border-[#242E3D]">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span>{hardware.hardware_tier}</span>
            <span className="text-[#3B485C]">|</span>
            <span>{hardware.registered_engines.length} Engines</span>
          </div>
        )}
      </div>
    </header>
  );
};
