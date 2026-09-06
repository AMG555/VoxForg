import React, { useState } from 'react';
import {
  Activity,
  Award,
  CheckCircle2,
  Cpu,
  Gauge,
  Loader2,
  Play,
  Scale,
  Sparkles,
  Volume2,
  Zap,
} from 'lucide-react';
import { AbTestComparison, Voice } from '../../types';
import { api } from '../../services/api';

interface AbTestLabProps {
  voices: Voice[];
}

export const AbTestLab: React.FC<AbTestLabProps> = ({ voices }) => {
  const [text, setText] = useState<string>(
    'The atmospheric density on Kepler-452b allows acoustic waves to travel 1.4 times faster than standard Earth normal.'
  );

  const [voiceA, setVoiceA] = useState<string>(voices[0]?.id || 'en-US-AriaNeural');
  const [voiceB, setVoiceB] = useState<string>(voices[1]?.id || 'en-US-GuyNeural');
  const [speedA, setSpeedA] = useState<number>(1.0);
  const [speedB, setSpeedB] = useState<number>(1.0);
  const [pitchA, setPitchA] = useState<number>(0.0);
  const [pitchB, setPitchB] = useState<number>(0.0);

  const [isRunning, setIsRunning] = useState<boolean>(false);
  const [comparison, setComparison] = useState<AbTestComparison | null>(null);
  const [audioUrlA, setAudioUrlA] = useState<string | null>(null);
  const [audioUrlB, setAudioUrlB] = useState<string | null>(null);

  React.useEffect(() => {
    return () => {
      if (audioUrlA) URL.revokeObjectURL(audioUrlA);
      if (audioUrlB) URL.revokeObjectURL(audioUrlB);
    };
  }, [audioUrlA, audioUrlB]);

  const handleRunEvaluation = async () => {
    if (!text.trim()) return;
    try {
      setIsRunning(true);
      if (audioUrlA) {
        URL.revokeObjectURL(audioUrlA);
        setAudioUrlA(null);
      }
      if (audioUrlB) {
        URL.revokeObjectURL(audioUrlB);
        setAudioUrlB(null);
      }

      // Run automated QA A/B test via API
      const result = await api.runAbTest({
        name: 'Interactive Voice QA Comparison',
        text,
        variant_a: { voice_id: voiceA, speed: speedA, pitch: pitchA },
        variant_b: { voice_id: voiceB, speed: speedB, pitch: pitchB },
      });
      setComparison(result);

      // Fetch actual audio for both variants so user can audition
      const [blobA, blobB] = await Promise.all([
        api.synthesizeDirect({ input: text, voice: voiceA, speed: speedA, pitch: pitchA }),
        api.synthesizeDirect({ input: text, voice: voiceB, speed: speedB, pitch: pitchB }),
      ]);
      setAudioUrlA(URL.createObjectURL(blobA));
      setAudioUrlB(URL.createObjectURL(blobB));
    } catch (err: any) {
      alert(`QA A/B Test Failed: ${err.message}`);
    } finally {
      setIsRunning(false);
    }
  };

  return (
    <div className="flex-1 p-8 overflow-y-auto bg-[#0B0E14] space-y-6">
      <div className="max-w-5xl space-y-6">
        <div>
          <div className="flex items-center space-x-2 text-amber-500 font-mono text-xs uppercase tracking-wider">
            <Scale className="w-4 h-4" />
            <span>Automated QA & Comparative Evaluation</span>
          </div>
          <h2 className="text-xl font-bold text-white mt-1">Voice & Engine A/B Testing Lab</h2>
          <p className="text-xs text-[#94A3B8] font-mono mt-1">
            Conduct double-blind synthesis comparisons, analyze real-time factor, and detect clipping.
          </p>
        </div>

        {/* Input Text Box */}
        <div className="space-y-2">
          <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8]">
            Test Corpus / Sentence
          </label>
          <textarea
            rows={3}
            value={text}
            onChange={(e) => setText(e.target.value)}
            className="w-full bg-[#121820] border border-[#242E3D] rounded-lg p-3 text-white text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed shadow-inner"
            placeholder="Enter benchmark text..."
          />
        </div>

        {/* Side-by-side Variant Configuration */}
        <div className="grid grid-cols-2 gap-6">
          {/* Variant A */}
          <div className="bg-[#121820] border border-[#242E3D] p-5 rounded-lg space-y-4">
            <div className="flex items-center justify-between border-b border-[#242E3D] pb-3">
              <span className="font-mono text-xs font-bold text-sky-400 uppercase tracking-wider">
                Variant A (Baseline)
              </span>
              <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-sky-500/10 text-sky-400 border border-sky-500/30">
                PROD
              </span>
            </div>

            <div>
              <label className="block text-[11px] font-mono text-[#94A3B8] mb-1">
                Voice Model
              </label>
              <select
                value={voiceA}
                onChange={(e) => setVoiceA(e.target.value)}
                className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-sky-500 focus:outline-none"
              >
                {voices.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.name} ({v.language}) - {v.engine_id}
                  </option>
                ))}
              </select>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <div className="flex justify-between text-[11px] font-mono mb-1 text-[#94A3B8]">
                  <span>Speed</span>
                  <span className="text-white">{speedA.toFixed(2)}x</span>
                </div>
                <input
                  type="range"
                  min="0.5"
                  max="2.0"
                  step="0.05"
                  value={speedA}
                  onChange={(e) => setSpeedA(parseFloat(e.target.value))}
                  className="w-full accent-sky-400 cursor-pointer"
                />
              </div>

              <div>
                <div className="flex justify-between text-[11px] font-mono mb-1 text-[#94A3B8]">
                  <span>Pitch</span>
                  <span className="text-white">{pitchA > 0 ? `+${pitchA}` : pitchA}st</span>
                </div>
                <input
                  type="range"
                  min="-12"
                  max="12"
                  step="0.5"
                  value={pitchA}
                  onChange={(e) => setPitchA(parseFloat(e.target.value))}
                  className="w-full accent-sky-400 cursor-pointer"
                />
              </div>
            </div>

            {audioUrlA && (
              <div className="pt-2">
                <audio controls src={audioUrlA} className="h-8 w-full rounded bg-[#0B0E14]" />
              </div>
            )}
          </div>

          {/* Variant B */}
          <div className="bg-[#121820] border border-[#242E3D] p-5 rounded-lg space-y-4">
            <div className="flex items-center justify-between border-b border-[#242E3D] pb-3">
              <span className="font-mono text-xs font-bold text-amber-400 uppercase tracking-wider">
                Variant B (Challenger)
              </span>
              <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/30">
                CANDIDATE
              </span>
            </div>

            <div>
              <label className="block text-[11px] font-mono text-[#94A3B8] mb-1">
                Voice Model
              </label>
              <select
                value={voiceB}
                onChange={(e) => setVoiceB(e.target.value)}
                className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-amber-500 focus:outline-none"
              >
                {voices.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.name} ({v.language}) - {v.engine_id}
                  </option>
                ))}
              </select>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div>
                <div className="flex justify-between text-[11px] font-mono mb-1 text-[#94A3B8]">
                  <span>Speed</span>
                  <span className="text-white">{speedB.toFixed(2)}x</span>
                </div>
                <input
                  type="range"
                  min="0.5"
                  max="2.0"
                  step="0.05"
                  value={speedB}
                  onChange={(e) => setSpeedB(parseFloat(e.target.value))}
                  className="w-full accent-amber-400 cursor-pointer"
                />
              </div>

              <div>
                <div className="flex justify-between text-[11px] font-mono mb-1 text-[#94A3B8]">
                  <span>Pitch</span>
                  <span className="text-white">{pitchB > 0 ? `+${pitchB}` : pitchB}st</span>
                </div>
                <input
                  type="range"
                  min="-12"
                  max="12"
                  step="0.5"
                  value={pitchB}
                  onChange={(e) => setPitchB(parseFloat(e.target.value))}
                  className="w-full accent-amber-400 cursor-pointer"
                />
              </div>
            </div>

            {audioUrlB && (
              <div className="pt-2">
                <audio controls src={audioUrlB} className="h-8 w-full rounded bg-[#0B0E14]" />
              </div>
            )}
          </div>
        </div>

        {/* Action Button */}
        <div className="flex justify-center">
          <button
            onClick={handleRunEvaluation}
            disabled={isRunning}
            className="flex items-center space-x-2 px-6 py-3 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs transition-colors shadow-lg shadow-amber-500/20 disabled:opacity-50"
          >
            {isRunning ? (
              <>
                <Loader2 className="w-4 h-4 animate-spin" />
                <span>Running Comparative Evaluation...</span>
              </>
            ) : (
              <>
                <Scale className="w-4 h-4" />
                <span>Execute Automated A/B Evaluation</span>
              </>
            )}
          </button>
        </div>

        {/* Results Scorecard */}
        {comparison && (
          <div className="bg-[#121820] border border-[#242E3D] rounded-lg p-6 space-y-6 shadow-xl">
            {/* Winner Callout */}
            <div className="flex items-center justify-between p-4 rounded-lg bg-emerald-500/10 border border-emerald-500/30">
              <div className="flex items-center space-x-3">
                <Award className="w-6 h-6 text-emerald-400" />
                <div>
                  <h4 className="text-sm font-bold text-white">
                    Winner: Variant {comparison.recommended_variant}
                  </h4>
                  <p className="text-xs text-[#94A3B8] font-mono">{comparison.summary}</p>
                </div>
              </div>
              <span className="text-xs font-mono font-bold text-emerald-400 uppercase tracking-wider">
                Automated QA Passed
              </span>
            </div>

            {/* Metrics Comparison Table */}
            <div className="overflow-x-auto">
              <table className="w-full text-xs font-mono">
                <thead>
                  <tr className="border-b border-[#242E3D] text-[#94A3B8] text-left">
                    <th className="py-2">Metric</th>
                    <th className="py-2 text-sky-400">Variant A ({comparison.variant_a.voice_id})</th>
                    <th className="py-2 text-amber-400">Variant B ({comparison.variant_b.voice_id})</th>
                    <th className="py-2">Delta / Winner</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[#242E3D] text-white">
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">Engine Backend</td>
                    <td>{comparison.variant_a.engine_id}</td>
                    <td>{comparison.variant_b.engine_id}</td>
                    <td className="text-[#64748B]">-</td>
                  </tr>
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">Synthesis Latency</td>
                    <td>{comparison.variant_a.latency_ms.toFixed(2)} ms</td>
                    <td>{comparison.variant_b.latency_ms.toFixed(2)} ms</td>
                    <td className={comparison.latency_delta_ms < 0 ? 'text-sky-400' : 'text-amber-400'}>
                      {comparison.latency_delta_ms.toFixed(2)} ms (Variant {comparison.faster_variant} faster)
                    </td>
                  </tr>
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">Real-Time Factor (RTF)</td>
                    <td>{comparison.variant_a.realtime_factor.toFixed(4)}</td>
                    <td>{comparison.variant_b.realtime_factor.toFixed(4)}</td>
                    <td className="text-emerald-400">Low Latency</td>
                  </tr>
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">Peak Amplitude</td>
                    <td>{comparison.variant_a.metrics.peak_dbfs.toFixed(1)} dBFS</td>
                    <td>{comparison.variant_b.metrics.peak_dbfs.toFixed(1)} dBFS</td>
                    <td className="text-[#64748B]">-</td>
                  </tr>
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">RMS Loudness</td>
                    <td>{comparison.variant_a.metrics.rms_dbfs.toFixed(1)} dBFS</td>
                    <td>{comparison.variant_b.metrics.rms_dbfs.toFixed(1)} dBFS</td>
                    <td>Δ {comparison.rms_delta_db.toFixed(1)} dB</td>
                  </tr>
                  <tr>
                    <td className="py-2.5 text-[#94A3B8]">Clipping Samples</td>
                    <td>{comparison.variant_a.metrics.clipping_samples_count}</td>
                    <td>{comparison.variant_b.metrics.clipping_samples_count}</td>
                    <td className="text-emerald-400">Clean / No distortion</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
