import { X, Sliders, Trash2, Wand2, Volume2, Mic, Copy, Clipboard, FileText, Layers, Gauge, Radio, Users } from 'lucide-react';
import { PipelineNode, Voice } from '../../types';

interface NodeInspectorProps {
  node: PipelineNode | null;
  selectedNodes?: PipelineNode[];
  allNodes?: PipelineNode[];
  voices: Voice[];
  onClose: () => void;
  onUpdateParams: (nodeId: string, params: Record<string, any>) => void;
  onUpdateName?: (nodeId: string, name: string) => void;
  onDeleteNode?: (nodeId: string) => void;
  onDuplicateNode?: (nodeId: string) => void;
  onCopyNodeJson?: (node: PipelineNode) => void;
  onDeleteSelected?: () => void;
  onDuplicateSelected?: () => void;
  onCopySelectedJson?: () => void;
}

export const NodeInspector: React.FC<NodeInspectorProps> = ({
  node,
  selectedNodes,
  allNodes,
  voices,
  onClose,
  onUpdateParams,
  onUpdateName,
  onDeleteNode,
  onDuplicateNode,
  onCopyNodeJson,
  onDeleteSelected,
  onDuplicateSelected,
  onCopySelectedJson,
}) => {
  if (selectedNodes && selectedNodes.length > 1) {
    return (
      <div className="w-80 border-l border-[#242E3D] bg-[#121820]/95 backdrop-blur flex flex-col h-full text-[#F0F4F8] select-none z-30 shadow-2xl">
        {/* Header */}
        <div className="p-4 border-b border-[#242E3D] flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Sliders className="w-4 h-4 text-amber-500" />
            <span className="font-semibold text-sm">Selection ({selectedNodes.length} nodes)</span>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-[#1A222D] text-[#94A3B8] hover:text-white transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          <div className="p-3 rounded-lg bg-[#0B0E14] border border-[#242E3D] text-xs font-mono space-y-2">
            <div className="text-[#94A3B8] font-sans">
              {allNodes && selectedNodes.length === allNodes.length
                ? 'Complete workflow selected.'
                : `${selectedNodes.length} nodes selected across workflow.`}
            </div>
            <div className="text-[11px] text-amber-400">
              Shortcuts: Del (delete all), Ctrl+C (copy JSON), Ctrl+D (duplicate all).
            </div>
          </div>

          <div className="space-y-1.5">
            <span className="text-[11px] font-mono text-[#64748B] uppercase tracking-wider">
              Selected Nodes
            </span>
            <div className="space-y-1 max-h-60 overflow-y-auto pr-1">
              {selectedNodes.map((n) => (
                <div
                  key={n.id}
                  className="px-2.5 py-1.5 rounded bg-[#1A222D] border border-[#242E3D] flex items-center justify-between text-xs font-mono"
                >
                  <span className="truncate text-white">{n.name}</span>
                  <span className="text-[10px] text-[#94A3B8] uppercase ml-2">{n.node_type}</span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Multi-Node Actions Footer */}
        <div className="p-3 border-t border-[#242E3D] bg-[#0B0E14]/60 flex flex-col space-y-2 text-xs font-mono">
          <div className="flex items-center space-x-2">
            {onDuplicateSelected && (
              <button
                type="button"
                onClick={onDuplicateSelected}
                className="flex-1 flex items-center justify-center space-x-1.5 px-3 py-2 rounded bg-[#1A222D] hover:bg-[#242E3D] text-white transition-colors"
                title="Duplicate all selected nodes (Ctrl+D)"
              >
                <Copy className="w-3.5 h-3.5 text-amber-400" />
                <span>Duplicate ({selectedNodes.length})</span>
              </button>
            )}
            {onCopySelectedJson && (
              <button
                type="button"
                onClick={onCopySelectedJson}
                className="flex-1 flex items-center justify-center space-x-1.5 px-3 py-2 rounded bg-[#1A222D] hover:bg-[#242E3D] text-white transition-colors"
                title="Copy selection JSON (Ctrl+C)"
              >
                <Clipboard className="w-3.5 h-3.5 text-sky-400" />
                <span>Copy JSON</span>
              </button>
            )}
          </div>
          {onDeleteSelected && (
            <button
              type="button"
              onClick={onDeleteSelected}
              className="w-full flex items-center justify-center space-x-1.5 px-3 py-2 rounded bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 border border-rose-500/30 transition-colors"
              title="Delete all selected nodes (Del / Backspace)"
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>Delete All Selected ({selectedNodes.length})</span>
            </button>
          )}
        </div>
      </div>
    );
  }

  if (!node) return null;

  const handleChange = (key: string, value: any) => {
    onUpdateParams(node.id, {
      ...node.params,
      [key]: value,
    });
  };

  const handleAutoDetectSpeakers = () => {
    if (!allNodes) return;
    const textInputs = allNodes.filter((n) => n.node_type === 'text_input');
    const discoveredSpeakers = new Set<string>();

    for (const inputNode of textInputs) {
      const text = inputNode.params.text || '';
      const lines = text.split('\n');
      for (const line of lines) {
        const trimmed = line.trim();
        let speaker = '';
        if (trimmed.includes(':')) {
          speaker = trimmed.split(':')[0].trim();
        } else if (trimmed.startsWith('[') && trimmed.includes(']')) {
          speaker = trimmed.substring(1, trimmed.indexOf(']')).trim();
        } else if (trimmed.startsWith('(') && trimmed.includes(')')) {
          speaker = trimmed.substring(1, trimmed.indexOf(')')).trim();
        }
        if (speaker && speaker.length > 0 && speaker.length < 32) {
          discoveredSpeakers.add(speaker);
        }
      }
    }

    if (discoveredSpeakers.size > 0) {
      const currentMap = { ...(node.params.speaker_map || {}) };
      const speakerList = Array.from(discoveredSpeakers);
      speakerList.forEach((spk, idx) => {
        if (!currentMap[spk]) {
          const assignedVoice = voices[idx % voices.length]?.id || node.params.default_voice || 'en-US-AriaNeural';
          currentMap[spk] = assignedVoice;
        }
      });
      handleChange('speaker_map', currentMap);
    } else {
      alert('No speaker prefixes found in script! Format lines as "Speaker: Line" or "[Speaker] Line".');
    }
  };

  return (
    <aside className="w-80 border-l border-[#242E3D] bg-[#121820] flex flex-col h-full z-10 select-none shadow-xl">
      <div className="h-12 px-4 border-b border-[#242E3D] flex items-center justify-between bg-[#0B0E14]/40">
        <div className="flex items-center space-x-2">
          <Sliders className="w-4 h-4 text-amber-500" />
          <span className="text-xs font-semibold uppercase tracking-wider text-white">
            Node Inspector
          </span>
        </div>
        <div className="flex items-center space-x-1">
          {onCopyNodeJson && (
            <button
              onClick={() => onCopyNodeJson(node)}
              className="p-1 rounded text-[#94A3B8] hover:text-white hover:bg-[#1A222D] transition-colors"
              title="Copy Node JSON (Ctrl+C)"
            >
              <Clipboard className="w-3.5 h-3.5" />
            </button>
          )}
          {onDuplicateNode && (
            <button
              onClick={() => onDuplicateNode(node.id)}
              className="p-1 rounded text-[#94A3B8] hover:text-amber-400 hover:bg-[#1A222D] transition-colors"
              title="Duplicate Node (Ctrl+D)"
            >
              <Copy className="w-3.5 h-3.5" />
            </button>
          )}
          {onDeleteNode && (
            <button
              onClick={() => onDeleteNode(node.id)}
              className="p-1 rounded text-[#64748B] hover:text-rose-400 hover:bg-[#1A222D] transition-colors"
              title="Delete Node (Del)"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          )}
          <button
            onClick={onClose}
            className="p-1 rounded text-[#94A3B8] hover:text-white hover:bg-[#1A222D] transition-colors"
            title="Close Inspector"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      <div className="p-4 space-y-5 flex-1 overflow-y-auto font-sans text-xs">
        <div>
          <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
            Node Name
          </label>
          <input
            type="text"
            value={node.name}
            onChange={(e) => onUpdateName?.(node.id, e.target.value)}
            placeholder="Node Name"
            className="w-full bg-[#0B0E14] border border-[#242E3D] hover:border-[#38465a] focus:border-amber-500 rounded px-3 py-1.5 text-white font-mono text-xs focus:outline-none transition-colors"
          />
        </div>

        {node.node_type === 'character_voice' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-amber-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <Mic className="w-4 h-4 text-amber-400" />
              <span>Character Dialogue & Voice</span>
            </div>

            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Character Name
              </label>
              <input
                type="text"
                value={node.params.character_name || ''}
                onChange={(e) => handleChange('character_name', e.target.value)}
                placeholder="e.g. Captain, Narrator, AI Co-pilot"
                className="w-full bg-[#121820] border border-[#242E3D] focus:border-amber-500 rounded px-3 py-1.5 text-white font-mono text-xs focus:outline-none"
              />
            </div>

            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Assigned Voice
              </label>
              <select
                value={node.params.voice_id || 'en-US-AriaNeural'}
                onChange={(e) => handleChange('voice_id', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] focus:border-amber-500 rounded px-3 py-2 text-white text-xs font-mono focus:outline-none"
              >
                {voices
                  .filter((v) => v.engine_id !== 'mock-tts')
                  .map((v) => (
                    <option key={v.id} value={v.id}>
                      {v.name} ({v.language} · {v.engine_id})
                    </option>
                  ))}
              </select>
            </div>

            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Dialogue Line
              </label>
              <textarea
                rows={5}
                value={node.params.text || ''}
                onChange={(e) => handleChange('text', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] focus:border-amber-500 rounded p-2.5 text-white text-xs focus:outline-none resize-none leading-relaxed font-sans"
                placeholder="Enter character spoken line..."
              />
            </div>

            <div className="space-y-3 pt-2 border-t border-[#1A222D]">
              <div>
                <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                  <span>Speaking Rate</span>
                  <span className="text-amber-400">{(node.params.speed ?? 1.0).toFixed(2)}x</span>
                </div>
                <input
                  type="range"
                  min="0.5"
                  max="2.0"
                  step="0.05"
                  value={node.params.speed ?? 1.0}
                  onChange={(e) => handleChange('speed', parseFloat(e.target.value))}
                  className="w-full accent-amber-500 cursor-pointer"
                />
              </div>

              <div>
                <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                  <span>Pitch Shift</span>
                  <span className="text-amber-400">
                    {(node.params.pitch ?? 0) > 0 ? `+${node.params.pitch}` : node.params.pitch ?? 0} st
                  </span>
                </div>
                <input
                  type="range"
                  min="-12"
                  max="12"
                  step="1"
                  value={node.params.pitch ?? 0}
                  onChange={(e) => handleChange('pitch', parseInt(e.target.value, 10))}
                  className="w-full accent-amber-500 cursor-pointer"
                />
              </div>
            </div>
          </div>
        )}

        {node.node_type === 'text_input' && (
          <div>
            <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
              Script Text
            </label>
            <textarea
              rows={8}
              value={node.params.text || ''}
              onChange={(e) => handleChange('text', e.target.value)}
              className="w-full bg-[#0B0E14] border border-[#242E3D] rounded p-2.5 text-white text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed font-sans"
              placeholder="Enter narrative or dialog (e.g. Host: Welcome to the studio! \n Guest: Happy to be here!)"
            />
            <p className="text-[10px] text-[#64748B] font-mono mt-1.5">
              Tip: Prefix lines with <code className="text-amber-400 font-bold">Speaker:</code> or <code className="text-amber-400 font-bold">[Speaker]</code> for automatic multi-voice allocation.
            </p>
          </div>
        )}

        {node.node_type === 'speaker_parser' && (
          <div className="space-y-3 bg-[#0B0E14] p-3 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-purple-400 font-semibold text-xs">
              <Mic className="w-4 h-4" />
              <span>Speaker Script Parser</span>
            </div>
            <p className="text-[11px] text-[#94A3B8] leading-relaxed">
              Splits incoming narrative text into discrete chronological speaker segments for podcast and comms mastering.
            </p>
            <div className="bg-[#121820] p-2.5 rounded border border-[#242E3D] text-[10px] font-mono space-y-1 text-[#64748B]">
              <div className="text-[#94A3B8] font-bold uppercase text-[9px]">Supported Formats:</div>
              <div><span className="text-amber-400">Host:</span> Welcome to the show.</div>
              <div><span className="text-sky-400">[Guest]</span> Great to be here!</div>
              <div><span className="text-emerald-400">(Dispatch)</span> Radio check complete.</div>
            </div>
          </div>
        )}

        {node.node_type === 'voice_assigner' && (
          <div className="space-y-4">
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Default Voice Profile
              </label>
              <select
                value={node.params.default_voice || 'en-US-AriaNeural'}
                onChange={(e) => handleChange('default_voice', e.target.value)}
                className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-2 text-white text-xs focus:border-amber-500 focus:outline-none font-mono"
              >
                {voices.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.name} ({v.language})
                  </option>
                ))}
              </select>
            </div>

            {/* Multi-Speaker Mapping */}
            <div className="space-y-2 pt-2 border-t border-[#242E3D]">
              <div className="flex items-center justify-between">
                <span className="text-[11px] font-mono uppercase text-[#94A3B8]">
                  Speaker Voice Mapping
                </span>
                <div className="flex items-center space-x-2">
                  <button
                    type="button"
                    onClick={handleAutoDetectSpeakers}
                    className="flex items-center space-x-1 text-[10px] font-mono text-amber-400 hover:text-amber-300 bg-amber-500/10 hover:bg-amber-500/20 px-2 py-0.5 rounded border border-amber-500/30 transition-colors"
                    title="Auto-scan script nodes and populate all speakers"
                  >
                    <Wand2 className="w-3 h-3" />
                    <span>Auto-Detect</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      const speaker = prompt('Enter Speaker Name (e.g. Captain, Nav, Host, Guest):');
                      if (speaker && speaker.trim()) {
                        const currentMap = node.params.speaker_map || {};
                        handleChange('speaker_map', {
                          ...currentMap,
                          [speaker.trim()]: voices[0]?.id || 'en-US-AriaNeural',
                        });
                      }
                    }}
                    className="text-[10px] font-mono text-[#94A3B8] hover:text-white"
                  >
                    + Add
                  </button>
                </div>
              </div>

              {node.params.speaker_map &&
                Object.entries(node.params.speaker_map).map(([speaker, voiceId]) => (
                  <div key={speaker} className="bg-[#0B0E14] p-2.5 rounded border border-[#242E3D] space-y-1.5">
                    <div className="flex items-center justify-between">
                      <span className="font-semibold text-white text-xs">{speaker}</span>
                      <button
                        type="button"
                        onClick={() => {
                          const next = { ...node.params.speaker_map };
                          delete next[speaker];
                          handleChange('speaker_map', next);
                        }}
                        className="text-[#64748B] hover:text-rose-400 transition-colors"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                    <select
                      value={voiceId as string}
                      onChange={(e) => {
                        handleChange('speaker_map', {
                          ...node.params.speaker_map,
                          [speaker]: e.target.value,
                        });
                      }}
                      className="w-full bg-[#121820] border border-[#242E3D] rounded px-2 py-1 text-white text-[11px] font-mono"
                    >
                      {voices.map((v) => (
                        <option key={v.id} value={v.id}>
                          {v.name} ({v.language})
                        </option>
                      ))}
                    </select>
                  </div>
                ))}
            </div>
          </div>
        )}

        {node.node_type === 'audio_filter' && (
          <div className="space-y-5">
            {/* Peak Normalization */}
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-2">
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8]">
                <span className="font-semibold text-white">Peak Normalization</span>
                <span className="text-amber-400">
                  {((node.params.normalize_peak ?? 0.95) * 100).toFixed(0)}%
                </span>
              </div>
              <input
                type="range"
                min="0.5"
                max="1.0"
                step="0.01"
                value={node.params.normalize_peak ?? 0.95}
                onChange={(e) => handleChange('normalize_peak', parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>

            {/* Silence Trimmer */}
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-3">
              <div className="flex items-center justify-between">
                <label className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={Boolean(node.params.silence_trim)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        handleChange('silence_trim', { threshold_dbfs: -45, padding_ms: 30 });
                      } else {
                        const next = { ...node.params };
                        delete next.silence_trim;
                        onUpdateParams(node.id, next);
                      }
                    }}
                    className="accent-amber-500 rounded"
                  />
                  <span className="font-semibold text-white text-[11px]">Silence Trimmer</span>
                </label>
                {node.params.silence_trim && (
                  <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    Active
                  </span>
                )}
              </div>

              {node.params.silence_trim && (
                <div className="space-y-2 pt-1 border-t border-[#1A222D]">
                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Threshold</span>
                    <span className="text-white">{node.params.silence_trim.threshold_dbfs ?? -45} dBFS</span>
                  </div>
                  <input
                    type="range"
                    min="-60"
                    max="-20"
                    step="1"
                    value={node.params.silence_trim.threshold_dbfs ?? -45}
                    onChange={(e) =>
                      handleChange('silence_trim', {
                        ...node.params.silence_trim,
                        threshold_dbfs: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />

                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Padding Decay</span>
                    <span className="text-white">{node.params.silence_trim.padding_ms ?? 30} ms</span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="5"
                    value={node.params.silence_trim.padding_ms ?? 30}
                    onChange={(e) =>
                      handleChange('silence_trim', {
                        ...node.params.silence_trim,
                        padding_ms: parseInt(e.target.value, 10),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />
                </div>
              )}
            </div>

            {/* 3-Band Parametric EQ */}
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-3">
              <div className="flex items-center justify-between">
                <label className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={Boolean(node.params.eq)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        handleChange('eq', { low_gain_db: 0.0, mid_gain_db: 0.0, high_gain_db: 0.0 });
                      } else {
                        const next = { ...node.params };
                        delete next.eq;
                        onUpdateParams(node.id, next);
                      }
                    }}
                    className="accent-amber-500 rounded"
                  />
                  <span className="font-semibold text-white text-[11px]">3-Band Parametric EQ</span>
                </label>
                {node.params.eq && (
                  <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    Active
                  </span>
                )}
              </div>

              {node.params.eq && (
                <div className="space-y-2 pt-1 border-t border-[#1A222D]">
                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Low Shelf (250Hz)</span>
                    <span className="text-white">{(node.params.eq.low_gain_db ?? 0) > 0 ? `+${node.params.eq.low_gain_db}` : node.params.eq.low_gain_db ?? 0} dB</span>
                  </div>
                  <input
                    type="range"
                    min="-12"
                    max="12"
                    step="0.5"
                    value={node.params.eq.low_gain_db ?? 0}
                    onChange={(e) =>
                      handleChange('eq', {
                        ...node.params.eq,
                        low_gain_db: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />

                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Mid Peak (1kHz)</span>
                    <span className="text-white">{(node.params.eq.mid_gain_db ?? 0) > 0 ? `+${node.params.eq.mid_gain_db}` : node.params.eq.mid_gain_db ?? 0} dB</span>
                  </div>
                  <input
                    type="range"
                    min="-12"
                    max="12"
                    step="0.5"
                    value={node.params.eq.mid_gain_db ?? 0}
                    onChange={(e) =>
                      handleChange('eq', {
                        ...node.params.eq,
                        mid_gain_db: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />

                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>High Shelf (4kHz)</span>
                    <span className="text-white">{(node.params.eq.high_gain_db ?? 0) > 0 ? `+${node.params.eq.high_gain_db}` : node.params.eq.high_gain_db ?? 0} dB</span>
                  </div>
                  <input
                    type="range"
                    min="-12"
                    max="12"
                    step="0.5"
                    value={node.params.eq.high_gain_db ?? 0}
                    onChange={(e) =>
                      handleChange('eq', {
                        ...node.params.eq,
                        high_gain_db: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />
                </div>
              )}
            </div>

            {/* Dynamic Compressor */}
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-3">
              <div className="flex items-center justify-between">
                <label className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={Boolean(node.params.compressor)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        handleChange('compressor', {
                          threshold_dbfs: -18.0,
                          ratio: 3.0,
                          attack_ms: 15.0,
                          release_ms: 100.0,
                          makeup_gain_db: 2.0,
                        });
                      } else {
                        const next = { ...node.params };
                        delete next.compressor;
                        onUpdateParams(node.id, next);
                      }
                    }}
                    className="accent-amber-500 rounded"
                  />
                  <span className="font-semibold text-white text-[11px]">Dynamic Compressor</span>
                </label>
                {node.params.compressor && (
                  <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    Active
                  </span>
                )}
              </div>

              {node.params.compressor && (
                <div className="space-y-2 pt-1 border-t border-[#1A222D]">
                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Threshold</span>
                    <span className="text-white">{node.params.compressor.threshold_dbfs ?? -18} dBFS</span>
                  </div>
                  <input
                    type="range"
                    min="-36"
                    max="-6"
                    step="1"
                    value={node.params.compressor.threshold_dbfs ?? -18}
                    onChange={(e) =>
                      handleChange('compressor', {
                        ...node.params.compressor,
                        threshold_dbfs: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />

                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Ratio</span>
                    <span className="text-white">{node.params.compressor.ratio ?? 3}:1</span>
                  </div>
                  <input
                    type="range"
                    min="1.5"
                    max="8"
                    step="0.5"
                    value={node.params.compressor.ratio ?? 3}
                    onChange={(e) =>
                      handleChange('compressor', {
                        ...node.params.compressor,
                        ratio: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />

                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Makeup Gain</span>
                    <span className="text-white">+{node.params.compressor.makeup_gain_db ?? 2} dB</span>
                  </div>
                  <input
                    type="range"
                    min="0"
                    max="12"
                    step="0.5"
                    value={node.params.compressor.makeup_gain_db ?? 2}
                    onChange={(e) =>
                      handleChange('compressor', {
                        ...node.params.compressor,
                        makeup_gain_db: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />
                </div>
              )}
            </div>

            {/* Brickwall Limiter */}
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-3">
              <div className="flex items-center justify-between">
                <label className="flex items-center space-x-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={Boolean(node.params.limiter)}
                    onChange={(e) => {
                      if (e.target.checked) {
                        handleChange('limiter', { ceiling_dbfs: -0.5 });
                      } else {
                        const next = { ...node.params };
                        delete next.limiter;
                        onUpdateParams(node.id, next);
                      }
                    }}
                    className="accent-amber-500 rounded"
                  />
                  <span className="font-semibold text-white text-[11px]">Brickwall Limiter</span>
                </label>
                {node.params.limiter && (
                  <span className="text-[10px] font-mono text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20">
                    Active
                  </span>
                )}
              </div>

              {node.params.limiter && (
                <div className="space-y-2 pt-1 border-t border-[#1A222D]">
                  <div className="flex justify-between text-[10px] font-mono text-[#94A3B8]">
                    <span>Ceiling</span>
                    <span className="text-white">{node.params.limiter.ceiling_dbfs ?? -0.5} dBFS</span>
                  </div>
                  <input
                    type="range"
                    min="-6.0"
                    max="-0.1"
                    step="0.1"
                    value={node.params.limiter.ceiling_dbfs ?? -0.5}
                    onChange={(e) =>
                      handleChange('limiter', {
                        ...node.params.limiter,
                        ceiling_dbfs: parseFloat(e.target.value),
                      })
                    }
                    className="w-full accent-amber-500 cursor-pointer"
                  />
                </div>
              )}
            </div>
          </div>
        )}

        {node.node_type === 'synthesizer' && (
          <div className="space-y-4">
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Default Voice Override
              </label>
              <select
                value={node.params.voice || 'en-US-AriaNeural'}
                onChange={(e) => handleChange('voice', e.target.value)}
                className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-2 text-white text-xs focus:border-amber-500 focus:outline-none font-mono"
              >
                {voices.map((v) => (
                  <option key={v.id} value={v.id}>
                    {v.name} ({v.language})
                  </option>
                ))}
              </select>
            </div>

            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-2">
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8]">
                <span>Speaking Rate</span>
                <span className="text-white">{(node.params.speed ?? 1.0).toFixed(2)}x</span>
              </div>
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={node.params.speed ?? 1.0}
                onChange={(e) => handleChange('speed', parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>

            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-2">
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8]">
                <span>Pitch Shift</span>
                <span className="text-white">
                  {(node.params.pitch ?? 0) > 0 ? `+${node.params.pitch}` : node.params.pitch ?? 0} st
                </span>
              </div>
              <input
                type="range"
                min="-12"
                max="12"
                step="0.5"
                value={node.params.pitch ?? 0}
                onChange={(e) => handleChange('pitch', parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>
          </div>
        )}

        {node.node_type === 'audio_merge' && (
          <div className="space-y-4">
            <div className="bg-[#0B0E14] p-3 rounded border border-[#242E3D] space-y-3">
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8]">
                <span className="font-semibold text-white">Inter-Speaker Pause</span>
                <span className="text-amber-400 font-bold">{node.params.pause_ms ?? 150} ms</span>
              </div>
              <input
                type="range"
                min="0"
                max="1000"
                step="20"
                value={node.params.pause_ms ?? 150}
                onChange={(e) => handleChange('pause_ms', parseInt(e.target.value, 10))}
                className="w-full accent-amber-500 cursor-pointer"
              />
              <div className="grid grid-cols-2 gap-1.5 pt-1">
                {[
                  { label: 'Rapid Comms', ms: 80 },
                  { label: 'Conversational', ms: 150 },
                  { label: 'Podcast Studio', ms: 220 },
                  { label: 'Dramatic Pause', ms: 450 },
                ].map((p) => (
                  <button
                    key={p.ms}
                    type="button"
                    onClick={() => handleChange('pause_ms', p.ms)}
                    className={`py-1 px-2 text-[10px] font-mono rounded border transition-colors ${
                      (node.params.pause_ms ?? 150) === p.ms
                        ? 'bg-amber-500/10 border-amber-500 text-amber-400 font-bold'
                        : 'bg-[#121820] border-[#242E3D] text-[#94A3B8] hover:border-[#3B485C]'
                    }`}
                  >
                    {p.label} ({p.ms}ms)
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}

        {node.node_type === 'output_sink' && (
          <div className="space-y-3 bg-[#0B0E14] p-3 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-rose-400 font-semibold text-xs">
              <Volume2 className="w-4 h-4" />
              <span>Master Output Audio Sink</span>
            </div>
            <p className="text-[11px] text-[#94A3B8] leading-relaxed">
              Consolidates and encodes all processed multi-speaker branches into the final studio broadcast master.
            </p>
            <div className="space-y-1.5 pt-1 text-[11px] font-mono text-[#94A3B8] border-t border-[#1A222D]">
              <div className="flex justify-between">
                <span>Container:</span>
                <span className="text-white">RIFF WAV</span>
              </div>
              <div className="flex justify-between">
                <span>Encoding:</span>
                <span className="text-white">PCM Linear 16-bit</span>
              </div>
              <div className="flex justify-between">
                <span>Sample Rate:</span>
                <span className="text-white">24,000 Hz / 48,000 Hz</span>
              </div>
              <div className="flex justify-between">
                <span>Status:</span>
                <span className="text-emerald-400 font-bold">READY TO RENDER</span>
              </div>
            </div>
          </div>
        )}

        {node.node_type === 'asr_transcriber' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-sky-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <Radio className="w-4 h-4" />
              <span>ASR Audio Transcriber</span>
            </div>
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Model Engine
              </label>
              <select
                value={node.params.model || 'whisper-large-v3'}
                onChange={(e) => handleChange('model', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-sky-400 focus:outline-none"
              >
                <option value="whisper-large-v3">Whisper Large v3 (Studio Accuracy)</option>
                <option value="whisper-base">Whisper Base (Low Latency)</option>
                <option value="web-speech-native">Browser Native Web Speech</option>
              </select>
            </div>
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Language
              </label>
              <select
                value={node.params.language || 'auto'}
                onChange={(e) => handleChange('language', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-sky-400 focus:outline-none"
              >
                <option value="auto">Auto-Detect</option>
                <option value="en">English (en-US / en-GB)</option>
                <option value="es">Spanish (es-ES)</option>
                <option value="fr">French (fr-FR)</option>
                <option value="de">German (de-DE)</option>
                <option value="ja">Japanese (ja-JP)</option>
              </select>
            </div>
            <div className="flex items-center justify-between pt-1">
              <span className="text-[11px] text-[#94A3B8]">Word Timestamps</span>
              <input
                type="checkbox"
                checked={node.params.word_timestamps ?? true}
                onChange={(e) => handleChange('word_timestamps', e.target.checked)}
                className="rounded accent-sky-500 cursor-pointer"
              />
            </div>
          </div>
        )}

        {node.node_type === 'diarization' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-indigo-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <Users className="w-4 h-4" />
              <span>Multi-Speaker Diarization</span>
            </div>
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Max Speakers</span>
                <span className="text-indigo-400 font-bold">{node.params.max_speakers ?? 4}</span>
              </div>
              <input
                type="range"
                min="1"
                max="8"
                step="1"
                value={node.params.max_speakers ?? 4}
                onChange={(e) => handleChange('max_speakers', parseInt(e.target.value, 10))}
                className="w-full accent-indigo-500 cursor-pointer"
              />
            </div>
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Speaker Label Prefix
              </label>
              <input
                type="text"
                value={node.params.speaker_prefix || 'Speaker'}
                onChange={(e) => handleChange('speaker_prefix', e.target.value)}
                placeholder="Speaker"
                className="w-full bg-[#121820] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-indigo-400 focus:outline-none"
              />
            </div>
          </div>
        )}

        {node.node_type === 'document_chunker' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-emerald-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <FileText className="w-4 h-4" />
              <span>Document & Text Chunker</span>
            </div>
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Splitting Strategy
              </label>
              <select
                value={node.params.chunk_strategy || 'by_sentence'}
                onChange={(e) => handleChange('chunk_strategy', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-emerald-400 focus:outline-none"
              >
                <option value="by_sentence">By Sentence (Natural Flow)</option>
                <option value="by_paragraph">By Paragraph / Scene</option>
                <option value="by_token_limit">Token Window Budget</option>
              </select>
            </div>
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Max Chunk Size (chars)</span>
                <span className="text-emerald-400 font-bold">{node.params.max_chunk_chars ?? 600}</span>
              </div>
              <input
                type="range"
                min="100"
                max="3000"
                step="50"
                value={node.params.max_chunk_chars ?? 600}
                onChange={(e) => handleChange('max_chunk_chars', parseInt(e.target.value, 10))}
                className="w-full accent-emerald-500 cursor-pointer"
              />
            </div>
          </div>
        )}

        {node.node_type === 'audio_time_stretch' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-cyan-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <Gauge className="w-4 h-4" />
              <span>Time Stretch & Tempo</span>
            </div>
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Speed Ratio</span>
                <span className="text-cyan-400 font-bold">{(node.params.stretch_ratio ?? 1.0).toFixed(2)}x</span>
              </div>
              <input
                type="range"
                min="0.5"
                max="2.0"
                step="0.05"
                value={node.params.stretch_ratio ?? 1.0}
                onChange={(e) => handleChange('stretch_ratio', parseFloat(e.target.value))}
                className="w-full accent-cyan-500 cursor-pointer"
              />
            </div>
            <div className="flex items-center justify-between pt-1">
              <span className="text-[11px] text-[#94A3B8]">Preserve Pitch (WSOLA)</span>
              <input
                type="checkbox"
                checked={node.params.preserve_pitch ?? true}
                onChange={(e) => handleChange('preserve_pitch', e.target.checked)}
                className="rounded accent-cyan-500 cursor-pointer"
              />
            </div>
          </div>
        )}

        {node.node_type === 'audio_mux' && (
          <div className="space-y-4 bg-[#0B0E14] p-3.5 rounded-lg border border-[#242E3D]">
            <div className="flex items-center space-x-2 text-violet-400 font-semibold text-xs border-b border-[#242E3D] pb-2">
              <Layers className="w-4 h-4" />
              <span>Multi-Track Audio MUX</span>
            </div>
            <div>
              <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
                Mix Routing
              </label>
              <select
                value={node.params.mix_mode || 'stereo_master'}
                onChange={(e) => handleChange('mix_mode', e.target.value)}
                className="w-full bg-[#121820] border border-[#242E3D] rounded px-3 py-1.5 text-white text-xs font-mono focus:border-violet-400 focus:outline-none"
              >
                <option value="stereo_master">Stereo Master (Combined L/R)</option>
                <option value="stems_export">Individual Stems (Isolated)</option>
                <option value="binaural">3D Binaural Spatial</option>
              </select>
            </div>
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Ducking Attenuation</span>
                <span className="text-violet-400 font-bold">{node.params.ducking_db ?? -12} dB</span>
              </div>
              <input
                type="range"
                min="-30"
                max="0"
                step="1"
                value={node.params.ducking_db ?? -12}
                onChange={(e) => handleChange('ducking_db', parseInt(e.target.value, 10))}
                className="w-full accent-violet-500 cursor-pointer"
              />
            </div>
          </div>
        )}
      </div>

      {/* Node Actions Bottom Bar */}
      <div className="p-3 border-t border-[#242E3D] bg-[#0B0E14]/60 flex items-center justify-between text-xs font-mono">
        <div className="flex items-center space-x-2">
          {onDuplicateNode && (
            <button
              type="button"
              onClick={() => onDuplicateNode(node.id)}
              className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white transition-colors"
              title="Duplicate node (Ctrl+D)"
            >
              <Copy className="w-3.5 h-3.5 text-amber-400" />
              <span>Duplicate</span>
            </button>
          )}
          {onCopyNodeJson && (
            <button
              type="button"
              onClick={() => onCopyNodeJson(node)}
              className="flex items-center space-x-1.5 px-2.5 py-1.5 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white transition-colors"
              title="Copy JSON to clipboard"
            >
              <Clipboard className="w-3.5 h-3.5 text-sky-400" />
              <span>Copy JSON</span>
            </button>
          )}
        </div>
        {onDeleteNode && (
          <button
            type="button"
            onClick={() => onDeleteNode(node.id)}
            className="flex items-center space-x-1 px-2.5 py-1.5 rounded bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 border border-rose-500/30 transition-colors"
            title="Delete node (Del)"
          >
            <Trash2 className="w-3.5 h-3.5" />
            <span>Delete</span>
          </button>
        )}
      </div>
    </aside>
  );
};
