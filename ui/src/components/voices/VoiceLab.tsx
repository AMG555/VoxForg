import React, { useState } from 'react';
import { Play, Loader2, Volume2, Mic, Search, Sliders } from 'lucide-react';
import { Voice } from '../../types';
import { api } from '../../services/api';

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
  const [searchQuery, setSearchQuery] = useState<string>('');

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

  return (
    <div className="flex-1 flex overflow-hidden bg-[#0B0E14]">
      {/* Voices List Sidebar */}
      <div className="w-80 border-r border-[#242E3D] bg-[#121820] flex flex-col h-full select-none">
        <div className="p-3 border-b border-[#242E3D]">
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
              <audio controls src={audioUrl} className="h-10 w-96 rounded-lg bg-[#121820]" />
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
