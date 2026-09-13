import React, { useState, useEffect } from 'react';
import { Download, CheckCircle2, Cpu, HardDrive, Filter, AlertCircle, RefreshCw, Layers } from 'lucide-react';
import { CatalogItem } from '../../types';
import { api } from '../../services/api';

export const ModelCatalog: React.FC = () => {
  const [models, setModels] = useState<CatalogItem[]>([]);
  const [filterType, setFilterType] = useState<string>('all');
  const [installedOnly, setInstalledOnly] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [message, setMessage] = useState<{ text: string; type: 'success' | 'error' } | null>(null);

  const fetchModels = async () => {
    setLoading(true);
    try {
      const type = filterType === 'all' ? undefined : filterType;
      const data = await api.getCatalogModels(type, installedOnly);
      setModels(data);
    } catch {
      // Fallback offline curated models
      setModels([
        {
          id: 'piper-en-lessac-medium',
          name: 'Piper Lessac Medium',
          description: 'Fast, natural offline English neural voice model based on VITS / ONNX architecture',
          model_type: 'tts',
          format: 'onnx',
          size_bytes: 45 * 1024 * 1024,
          min_ram_mb: 512,
          requires_gpu: false,
          supported_languages: ['en-US'],
          status: 'Installed',
          local_path: 'models/piper-en-lessac-medium.onnx',
        },
        {
          id: 'kokoro-v0_19',
          name: 'Kokoro 82M Multilingual',
          description: 'Lightweight 82M parameter speech synthesis neural network with natural intonation',
          model_type: 'tts',
          format: 'onnx',
          size_bytes: 320 * 1024 * 1024,
          min_ram_mb: 2048,
          requires_gpu: false,
          supported_languages: ['en', 'es', 'fr', 'ja', 'zh'],
          status: 'Available',
        },
        {
          id: 'qwen3-tts-base',
          name: 'Qwen3-TTS Zero-Shot Base',
          description: 'Zero-shot voice cloning transformer conditioned on 512-dimensional speaker embeddings',
          model_type: 'tts',
          format: 'safetensors',
          size_bytes: 1200 * 1024 * 1024,
          min_ram_mb: 4096,
          requires_gpu: false,
          supported_languages: ['en', 'zh'],
          status: 'Available',
        },
        {
          id: 'whisper-base',
          name: 'Whisper Base Multilingual',
          description: 'General-purpose speech-to-text model with VAD segmentation and timestamp alignment',
          model_type: 'asr',
          format: 'onnx',
          size_bytes: 140 * 1024 * 1024,
          min_ram_mb: 1024,
          requires_gpu: false,
          supported_languages: ['en', 'es', 'fr', 'de', 'zh'],
          status: 'Installed',
          local_path: 'models/whisper-base.onnx',
        },
        {
          id: 'silero-vad',
          name: 'Silero Voice Activity Detector',
          description: 'Enterprise voice presence and pause detector neural network',
          model_type: 'vad',
          format: 'onnx',
          size_bytes: 5 * 1024 * 1024,
          min_ram_mb: 256,
          requires_gpu: false,
          supported_languages: ['*'],
          status: 'Installed',
          local_path: 'models/silero_vad.onnx',
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchModels();
  }, [filterType, installedOnly]);

  const handleInstall = async (id: string) => {
    setInstallingId(id);
    setMessage(null);
    try {
      await api.installModel(id);
      setMessage({ text: `Model '${id}' installed successfully!`, type: 'success' });
      await fetchModels();
    } catch (err: any) {
      setMessage({ text: err.message || `Failed to install '${id}'`, type: 'error' });
    } finally {
      setInstallingId(null);
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-[#0B0E14] overflow-y-auto">
      <div className="max-w-7xl w-full mx-auto p-6 space-y-6">
        {/* Header */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-[#242E3D] pb-6">
          <div>
            <h1 className="text-xl font-bold tracking-tight text-white flex items-center space-x-2">
              <Layers className="w-5 h-5 text-amber-500" />
              <span>Model Catalogue & Neural Weights</span>
            </h1>
            <p className="text-xs text-[#94A3B8] mt-1">
              Browse open-weights neural speech engines, verify host hardware requirements, and manage local models.
            </p>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={fetchModels}
              disabled={loading}
              className="flex items-center space-x-1.5 px-3 py-1.5 rounded text-xs font-mono bg-[#1A222D] text-[#94A3B8] hover:text-white border border-[#242E3D] transition-colors"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
              <span>Refresh</span>
            </button>
          </div>
        </div>

        {/* Status Message */}
        {message && (
          <div
            className={`p-3 rounded text-xs flex items-center space-x-2 ${
              message.type === 'success'
                ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                : 'bg-red-500/10 text-red-400 border border-red-500/20'
            }`}
          >
            {message.type === 'success' ? (
              <CheckCircle2 className="w-4 h-4 flex-shrink-0" />
            ) : (
              <AlertCircle className="w-4 h-4 flex-shrink-0" />
            )}
            <span>{message.text}</span>
          </div>
        )}

        {/* Filter Bar */}
        <div className="flex flex-wrap items-center justify-between gap-4 bg-[#121820] p-3 rounded-lg border border-[#242E3D]">
          <div className="flex items-center space-x-2">
            <Filter className="w-3.5 h-3.5 text-amber-500" />
            <span className="text-xs font-mono text-[#94A3B8]">Domain:</span>
            {(['all', 'tts', 'asr', 'vad'] as const).map((type) => (
              <button
                key={type}
                onClick={() => setFilterType(type)}
                className={`px-2.5 py-1 rounded text-xs font-mono uppercase transition-colors ${
                  filterType === type
                    ? 'bg-amber-500/20 text-amber-400 border border-amber-500/40'
                    : 'text-[#94A3B8] hover:text-white hover:bg-[#1A222D]'
                }`}
              >
                {type}
              </button>
            ))}
          </div>

          <label className="flex items-center space-x-2 text-xs text-[#94A3B8] cursor-pointer select-none">
            <input
              type="checkbox"
              checked={installedOnly}
              onChange={(e) => setInstalledOnly(e.target.checked)}
              className="rounded bg-[#1A222D] border-[#242E3D] text-amber-500 focus:ring-0 focus:ring-offset-0"
            />
            <span>Show installed models only</span>
          </label>
        </div>

        {/* Models Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {models.map((model) => {
            const isInstalled = model.status === 'Installed';
            const isInstalling = installingId === model.id;
            const sizeMb = Math.round(model.size_bytes / (1024 * 1024));

            return (
              <div
                key={model.id}
                className="bg-[#121820] rounded-lg border border-[#242E3D] p-5 flex flex-col justify-between hover:border-[#3B485C] transition-colors group"
              >
                <div>
                  <div className="flex items-start justify-between gap-2">
                    <div>
                      <span className="text-[10px] font-mono px-2 py-0.5 rounded uppercase font-bold bg-[#1A222D] text-amber-400 border border-[#242E3D]">
                        {model.model_type}
                      </span>
                      <span className="ml-2 text-[10px] font-mono uppercase text-[#64748B]">
                        {model.format}
                      </span>
                      <h3 className="font-bold text-sm text-white mt-2 group-hover:text-amber-400 transition-colors">
                        {model.name}
                      </h3>
                      <div className="text-[11px] font-mono text-[#64748B] mt-0.5">{model.id}</div>
                    </div>

                    {isInstalled ? (
                      <span className="flex items-center space-x-1 text-[11px] font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        <CheckCircle2 className="w-3 h-3" />
                        <span>READY</span>
                      </span>
                    ) : (
                      <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-[#1A222D] text-[#94A3B8] border border-[#242E3D]">
                        AVAILABLE
                      </span>
                    )}
                  </div>

                  <p className="text-xs text-[#94A3B8] mt-3 leading-relaxed">{model.description}</p>

                  <div className="mt-4 pt-3 border-t border-[#242E3D]/50 flex flex-wrap gap-3 text-xs text-[#64748B] font-mono">
                    <div className="flex items-center space-x-1">
                      <HardDrive className="w-3.5 h-3.5 text-amber-500" />
                      <span>{sizeMb} MB</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <Cpu className="w-3.5 h-3.5 text-amber-500" />
                      <span>Min {model.min_ram_mb} MB</span>
                    </div>
                  </div>

                  <div className="mt-3 flex flex-wrap gap-1">
                    {model.supported_languages.map((lang) => (
                      <span
                        key={lang}
                        className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-[#1A222D] text-[#94A3B8]"
                      >
                        {lang}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="mt-5 pt-4 border-t border-[#242E3D]">
                  {isInstalled ? (
                    <div className="text-[11px] font-mono text-emerald-400/80 flex items-center space-x-1">
                      <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400" />
                      <span className="truncate">Loaded: {model.local_path || 'active'}</span>
                    </div>
                  ) : (
                    <button
                      onClick={() => handleInstall(model.id)}
                      disabled={isInstalling}
                      className="w-full flex items-center justify-center space-x-2 py-2 px-3 rounded bg-amber-500/10 hover:bg-amber-500/20 text-amber-400 border border-amber-500/30 font-mono text-xs font-semibold transition-colors disabled:opacity-50"
                    >
                      {isInstalling ? (
                        <>
                          <RefreshCw className="w-3.5 h-3.5 animate-spin" />
                          <span>Downloading weights...</span>
                        </>
                      ) : (
                        <>
                          <Download className="w-3.5 h-3.5" />
                          <span>Install Weights ({sizeMb} MB)</span>
                        </>
                      )}
                    </button>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
