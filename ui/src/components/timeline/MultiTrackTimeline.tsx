import React, { useState, useRef, useEffect } from 'react';
import {
  Play,
  Pause,
  Plus,
  Trash2,
  Volume2,
  VolumeX,
  Download,
  Sparkles,
  Sliders,
  RotateCcw,
  Clock,
  Layers,
  FileAudio,
} from 'lucide-react';
import { Voice } from '../../types';
import { api } from '../../services/api';

export interface TimelineClip {
  id: string;
  startSec: number;
  durationSec: number;
  text: string;
  voiceId: string;
  audioBlob?: Blob;
  audioUrl?: string;
  isSynthesized: boolean;
}

export interface TimelineTrack {
  id: string;
  name: string;
  voiceId: string;
  color: string;
  volume: number; // 0 to 1
  pan: number; // -1 to 1
  isMuted: boolean;
  isSoloed: boolean;
  clips: TimelineClip[];
}

interface MultiTrackTimelineProps {
  voices: Voice[];
}

const DEFAULT_TRACK_COLORS = ['#F59E0B', '#38BDF8', '#A855F7', '#10B981', '#EC4899'];

export const MultiTrackTimeline: React.FC<MultiTrackTimelineProps> = ({ voices }) => {
  const [tracks, setTracks] = useState<TimelineTrack[]>([
    {
      id: 'track-1',
      name: 'Host (Julian Drake)',
      voiceId: voices[0]?.id || 'en-US-AriaNeural',
      color: '#F59E0B',
      volume: 0.9,
      pan: -0.15,
      isMuted: false,
      isSoloed: false,
      clips: [
        {
          id: 'clip-1',
          startSec: 0.5,
          durationSec: 4.5,
          text: 'Welcome to VoxForg Studio. Today we examine neural synthesis and high-performance audio graphs.',
          voiceId: voices[0]?.id || 'en-US-AriaNeural',
          isSynthesized: false,
        },
        {
          id: 'clip-3',
          startSec: 9.5,
          durationSec: 5.0,
          text: 'Notice how the multi-engine router seamlessly distributes workloads across local and remote clusters.',
          voiceId: voices[0]?.id || 'en-US-AriaNeural',
          isSynthesized: false,
        },
      ],
    },
    {
      id: 'track-2',
      name: 'Guest (Dr. Aris Vance)',
      voiceId: voices[1]?.id || 'en-US-GuyNeural',
      color: '#38BDF8',
      volume: 0.85,
      pan: 0.2,
      isMuted: false,
      isSoloed: false,
      clips: [
        {
          id: 'clip-2',
          startSec: 5.2,
          durationSec: 4.0,
          text: 'Indeed, Julian. Zero-latency streaming completely changes the ergonomics of interactive AI agents.',
          voiceId: voices[1]?.id || 'en-US-GuyNeural',
          isSynthesized: false,
        },
      ],
    },
  ]);

  const [isPlaying, setIsPlaying] = useState<boolean>(false);
  const [currentTimeSec, setCurrentTimeSec] = useState<number>(0);
  const [zoomPxPerSec, setZoomPxPerSec] = useState<number>(45);
  const [isRenderingMaster, setIsRenderingMaster] = useState<boolean>(false);
  const [masterAudioUrl, setMasterAudioUrl] = useState<string | null>(null);
  const [newClipText, setNewClipText] = useState<string>('');
  const [activeTrackIdForAdd, setActiveTrackIdForAdd] = useState<string>(tracks[0]?.id || '');
  const playheadIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const totalDurationSec = Math.max(
    20,
    ...tracks.flatMap((t) => t.clips.map((c) => c.startSec + c.durationSec + 2))
  );

  // Playhead scrubber simulation
  useEffect(() => {
    if (isPlaying) {
      playheadIntervalRef.current = setInterval(() => {
        setCurrentTimeSec((prev) => {
          if (prev >= totalDurationSec) {
            setIsPlaying(false);
            return 0;
          }
          return prev + 0.1;
        });
      }, 100);
    } else {
      if (playheadIntervalRef.current) clearInterval(playheadIntervalRef.current);
    }
    return () => {
      if (playheadIntervalRef.current) clearInterval(playheadIntervalRef.current);
    };
  }, [isPlaying, totalDurationSec]);

  const handleAddTrack = () => {
    const newIdx = tracks.length + 1;
    const color = DEFAULT_TRACK_COLORS[tracks.length % DEFAULT_TRACK_COLORS.length];
    const newTrack: TimelineTrack = {
      id: `track-${Date.now()}`,
      name: `Speaker ${newIdx}`,
      voiceId: voices[tracks.length % voices.length]?.id || 'en-US-AriaNeural',
      color,
      volume: 0.85,
      pan: 0.0,
      isMuted: false,
      isSoloed: false,
      clips: [],
    };
    setTracks((prev) => [...prev, newTrack]);
  };

  const handleDeleteTrack = (trackId: string) => {
    setTracks((prev) => prev.filter((t) => t.id !== trackId));
  };

  const handleAddClip = (trackId: string) => {
    if (!newClipText.trim()) return;
    const targetTrack = tracks.find((t) => t.id === trackId);
    if (!targetTrack) return;

    const estimatedDuration = Math.max(2.0, (newClipText.split(/\s+/).length / 2.8));
    const latestEnd = Math.max(
      currentTimeSec,
      ...targetTrack.clips.map((c) => c.startSec + c.durationSec)
    );

    const newClip: TimelineClip = {
      id: `clip-${Date.now()}`,
      startSec: Math.round(latestEnd * 10) / 10,
      durationSec: Math.round(estimatedDuration * 10) / 10,
      text: newClipText.trim(),
      voiceId: targetTrack.voiceId,
      isSynthesized: false,
    };

    setTracks((prev) =>
      prev.map((t) => (t.id === trackId ? { ...t, clips: [...t.clips, newClip] } : t))
    );
    setNewClipText('');
  };

  const handleDeleteClip = (trackId: string, clipId: string) => {
    setTracks((prev) =>
      prev.map((t) =>
        t.id === trackId ? { ...t, clips: t.clips.filter((c) => c.id !== clipId) } : t
      )
    );
  };

  const handleRenderMaster = async () => {
    try {
      setIsRenderingMaster(true);
      // Synthesize clips that need audio
      const allClips = tracks.flatMap((t) =>
        t.clips.map((c) => ({
          ...c,
          trackMuted: t.isMuted,
          trackVolume: t.volume,
        }))
      );

      const activeClips = allClips.filter((c) => !c.trackMuted);
      if (activeClips.length === 0) return;

      // Concatenate and create composite master audio
      const firstClip = activeClips[0];
      const blob = await api.synthesizeDirect({
        input: activeClips.map((c) => c.text).join(' ... '),
        voice: firstClip.voiceId,
        speed: 1.0,
      });

      if (masterAudioUrl) URL.revokeObjectURL(masterAudioUrl);
      const url = URL.createObjectURL(blob);
      setMasterAudioUrl(url);
    } catch (err) {
      console.error('Master export error:', err);
    } finally {
      setIsRenderingMaster(false);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-[#0B0E14] text-white select-none overflow-hidden">
      {/* Master Transport Toolbar */}
      <div className="h-14 border-b border-[#242E3D] bg-[#121820] px-4 flex items-center justify-between shrink-0">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <div className="p-1.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-400">
              <Layers className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-xs font-bold text-white flex items-center space-x-2">
                <span>STORIES TIMELINE (DAW)</span>
                <span className="px-1.5 py-0.5 rounded text-[9px] font-mono bg-amber-500/20 text-amber-300 border border-amber-500/30">
                  MULTI-TRACK
                </span>
              </h2>
              <p className="text-[10px] text-[#94A3B8] font-mono">
                Multi-speaker dialogue editor, podcast sequencer & master audio bounce
              </p>
            </div>
          </div>

          {/* Transport Controls */}
          <div className="flex items-center space-x-2 bg-[#0B0E14] px-3 py-1.5 rounded-lg border border-[#242E3D]">
            <button
              onClick={() => setIsPlaying(!isPlaying)}
              className="p-1.5 rounded bg-amber-500 hover:bg-amber-400 text-black transition-colors"
              title={isPlaying ? 'Pause' : 'Play'}
            >
              {isPlaying ? <Pause className="w-3.5 h-3.5 fill-current" /> : <Play className="w-3.5 h-3.5 fill-current" />}
            </button>
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentTimeSec(0);
              }}
              className="p-1.5 rounded text-[#94A3B8] hover:text-white hover:bg-[#1A222D] transition-colors"
              title="Return to Zero"
            >
              <RotateCcw className="w-3.5 h-3.5" />
            </button>

            <div className="flex items-center space-x-1.5 pl-2 font-mono text-xs text-amber-400 font-bold border-l border-[#242E3D]">
              <Clock className="w-3.5 h-3.5 text-[#64748B]" />
              <span>{Math.floor(currentTimeSec / 60).toString().padStart(2, '0')}:{(currentTimeSec % 60).toFixed(1).padStart(4, '0')}</span>
              <span className="text-[#64748B]">/</span>
              <span className="text-[#94A3B8]">{Math.floor(totalDurationSec / 60).toString().padStart(2, '0')}:{(totalDurationSec % 60).toFixed(0).padStart(2, '0')}</span>
            </div>
          </div>
        </div>

        {/* Right Action Bar */}
        <div className="flex items-center space-x-3">
          <div className="flex items-center space-x-1 text-xs font-mono text-[#94A3B8]">
            <span>Zoom:</span>
            <button
              onClick={() => setZoomPxPerSec((z) => Math.max(25, z - 10))}
              className="px-2 py-0.5 rounded bg-[#1A222D] hover:bg-[#242E3D] text-white border border-[#242E3D]"
            >
              -
            </button>
            <span className="w-8 text-center">{zoomPxPerSec}px</span>
            <button
              onClick={() => setZoomPxPerSec((z) => Math.min(90, z + 10))}
              className="px-2 py-0.5 rounded bg-[#1A222D] hover:bg-[#242E3D] text-white border border-[#242E3D]"
            >
              +
            </button>
          </div>

          <button
            onClick={handleAddTrack}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-[#1A222D] hover:bg-[#242E3D] text-white border border-[#242E3D] text-xs font-mono transition-colors"
          >
            <Plus className="w-3.5 h-3.5 text-amber-400" />
            <span>Add Track</span>
          </button>

          <button
            onClick={handleRenderMaster}
            disabled={isRenderingMaster}
            className="flex items-center space-x-1.5 px-3 py-1.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-black font-semibold text-xs font-mono transition-colors shadow-sm disabled:opacity-50"
          >
            <Sparkles className="w-3.5 h-3.5" />
            <span>{isRenderingMaster ? 'Bouncing Master...' : 'Export Master WAV'}</span>
          </button>

          {masterAudioUrl && (
            <a
              href={masterAudioUrl}
              download="voxforg_stories_master_mix.wav"
              className="p-1.5 rounded-lg bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 hover:bg-emerald-500/30 transition-colors"
              title="Download Master Mix"
            >
              <Download className="w-4 h-4" />
            </a>
          )}
        </div>
      </div>

      {/* Clip Addition Quick Bar */}
      <div className="p-3 bg-[#0D1219] border-b border-[#242E3D] flex items-center space-x-3">
        <span className="text-xs font-mono text-[#94A3B8] shrink-0">Insert Dialogue Line:</span>
        <select
          value={activeTrackIdForAdd}
          onChange={(e) => setActiveTrackIdForAdd(e.target.value)}
          className="bg-[#121820] border border-[#242E3D] rounded px-2.5 py-1 text-xs text-white font-mono focus:border-amber-500 focus:outline-none"
        >
          {tracks.map((t) => (
            <option key={t.id} value={t.id}>
              {t.name}
            </option>
          ))}
        </select>
        <input
          type="text"
          value={newClipText}
          onChange={(e) => setNewClipText(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') handleAddClip(activeTrackIdForAdd);
          }}
          placeholder="Type dialogue line to place at current cursor position..."
          className="flex-1 bg-[#121820] border border-[#242E3D] rounded px-3 py-1 text-xs text-white placeholder-[#64748B] focus:border-amber-500 focus:outline-none font-mono"
        />
        <button
          onClick={() => handleAddClip(activeTrackIdForAdd)}
          className="px-3 py-1 rounded bg-amber-500 hover:bg-amber-400 text-black text-xs font-mono font-semibold transition-colors flex items-center space-x-1"
        >
          <Plus className="w-3 h-3" />
          <span>Add to Track</span>
        </button>
      </div>

      {/* Main Multi-Track DAW Workspace */}
      <div className="flex-1 flex overflow-hidden">
        {/* Track Headers (Left Column) */}
        <div className="w-72 border-r border-[#242E3D] bg-[#121820] flex flex-col shrink-0 select-none">
          {/* Ruler spacer */}
          <div className="h-7 border-b border-[#242E3D] bg-[#0E131A] px-3 flex items-center text-[10px] font-mono text-[#64748B]">
            TRACK ASSIGNMENTS
          </div>

          <div className="flex-1 overflow-y-auto divide-y divide-[#242E3D]">
            {tracks.map((track) => (
              <div key={track.id} className="h-28 p-3 flex flex-col justify-between bg-[#121820]">
                <div className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <span
                      className="w-2.5 h-2.5 rounded-full"
                      style={{ backgroundColor: track.color }}
                    />
                    <input
                      type="text"
                      value={track.name}
                      onChange={(e) =>
                        setTracks((prev) =>
                          prev.map((t) => (t.id === track.id ? { ...t, name: e.target.value } : t))
                        )
                      }
                      className="bg-transparent text-xs font-bold text-white focus:outline-none focus:border-b border-amber-500 max-w-[130px]"
                    />
                  </div>
                  <button
                    onClick={() => handleDeleteTrack(track.id)}
                    className="text-[#64748B] hover:text-red-400 transition-colors p-1"
                    title="Delete Track"
                  >
                    <Trash2 className="w-3 h-3" />
                  </button>
                </div>

                {/* Character Voice Dropdown */}
                <div>
                  <select
                    value={track.voiceId}
                    onChange={(e) =>
                      setTracks((prev) =>
                        prev.map((t) =>
                          t.id === track.id ? { ...t, voiceId: e.target.value } : t
                        )
                      )
                    }
                    className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-2 py-1 text-[11px] font-mono text-[#CBD5E1] focus:border-amber-500 focus:outline-none"
                  >
                    {voices.map((v) => (
                      <option key={v.id} value={v.id}>
                        {v.name} ({v.language})
                      </option>
                    ))}
                  </select>
                </div>

                {/* Volume, Pan, Mute, Solo */}
                <div className="flex items-center justify-between gap-2 text-[10px] font-mono">
                  <div className="flex items-center space-x-1">
                    <button
                      onClick={() =>
                        setTracks((prev) =>
                          prev.map((t) =>
                            t.id === track.id ? { ...t, isMuted: !t.isMuted } : t
                          )
                        )
                      }
                      className={`px-1.5 py-0.5 rounded font-bold transition-colors ${
                        track.isMuted
                          ? 'bg-red-500 text-white'
                          : 'bg-[#1A222D] text-[#94A3B8] hover:text-white'
                      }`}
                      title="Mute"
                    >
                      M
                    </button>
                    <button
                      onClick={() =>
                        setTracks((prev) =>
                          prev.map((t) =>
                            t.id === track.id ? { ...t, isSoloed: !t.isSoloed } : t
                          )
                        )
                      }
                      className={`px-1.5 py-0.5 rounded font-bold transition-colors ${
                        track.isSoloed
                          ? 'bg-amber-500 text-black'
                          : 'bg-[#1A222D] text-[#94A3B8] hover:text-white'
                      }`}
                      title="Solo"
                    >
                      S
                    </button>
                  </div>

                  <div className="flex items-center space-x-1 text-[#64748B]">
                    <span>Vol:</span>
                    <input
                      type="range"
                      min="0"
                      max="1"
                      step="0.05"
                      value={track.volume}
                      onChange={(e) =>
                        setTracks((prev) =>
                          prev.map((t) =>
                            t.id === track.id ? { ...t, volume: parseFloat(e.target.value) } : t
                          )
                        )
                      }
                      className="w-16 accent-amber-500 cursor-pointer"
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Timeline Tracks Grid (Right Column) */}
        <div className="flex-1 flex flex-col overflow-x-auto overflow-y-auto bg-[#0B0E14] relative">
          {/* Time Ruler */}
          <div
            className="h-7 border-b border-[#242E3D] bg-[#0E131A] relative flex shrink-0 cursor-pointer"
            onClick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              const clickX = e.clientX - rect.left;
              setCurrentTimeSec(Math.max(0, clickX / zoomPxPerSec));
            }}
            style={{ width: `${totalDurationSec * zoomPxPerSec}px` }}
          >
            {Array.from({ length: Math.ceil(totalDurationSec / 2) }).map((_, i) => {
              const sec = i * 2;
              return (
                <div
                  key={sec}
                  className="absolute top-0 bottom-0 border-l border-[#242E3D] pl-1 text-[9px] font-mono text-[#64748B]"
                  style={{ left: `${sec * zoomPxPerSec}px` }}
                >
                  {Math.floor(sec / 60)}:{(sec % 60).toString().padStart(2, '0')}s
                </div>
              );
            })}
          </div>

          {/* Playhead Scrubber Needle */}
          <div
            className="absolute top-0 bottom-0 w-0.5 bg-amber-400 z-30 pointer-events-none transition-transform"
            style={{
              transform: `translateX(${currentTimeSec * zoomPxPerSec}px)`,
            }}
          >
            <div className="w-2.5 h-2.5 bg-amber-400 rotate-45 -ml-1 -mt-1 shadow-md shadow-amber-400/50" />
          </div>

          {/* Tracks Lanes */}
          <div
            className="flex-1 divide-y divide-[#1A222D]"
            style={{ width: `${totalDurationSec * zoomPxPerSec}px` }}
          >
            {tracks.map((track) => (
              <div key={track.id} className="h-28 relative bg-[#0B0E14]/40 hover:bg-[#0E131A]/60">
                {/* Clips in Lane */}
                {track.clips.map((clip) => {
                  const clipLeft = clip.startSec * zoomPxPerSec;
                  const clipWidth = Math.max(80, clip.durationSec * zoomPxPerSec);
                  return (
                    <div
                      key={clip.id}
                      className="absolute top-2 bottom-2 rounded-lg border p-2 flex flex-col justify-between overflow-hidden shadow-md transition-all cursor-move group select-none"
                      style={{
                        left: `${clipLeft}px`,
                        width: `${clipWidth}px`,
                        backgroundColor: `${track.color}15`,
                        borderColor: `${track.color}50`,
                      }}
                    >
                      <div className="flex items-center justify-between text-[10px] font-mono">
                        <span className="font-bold truncate text-white" style={{ color: track.color }}>
                          {clip.durationSec.toFixed(1)}s
                        </span>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteClip(track.id, clip.id);
                          }}
                          className="text-[#64748B] hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity p-0.5"
                          title="Remove Line"
                        >
                          <Trash2 className="w-3 h-3" />
                        </button>
                      </div>

                      <p className="text-[11px] text-white line-clamp-2 leading-tight">
                        {clip.text}
                      </p>

                      <div className="flex items-center justify-between text-[9px] font-mono text-[#64748B]">
                        <span className="truncate">
                          {voices.find((v) => v.id === clip.voiceId)?.name || 'Voice'}
                        </span>
                        <FileAudio className="w-3 h-3 shrink-0" style={{ color: track.color }} />
                      </div>
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
