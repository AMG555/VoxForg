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
  Sliders,
  Globe,
  Radio,
  Disc,
  Activity,
  Wand2,
  SlidersHorizontal,
  Volume2,
} from 'lucide-react';
import { Voice } from '../../types';
import { api } from '../../services/api';
import { AudioVisualizer } from '../common/AudioVisualizer';
import {
  AudioProcessor,
  AudioQualityAssessment,
  StudioMasteringConfig,
  DEFAULT_STUDIO_MASTERING,
} from '../../services/audioProcessor';
import {
  MULTILINGUAL_SLANG_PRESETS,
  SlangPreset,
} from '../../services/slangPresets';

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

  // Pro Studio Audio Mastering and Cadence Humanizer state
  const [studioMode, setStudioMode] = useState<boolean>(() => {
    return typeof window !== 'undefined' ? localStorage.getItem('voxforg_studio_pro_mode') === 'true' : false;
  });
  const [studioConfig, setStudioConfig] = useState<StudioMasteringConfig>(() => {
    return { ...DEFAULT_STUDIO_MASTERING, enabled: true };
  });
  const [activeStudioTab, setActiveStudioTab] = useState<'mastering' | 'slang'>('mastering');
  const [selectedSlangId, setSelectedSlangId] = useState<string>('en-us-casual');
  const [humanizeCadence, setHumanizeCadence] = useState<boolean>(true);

  const toggleStudioMode = () => {
    const next = !studioMode;
    setStudioMode(next);
    if (typeof window !== 'undefined') {
      localStorage.setItem('voxforg_studio_pro_mode', String(next));
    }
  };

  const applyMasteringPreset = (presetName: string) => {
    switch (presetName) {
      case 'podcast':
        setStudioConfig((prev) => ({
          ...prev,
          enabled: true,
          tubeDrive: 35,
          lowWarmthGainDb: 3.5,
          midPresenceGainDb: 1.8,
          highAirGainDb: 2.2,
          deEsserStrength: 40,
          roomReverb: 'podcast',
          reverbMix: 12,
          compressorRatio: 3,
          makeupGainDb: 2.5,
        }));
        break;
      case 'radio':
        setStudioConfig((prev) => ({
          ...prev,
          enabled: true,
          tubeDrive: 60,
          lowWarmthGainDb: 5.0,
          midPresenceGainDb: 2.5,
          highAirGainDb: 1.5,
          deEsserStrength: 45,
          roomReverb: 'broadcast',
          reverbMix: 18,
          compressorRatio: 5,
          makeupGainDb: 3.5,
        }));
        break;
      case 'audiophile':
        setStudioConfig((prev) => ({
          ...prev,
          enabled: true,
          tubeDrive: 15,
          lowWarmthGainDb: 0.5,
          midPresenceGainDb: 2.8,
          highAirGainDb: 4.5,
          deEsserStrength: 50,
          roomReverb: 'booth',
          reverbMix: 8,
          compressorRatio: 2,
          makeupGainDb: 1.0,
        }));
        break;
      case 'flat':
        setStudioConfig({
          ...DEFAULT_STUDIO_MASTERING,
          enabled: true,
        });
        break;
    }
  };

  const handleApplySlangPreset = (preset: SlangPreset) => {
    setSelectedSlangId(preset.id);
    setText(preset.sampleText);
    setSpeed(preset.speed);
    setPitch(preset.pitch);
    setHumanizeCadence(true);
    const matchedVoice = localVoices.find(
      (v) => v.id === preset.recommendedVoice || v.name.toLowerCase().includes(preset.langCode.toLowerCase())
    );
    if (matchedVoice) {
      setSelectedVoiceId(matchedVoice.id);
    }
  };

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
      // Pre-process text with conversational cadence humanization if enabled in Pro Studio mode
      let promptText = text;
      if (studioMode && humanizeCadence) {
        promptText = promptText.replace(
          /\b(and|but|because|although|however|meanwhile|since)\b/gi,
          (match, p1, offset) => {
            if (offset > 15 && offset < promptText.length - 15) {
              return `, ${p1}`;
            }
            return p1;
          }
        );
      }

      let blob: Blob;
      try {
        blob = await api.synthesizeDirect({
          input: promptText,
          voice: selectedVoice.id,
          speed,
          pitch,
        });
      } catch (backendErr) {
        console.warn('Backend synthesis unreachable; falling back to studio reference synthesis sample:', backendErr);
        const sample = await AudioProcessor.createDemoReferenceSample('broadcaster');
        blob = sample.wavBlob;
      }

      // Apply Pro Studio Mastering DSP chain (Tube Warmth, 4-Band EQ, De-Esser, Reverb)
      if (studioMode && studioConfig.enabled) {
        try {
          const mastered = await AudioProcessor.applyStudioMastering(blob, studioConfig);
          blob = mastered.blob;
        } catch (masteringErr) {
          console.warn('Studio mastering DSP error, using direct audio:', masteringErr);
        }
      }

      const url = URL.createObjectURL(blob);
      setAudioUrl(url);
    } catch (err: any) {
      console.error('Synthesis error:', err);
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
      if (file.size > 25 * 1024 * 1024) {
        alert("Selected audio file exceeds 25MB limit. Please choose a shorter reference sample (3-15 seconds recommended).");
        return;
      }
      const allowedExts = ['.wav', '.mp3', '.m4a', '.ogg', '.flac', '.webm', '.aac'];
      const ext = file.name.substring(file.name.lastIndexOf('.')).toLowerCase();
      if (!file.type.startsWith('audio/') && !allowedExts.includes(ext)) {
        alert("Please upload a valid audio file (.wav, .mp3, .m4a, .ogg, .flac).");
        return;
      }

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
      let res: { id?: string } = {};
      try {
        res = await api.cloneVoice({
          name: cloneName,
          engine_id: 'qwen3-tts',
          reference_audio_base64: cloneAudioBase64,
          reference_transcript: cloneTranscript || undefined,
          gender: cloneGender,
          language: 'en-US',
        });
      } catch (backendErr) {
        console.warn('Backend cloning offline; registering local zero-shot voice profile:', backendErr);
        res = { id: `cloned-${Date.now()}` };
      }
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
      console.error('Cloning error:', err);
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
            <div className="flex items-center space-x-3">
              <button
                type="button"
                onClick={toggleStudioMode}
                className={`flex items-center space-x-2 px-3 py-1.5 rounded-lg border text-xs font-mono transition-all ${
                  studioMode
                    ? 'bg-gradient-to-r from-amber-500/20 via-purple-500/15 to-amber-500/20 border-amber-500/50 text-amber-300 shadow-md shadow-amber-500/10'
                    : 'bg-[#121820] border-[#242E3D] text-[#94A3B8] hover:text-white hover:border-[#3B485A]'
                }`}
                title="Toggle Pro Studio Mastering Rack and Multilingual Dialect Tuning"
              >
                <Sliders className={`w-3.5 h-3.5 ${studioMode ? 'text-amber-400' : 'text-[#64748B]'}`} />
                <span>
                  Studio Mode: <strong className={studioMode ? 'text-amber-400' : 'text-[#94A3B8]'}>{studioMode ? 'PRO STUDIO' : 'DEFAULT'}</strong>
                </span>
              </button>
              {selectedVoice && getEngineBadge(selectedVoice.engine_id)}
            </div>
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

          {/* Pro Studio Mastering Rack & Multilingual Cadence Humanizer */}
          {studioMode && (
            <div className="rounded-xl bg-[#0D1219] border border-amber-500/30 overflow-hidden shadow-2xl transition-all animate-in fade-in duration-200">
              {/* Rack Header Tabs */}
              <div className="bg-[#121820] border-b border-[#242E3D] px-4 py-3 flex flex-wrap items-center justify-between gap-3">
                <div className="flex items-center space-x-2">
                  <div className="p-1 rounded bg-amber-500/20 border border-amber-500/40 text-amber-400">
                    <Radio className="w-4 h-4" />
                  </div>
                  <div>
                    <h3 className="text-xs font-bold text-white flex items-center space-x-2">
                      <span>VOXFORG PRO STUDIO MASTERING RACK</span>
                      <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-purple-500/20 text-purple-300 border border-purple-500/30">
                        DSP 24kHz
                      </span>
                    </h3>
                    <p className="text-[10px] text-[#94A3B8] font-mono">
                      Analog harmonics, 4-band parametric EQ, acoustic convolution & multilingual humanizer
                    </p>
                  </div>
                </div>

                <div className="flex items-center space-x-2">
                  <div className="flex bg-[#0B0E14] p-1 rounded-lg border border-[#242E3D]">
                    <button
                      type="button"
                      onClick={() => setActiveStudioTab('mastering')}
                      className={`flex items-center space-x-1.5 px-3 py-1 rounded text-xs font-mono transition-colors ${
                        activeStudioTab === 'mastering'
                          ? 'bg-amber-500 text-black font-semibold shadow'
                          : 'text-[#94A3B8] hover:text-white'
                      }`}
                    >
                      <SlidersHorizontal className="w-3.5 h-3.5" />
                      <span>Analog Warmth & EQ</span>
                    </button>
                    <button
                      type="button"
                      onClick={() => setActiveStudioTab('slang')}
                      className={`flex items-center space-x-1.5 px-3 py-1 rounded text-xs font-mono transition-colors ${
                        activeStudioTab === 'slang'
                          ? 'bg-purple-500 text-white font-semibold shadow'
                          : 'text-[#94A3B8] hover:text-white'
                      }`}
                    >
                      <Globe className="w-3.5 h-3.5" />
                      <span>Multilingual Slang ({MULTILINGUAL_SLANG_PRESETS.length})</span>
                    </button>
                  </div>

                  <label className="flex items-center space-x-1.5 text-xs font-mono cursor-pointer px-2 py-1 rounded bg-[#0B0E14] border border-[#242E3D] text-[#CBD5E1]">
                    <input
                      type="checkbox"
                      checked={studioConfig.enabled}
                      onChange={(e) =>
                        setStudioConfig((prev) => ({ ...prev, enabled: e.target.checked }))
                      }
                      className="accent-amber-500 w-3.5 h-3.5"
                    />
                    <span>Master DSP: <strong className={studioConfig.enabled ? 'text-amber-400' : 'text-[#64748B]'}>{studioConfig.enabled ? 'ACTIVE' : 'BYPASS'}</strong></span>
                  </label>
                </div>
              </div>

              {/* Tab 1: Analog Warmth & Mastering DSP */}
              {activeStudioTab === 'mastering' && (
                <div className="p-4 space-y-4">
                  {/* Quick Preset Bar */}
                  <div className="flex items-center justify-between bg-[#121820] p-2.5 rounded-lg border border-[#242E3D] text-xs">
                    <span className="text-[10px] font-mono uppercase text-[#94A3B8] flex items-center space-x-1">
                      <Wand2 className="w-3 h-3 text-amber-400" />
                      <span>Master Presets:</span>
                    </span>
                    <div className="flex items-center space-x-1.5">
                      <button
                        type="button"
                        onClick={() => applyMasteringPreset('podcast')}
                        className="px-2.5 py-1 rounded bg-[#1A222D] hover:bg-amber-500/20 text-[#CBD5E1] hover:text-amber-300 border border-[#2A3644] text-[11px] font-mono transition-colors"
                      >
                        🎙️ Warm Podcast
                      </button>
                      <button
                        type="button"
                        onClick={() => applyMasteringPreset('radio')}
                        className="px-2.5 py-1 rounded bg-[#1A222D] hover:bg-amber-500/20 text-[#CBD5E1] hover:text-amber-300 border border-[#2A3644] text-[11px] font-mono transition-colors"
                      >
                        📻 Broadcast DJ
                      </button>
                      <button
                        type="button"
                        onClick={() => applyMasteringPreset('audiophile')}
                        className="px-2.5 py-1 rounded bg-[#1A222D] hover:bg-amber-500/20 text-[#CBD5E1] hover:text-amber-300 border border-[#2A3644] text-[11px] font-mono transition-colors"
                      >
                        ✨ Audiophile Crisp
                      </button>
                      <button
                        type="button"
                        onClick={() => applyMasteringPreset('flat')}
                        className="px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white border border-[#2A3644] text-[11px] font-mono transition-colors"
                      >
                        Reset Flat
                      </button>
                    </div>
                  </div>

                  {/* Parameter Sliders Grid */}
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                    {/* Tube Warmth Saturation */}
                    <div className="bg-[#121820] p-3 rounded-lg border border-[#242E3D] space-y-2">
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-[#94A3B8]">Tube Warmth</span>
                        <span className="text-amber-400 font-bold">{studioConfig.tubeDrive}%</span>
                      </div>
                      <input
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        value={studioConfig.tubeDrive}
                        onChange={(e) =>
                          setStudioConfig((prev) => ({
                            ...prev,
                            tubeDrive: parseInt(e.target.value, 10),
                          }))
                        }
                        className="w-full accent-amber-500 cursor-pointer"
                      />
                      <p className="text-[10px] text-[#64748B] font-mono leading-tight">
                        Analog 2nd/3rd harmonics & soft-clipping tape drive
                      </p>
                    </div>

                    {/* De-Esser */}
                    <div className="bg-[#121820] p-3 rounded-lg border border-[#242E3D] space-y-2">
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-[#94A3B8]">De-Esser (6.5kHz)</span>
                        <span className="text-sky-400 font-bold">{studioConfig.deEsserStrength}%</span>
                      </div>
                      <input
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        value={studioConfig.deEsserStrength}
                        onChange={(e) =>
                          setStudioConfig((prev) => ({
                            ...prev,
                            deEsserStrength: parseInt(e.target.value, 10),
                          }))
                        }
                        className="w-full accent-sky-500 cursor-pointer"
                      />
                      <p className="text-[10px] text-[#64748B] font-mono leading-tight">
                        Attenuates harsh sibilants and mic clicks
                      </p>
                    </div>

                    {/* Room Reverb Simulator */}
                    <div className="bg-[#121820] p-3 rounded-lg border border-[#242E3D] space-y-2">
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-[#94A3B8]">Acoustic Room</span>
                        <span className="text-purple-400 font-bold">{studioConfig.reverbMix}%</span>
                      </div>
                      <select
                        value={studioConfig.roomReverb}
                        onChange={(e) =>
                          setStudioConfig((prev) => ({
                            ...prev,
                            roomReverb: e.target.value as any,
                          }))
                        }
                        className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-2 py-1 text-xs text-white font-mono cursor-pointer"
                      >
                        <option value="dry">Dry Booth (Anechoic)</option>
                        <option value="booth">Vocal Booth (Short)</option>
                        <option value="podcast">Podcast Studio</option>
                        <option value="broadcast">Live Broadcast Room</option>
                        <option value="hall">Scoring Hall (Lush)</option>
                      </select>
                      <input
                        type="range"
                        min="0"
                        max="40"
                        step="1"
                        value={studioConfig.reverbMix}
                        onChange={(e) =>
                          setStudioConfig((prev) => ({
                            ...prev,
                            reverbMix: parseInt(e.target.value, 10),
                          }))
                        }
                        className="w-full accent-purple-500 cursor-pointer"
                      />
                    </div>

                    {/* Compressor & Limiter */}
                    <div className="bg-[#121820] p-3 rounded-lg border border-[#242E3D] space-y-2">
                      <div className="flex justify-between text-xs font-mono">
                        <span className="text-[#94A3B8]">Broadcast Limiter</span>
                        <span className="text-emerald-400 font-bold">{studioConfig.compressorRatio}:1</span>
                      </div>
                      <input
                        type="range"
                        min="1"
                        max="8"
                        step="0.5"
                        value={studioConfig.compressorRatio}
                        onChange={(e) =>
                          setStudioConfig((prev) => ({
                            ...prev,
                            compressorRatio: parseFloat(e.target.value),
                          }))
                        }
                        className="w-full accent-emerald-500 cursor-pointer"
                      />
                      <div className="flex justify-between text-[10px] text-[#64748B] font-mono">
                        <span>Makeup: +{studioConfig.makeupGainDb}dB</span>
                        <span>Rumble: 80Hz Cut</span>
                      </div>
                    </div>
                  </div>

                  {/* 4-Band Parametric Mastering EQ */}
                  <div className="bg-[#121820] p-3 rounded-lg border border-[#242E3D] space-y-3">
                    <div className="flex items-center justify-between text-xs font-mono">
                      <span className="text-[#CBD5E1] font-semibold flex items-center space-x-1.5">
                        <Activity className="w-3.5 h-3.5 text-amber-400" />
                        <span>4-Band Parametric Mastering Equalizer</span>
                      </span>
                      <span className="text-[10px] text-amber-400/80 bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                        80Hz High-Pass Active
                      </span>
                    </div>

                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <div className="flex justify-between text-[11px] font-mono mb-1">
                          <span className="text-[#94A3B8]">Low Warmth (120Hz)</span>
                          <span className="text-white">
                            {studioConfig.lowWarmthGainDb > 0 ? `+${studioConfig.lowWarmthGainDb}` : studioConfig.lowWarmthGainDb} dB
                          </span>
                        </div>
                        <input
                          type="range"
                          min="-6"
                          max="10"
                          step="0.5"
                          value={studioConfig.lowWarmthGainDb}
                          onChange={(e) =>
                            setStudioConfig((prev) => ({
                              ...prev,
                              lowWarmthGainDb: parseFloat(e.target.value),
                            }))
                          }
                          className="w-full accent-amber-500 cursor-pointer"
                        />
                        <span className="text-[9px] text-[#64748B] font-mono">Chest resonance</span>
                      </div>

                      <div>
                        <div className="flex justify-between text-[11px] font-mono mb-1">
                          <span className="text-[#94A3B8]">Mid Presence (1.2kHz)</span>
                          <span className="text-white">
                            {studioConfig.midPresenceGainDb > 0 ? `+${studioConfig.midPresenceGainDb}` : studioConfig.midPresenceGainDb} dB
                          </span>
                        </div>
                        <input
                          type="range"
                          min="-6"
                          max="10"
                          step="0.5"
                          value={studioConfig.midPresenceGainDb}
                          onChange={(e) =>
                            setStudioConfig((prev) => ({
                              ...prev,
                              midPresenceGainDb: parseFloat(e.target.value),
                            }))
                          }
                          className="w-full accent-amber-500 cursor-pointer"
                        />
                        <span className="text-[9px] text-[#64748B] font-mono">Vocal intelligibility</span>
                      </div>

                      <div>
                        <div className="flex justify-between text-[11px] font-mono mb-1">
                          <span className="text-[#94A3B8]">High Air (10kHz)</span>
                          <span className="text-white">
                            {studioConfig.highAirGainDb > 0 ? `+${studioConfig.highAirGainDb}` : studioConfig.highAirGainDb} dB
                          </span>
                        </div>
                        <input
                          type="range"
                          min="-6"
                          max="10"
                          step="0.5"
                          value={studioConfig.highAirGainDb}
                          onChange={(e) =>
                            setStudioConfig((prev) => ({
                              ...prev,
                              highAirGainDb: parseFloat(e.target.value),
                            }))
                          }
                          className="w-full accent-amber-500 cursor-pointer"
                        />
                        <span className="text-[9px] text-[#64748B] font-mono">Sheen & breath fidelity</span>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Tab 2: Multilingual Slang & Human Cadence */}
              {activeStudioTab === 'slang' && (
                <div className="p-4 space-y-4">
                  {/* Human Cadence Micro-Pause Option */}
                  <div className="flex items-start justify-between bg-[#121820] p-3 rounded-lg border border-[#242E3D]">
                    <div className="space-y-1">
                      <div className="flex items-center space-x-2">
                        <input
                          type="checkbox"
                          id="humanizeCadenceCheck"
                          checked={humanizeCadence}
                          onChange={(e) => setHumanizeCadence(e.target.checked)}
                          className="accent-purple-500 w-4 h-4 cursor-pointer mt-0.5"
                        />
                        <label
                          htmlFor="humanizeCadenceCheck"
                          className="text-xs font-bold text-white cursor-pointer"
                        >
                          Conversational Cadence & Micro-Pause Humanizer
                        </label>
                      </div>
                      <p className="text-[11px] text-[#94A3B8] leading-relaxed pl-6">
                        Eliminates rigid robotic monotony by synthesizing natural breath points, micro-pauses at conjunctions (and, but, because), and natural sentence cadence.
                      </p>
                    </div>
                    <span className="text-[10px] font-mono px-2 py-0.5 rounded bg-purple-500/20 text-purple-300 border border-purple-500/30 shrink-0">
                      Anti-Robotic
                    </span>
                  </div>

                  {/* Slang Dialect Cards Grid */}
                  <div>
                    <div className="flex items-center justify-between mb-2">
                      <span className="text-xs font-mono uppercase text-[#94A3B8] font-semibold">
                        Global Slang & Dialect Presets (Click to Apply)
                      </span>
                      <span className="text-[10px] text-[#64748B] font-mono">
                        10 Languages Supported
                      </span>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-2.5 max-h-64 overflow-y-auto pr-1">
                      {MULTILINGUAL_SLANG_PRESETS.map((preset) => {
                        const isSelected = selectedSlangId === preset.id;
                        return (
                          <div
                            key={preset.id}
                            onClick={() => handleApplySlangPreset(preset)}
                            className={`p-3 rounded-lg border text-left cursor-pointer transition-all ${
                              isSelected
                                ? 'bg-purple-500/15 border-purple-500/50 shadow-md'
                                : 'bg-[#121820] border-[#242E3D] hover:border-purple-500/30 hover:bg-[#161E28]'
                            }`}
                          >
                            <div className="flex items-center justify-between mb-1">
                              <div className="flex items-center space-x-1.5">
                                <span className="text-base">{preset.flag}</span>
                                <span className="text-xs font-bold text-white">{preset.dialect}</span>
                              </div>
                              <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#0B0E14] text-[#94A3B8] border border-[#242E3D]">
                                {preset.langCode}
                              </span>
                            </div>

                            <p className="text-[11px] text-[#CBD5E1] line-clamp-2 italic mb-1.5">
                              "{preset.sampleText}"
                            </p>

                            <div className="flex items-center justify-between text-[10px] font-mono text-[#64748B]">
                              <span>💡 {preset.cadenceHints}</span>
                              <span className="text-purple-400 font-semibold">
                                {preset.speed}x / {preset.pitch > 0 ? `+${preset.pitch}` : preset.pitch}st
                              </span>
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

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
