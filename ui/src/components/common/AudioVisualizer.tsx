import React, { useEffect, useRef, useState } from 'react';
import { Activity, BarChart2 } from 'lucide-react';

interface AudioVisualizerProps {
  audioElement: HTMLAudioElement | null;
  isPlaying?: boolean;
}

const sourceMap = new WeakMap<HTMLAudioElement, { analyser: AnalyserNode; ctx: AudioContext }>();

export const AudioVisualizer: React.FC<AudioVisualizerProps> = ({ audioElement, isPlaying }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [mode, setMode] = useState<'bars' | 'wave'>('bars');
  const animFrameId = useRef<number | null>(null);

  useEffect(() => {
    if (!audioElement) return;

    let analyserNode: AnalyserNode | null = null;

    try {
      if (sourceMap.has(audioElement)) {
        const stored = sourceMap.get(audioElement)!;
        analyserNode = stored.analyser;
      } else {
        const AudioCtxClass = window.AudioContext || (window as any).webkitAudioContext;
        if (AudioCtxClass) {
          const ctx = new AudioCtxClass();
          const analyser = ctx.createAnalyser();
          analyser.fftSize = 256;
          const source = ctx.createMediaElementSource(audioElement);
          source.connect(analyser);
          analyser.connect(ctx.destination);
          sourceMap.set(audioElement, { analyser, ctx });
          analyserNode = analyser;
        }
      }
    } catch (e) {
      // Audio element might already be connected or CORS restricted
    }

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx2d = canvas.getContext('2d');
    if (!ctx2d) return;

    const bufferLength = analyserNode ? analyserNode.frequencyBinCount : 64;
    const dataArray = new Uint8Array(bufferLength);

    const render = () => {
      animFrameId.current = requestAnimationFrame(render);
      const width = canvas.width;
      const height = canvas.height;

      ctx2d.clearRect(0, 0, width, height);

      // Background grid line
      ctx2d.strokeStyle = '#1E293B';
      ctx2d.lineWidth = 1;
      ctx2d.beginPath();
      ctx2d.moveTo(0, height / 2);
      ctx2d.lineTo(width, height / 2);
      ctx2d.stroke();

      if (analyserNode && isPlaying) {
        if (mode === 'bars') {
          analyserNode.getByteFrequencyData(dataArray);
          const barWidth = (width / bufferLength) * 2.2;
          let x = 0;

          for (let i = 0; i < bufferLength; i++) {
            const barHeight = (dataArray[i] / 255) * height * 0.9;
            const gradient = ctx2d.createLinearGradient(0, height, 0, height - barHeight);
            gradient.addColorStop(0, '#F59E0B');
            gradient.addColorStop(1, '#06B6D4');

            ctx2d.fillStyle = gradient;
            ctx2d.fillRect(x, height - barHeight, barWidth - 1, barHeight);
            x += barWidth;
          }
        } else {
          analyserNode.getByteTimeDomainData(dataArray);
          ctx2d.lineWidth = 2;
          ctx2d.strokeStyle = '#06B6D4';
          ctx2d.beginPath();

          const sliceWidth = (width * 1.0) / bufferLength;
          let x = 0;

          for (let i = 0; i < bufferLength; i++) {
            const v = dataArray[i] / 128.0;
            const y = (v * height) / 2;

            if (i === 0) {
              ctx2d.moveTo(x, y);
            } else {
              ctx2d.lineTo(x, y);
            }
            x += sliceWidth;
          }
          ctx2d.lineTo(width, height / 2);
          ctx2d.stroke();
        }
      } else {
        // Idle animation / flatline
        ctx2d.strokeStyle = '#334155';
        ctx2d.lineWidth = 1.5;
        ctx2d.beginPath();
        ctx2d.moveTo(0, height / 2);
        ctx2d.lineTo(width, height / 2);
        ctx2d.stroke();
      }
    };

    render();

    return () => {
      if (animFrameId.current) {
        cancelAnimationFrame(animFrameId.current);
      }
    };
  }, [audioElement, isPlaying, mode]);

  return (
    <div className="relative bg-[#0F141C] border border-[#242E3D] rounded-lg p-3 overflow-hidden shadow-inner">
      <div className="flex items-center justify-between mb-2">
        <span className="text-[10px] font-mono uppercase tracking-wider text-[#94A3B8] flex items-center gap-1.5">
          <span className={`w-1.5 h-1.5 rounded-full ${isPlaying ? 'bg-emerald-400 animate-pulse' : 'bg-slate-600'}`} />
          Real-Time Audio Telemetry
        </span>
        <div className="flex items-center space-x-1 bg-[#1A222D] p-0.5 rounded border border-[#242E3D]">
          <button
            onClick={() => setMode('bars')}
            className={`px-1.5 py-0.5 rounded text-[10px] flex items-center gap-1 transition-colors ${
              mode === 'bars' ? 'bg-amber-500 text-black font-semibold' : 'text-[#94A3B8] hover:text-white'
            }`}
            title="Frequency Spectrum"
          >
            <BarChart2 className="w-3 h-3" />
            <span>FFT</span>
          </button>
          <button
            onClick={() => setMode('wave')}
            className={`px-1.5 py-0.5 rounded text-[10px] flex items-center gap-1 transition-colors ${
              mode === 'wave' ? 'bg-cyan-500 text-black font-semibold' : 'text-[#94A3B8] hover:text-white'
            }`}
            title="Oscilloscope Waveform"
          >
            <Activity className="w-3 h-3" />
            <span>Wave</span>
          </button>
        </div>
      </div>
      <canvas
        ref={canvasRef}
        width={480}
        height={80}
        className="w-full h-20 rounded bg-[#0B0E14] border border-[#1E293B]"
      />
    </div>
  );
};
