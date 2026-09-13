import React, { useState, useRef } from 'react';
import { Play, Loader2, Search, Sparkles, Upload, X, Mic } from 'lucide-react';
import { Voice } from '../../types';
import { api } from '../../services/api';
import { AudioVisualizer } from '../common/AudioVisualizer';

interface VoiceLabProps {
  voices: Voice[];
}

export const VoiceLab: React.FC<VoiceLabProps> = ({ voices }) => {
  const [selectedVoiceId, setSelectedVoiceId] = useState<string>('en-US-AriaNeural');
  const [text, setText] = useState<string>(
    'The atmospheric density on Kepler-452b allows acoustic waves to travel 1.4 times faster than standard Earth normal.'
  );
  const [speed, setSpeed] = useState<number>(1.0);
  const [pitch, setPitch] = useState<number>(0.0);
  const [isGenerating, setIsGenerating] = useState<boolean>(false);
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // Voice cloning studio state
  const [showCloneModal, setShowCloneModal] = useState<boolean>(false);
  const [cloneName, setCloneName] = useState<string>('');
  const [cloneTranscript, setCloneTranscript] = useState<string>('');
  const [cloneGender, setCloneGender] = useState<'male' | 'female' | 'neutral'>('neutral');
  const [cloneAudioBase64, setCloneAudioBase64] = useState<string>('');
  const [cloneFileName, setCloneFileName] = useState<string>('');
  const [isCloning, setIsCloning] = useState<boolean>(false);

  React.useEffect(() => {
    return () => {
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
      }
    };
  }, [audioUrl]);

  const filteredVoices = voices.filter(
    (v) =>
      v.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      v.language.toLowerCase().includes(searchQuery.toLowerCase()) ||
      v.engine_id.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const selectedVoice = voices.find((v) => v.id === selectedVoiceId) || voices[0];

  const handleGenerate = async () => {
    if (!text.trim()) return;
    try {
      setIsGenerating(true);
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }
      const blob = await api.synthesizeDirect({
        input: text,
        voice: selectedVoiceId,
        speed,
        pitch,
      });
      const url = URL.createObjectURL(blob);
      setAudioUrl(url);
    } catch (err: any) {
      alert(`Synthesis Error: ${err.message}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setCloneFileName(file.name);
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result as string;
        const base64 = result.includes(',') ? result.split(',')[1] : result;
        setCloneAudioBase64(base64);
      };
      reader.readAsDataURL(file);
    }
  };

  const handleCloneSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!cloneName || !cloneAudioBase64) return;
    setIsCloning(true);
    try {
      const res = await api.cloneVoice({
        name: cloneName,
        engine_id: 'qwen3-tts',
        reference_audio_base64: cloneAudioBase64,
        reference_transcript: cloneTranscript || undefined,
        gender: cloneGender,
        language: 'en-US',
      });
      const newVoice: Voice = {
        id: res.id || `cloned-${Date.now()}`,
        name: `${cloneName} (Cloned)`,
        engine_id: 'qwen3-tts',
        language: 'en-US',
        gender: cloneGender,
        sample_rate_hz: 24000,
        tags: ['cloned', 'zero-shot'],
        description: 'Zero-shot voice profile generated from reference sample',
      };
      voices.unshift(newVoice);
      setSelectedVoiceId(newVoice.id);
      setShowCloneModal(false);
      setCloneName('');
      setCloneTranscript('');
      setCloneAudioBase64('');
      setCloneFileName('');
    } catch (err: any) {
      alert(`Cloning failed: ${err.message}`);
    } finally {
      setIsCloning(false);
    }
  };

  return (
    <div className="flex-1 flex overflow-hidden bg-[#0B0E14] relative">
      {/* Voices List Sidebar */}
      <div className="w-80 border-r border-[#242E3D] bg-[#121820] flex flex-col h-full select-none">
        <div className="p-3 border-b border-[#242E3D] space-y-2">
          <button
            onClick={() => setShowCloneModal(true)}
            className="w-full flex items-center justify-center space-x-2 py-1.5 px-3 rounded bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 text-xs font-mono font-semibold transition-colors"
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>Clone New Voice</span>
          </button>

          <div className="relative">
            <Search className="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-[#64748B]" />
            <input
              type="text"
              placeholder="Search voices, languages..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-[#0B0E14] border border-[#242E3D] rounded pl-8 pr-3 py-1.5 text-xs text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none"
            />
          </div>
        </div>

        <div className="flex-1 overflow-y-auto divide-y divide-[#242E3D]">
          {filteredVoices.map((voice) => (
            <div
              key={voice.id}
              onClick={() => setSelectedVoiceId(voice.id)}
              className={`p-3 cursor-pointer transition-colors ${
                selectedVoiceId === voice.id
                  ? 'bg-amber-500/10 border-l-2 border-amber-500'
                  : 'hover:bg-[#1A222D]/60'
              }`}
            >
              <div className="flex items-center justify-between">
                <h4 className="text-xs font-semibold text-white">{voice.name}</h4>
                <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
                  {voice.language}
                </span>
              </div>
              <div className="flex items-center space-x-2 mt-1 text-[11px] font-mono text-[#64748B]">
                <span>{voice.engine_id}</span>
                <span>•</span>
                <span>{voice.sample_rate_hz} Hz</span>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Synthesis Playground */}
      <div className="flex-1 p-8 overflow-y-auto space-y-6">
        <div className="max-w-3xl space-y-6">
          <div>
            <span className="text-[11px] font-mono uppercase text-amber-500 tracking-wider">
              Voice Synthesis Playground
            </span>
            <h2 className="text-xl font-bold text-white mt-0.5">
              {selectedVoice?.name || 'Selected Voice'}
            </h2>
            <p className="text-xs text-[#94A3B8] font-mono mt-1">
              Engine: {selectedVoice?.engine_id} | Sample Rate: {selectedVoice?.sample_rate_hz}Hz
            </p>
          </div>

          <div className="space-y-2">
            <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8]">
              Input Prompt
            </label>
            <textarea
              rows={4}
              value={text}
              onChange={(e) => setText(e.target.value)}
              className="w-full bg-[#121820] border border-[#242E3D] rounded-lg p-3 text-white text-sm focus:border-amber-500 focus:outline-none resize-none leading-relaxed shadow-inner"
              placeholder="Enter text to synthesize..."
            />
          </div>

          <div className="grid grid-cols-2 gap-6 bg-[#121820] p-4 rounded-lg border border-[#242E3D]">
            <div>
              <div className="flex justify-between text-xs font-mono mb-1">
                <span className="text-[#94A3B8]">Speaking Rate</span>
                <span className="text-white">{speed.toFixed(2)}x</span>
              </div>
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={speed}
                onChange={(e) => setSpeed(parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>

            <div>
              <div className="flex justify-between text-xs font-mono mb-1">
                <span className="text-[#94A3B8]">Pitch Shift</span>
                <span className="text-white">{pitch > 0 ? `+${pitch}` : pitch} st</span>
              </div>
              <input
                type="range"
                min="-12"
                max="12"
                step="0.5"
                value={pitch}
                onChange={(e) => setPitch(parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>
          </div>

          <div className="space-y-4">
            <div className="flex items-center space-x-4">
              <button
                onClick={handleGenerate}
                disabled={isGenerating}
                className="flex items-center space-x-2 px-5 py-2.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs transition-colors shadow-lg shadow-amber-500/20 disabled:opacity-50"
              >
                {isGenerating ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    <span>Synthesizing...</span>
                  </>
                ) : (
                  <>
                    <Play className="w-4 h-4 fill-black" />
                    <span>Generate Audio</span>
                  </>
                )}
              </button>

              {audioUrl && (
                <audio
                  ref={audioRef}
                  controls
                  src={audioUrl}
                  onPlay={() => setIsPlaying(true)}
                  onPause={() => setIsPlaying(false)}
                  onEnded={() => setIsPlaying(false)}
                  className="h-10 w-96 rounded-lg bg-[#121820]"
                />
              )}
            </div>

            {audioUrl && (
              <AudioVisualizer
                audioElement={audioRef.current}
                isPlaying={isPlaying}
              />
            )}
          </div>
        </div>
      </div>

      {/* Voice Cloning Studio Modal */}
      {showCloneModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm p-4">
          <div className="bg-[#121820] border border-[#242E3D] rounded-xl w-full max-w-lg overflow-hidden shadow-2xl animate-in fade-in zoom-in duration-150">
            <div className="p-4 border-b border-[#242E3D] flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <div className="p-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400">
                  <Sparkles className="w-4 h-4" />
                </div>
                <div>
                  <h3 className="text-sm font-bold text-white">Voice Cloning Studio</h3>
                  <p className="text-[11px] text-[#94A3B8] font-mono">Zero-shot speaker acoustic embedding extraction</p>
                </div>
              </div>
              <button
                onClick={() => setShowCloneModal(false)}
                className="p-1 rounded-lg text-[#64748B] hover:text-white hover:bg-[#1A222D] transition-colors"
              >
                <X className="w-4 h-4" />
              </button>
            </div>

            <form onSubmit={handleCloneSubmit} className="p-5 space-y-4">
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1">
                  Voice Name *
                </label>
                <input
                  type="text"
                  required
                  value={cloneName}
                  onChange={(e) => setCloneName(e.target.value)}
                  placeholder="e.g. Rachel Nova (Podcaster)"
                  className="w-full bg-[#0B0E14] border border-[#242E3D] rounded-lg px-3 py-2 text-white text-xs focus:border-amber-500 focus:outline-none"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1">
                  Reference Audio Sample *
                </label>
                <label className="flex flex-col items-center justify-center border border-dashed border-[#242E3D] hover:border-amber-500/50 bg-[#0B0E14] rounded-lg p-4 cursor-pointer transition-colors group">
                  <Upload className="w-6 h-6 text-[#64748B] group-hover:text-amber-400 mb-2 transition-colors" />
                  <span className="text-xs text-[#94A3B8] group-hover:text-white">
                    {cloneFileName ? cloneFileName : 'Select WAV, MP3, or FLAC reference audio'}
                  </span>
                  <span className="text-[10px] text-[#64748B] font-mono mt-1">3 - 15 seconds clean speech recommended</span>
                  <input
                    type="file"
                    accept="audio/*"
                    onChange={handleFileChange}
                    className="hidden"
                  />
                </label>
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1">
                  Reference Transcript <span className="text-[#64748B] lowercase">(optional, improves alignment)</span>
                </label>
                <textarea
                  rows={2}
                  value={cloneTranscript}
                  onChange={(e) => setCloneTranscript(e.target.value)}
                  placeholder="What was spoken in the reference audio sample..."
                  className="w-full bg-[#0B0E14] border border-[#242E3D] rounded-lg p-2 text-white text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed"
                />
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1">
                  Gender Target
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {(['neutral', 'female', 'male'] as const).map((g) => (
                    <button
                      key={g}
                      type="button"
                      onClick={() => setCloneGender(g)}
                      className={`py-1.5 px-3 rounded-lg text-xs font-mono capitalize border transition-all ${
                        cloneGender === g
                          ? 'bg-amber-500/10 border-amber-500 text-amber-400'
                          : 'bg-[#0B0E14] border-[#242E3D] text-[#94A3B8] hover:border-[#3B485C]'
                      }`}
                    >
                      {g}
                    </button>
                  ))}
                </div>
              </div>

              <div className="pt-2 flex items-center justify-end space-x-3 border-t border-[#242E3D]">
                <button
                  type="button"
                  onClick={() => setShowCloneModal(false)}
                  className="px-4 py-2 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-semibold transition-colors"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isCloning || !cloneName || !cloneAudioBase64}
                  className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-amber-500 hover:bg-amber-400 text-black text-xs font-semibold transition-colors disabled:opacity-50"
                >
                  {isCloning ? (
                    <>
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span>Extracting Profile...</span>
                    </>
                  ) : (
                    <>
                      <Mic className="w-3.5 h-3.5" />
                      <span>Clone Profile</span>
                    </>
                  )}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
