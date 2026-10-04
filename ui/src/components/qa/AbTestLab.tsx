import React, { useState } from 'react';
import { Award, Loader2, Scale, Eye, EyeOff, Sparkles, Star, ShieldCheck, Upload, CheckCircle2, AlertTriangle, FileAudio } from 'lucide-react';
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

  // Double-Blind QA Trial Mode State
  const [isBlindTest, setIsBlindTest] = useState<boolean>(false);
  const [isUnblinded, setIsUnblinded] = useState<boolean>(false);
  const [blindSwap, setBlindSwap] = useState<boolean>(false);
  const [ratingAlpha, setRatingAlpha] = useState<number>(0);
  const [ratingBeta, setRatingBeta] = useState<number>(0);

  // Audio Provenance & Watermark Verification State
  const [verifyingWatermark, setVerifyingWatermark] = useState<boolean>(false);
  const [watermarkResult, setWatermarkResult] = useState<{
    is_detected: boolean;
    confidence: number;
    payload?: number;
    signature_match: boolean;
    sample_rate: number;
    duration_seconds: number;
  } | null>(null);
  const [watermarkFileName, setWatermarkFileName] = useState<string>('');

  const handleVerifyWatermarkFile = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    try {
      setVerifyingWatermark(true);
      setWatermarkFileName(file.name);
      const buffer = await file.arrayBuffer();
      const base64 = btoa(
        new Uint8Array(buffer).reduce((data, byte) => data + String.fromCharCode(byte), '')
      );
      const res = await api.verifyWatermark(base64);
      setWatermarkResult(res);
    } catch (err: any) {
      alert(`Watermark detection failed: ${err.message}`);
    } finally {
      setVerifyingWatermark(false);
    }
  };

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
      setIsUnblinded(false);
      setRatingAlpha(0);
      setRatingBeta(0);
      setBlindSwap(Math.random() > 0.5);

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
        <div className="flex flex-wrap items-start justify-between gap-4">
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

          <button
            type="button"
            onClick={() => {
              setIsBlindTest(!isBlindTest);
              setIsUnblinded(false);
            }}
            className={`flex items-center space-x-2 px-3 py-1.5 rounded-lg border text-xs font-mono transition-all ${
              isBlindTest
                ? 'bg-purple-500/20 text-purple-300 border-purple-500/50 shadow-md shadow-purple-500/10'
                : 'bg-[#121820] text-[#94A3B8] border-[#242E3D] hover:text-white'
            }`}
          >
            {isBlindTest ? <EyeOff className="w-3.5 h-3.5 text-purple-400" /> : <Eye className="w-3.5 h-3.5 text-[#64748B]" />}
            <span>Double-Blind Trial: {isBlindTest ? 'ACTIVE' : 'OFF'}</span>
          </button>
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
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
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

            {!isBlindTest && audioUrlA && (
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

            {!isBlindTest && audioUrlB && (
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
                <span>{isBlindTest ? 'Start Double-Blind Audition Trial' : 'Execute Automated A/B Evaluation'}</span>
              </>
            )}
          </button>
        </div>

        {/* Double-Blind Audition Deck */}
        {isBlindTest && audioUrlA && audioUrlB && !isUnblinded && (
          <div className="p-6 rounded-xl bg-gradient-to-br from-[#121820] to-[#0D1219] border border-purple-500/40 space-y-6 shadow-2xl animate-in fade-in duration-200">
            <div className="flex items-center justify-between border-b border-[#242E3D] pb-3">
              <div className="flex items-center space-x-2">
                <EyeOff className="w-4 h-4 text-purple-400" />
                <span className="text-xs font-bold text-white font-mono uppercase tracking-wider">
                  Blind Audition Session (Randomized Playback)
                </span>
              </div>
              <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20">
                Double-Blind Active
              </span>
            </div>

            <p className="text-xs text-[#94A3B8]">
              Listen to both anonymized samples below. Rate the perceived naturalness and clarity of each sample (MOS 1–5), then unblind to reveal models.
            </p>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {/* Sample Alpha */}
              <div className="p-4 rounded-lg bg-[#0B0E14] border border-[#242E3D] space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold text-purple-400 font-mono">Sample Alpha</span>
                  <span className="text-[10px] text-[#64748B] font-mono">Anonymized</span>
                </div>
                <audio controls src={blindSwap ? audioUrlB : audioUrlA} className="w-full h-8 rounded bg-[#121820]" />
                <div>
                  <div className="text-[11px] font-mono text-[#94A3B8] mb-1">Perceived Naturalness (MOS):</div>
                  <div className="flex items-center space-x-1">
                    {[1, 2, 3, 4, 5].map((star) => (
                      <button
                        key={star}
                        type="button"
                        onClick={() => setRatingAlpha(star)}
                        className={`p-1 rounded ${ratingAlpha >= star ? 'text-amber-400' : 'text-[#334155] hover:text-amber-400/50'}`}
                      >
                        <Star className="w-4 h-4 fill-current" />
                      </button>
                    ))}
                    <span className="text-xs font-mono font-bold text-white ml-2">
                      {ratingAlpha > 0 ? `${ratingAlpha}.0` : '--'}
                    </span>
                  </div>
                </div>
              </div>

              {/* Sample Beta */}
              <div className="p-4 rounded-lg bg-[#0B0E14] border border-[#242E3D] space-y-3">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold text-sky-400 font-mono">Sample Beta</span>
                  <span className="text-[10px] text-[#64748B] font-mono">Anonymized</span>
                </div>
                <audio controls src={blindSwap ? audioUrlA : audioUrlB} className="w-full h-8 rounded bg-[#121820]" />
                <div>
                  <div className="text-[11px] font-mono text-[#94A3B8] mb-1">Perceived Naturalness (MOS):</div>
                  <div className="flex items-center space-x-1">
                    {[1, 2, 3, 4, 5].map((star) => (
                      <button
                        key={star}
                        type="button"
                        onClick={() => setRatingBeta(star)}
                        className={`p-1 rounded ${ratingBeta >= star ? 'text-amber-400' : 'text-[#334155] hover:text-amber-400/50'}`}
                      >
                        <Star className="w-4 h-4 fill-current" />
                      </button>
                    ))}
                    <span className="text-xs font-mono font-bold text-white ml-2">
                      {ratingBeta > 0 ? `${ratingBeta}.0` : '--'}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div className="flex justify-end pt-2">
              <button
                type="button"
                onClick={() => setIsUnblinded(true)}
                className="flex items-center space-x-2 px-5 py-2.5 rounded-lg bg-gradient-to-r from-purple-500 to-indigo-600 hover:from-purple-400 hover:to-indigo-500 text-white font-bold text-xs shadow-lg transition-all"
              >
                <Sparkles className="w-3.5 h-3.5" />
                <span>Reveal Identities & Verdict</span>
              </button>
            </div>
          </div>
        )}

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

        {/* Audio Provenance & Watermark Verifier */}
        <div className="p-6 rounded-2xl bg-[#121820] border border-[#242E3D] space-y-4 shadow-xl">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <div className="p-2 rounded-xl bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
                <ShieldCheck className="w-5 h-5" />
              </div>
              <div>
                <h3 className="text-sm font-bold text-white tracking-tight flex items-center space-x-2">
                  <span>Audio Provenance & Watermark Verifier</span>
                  <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-500/20 text-emerald-300 border border-emerald-500/40">
                    Spread-Spectrum
                  </span>
                </h3>
                <p className="text-xs text-[#94A3B8]">
                  Detect imperceptible 16-bit neural provenance signatures and check for post-synthesis audio tampering
                </p>
              </div>
            </div>
            <label className="flex items-center space-x-2 px-3.5 py-2 rounded-xl bg-[#1A222D] hover:bg-[#242E3D] border border-[#242E3D] text-xs text-white cursor-pointer transition-colors shadow-sm">
              <Upload className="w-3.5 h-3.5 text-emerald-400" />
              <span>Verify Audio File</span>
              <input
                type="file"
                accept=".wav,.mp3"
                onChange={handleVerifyWatermarkFile}
                className="hidden"
              />
            </label>
          </div>

          {verifyingWatermark && (
            <div className="p-4 rounded-xl bg-[#0B0E14] border border-[#242E3D] flex items-center justify-center space-x-2 text-xs text-amber-400">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>Computing cross-correlation with spread-spectrum chip sequence...</span>
            </div>
          )}

          {watermarkResult && !verifyingWatermark && (
            <div className="p-4 rounded-xl bg-[#0B0E14] border border-[#242E3D] space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <FileAudio className="w-4 h-4 text-sky-400" />
                  <span className="text-xs font-mono font-semibold text-white truncate max-w-xs">
                    {watermarkFileName}
                  </span>
                  <span className="text-[10px] text-[#64748B] font-mono">
                    ({watermarkResult.duration_seconds.toFixed(2)}s @ {watermarkResult.sample_rate}Hz)
                  </span>
                </div>
                {watermarkResult.is_detected && watermarkResult.signature_match ? (
                  <span className="px-2.5 py-1 rounded-full text-xs font-mono font-bold bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 flex items-center space-x-1">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    <span>AUTHENTIC VOXFORG SIGNATURE</span>
                  </span>
                ) : (
                  <span className="px-2.5 py-1 rounded-full text-xs font-mono font-bold bg-amber-500/20 text-amber-400 border border-amber-500/40 flex items-center space-x-1">
                    <AlertTriangle className="w-3.5 h-3.5" />
                    <span>NO PROVENANCE WATERMARK</span>
                  </span>
                )}
              </div>

              <div className="grid grid-cols-3 gap-3 pt-2 border-t border-[#242E3D]/50 text-xs font-mono">
                <div className="p-3 rounded-lg bg-[#161F2C] border border-[#242E3D]">
                  <span className="text-[10px] text-[#94A3B8] block">Correlation Confidence</span>
                  <span className="text-base font-bold text-white">
                    {(watermarkResult.confidence * 100).toFixed(1)}%
                  </span>
                </div>
                <div className="p-3 rounded-lg bg-[#161F2C] border border-[#242E3D]">
                  <span className="text-[10px] text-[#94A3B8] block">Decoded Payload</span>
                  <span className="text-base font-bold text-amber-400">
                    {watermarkResult.payload
                      ? `0x${watermarkResult.payload.toString(16).toUpperCase()}`
                      : 'None'}
                  </span>
                </div>
                <div className="p-3 rounded-lg bg-[#161F2C] border border-[#242E3D]">
                  <span className="text-[10px] text-[#94A3B8] block">Tamper Check</span>
                  <span
                    className={`text-base font-bold ${
                      watermarkResult.signature_match ? 'text-emerald-400' : 'text-[#64748B]'
                    }`}
                  >
                    {watermarkResult.signature_match ? 'PASS (Untampered)' : 'N/A'}
                  </span>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
