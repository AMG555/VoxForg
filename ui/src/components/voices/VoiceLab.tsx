import React, { useState, useRef, useEffect } from 'react';
import {
  Play,
  Loader2,
  Search,
  Sparkles,
  Upload,
  X,
  Mic,
  Square,
  RotateCcw,
  CheckCircle2,
  AlertTriangle,
  Key,
  Layers,
  Check,
} from 'lucide-react';
import { Voice } from '../../types';
import { api } from '../../services/api';
import { AudioVisualizer } from '../common/AudioVisualizer';
import { AudioProcessor, AudioQualityAssessment } from '../../services/audioProcessor';

interface VoiceLabProps {
  voices: Voice[];
  onVoiceCreated?: (voice: Voice) => void;
}

export const VoiceLab: React.FC<VoiceLabProps> = ({ voices, onVoiceCreated }) => {
  const [localVoices, setLocalVoices] = useState<Voice[]>(voices);
  useEffect(() => {
    setLocalVoices(voices);
  }, [voices]);
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

  // Router Multi-Platform config state
  const [routerProvider, setRouterProvider] = useState<'openrouter' | 'openai' | 'groq' | 'together' | 'custom'>(() => {
    return (typeof window !== 'undefined' ? (localStorage.getItem('voxforg_router_provider') as any) : null) || 'openrouter';
  });
  const [apiKeyInput, setApiKeyInput] = useState<string>(() => {
    return typeof window !== 'undefined' ? localStorage.getItem('voxforg_api_key') || '' : '';
  });
  const [routerModelInput, setRouterModelInput] = useState<string>(() => {
    return typeof window !== 'undefined' ? localStorage.getItem('voxforg_router_model') || 'openai/tts-1' : 'openai/tts-1';
  });
  const [routerCustomUrl, setRouterCustomUrl] = useState<string>(() => {
    return typeof window !== 'undefined' ? localStorage.getItem('voxforg_router_url') || '' : '';
  });
  const [keySaved, setKeySaved] = useState<boolean>(false);

  // Voice cloning studio state
  const [showCloneModal, setShowCloneModal] = useState<boolean>(false);
  const [cloneMode, setCloneMode] = useState<'record' | 'upload'>('record');
  const [cloneName, setCloneName] = useState<string>('');
  const [cloneTranscript, setCloneTranscript] = useState<string>('');
  const [cloneGender, setCloneGender] = useState<'male' | 'female' | 'neutral'>('neutral');
  const [cloneAudioBase64, setCloneAudioBase64] = useState<string>('');
  const [cloneFileName, setCloneFileName] = useState<string>('');
  const [isCloning, setIsCloning] = useState<boolean>(false);
  const [audioQuality, setAudioQuality] = useState<AudioQualityAssessment | null>(null);
  const [isAutoTranscribing, setIsAutoTranscribing] = useState<boolean>(false);

  // Microphone recording state
  const [isRecording, setIsRecording] = useState<boolean>(false);
  const [recordingSeconds, setRecordingSeconds] = useState<number>(0);
  const [recordedAudioUrl, setRecordedAudioUrl] = useState<string | null>(null);
  const [isPreviewPlaying, setIsPreviewPlaying] = useState<boolean>(false);
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const speechRecognitionRef = useRef<any>(null);
  const audioChunksRef = useRef<Blob[]>([]);
  const timerRef = useRef<any>(null);
  const previewAudioRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    return () => {
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
      }
      if (recordedAudioUrl) {
        URL.revokeObjectURL(recordedAudioUrl);
      }
      if (timerRef.current) {
        clearInterval(timerRef.current);
      }
      if (speechRecognitionRef.current) {
        try { speechRecognitionRef.current.stop(); } catch {}
      }
    };
  }, [audioUrl, recordedAudioUrl]);

  // Clean neural voices list: eliminate mock sine voices and sort working neural voices to top
  const filteredVoices = localVoices
    .filter((v) => v.engine_id !== 'mock-tts')
    .filter(
      (v) =>
        v.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        v.language.toLowerCase().includes(searchQuery.toLowerCase()) ||
        v.engine_id.toLowerCase().includes(searchQuery.toLowerCase())
    )
    .sort((a, b) => {
      const getPriority = (v: Voice) => {
        if (v.tags?.includes('cloned')) return 0;
        if (v.engine_id === 'edge-tts') return 1;
        if (v.engine_id === 'piper-tts') return 2;
        return 3;
      };
      return getPriority(a) - getPriority(b);
    });

  const selectedVoice =
    filteredVoices.find((v) => v.id === selectedVoiceId) ||
    filteredVoices.find((v) => v.engine_id === 'edge-tts') ||
    filteredVoices[0] ||
    localVoices[0] ||
    voices[0];

  const isRouterConfigured = Boolean(apiKeyInput.trim() && routerModelInput.trim());
  const canGenerate = selectedVoice?.engine_id !== 'openai-router' || isRouterConfigured;

  const handleGenerate = async () => {
    if (!text.trim() || !canGenerate) return;
    try {
      setIsGenerating(true);
      if (audioUrl) {
        URL.revokeObjectURL(audioUrl);
        setAudioUrl(null);
      }
      const blob = await api.synthesizeDirect({
        input: text,
        voice: selectedVoice.id,
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

  const handleProviderChange = (provider: 'openrouter' | 'openai' | 'groq' | 'together' | 'custom') => {
    setRouterProvider(provider);
    let defaultModel = routerModelInput;
    if (provider === 'openrouter') defaultModel = 'openai/tts-1';
    else if (provider === 'openai') defaultModel = 'tts-1';
    else if (provider === 'groq') defaultModel = 'whisper-large-v3';
    else if (provider === 'together') defaultModel = 'cartesia/sonic';

    setRouterModelInput(defaultModel);
    if (typeof window !== 'undefined') {
      localStorage.setItem('voxforg_router_provider', provider);
      localStorage.setItem('voxforg_router_model', defaultModel);
    }
  };

  const handleSaveRouterConfig = () => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('voxforg_router_provider', routerProvider);
      localStorage.setItem('voxforg_api_key', apiKeyInput.trim());
      localStorage.setItem('voxforg_router_model', routerModelInput.trim());
      if (routerCustomUrl.trim()) {
        localStorage.setItem('voxforg_router_url', routerCustomUrl.trim());
      }
      setKeySaved(true);
      setTimeout(() => setKeySaved(false), 2500);
    }
  };

  // Microphone audio capture
  const startRecording = async () => {
    try {
      audioChunksRef.current = [];
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
      });

      // Cross-browser MIME type check
      const supportedMime = [
        'audio/webm;codecs=opus',
        'audio/webm',
        'audio/ogg;codecs=opus',
        'audio/mp4',
      ].find((t) => typeof MediaRecorder !== 'undefined' && MediaRecorder.isTypeSupported(t)) || '';

      const recorderOptions = supportedMime ? { mimeType: supportedMime } : undefined;
      const mediaRecorder = new MediaRecorder(stream, recorderOptions);
      mediaRecorderRef.current = mediaRecorder;

      // Start automatic speech recognition to transcribe reference speech
      if (typeof window !== 'undefined') {
        const SpeechRec = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
        if (SpeechRec) {
          try {
            const recognizer = new SpeechRec();
            recognizer.continuous = true;
            recognizer.interimResults = true;
            recognizer.lang = 'en-US';
            recognizer.onresult = (event: any) => {
              let transcript = '';
              for (let i = 0; i < event.results.length; i++) {
                transcript += event.results[i][0].transcript + ' ';
              }
              if (transcript.trim()) {
                setCloneTranscript(transcript.trim());
              }
            };
            recognizer.onerror = () => setIsAutoTranscribing(false);
            recognizer.onend = () => setIsAutoTranscribing(false);
            recognizer.start();
            speechRecognitionRef.current = recognizer;
            setIsAutoTranscribing(true);
          } catch {}
        }
      }

      mediaRecorder.ondataavailable = (event) => {
        if (event.data && event.data.size > 0) {
          audioChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = async () => {
        const chosenMime = mediaRecorder.mimeType || supportedMime || 'audio/webm';
        const rawBlob = new Blob(audioChunksRef.current, { type: chosenMime });

        try {
          // Preprocess: 80Hz rumble cut, silence gating, -1.0 dBFS normalization, 24kHz 16-bit WAV
          const result = await AudioProcessor.preprocessForCloning(rawBlob);
          if (recordedAudioUrl) {
            URL.revokeObjectURL(recordedAudioUrl);
          }
          const url = URL.createObjectURL(result.wavBlob);
          setRecordedAudioUrl(url);
          setCloneAudioBase64(result.wavBase64);
          setAudioQuality(result.metrics);
          setCloneFileName(`Enhanced Studio WAV (${result.durationFormatted})`);
        } catch {
          // Fallback to direct raw blob if Web Audio decode is unsupported
          const url = URL.createObjectURL(rawBlob);
          setRecordedAudioUrl(url);
          const reader = new FileReader();
          reader.onloadend = () => {
            const base64data = reader.result as string;
            const base64 = base64data.includes(',') ? base64data.split(',')[1] : base64data;
            setCloneAudioBase64(base64);
            setCloneFileName(`Microphone Sample (${recordingSeconds}s)`);
          };
          reader.readAsDataURL(rawBlob);
        }

        // Stop all audio tracks to release microphone hardware
        stream.getTracks().forEach((track) => track.stop());
      };

      mediaRecorder.start(250);
      setIsRecording(true);
      setRecordingSeconds(0);

      timerRef.current = setInterval(() => {
        setRecordingSeconds((prev) => {
          if (prev >= 20) {
            stopRecording();
            return 20;
          }
          return prev + 1;
        });
      }, 1000);
    } catch (err: any) {
      alert(`Microphone access error: ${err.message}`);
    }
  };

  const stopRecording = () => {
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
      mediaRecorderRef.current.stop();
    }
    if (timerRef.current) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
    if (speechRecognitionRef.current) {
      try {
        speechRecognitionRef.current.stop();
      } catch {}
      speechRecognitionRef.current = null;
    }
    setIsAutoTranscribing(false);
    setIsRecording(false);
  };

  const resetRecording = () => {
    if (recordedAudioUrl) {
      URL.revokeObjectURL(recordedAudioUrl);
    }
    setRecordedAudioUrl(null);
    setCloneAudioBase64('');
    setCloneFileName('');
    setAudioQuality(null);
    setRecordingSeconds(0);
    if (speechRecognitionRef.current) {
      try {
        speechRecognitionRef.current.stop();
      } catch {}
      speechRecognitionRef.current = null;
    }
    setIsAutoTranscribing(false);
  };

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setCloneFileName(file.name);
      try {
        const result = await AudioProcessor.preprocessForCloning(file);
        if (recordedAudioUrl) URL.revokeObjectURL(recordedAudioUrl);
        setRecordedAudioUrl(URL.createObjectURL(result.wavBlob));
        setCloneAudioBase64(result.wavBase64);
        setAudioQuality(result.metrics);
        setCloneFileName(`${file.name} (Enhanced 24kHz Studio WAV)`);
      } catch {
        const reader = new FileReader();
        reader.onload = () => {
          const result = reader.result as string;
          const base64 = result.includes(',') ? result.split(',')[1] : result;
          setCloneAudioBase64(base64);
        };
        reader.readAsDataURL(file);
      }
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
      setLocalVoices((prev) => [newVoice, ...prev.filter((v) => v.id !== newVoice.id)]);
      onVoiceCreated?.(newVoice);
      setSelectedVoiceId(newVoice.id);
      setShowCloneModal(false);
      setCloneName('');
      setCloneTranscript('');
      setCloneAudioBase64('');
      setCloneFileName('');
      resetRecording();
    } catch (err: any) {
      alert(`Cloning failed: ${err.message}`);
    } finally {
      setIsCloning(false);
    }
  };

  const handleLoadDemoSample = async (type: 'broadcaster' | 'dispatcher') => {
    try {
      const demo = await AudioProcessor.createDemoReferenceSample(type);
      if (recordedAudioUrl) URL.revokeObjectURL(recordedAudioUrl);
      setRecordedAudioUrl(URL.createObjectURL(demo.wavBlob));
      setCloneAudioBase64(demo.wavBase64);
      setAudioQuality(demo.metrics);
      setCloneName(demo.name);
      setCloneTranscript(demo.transcript);
      setCloneFileName(`${demo.name} (Enhanced 24kHz Reference WAV)`);
      setCloneMode('upload');
    } catch (err) {
      console.error(err);
    }
  };

  const getEngineBadge = (engineId: string) => {
    switch (engineId) {
      case 'edge-tts':
        return (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center space-x-1">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse" />
            <span>CLOUD RELAY</span>
          </span>
        );
      case 'piper-tts':
        return (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-amber-500/10 text-amber-400 border border-amber-500/20">
            LOCAL NEURAL
          </span>
        );
      case 'openai-router':
        return (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-sky-500/10 text-sky-400 border border-sky-500/20">
            OPENROUTER API
          </span>
        );
      case 'qwen3-tts':
        return (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-purple-500/10 text-purple-400 border border-purple-500/20">
            ZERO-SHOT
          </span>
        );
      case 'mock-tts':
      default:
        return (
          <span className="text-[9px] font-mono px-1.5 py-0.5 rounded bg-slate-500/10 text-slate-400 border border-slate-500/20">
            MOCK HARNESS
          </span>
        );
    }
  };

  return (
    <div className="flex-1 flex overflow-hidden bg-[#0B0E14] relative">
      {/* Voices List Sidebar */}
      <div className="w-80 border-r border-[#242E3D] bg-[#121820] flex flex-col h-full select-none">
        <div className="p-3 border-b border-[#242E3D] space-y-2">
          <button
            onClick={() => setShowCloneModal(true)}
            className="w-full flex items-center justify-center space-x-2 py-2 px-3 rounded bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 text-xs font-mono font-semibold transition-colors shadow-sm"
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
              <div className="flex items-center justify-between mt-1.5 text-[11px] font-mono text-[#64748B]">
                <span>{voice.sample_rate_hz} Hz</span>
                {getEngineBadge(voice.engine_id)}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Synthesis Playground */}
      <div className="flex-1 p-8 overflow-y-auto space-y-6">
        <div className="max-w-3xl space-y-6">
          <div className="flex items-start justify-between gap-4">
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
            {selectedVoice && getEngineBadge(selectedVoice.engine_id)}
          </div>

          {/* Contextual Engine Diagnostics & Actions */}
          {selectedVoice?.engine_id === 'piper-tts' && (
            <div className="p-3.5 rounded-lg bg-amber-500/10 border border-amber-500/25 flex items-start space-x-3 text-xs">
              <Layers className="w-4 h-4 text-amber-400 flex-shrink-0 mt-0.5" />
              <div className="flex-1">
                <div className="font-semibold text-amber-300">Piper Neural Voice Engine (Local ONNX)</div>
                <p className="text-[#94A3B8] text-[11px] mt-0.5">
                  Generates natural speech 100% offline. Ensure <strong>piper-en-lessac-medium</strong> weights are downloaded from the <strong>Model Catalog</strong> tab to enable local neural synthesis.
                </p>
              </div>
            </div>
          )}

          {selectedVoice?.engine_id === 'openai-router' && (
            <div className="p-4 rounded-xl bg-[#121820] border border-sky-500/30 space-y-3.5 text-xs shadow-xl">
              <div className="flex items-center justify-between border-b border-[#242E3D] pb-2">
                <div className="flex items-center space-x-2 text-sky-400 font-semibold">
                  <Key className="w-4 h-4" />
                  <span>External Router Platform & Credentials</span>
                </div>
                <span className="text-[10px] font-mono text-[#64748B]">Stored in browser</span>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-[10px] font-mono uppercase text-[#94A3B8] mb-1">
                    Router Provider
                  </label>
                  <select
                    value={routerProvider}
                    onChange={(e) => handleProviderChange(e.target.value as any)}
                    className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-2.5 py-1.5 text-white font-mono text-xs focus:border-sky-500 focus:outline-none cursor-pointer"
                  >
                    <option value="openrouter">OpenRouter (openrouter.ai)</option>
                    <option value="openai">OpenAI Direct (api.openai.com)</option>
                    <option value="groq">Groq Cloud (api.groq.com)</option>
                    <option value="together">Together AI (together.xyz)</option>
                    <option value="custom">Custom Server / Local URL</option>
                  </select>
                </div>

                <div>
                  <label className="block text-[10px] font-mono uppercase text-[#94A3B8] mb-1">
                    Exact Model ID *
                  </label>
                  <input
                    type="text"
                    value={routerModelInput}
                    onChange={(e) => setRouterModelInput(e.target.value)}
                    placeholder="e.g. openai/tts-1, tts-1"
                    className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-2.5 py-1.5 text-white font-mono text-xs focus:border-sky-500 focus:outline-none"
                  />
                </div>
              </div>

              {routerProvider === 'custom' && (
                <div>
                  <label className="block text-[10px] font-mono uppercase text-[#94A3B8] mb-1">
                    Custom Base URL *
                  </label>
                  <input
                    type="text"
                    value={routerCustomUrl}
                    onChange={(e) => setRouterCustomUrl(e.target.value)}
                    placeholder="e.g. http://localhost:8000/v1 or https://my-proxy/v1"
                    className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-2.5 py-1.5 text-white font-mono text-xs focus:border-sky-500 focus:outline-none"
                  />
                </div>
              )}

              <div>
                <label className="block text-[10px] font-mono uppercase text-[#94A3B8] mb-1">
                  API Key / Bearer Token *
                </label>
                <div className="flex items-center space-x-2">
                  <input
                    type="password"
                    placeholder={
                      routerProvider === 'openrouter'
                        ? 'sk-or-v1-...'
                        : routerProvider === 'openai'
                        ? 'sk-proj-...'
                        : routerProvider === 'groq'
                        ? 'gsk_...'
                        : 'Enter provider API key...'
                    }
                    value={apiKeyInput}
                    onChange={(e) => setApiKeyInput(e.target.value)}
                    className="flex-1 bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-1.5 text-xs text-white font-mono focus:border-sky-500 focus:outline-none"
                  />
                  <button
                    onClick={handleSaveRouterConfig}
                    className="px-3.5 py-1.5 rounded bg-sky-500 hover:bg-sky-400 text-black font-semibold text-xs transition-colors flex items-center space-x-1"
                  >
                    {keySaved ? (
                      <>
                        <Check className="w-3.5 h-3.5" />
                        <span>Saved!</span>
                      </>
                    ) : (
                      <span>Save Config</span>
                    )}
                  </button>
                </div>
              </div>

              {!isRouterConfigured && (
                <div className="p-2.5 rounded bg-amber-500/10 border border-amber-500/20 text-amber-300 text-[11px] font-mono flex items-center space-x-2">
                  <AlertTriangle className="w-4 h-4 text-amber-400 shrink-0" />
                  <span>Enter API key and exact Model ID to enable external neural synthesis.</span>
                </div>
              )}
            </div>
          )}

          {selectedVoice?.engine_id === 'mock-tts' && (
            <div className="p-3 rounded-lg bg-slate-800/60 border border-slate-700 flex items-center space-x-2 text-xs text-[#94A3B8]">
              <AlertTriangle className="w-4 h-4 text-amber-400 flex-shrink-0" />
              <span>
                <strong>Synthetic Test Harness:</strong> Generates mathematical harmonic tones for buffer and latency testing. Use <strong>Edge-TTS</strong> for human speech.
              </span>
            </div>
          )}

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
            {canGenerate ? (
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
            ) : (
              <div className="p-4 rounded-xl bg-amber-500/10 border border-amber-500/30 flex items-start space-x-3 text-xs text-amber-300 font-mono">
                <AlertTriangle className="w-5 h-5 text-amber-400 shrink-0 mt-0.5" />
                <div>
                  <div className="font-semibold text-white">Generate Audio Hidden: Router Setup Required</div>
                  <div className="text-[11px] text-[#94A3B8] mt-1 leading-relaxed">
                    This voice routes through an external API. Add your working API key and exact Model ID in the configuration panel above to unlock audio generation.
                  </div>
                </div>
              </div>
            )}

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
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-sm p-4">
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
                onClick={() => {
                  if (isRecording) stopRecording();
                  setShowCloneModal(false);
                }}
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

              {/* Input Mode Selector: Instant Microphone vs File Upload */}
              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1.5">
                  Reference Audio Source *
                </label>
                <div className="grid grid-cols-2 gap-2 mb-3">
                  <button
                    type="button"
                    onClick={() => {
                      setCloneMode('record');
                      resetRecording();
                    }}
                    className={`py-2 px-3 rounded-lg text-xs font-medium border flex items-center justify-center space-x-2 transition-all ${
                      cloneMode === 'record'
                        ? 'bg-amber-500/10 border-amber-500 text-amber-400 font-semibold'
                        : 'bg-[#0B0E14] border-[#242E3D] text-[#94A3B8] hover:border-[#3B485C]'
                    }`}
                  >
                    <Mic className="w-3.5 h-3.5" />
                    <span>Instant Microphone</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => {
                      setCloneMode('upload');
                      resetRecording();
                    }}
                    className={`py-2 px-3 rounded-lg text-xs font-medium border flex items-center justify-center space-x-2 transition-all ${
                      cloneMode === 'upload'
                        ? 'bg-amber-500/10 border-amber-500 text-amber-400 font-semibold'
                        : 'bg-[#0B0E14] border-[#242E3D] text-[#94A3B8] hover:border-[#3B485C]'
                    }`}
                  >
                    <Upload className="w-3.5 h-3.5" />
                    <span>Upload Audio File</span>
                  </button>
                </div>

                <div className="flex items-center space-x-2 mb-3 px-1">
                  <span className="text-[10px] font-mono text-[#64748B] uppercase">Quick Demo:</span>
                  <button
                    type="button"
                    onClick={() => handleLoadDemoSample('broadcaster')}
                    className="px-2.5 py-1 text-[11px] font-mono rounded-md bg-[#0B0E14] hover:bg-[#1A222D] text-[#94A3B8] hover:text-amber-400 border border-[#242E3D] hover:border-amber-500/40 transition-colors flex items-center space-x-1"
                  >
                    <Sparkles className="w-3 h-3 text-amber-400" />
                    <span>Studio Host (5.5s)</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => handleLoadDemoSample('dispatcher')}
                    className="px-2.5 py-1 text-[11px] font-mono rounded-md bg-[#0B0E14] hover:bg-[#1A222D] text-[#94A3B8] hover:text-sky-400 border border-[#242E3D] hover:border-sky-500/40 transition-colors flex items-center space-x-1"
                  >
                    <Sparkles className="w-3 h-3 text-sky-400" />
                    <span>Orbital Dispatch (4.8s)</span>
                  </button>
                </div>

                {cloneMode === 'record' ? (
                  <div className="border border-[#242E3D] bg-[#0B0E14] rounded-lg p-4 flex flex-col items-center justify-center space-y-3">
                    {recordedAudioUrl ? (
                      <div className="w-full space-y-3">
                        <div className="flex items-center justify-between text-xs text-[#94A3B8]">
                          <span className="flex items-center space-x-1.5 text-emerald-400 font-mono">
                            <CheckCircle2 className="w-4 h-4" />
                            <span>Audio Sample Captured ({recordingSeconds}s)</span>
                          </span>
                          <button
                            type="button"
                            onClick={resetRecording}
                            className="flex items-center space-x-1 text-xs text-[#64748B] hover:text-amber-400 transition-colors"
                          >
                            <RotateCcw className="w-3.5 h-3.5" />
                            <span>Record Again</span>
                          </button>
                        </div>

                        <div className="flex items-center space-x-2.5 bg-[#121820] p-2.5 rounded-lg border border-[#242E3D]">
                          <button
                            type="button"
                            onClick={() => {
                              if (!previewAudioRef.current) return;
                              if (isPreviewPlaying) {
                                previewAudioRef.current.pause();
                                setIsPreviewPlaying(false);
                              } else {
                                previewAudioRef.current.currentTime = 0;
                                previewAudioRef.current.play().catch(console.error);
                                setIsPreviewPlaying(true);
                              }
                            }}
                            className="px-3.5 py-1.5 rounded-md bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs flex items-center space-x-1.5 transition-colors shadow-sm shrink-0"
                          >
                            {isPreviewPlaying ? (
                              <>
                                <Square className="w-3.5 h-3.5 fill-black" />
                                <span>Stop</span>
                              </>
                            ) : (
                              <>
                                <Play className="w-3.5 h-3.5 fill-black" />
                                <span>Play Preview</span>
                              </>
                            )}
                          </button>

                          <audio
                            ref={previewAudioRef}
                            controls
                            src={recordedAudioUrl}
                            onPlay={() => setIsPreviewPlaying(true)}
                            onPause={() => setIsPreviewPlaying(false)}
                            onEnded={() => setIsPreviewPlaying(false)}
                            className="flex-1 h-8 rounded bg-[#0B0E14]"
                          />
                        </div>
                      </div>
                    ) : isRecording ? (
                      <div className="flex flex-col items-center space-y-3 py-2">
                        <div className="flex items-center space-x-2 text-rose-400 animate-pulse font-mono text-xs font-bold">
                          <span className="w-2.5 h-2.5 rounded-full bg-rose-500" />
                          <span>RECORDING: 00:{recordingSeconds.toString().padStart(2, '0')} / 00:20</span>
                        </div>
                        <div className="flex items-center space-x-1">
                          <span className="w-1 h-4 bg-amber-500 animate-pulse" />
                          <span className="w-1 h-7 bg-amber-400 animate-pulse delay-75" />
                          <span className="w-1 h-5 bg-amber-500 animate-pulse delay-100" />
                          <span className="w-1 h-8 bg-amber-400 animate-pulse delay-150" />
                          <span className="w-1 h-4 bg-amber-500 animate-pulse delay-200" />
                        </div>
                        <button
                          type="button"
                          onClick={stopRecording}
                          className="flex items-center space-x-2 px-4 py-1.5 rounded-lg bg-rose-500 hover:bg-rose-600 text-white text-xs font-semibold transition-colors"
                        >
                          <Square className="w-3.5 h-3.5 fill-white" />
                          <span>Stop Recording</span>
                        </button>
                      </div>
                    ) : (
                      <div className="flex flex-col items-center space-y-2 py-2">
                        <button
                          type="button"
                          onClick={startRecording}
                          className="w-12 h-12 rounded-full bg-amber-500/15 hover:bg-amber-500/25 border-2 border-amber-500 text-amber-400 flex items-center justify-center transition-transform hover:scale-105"
                        >
                          <Mic className="w-6 h-6" />
                        </button>
                        <div className="text-center">
                          <p className="text-xs font-semibold text-white">Click to Start Recording</p>
                          <p className="text-[10px] text-[#64748B] font-mono mt-0.5">
                            Speak clearly for 5 to 15 seconds to capture vocal timbre
                          </p>
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
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
                )}

                {/* Real-Time Audio Quality Diagnostics */}
                {audioQuality && (
                  <div className="bg-[#0B0E14] border border-amber-500/30 rounded-xl p-3 space-y-2.5 font-mono text-xs shadow-inner">
                    <div className="flex items-center justify-between border-b border-[#242E3D] pb-1.5">
                      <div className="flex items-center space-x-1.5 text-amber-400 font-semibold text-[11px] uppercase tracking-wider">
                        <Sparkles className="w-3.5 h-3.5" />
                        <span>Acoustic Quality Diagnostics</span>
                      </div>
                      <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20 flex items-center gap-1">
                        <CheckCircle2 className="w-3 h-3" />
                        <span>24kHz Studio WAV</span>
                      </span>
                    </div>

                    <div className="grid grid-cols-3 gap-2 text-[11px]">
                      <div className="bg-[#121820] p-2 rounded border border-[#242E3D]">
                        <div className="text-[#64748B] text-[10px] uppercase">Duration</div>
                        <div className={`font-semibold mt-0.5 ${audioQuality.durationQuality === 'optimal' ? 'text-emerald-400' : 'text-amber-400'}`}>
                          {audioQuality.durationSeconds}s
                        </div>
                      </div>
                      <div className="bg-[#121820] p-2 rounded border border-[#242E3D]">
                        <div className="text-[#64748B] text-[10px] uppercase">Clarity SNR</div>
                        <div className={`font-semibold mt-0.5 ${audioQuality.clarityRating === 'excellent' ? 'text-emerald-400' : 'text-amber-400'}`}>
                          ~{audioQuality.snrEstimateDb} dB ({audioQuality.clarityRating})
                        </div>
                      </div>
                      <div className="bg-[#121820] p-2 rounded border border-[#242E3D]">
                        <div className="text-[#64748B] text-[10px] uppercase">Peak Level</div>
                        <div className="font-semibold text-white mt-0.5">
                          {audioQuality.peakDbfs} dBFS
                        </div>
                      </div>
                    </div>

                    <div className="text-[10px] text-[#94A3B8] flex items-center space-x-1.5 bg-[#121820] px-2.5 py-1.5 rounded border border-[#242E3D]">
                      <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse shrink-0" />
                      <span className="truncate">{audioQuality.clarityMessage} • 80Hz rumble filter & -1dBFS normalization applied</span>
                    </div>
                  </div>
                )}
              </div>

              <div>
                <label className="block text-xs font-semibold uppercase tracking-wider text-[#94A3B8] mb-1">
                  Reference Transcript{' '}
                  {isAutoTranscribing ? (
                    <span className="text-amber-400 font-mono text-[10px] animate-pulse">
                      (Listening & Auto-Transcribing...)
                    </span>
                  ) : cloneTranscript ? (
                    <span className="text-emerald-400 font-mono text-[10px]">
                      (✓ Spoken Transcript Captured)
                    </span>
                  ) : (
                    <span className="text-[#64748B] lowercase font-normal">(optional, auto-transcribes on microphone)</span>
                  )}
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
                  onClick={() => {
                    if (isRecording) stopRecording();
                    setShowCloneModal(false);
                  }}
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
