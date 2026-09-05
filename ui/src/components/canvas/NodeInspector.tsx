import React from 'react';
import { X, Sliders, Play } from 'lucide-react';
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
          <div className="space-y-4">
            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#94A3B8] mb-1">
                <span>Peak Normalization Limit</span>
                <span className="text-white">
                  {((node.params.normalize_peak || 0.95) * 100).toFixed(0)}%
                </span>
              </div>
              <input
                type="range"
                min="0.5"
                max="1.0"
                step="0.01"
                value={node.params.normalize_peak || 0.95}
                onChange={(e) => handleChange('normalize_peak', parseFloat(e.target.value))}
                className="w-full accent-amber-500 cursor-pointer"
              />
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
