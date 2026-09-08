import React from 'react';
import { X, Sliders } from 'lucide-react';
import { PipelineNode, Voice } from '../../types';

interface NodeInspectorProps {
  node: PipelineNode | null;
  voices: Voice[];
  onClose: () => void;
  onUpdateParams: (nodeId: string, params: Record<string, any>) => void;
}

export const NodeInspector: React.FC<NodeInspectorProps> = ({
  node,
  voices,
  onClose,
  onUpdateParams,
}) => {
  if (!node) return null;

  const handleChange = (key: string, value: any) => {
    onUpdateParams(node.id, {
      ...node.params,
      [key]: value,
    });
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
        <button
          onClick={onClose}
          className="p-1 rounded text-[#94A3B8] hover:text-white hover:bg-[#1A222D]"
        >
          <X className="w-4 h-4" />
        </button>
      </div>

      <div className="p-4 space-y-5 flex-1 overflow-y-auto font-sans text-xs">
        <div>
          <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
            Node Name
          </label>
          <input
            type="text"
            value={node.name}
            readOnly
            className="w-full bg-[#0B0E14] border border-[#242E3D] rounded px-3 py-1.5 text-white font-mono text-xs focus:outline-none"
          />
        </div>

        {node.node_type === 'text_input' && (
          <div>
            <label className="block text-[11px] font-mono uppercase text-[#94A3B8] mb-1">
              Script Text
            </label>
            <textarea
              rows={8}
              value={node.params.text || ''}
              onChange={(e) => handleChange('text', e.target.value)}
              className="w-full bg-[#0B0E14] border border-[#242E3D] rounded p-2.5 text-white text-xs focus:border-amber-500 focus:outline-none resize-none leading-relaxed"
              placeholder="Enter narrative or dialog (e.g. Alice: Greetings!)"
            />
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
                    {v.name} ({v.language}) - {v.engine_id}
                  </option>
                ))}
              </select>
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

        {node.node_type === 'audio_merge' && (
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Pause Between Segments</span>
                <span className="text-white">{node.params.pause_ms || 150} ms</span>
              </div>
              <input
                type="range"
                min="0"
                max="1000"
                step="25"
                value={node.params.pause_ms || 150}
                onChange={(e) => handleChange('pause_ms', parseInt(e.target.value, 10))}
                className="w-full accent-amber-500 cursor-pointer"
              />
            </div>
          </div>
        )}
      </div>
    </aside>
  );
};
