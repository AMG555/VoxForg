import React, { useState } from 'react';
import {
  X,
  Sparkles,
  Mic,
  Sliders,
  Cpu,
  CheckCircle2,
  AlertCircle,
  Copy,
  Check,
  BookOpen,
  Waves,
  Smile,
  Volume2,
} from 'lucide-react';

interface VoiceTipsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onApplyPrompt?: (text: string) => void;
}

export const VoiceTipsModal: React.FC<VoiceTipsModalProps> = ({
  isOpen,
  onClose,
  onApplyPrompt,
}) => {
  const [activeTab, setActiveTab] = useState<
    'audio' | 'modulation' | 'tags' | 'mastering' | 'sidecar'
  >('audio');
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  if (!isOpen) return null;

  const handleCopy = (text: string, index: number) => {
    navigator.clipboard.writeText(text);
    setCopiedIndex(index);
    setTimeout(() => setCopiedIndex(null), 2000);
  };

  const samplePrompts = [
    {
      title: 'Cinematic & Intimate Storytelling',
      text: '"Do you see how the morning sun transforms the room into a golden paradise?" he whispered with wonder in his voice as his eyes watched the play of light on the old wooden floorboards.',
      tip: 'Notice how commas and question marks sculpt organic breathing pauses and pitch contours.',
    },
    {
      title: 'Conversational Drama with Emotion Tags',
      text: 'Wait... did you hear that? [whisper] I think someone is outside. [sigh] Let me check before we make any noise.',
      tip: 'The [whisper] and [sigh] paralinguistic tags inject organic vocal fold relaxation.',
    },
    {
      title: 'Broadcast Authoritative Pacing',
      text: 'Good evening. Breaking reports tonight confirm that the deep space telescope has detected unprecedented atmospheric biosignatures — marking a transformative milestone in modern astrophysics.',
      tip: 'Em-dashes (—) produce authoritative 120ms breathing pauses without losing momentum.',
    },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/85 backdrop-blur-md p-4 sm:p-6 overflow-y-auto animate-in fade-in duration-200">
      <div className="bg-[#0E131A] border border-[#242E3D] rounded-2xl w-full max-w-3xl max-h-[92vh] flex flex-col shadow-2xl overflow-hidden border-sky-500/20">
        {/* Header */}
        <div className="p-4 sm:p-5 border-b border-[#242E3D] bg-[#121822] flex items-center justify-between shrink-0">
          <div className="flex items-center space-x-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-sky-500/20 to-blue-600/20 border border-sky-500/40 flex items-center justify-center shadow-lg shadow-sky-500/10">
              <span className="font-mono font-bold text-sky-400 text-base">i</span>
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h3 className="text-base font-bold text-white tracking-tight">
                  Voice Optimization & Realism Studio Guide
                </h3>
                <span className="px-2 py-0.5 rounded text-[10px] font-mono font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                  100% Fidelity Tips
                </span>
              </div>
              <p className="text-xs text-[#94A3B8] font-mono mt-0.5">
                Pro tips to maximize vocal modulation, speaker cloning accuracy, and human realism
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-lg text-[#64748B] hover:text-white hover:bg-[#1A222D] transition-colors"
            title="Close Guide"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex items-center px-4 sm:px-6 pt-3 border-b border-[#242E3D] bg-[#0E131A] space-x-1 sm:space-x-2 overflow-x-auto shrink-0 select-none scrollbar-none">
          <button
            type="button"
            onClick={() => setActiveTab('audio')}
            className={`flex items-center space-x-2 px-3 py-2 text-xs font-mono font-medium border-b-2 transition-all whitespace-nowrap ${
              activeTab === 'audio'
                ? 'border-sky-400 text-sky-300 bg-sky-500/5'
                : 'border-transparent text-[#94A3B8] hover:text-white'
            }`}
          >
            <Mic className="w-3.5 h-3.5" />
            <span>1. Reference Audio Secrets</span>
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('modulation')}
            className={`flex items-center space-x-2 px-3 py-2 text-xs font-mono font-medium border-b-2 transition-all whitespace-nowrap ${
              activeTab === 'modulation'
                ? 'border-amber-400 text-amber-300 bg-amber-500/5'
                : 'border-transparent text-[#94A3B8] hover:text-white'
            }`}
          >
            <Waves className="w-3.5 h-3.5" />
            <span>2. Modulation & Punctuation</span>
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('tags')}
            className={`flex items-center space-x-2 px-3 py-2 text-xs font-mono font-medium border-b-2 transition-all whitespace-nowrap ${
              activeTab === 'tags'
                ? 'border-purple-400 text-purple-300 bg-purple-500/5'
                : 'border-transparent text-[#94A3B8] hover:text-white'
            }`}
          >
            <Smile className="w-3.5 h-3.5" />
            <span>3. Emotion & Speech Tags</span>
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('mastering')}
            className={`flex items-center space-x-2 px-3 py-2 text-xs font-mono font-medium border-b-2 transition-all whitespace-nowrap ${
              activeTab === 'mastering'
                ? 'border-emerald-400 text-emerald-300 bg-emerald-500/5'
                : 'border-transparent text-[#94A3B8] hover:text-white'
            }`}
          >
            <Sliders className="w-3.5 h-3.5" />
            <span>4. Studio Mastering Rack</span>
          </button>
          <button
            type="button"
            onClick={() => setActiveTab('sidecar')}
            className={`flex items-center space-x-2 px-3 py-2 text-xs font-mono font-medium border-b-2 transition-all whitespace-nowrap ${
              activeTab === 'sidecar'
                ? 'border-cyan-400 text-cyan-300 bg-cyan-500/5'
                : 'border-transparent text-[#94A3B8] hover:text-white'
            }`}
          >
            <Cpu className="w-3.5 h-3.5" />
            <span>5. Neural Sidecar (100% Clone)</span>
          </button>
        </div>

        {/* Scrollable Content Body */}
        <div className="p-5 sm:p-6 overflow-y-auto flex-1 space-y-5 text-sm">
          {/* TAB 1: Reference Audio Secrets */}
          {activeTab === 'audio' && (
            <div className="space-y-4 animate-in fade-in duration-150">
              <div className="p-4 rounded-xl bg-sky-500/10 border border-sky-500/30 flex items-start space-x-3">
                <Sparkles className="w-5 h-5 text-sky-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="font-semibold text-sky-200 text-sm">The Zero-Shot Cloning Principle</h4>
                  <p className="text-xs text-[#94A3B8] mt-1 leading-relaxed">
                    Voice cloning extracts 512 vocal tract formant resonance coordinates and fundamental pitch dynamics ($F_0$).
                    The synthesized voice will replicate every acoustic trait of your reference sample — including room reflections, mic proximity, and vocal energy.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="p-4 rounded-xl bg-[#121822] border border-emerald-500/30 space-y-2.5">
                  <div className="flex items-center space-x-2 text-emerald-400 font-semibold text-xs uppercase tracking-wider">
                    <CheckCircle2 className="w-4 h-4" />
                    <span>Best Practices (Do This)</span>
                  </div>
                  <ul className="text-xs text-[#CBD5E1] space-y-2 list-disc list-inside">
                    <li><strong className="text-white">5–12 Seconds Duration:</strong> Optimal length for accurate pitch-mean and formant extraction without overfitting.</li>
                    <li><strong className="text-white">Dry Acoustic Space:</strong> Record in a quiet closet, padded room, or vocal booth with zero room echo.</li>
                    <li><strong className="text-white">Natural Conversational Flow:</strong> Speak with animated, natural inflection. Variety in pitch contour gives the AI rich pitch dynamics.</li>
                    <li><strong className="text-white">24kHz / 48kHz Clean WAV:</strong> Uncompressed audio preserves high-frequency harmonic air above 8kHz.</li>
                  </ul>
                </div>

                <div className="p-4 rounded-xl bg-[#121822] border border-rose-500/30 space-y-2.5">
                  <div className="flex items-center space-x-2 text-rose-400 font-semibold text-xs uppercase tracking-wider">
                    <AlertCircle className="w-4 h-4" />
                    <span>Common Pitfalls (Avoid This)</span>
                  </div>
                  <ul className="text-xs text-[#CBD5E1] space-y-2 list-disc list-inside">
                    <li><strong className="text-white">Background Noise & Music:</strong> Never clone audio with background music, AC hum, or computer fans.</li>
                    <li><strong className="text-white">Room Echo & Reverb:</strong> Avoid large tiled rooms; synthetic clones will duplicate hollow echo.</li>
                    <li><strong className="text-white">Monotone or Whispered Input:</strong> Monotone reference audio causes the engine to synthesize flat, lifeless prosody.</li>
                    <li><strong className="text-white">Low Bitrate MP3s:</strong> MP3 compression artifacts warp formant filter frequencies.</li>
                  </ul>
                </div>
              </div>

              <div className="p-3.5 rounded-lg bg-[#0A0E14] border border-[#242E3D] font-mono text-xs text-[#94A3B8]">
                💡 <span className="text-white font-semibold">Pro Studio Tip:</span> Click the <strong className="text-amber-400">Assess Audio Quality</strong> button inside the Clone Voice modal before saving. VoxForg will instantly verify SNR ratio, clipping thresholds, and frequency response!
              </div>
            </div>
          )}

          {/* TAB 2: Modulation & Punctuation */}
          {activeTab === 'modulation' && (
            <div className="space-y-4 animate-in fade-in duration-150">
              <div className="p-4 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-start space-x-3">
                <Waves className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="font-semibold text-amber-200 text-sm">Punctuation Dictates Human Breath & Melody</h4>
                  <p className="text-xs text-[#94A3B8] mt-1 leading-relaxed">
                    Neural speech engines treat punctuation marks as acoustic breath markers and melodic pitch shifts.
                    Crafting punctuation is the single most effective way to shape human expression!
                  </p>
                </div>
              </div>

              <div className="space-y-2.5">
                <h5 className="text-xs font-semibold uppercase tracking-wider text-[#94A3B8]">Punctuation Cadence Rules</h5>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
                  <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <span className="text-amber-400 font-mono font-bold text-sm">Commas (,)</span>
                    <p className="text-[#94A3B8] mt-1">Generates 60–90ms gentle pause and slight pitch suspension. Use often to segment phrases.</p>
                  </div>
                  <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <span className="text-sky-400 font-mono font-bold text-sm">Em-Dashes (—)</span>
                    <p className="text-[#94A3B8] mt-1">Inserts a deliberate 120ms dramatic breath pause without dropping sentence energy.</p>
                  </div>
                  <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <span className="text-purple-400 font-mono font-bold text-sm">Ellipses (...)</span>
                    <p className="text-[#94A3B8] mt-1">Creates a thoughtful trailing hesitation, lowering speech volume and slowing pace.</p>
                  </div>
                </div>
              </div>

              <div className="space-y-2.5 pt-2">
                <div className="flex items-center justify-between">
                  <h5 className="text-xs font-semibold uppercase tracking-wider text-[#94A3B8]">High-Modulation Sample Prompts</h5>
                  <span className="text-[11px] text-[#64748B] font-mono">Click to copy or use</span>
                </div>
                <div className="space-y-2.5">
                  {samplePrompts.map((sample, idx) => (
                    <div
                      key={idx}
                      className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] hover:border-amber-500/40 transition-colors space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-xs font-semibold text-white">{sample.title}</span>
                        <div className="flex items-center space-x-1.5">
                          {onApplyPrompt && (
                            <button
                              type="button"
                              onClick={() => {
                                onApplyPrompt(sample.text);
                                onClose();
                              }}
                              className="px-2 py-0.5 rounded text-[11px] font-mono bg-amber-500/10 text-amber-300 hover:bg-amber-500/20 border border-amber-500/30 transition-colors"
                            >
                              Load in Studio
                            </button>
                          )}
                          <button
                            type="button"
                            onClick={() => handleCopy(sample.text, idx)}
                            className="p-1 rounded text-[#94A3B8] hover:text-white transition-colors"
                            title="Copy prompt"
                          >
                            {copiedIndex === idx ? (
                              <Check className="w-3.5 h-3.5 text-emerald-400" />
                            ) : (
                              <Copy className="w-3.5 h-3.5" />
                            )}
                          </button>
                        </div>
                      </div>
                      <p className="text-xs text-[#CBD5E1] font-serif italic bg-[#0A0E14] p-2.5 rounded-lg border border-[#1A222D]">
                        {sample.text}
                      </p>
                      <p className="text-[11px] text-[#94A3B8] font-mono">💡 {sample.tip}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          )}

          {/* TAB 3: Emotion & Speech Tags */}
          {activeTab === 'tags' && (
            <div className="space-y-4 animate-in fade-in duration-150">
              <div className="p-4 rounded-xl bg-purple-500/10 border border-purple-500/30 flex items-start space-x-3">
                <Smile className="w-5 h-5 text-purple-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="font-semibold text-purple-200 text-sm">Paralinguistic Emotional Expression Tags</h4>
                  <p className="text-xs text-[#94A3B8] mt-1 leading-relaxed">
                    VoxForg includes a physiological biometric paralinguistic tag processor. Inserting these bracketed tags inline in your prompt instructs the acoustic synthesis pipeline to inject natural non-verbal human vocal inflections.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-0.5 rounded font-mono font-bold bg-sky-500/15 text-sky-300 border border-sky-500/30">
                      [whisper]
                    </span>
                    <span className="text-white font-medium">Hushed Intimacy</span>
                  </div>
                  <p className="text-[#94A3B8] text-[11px]">Reduces vocal fold phonation, increases breathy aspiration and softens high frequencies.</p>
                </div>

                <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-0.5 rounded font-mono font-bold bg-amber-500/15 text-amber-300 border border-amber-500/30">
                      [laugh] / [chuckle]
                    </span>
                    <span className="text-white font-medium">Playful Warmth</span>
                  </div>
                  <p className="text-[#94A3B8] text-[11px]">Injects harmonic breath rhythm and pitch elevation representing amusement.</p>
                </div>

                <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-0.5 rounded font-mono font-bold bg-indigo-500/15 text-indigo-300 border border-indigo-500/30">
                      [sigh]
                    </span>
                    <span className="text-white font-medium">Relief or Weariness</span>
                  </div>
                  <p className="text-[#94A3B8] text-[11px]">Generates an audible exhalation with declining pitch contour before the phrase.</p>
                </div>

                <div className="p-3 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="px-2 py-0.5 rounded font-mono font-bold bg-emerald-500/15 text-emerald-300 border border-emerald-500/30">
                      [gasp]
                    </span>
                    <span className="text-white font-medium">Sudden Realization</span>
                  </div>
                  <p className="text-[#94A3B8] text-[11px]">Injects a sharp pre-speech inhalation aspiration burst, conveying shock or urgency.</p>
                </div>
              </div>

              <div className="p-3.5 rounded-lg bg-[#0A0E14] border border-[#242E3D] text-xs font-mono text-[#94A3B8]">
                💡 <span className="text-white font-semibold">Quick Usage:</span> In the Text-to-Speech studio, click the quick-insert tag buttons right above the prompt box to place tags wherever you want emotional emphasis!
              </div>
            </div>
          )}

          {/* TAB 4: Studio Mastering Rack */}
          {activeTab === 'mastering' && (
            <div className="space-y-4 animate-in fade-in duration-150">
              <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/30 flex items-start space-x-3">
                <Sliders className="w-5 h-5 text-emerald-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="font-semibold text-emerald-200 text-sm">Pro Studio Mastering DSP Rack</h4>
                  <p className="text-xs text-[#94A3B8] mt-1 leading-relaxed">
                    Raw neural output sounds clean but sterile. Toggling <strong className="text-amber-400">PRO STUDIO</strong> mode passes audio through our 64-bit WebAudio mastering pipeline.
                  </p>
                </div>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-xs">
                <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <span className="text-amber-400 font-semibold flex items-center space-x-1.5">
                    <Volume2 className="w-3.5 h-3.5" />
                    <span>Analog Tube Warmth (25–40%)</span>
                  </span>
                  <p className="text-[#94A3B8] text-[11px]">Adds subtle asymmetric even-order tube harmonics, recreating the rich chest resonance of expensive Neumann studio condenser microphones.</p>
                </div>

                <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <span className="text-sky-400 font-semibold flex items-center space-x-1.5">
                    <Sliders className="w-3.5 h-3.5" />
                    <span>6.5kHz Dynamic De-Esser (30–50%)</span>
                  </span>
                  <p className="text-[#94A3B8] text-[11px]">Suppresses harsh high-frequency sibilance ("s", "sh", "ch") without dulling vocal presence or clarity.</p>
                </div>

                <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <span className="text-purple-400 font-semibold flex items-center space-x-1.5">
                    <Waves className="w-3.5 h-3.5" />
                    <span>Room & Studio Reverb (10–15%)</span>
                  </span>
                  <p className="text-[#94A3B8] text-[11px]">Gives dry audio a sense of real three-dimensional acoustic space. Use "Podcast" for intimate interviews or "Studio" for voiceovers.</p>
                </div>

                <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] space-y-1">
                  <span className="text-emerald-400 font-semibold flex items-center space-x-1.5">
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    <span>Broadcast Brickwall Limiter</span>
                  </span>
                  <p className="text-[#94A3B8] text-[11px]">Prevents inter-sample clipping distortion (-0.5 dBFS ceiling) and boosts commercial loudness and punch.</p>
                </div>
              </div>
            </div>
          )}

          {/* TAB 5: Neural Sidecar */}
          {activeTab === 'sidecar' && (
            <div className="space-y-4 animate-in fade-in duration-150">
              <div className="p-4 rounded-xl bg-cyan-500/10 border border-cyan-500/30 flex items-start space-x-3">
                <Cpu className="w-5 h-5 text-cyan-400 shrink-0 mt-0.5" />
                <div>
                  <h4 className="font-semibold text-cyan-200 text-sm">100% Neural Zero-Shot Diffusion Sidecar</h4>
                  <p className="text-xs text-[#94A3B8] mt-1 leading-relaxed">
                    By default, VoxForg runs its high-realism biometric DSP engine. For true 100% neural cloning where every breath, accent, and inflection matches your clone sample with zero compromise, VoxForg automatically detects external diffusion sidecars!
                  </p>
                </div>
              </div>

              <div className="space-y-2.5">
                <h5 className="text-xs font-semibold uppercase tracking-wider text-[#94A3B8]">Supported Neural Sidecars</h5>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 text-xs">
                  <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <div className="flex items-center justify-between">
                      <span className="text-white font-bold">CosyVoice 2</span>
                      <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-cyan-500/20 text-cyan-300">Port 9000</span>
                    </div>
                    <p className="text-[#94A3B8] mt-1.5 text-[11px]">Alibaba's state-of-the-art multi-lingual diffusion model with zero-shot emotional transfer.</p>
                  </div>

                  <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <div className="flex items-center justify-between">
                      <span className="text-white font-bold">F5-TTS</span>
                      <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-indigo-500/20 text-indigo-300">Port 8880</span>
                    </div>
                    <p className="text-[#94A3B8] mt-1.5 text-[11px]">Flow-matching non-autoregressive neural TTS engine for ultra-fast expressive cloning.</p>
                  </div>

                  <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D]">
                    <div className="flex items-center justify-between">
                      <span className="text-white font-bold">Chatterbox / ChatTTS</span>
                      <span className="px-1.5 py-0.5 rounded text-[10px] font-mono bg-amber-500/20 text-amber-300">Port 5000</span>
                    </div>
                    <p className="text-[#94A3B8] mt-1.5 text-[11px]">Conversational engine optimized for natural laughter, sighs, and conversational speech.</p>
                  </div>
                </div>
              </div>

              <div className="p-3.5 rounded-xl bg-[#121822] border border-[#242E3D] space-y-2">
                <h5 className="text-xs font-semibold text-white">How Auto-Detection Works:</h5>
                <p className="text-xs text-[#94A3B8] leading-relaxed">
                  Simply run any of the above models locally (or in Docker/WSL). VoxForg continuously probes ports <code className="text-cyan-300 bg-black/40 px-1 py-0.5 rounded">9000</code>, <code className="text-indigo-300 bg-black/40 px-1 py-0.5 rounded">8880</code>, and <code className="text-amber-300 bg-black/40 px-1 py-0.5 rounded">5000</code>. When detected, VoxForg transparently routes reference audio directly to the neural sidecar with zero manual configuration!
                </p>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-[#242E3D] bg-[#121822] shrink-0 flex items-center justify-between">
          <div className="flex items-center space-x-2 text-xs text-[#94A3B8] font-mono">
            <BookOpen className="w-3.5 h-3.5 text-sky-400" />
            <span>VoxForg Biometric Audio Architecture v0.2.0</span>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="px-5 py-2 rounded-lg bg-sky-500 hover:bg-sky-400 text-black text-xs font-semibold font-mono transition-colors shadow-lg shadow-sky-500/20"
          >
            Got It
          </button>
        </div>
      </div>
    </div>
  );
};
