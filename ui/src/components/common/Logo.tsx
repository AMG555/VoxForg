import React from 'react';

interface LogoProps {
  size?: number;
  className?: string;
  showText?: boolean;
}

export const Logo: React.FC<LogoProps> = ({ size = 28, className = '', showText = true }) => {
  return (
    <div className={`flex items-center space-x-2.5 select-none ${className}`}>
      <div
        className="relative flex items-center justify-center rounded-lg bg-gradient-to-br from-[#1E293B] to-[#0B0E14] border border-amber-500/40 p-1 shadow-lg shadow-amber-500/10 group cursor-pointer"
        style={{ width: size, height: size }}
      >
        <svg
          viewBox="0 0 64 64"
          className="w-full h-full"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <defs>
            <linearGradient id="logoWave" x1="0%" y1="0%" x2="100%" y2="0%">
              <stop offset="0%" stopColor="#38BDF8" />
              <stop offset="100%" stopColor="#F59E0B" />
            </linearGradient>
            <linearGradient id="logoGold" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#FCD34D" />
              <stop offset="100%" stopColor="#D97706" />
            </linearGradient>
          </defs>
          {/* Waveform harmonic bars */}
          <rect x="8" y="26" width="3.5" height="12" rx="1.75" fill="#38BDF8" opacity="0.85" />
          <rect x="15" y="19" width="3.5" height="26" rx="1.75" fill="#38BDF8" />
          <rect x="22" y="13" width="3.5" height="38" rx="1.75" fill="#38BDF8" />
          
          <rect x="38" y="13" width="3.5" height="38" rx="1.75" fill="#F59E0B" />
          <rect x="45" y="19" width="3.5" height="26" rx="1.75" fill="#F59E0B" />
          <rect x="52" y="26" width="3.5" height="12" rx="1.75" fill="#F59E0B" opacity="0.85" />

          {/* Stylized Forge Anvil */}
          <path
            d="M 23 27 L 41 27 C 45 27 48 29 46 32 C 45 34 42 35 39 35 L 35 35 L 34 43 L 40 46 L 24 46 L 30 43 L 29 35 L 25 35 C 22 35 20 33 21 31 C 21 29 23 27 23 27 Z"
            fill="url(#logoGold)"
            stroke="#FDE68A"
            strokeWidth="0.75"
          />
          <polygon points="32,23 34,28 31,30 33,34 29,35 30,30 29,29" fill="#FFF" />
        </svg>
      </div>

      {showText && (
        <div className="flex flex-col">
          <div className="flex items-center space-x-1.5">
            <span className="font-extrabold tracking-wider text-sm font-mono text-white">
              VOX<span className="text-amber-400">FORG</span>
            </span>
            <span className="text-[9px] uppercase font-mono px-1 py-0.2 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
              v0.1.0
            </span>
          </div>
          <span className="text-[9px] text-[#64748B] font-mono tracking-tight -mt-0.5">
            NEURAL SPEECH STUDIO
          </span>
        </div>
      )}
    </div>
  );
};
