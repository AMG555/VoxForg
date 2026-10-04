import React, { useState, useEffect, useRef } from 'react';
import {
  Mic,
  Square,
  Sparkles,
  Copy,
  Check,
  Trash2,
  X,
  Keyboard,
  Activity,
  ArrowRight,
} from 'lucide-react';
import { api } from '../../services/api';
import { AudioProcessor } from '../../services/audioProcessor';

interface DictationBarProps {
  isOpen: boolean;
  onClose: () => void;
  onSendToVoiceLab?: (text: string) => void;
}

export const DictationBar: React.FC<DictationBarProps> = ({
  isOpen,
  onClose,
  onSendToVoiceLab,
}) => {
  const [isRecording, setIsRecording] = useState<boolean>(false);
  const [recordingDurationSec, setRecordingDurationSec] = useState<number>(0);
  const [rmsLevel, setRmsLevel] = useState<number>(0);
  const [rawTranscript, setRawTranscript] = useState<string>('');
  const [refinedTranscript, setRefinedTranscript] = useState<string>('');
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [statusMessage, setStatusMessage] = useState<string>('');
  const [copied, setCopied] = useState<boolean>(false);

  // Refinement settings
  const [removeFillers, setRemoveFillers] = useState<boolean>(true);
  const [fixRepetitions, setFixRepetitions] = useState<boolean>(true);
  const [removedCount, setRemovedCount] = useState<number>(0);
  const [aecEnabled, setAecEnabled] = useState<boolean>(true);

  // Audio Recording Refs
  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const audioChunksRef = useRef<Blob[]>([]);
  const timerIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animFrameRef = useRef<number | null>(null);

  // Global hotkey listener (Ctrl+Shift+D or Space hold when open)
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Toggle dictation with Ctrl+Shift+D
      if (e.ctrlKey && e.shiftKey && e.code === 'KeyD') {
        e.preventDefault();
        if (isRecording) {
          stopRecording();
        } else {
          startRecording();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isRecording]);

  const startRecording = async () => {
    try {
      setStatusMessage('Accessing studio microphone...');
      audioChunksRef.current = [];
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          channelCount: 1,
          sampleRate: 24000,
          echoCancellation: aecEnabled,
          noiseSuppression: true,
        },
      });

      // Set up WebAudio analyser for live RMS waveform meter
      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      const audioCtx = new AudioCtx();
      audioContextRef.current = audioCtx;
      const source = audioCtx.createMediaStreamSource(stream);
      const analyser = audioCtx.createAnalyser();
      analyser.fftSize = 256;
      source.connect(analyser);
      analyserRef.current = analyser;

      const dataArray = new Uint8Array(analyser.frequencyBinCount);
      const updateLevel = () => {
        analyser.getByteFrequencyData(dataArray);
        let sum = 0;
        for (let i = 0; i < dataArray.length; i++) {
          sum += dataArray[i];
        }
        const avg = sum / dataArray.length;
        setRmsLevel(Math.min(100, Math.round((avg / 128) * 100)));
        animFrameRef.current = requestAnimationFrame(updateLevel);
      };
      updateLevel();

      const mediaRecorder = new MediaRecorder(stream);
      mediaRecorderRef.current = mediaRecorder;

      mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          audioChunksRef.current.push(event.data);
        }
      };

      mediaRecorder.onstop = async () => {
        if (animFrameRef.current) cancelAnimationFrame(animFrameRef.current);
        stream.getTracks().forEach((t) => t.stop());
        if (audioContextRef.current && audioContextRef.current.state !== 'closed') {
          audioContextRef.current.close().catch(() => {});
        }
        setRmsLevel(0);
        await processRecordedAudio();
      };

      mediaRecorder.start(250);
      setIsRecording(true);
      setRecordingDurationSec(0);
      setStatusMessage('Listening & recording speech...');

      timerIntervalRef.current = setInterval(() => {
        setRecordingDurationSec((prev) => prev + 0.1);
      }, 100);
    } catch (err: any) {
      console.error('Microphone error:', err);
      setStatusMessage(`Microphone error: ${err.message || 'Permission denied'}`);
      setIsRecording(false);
    }
  };

  const stopRecording = () => {
    if (timerIntervalRef.current) {
      clearInterval(timerIntervalRef.current);
      timerIntervalRef.current = null;
    }
    if (mediaRecorderRef.current && mediaRecorderRef.current.state !== 'inactive') {
      mediaRecorderRef.current.stop();
    }
    setIsRecording(false);
  };

  const processRecordedAudio = async () => {
    if (audioChunksRef.current.length === 0) return;

    try {
      setIsProcessing(true);
      setStatusMessage('Preprocessing audio & transcribing via Whisper...');
      const rawBlob = new Blob(audioChunksRef.current, { type: 'audio/webm' });

      // Preprocess through studio audio processor to get clean WAV base64
      const preprocessed = await AudioProcessor.preprocessForCloning(rawBlob);

      // Transcribe via ASR
      const transcription = await api.transcribeAudio(preprocessed.wavBase64);
      const text = transcription.text.trim();
      setRawTranscript(text);

      if (!text) {
        setStatusMessage('No clear speech detected in recorded audio.');
        setIsProcessing(false);
        return;
      }

      // Automatically refine text through verbal filler remover & repetition filter
      setStatusMessage('Refining text & stripping verbal fillers...');
      const refinement = await api.refineText(text, {
        remove_fillers: removeFillers,
        fix_repetitions: fixRepetitions,
        punctuation_pass: true,
      });

      setRefinedTranscript(refinement.refined_text);
      setRemovedCount(refinement.removed_fillers + refinement.cleaned_repetitions);
      setStatusMessage('Transcription & neural text refinement complete!');
    } catch (err: any) {
      console.error('Dictation processing error:', err);
      setStatusMessage(`Error: ${err.message || 'Processing failed'}`);
    } finally {
      setIsProcessing(false);
    }
  };

  const handleCopy = () => {
    const textToCopy = refinedTranscript || rawTranscript;
    if (!textToCopy) return;
    navigator.clipboard.writeText(textToCopy);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleClear = () => {
    setRawTranscript('');
    setRefinedTranscript('');
    setStatusMessage('');
    setRemovedCount(0);
  };

  if (!isOpen) return null;

  return (
    <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 w-full max-w-2xl px-4 animate-in fade-in slide-in-from-bottom-4 duration-200">
      <div className="bg-[#121820]/95 backdrop-blur-md border border-[#242E3D] rounded-2xl shadow-2xl p-5 text-white">
        {/* Header */}
        <div className="flex items-center justify-between pb-3 border-b border-[#242E3D]">
          <div className="flex items-center space-x-2.5">
            <div className={`p-1.5 rounded-lg ${isRecording ? 'bg-red-500/20 text-red-400 animate-pulse' : 'bg-amber-500/10 text-amber-400'}`}>
              <Mic className="w-4 h-4" />
            </div>
            <div>
              <div className="flex items-center space-x-2">
                <h3 className="text-xs font-bold uppercase tracking-wider text-white">
                  VOICEBOX DICTATION & PUSH-TO-TALK
                </h3>
                <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-[#1A222D] text-[#94A3B8] border border-[#242E3D] flex items-center space-x-1">
                  <Keyboard className="w-2.5 h-2.5" />
                  <span>Ctrl+Shift+D</span>
                </span>
              </div>
              <p className="text-[10px] text-[#94A3B8] font-mono">
                Real-time Whisper capture with verbal filler & hallucination cleaner
              </p>
            </div>
          </div>

          <button
            onClick={onClose}
            className="text-[#94A3B8] hover:text-white p-1 rounded-lg hover:bg-[#1A222D] transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Live Audio Monitor & Controls */}
        <div className="mt-4 flex items-center justify-between bg-[#0B0E14] px-4 py-3 rounded-xl border border-[#242E3D]">
          <div className="flex items-center space-x-4">
            <button
              onClick={isRecording ? stopRecording : startRecording}
              disabled={isProcessing}
              className={`flex items-center space-x-2 px-4 py-2 rounded-xl text-xs font-bold transition-all shadow-lg ${
                isRecording
                  ? 'bg-red-500 hover:bg-red-600 text-white animate-pulse'
                  : 'bg-amber-500 hover:bg-amber-400 text-black'
              }`}
            >
              {isRecording ? (
                <>
                  <Square className="w-4 h-4 fill-current" />
                  <span>Stop Dictating</span>
                </>
              ) : (
                <>
                  <Mic className="w-4 h-4" />
                  <span>Start Push-to-Talk</span>
                </>
              )}
            </button>

            {/* Live VU / RMS Meter */}
            <div className="flex items-center space-x-2">
              <span className="text-[11px] font-mono text-[#94A3B8]">
                {recordingDurationSec.toFixed(1)}s
              </span>
              <div className="w-32 h-2.5 bg-[#1A222D] rounded-full overflow-hidden flex items-center p-0.5">
                <div
                  className="h-full rounded-full transition-all duration-75 bg-gradient-to-r from-emerald-500 via-amber-500 to-red-500"
                  style={{ width: `${rmsLevel}%` }}
                />
              </div>
            </div>
          </div>

          {/* Quick Refinement Toggles */}
          <div className="flex items-center space-x-3 text-xs font-mono">
            <label className="flex items-center space-x-1.5 cursor-pointer text-[#94A3B8] hover:text-white">
              <input
                type="checkbox"
                checked={removeFillers}
                onChange={(e) => setRemoveFillers(e.target.checked)}
                className="rounded border-[#3B485C] bg-[#1A222D] text-amber-500 focus:ring-0"
              />
              <span>Strip Um/Uh</span>
            </label>

            <label className="flex items-center space-x-1.5 cursor-pointer text-[#94A3B8] hover:text-white">
              <input
                type="checkbox"
                checked={fixRepetitions}
                onChange={(e) => setFixRepetitions(e.target.checked)}
                className="rounded border-[#3B485C] bg-[#1A222D] text-amber-500 focus:ring-0"
              />
              <span>Anti-Loop</span>
            </label>

            <button
              type="button"
              onClick={() => setAecEnabled(!aecEnabled)}
              className={`flex items-center space-x-1 px-2 py-0.5 rounded text-[11px] font-mono border transition-colors ${
                aecEnabled
                  ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'
                  : 'bg-[#121820] text-[#64748B] border-[#242E3D]'
              }`}
              title="Acoustic Echo Cancellation: eliminates feedback from speakers"
            >
              <Activity className="w-3 h-3" />
              <span>AEC {aecEnabled ? 'ON' : 'OFF'}</span>
            </button>
          </div>
        </div>

        {/* Status Message */}
        {statusMessage && (
          <div className="mt-2 text-[11px] font-mono text-amber-400 flex items-center space-x-1.5">
            <Activity className="w-3 h-3 animate-spin" />
            <span>{statusMessage}</span>
          </div>
        )}

        {/* Refined Output Box */}
        {(refinedTranscript || rawTranscript) && (
          <div className="mt-4 space-y-2">
            <div className="flex items-center justify-between text-[11px] text-[#94A3B8] font-mono">
              <span className="flex items-center space-x-1 text-white">
                <Sparkles className="w-3.5 h-3.5 text-amber-400" />
                <span>Refined Transcript:</span>
                {removedCount > 0 && (
                  <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    Cleaned {removedCount} fillers/loops
                  </span>
                )}
              </span>
              <span>{(refinedTranscript || rawTranscript).split(/\s+/).filter(Boolean).length} words</span>
            </div>

            <div className="bg-[#0B0E14] p-3.5 rounded-xl border border-[#242E3D] text-sm text-[#F8FAFC] leading-relaxed font-sans max-h-36 overflow-y-auto select-text">
              {refinedTranscript || rawTranscript}
            </div>

            {/* Actions Toolbar */}
            <div className="flex items-center justify-between pt-2">
              <div className="flex items-center space-x-2">
                <button
                  onClick={handleCopy}
                  className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 text-amber-300 border border-amber-500/30 text-xs font-medium transition-colors"
                >
                  {copied ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
                  <span>{copied ? 'Copied!' : 'Copy to Clipboard'}</span>
                </button>

                {onSendToVoiceLab && (
                  <button
                    onClick={() => onSendToVoiceLab(refinedTranscript || rawTranscript)}
                    className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white text-xs font-medium transition-colors"
                  >
                    <span>Send to Voice Lab</span>
                    <ArrowRight className="w-3.5 h-3.5" />
                  </button>
                )}
              </div>

              <button
                onClick={handleClear}
                className="text-[#94A3B8] hover:text-red-400 p-1.5 rounded-lg hover:bg-[#1A222D] transition-colors"
                title="Clear transcript"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
