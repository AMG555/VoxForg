import {
  AbTestComparison,
  AbTestScenario,
  CatalogItem,
  CloneVoicePayload,
  HardwareInfo,
  PipelineDefinition,
  Voice,
} from '../types';

const BASE_URL = '';

const getHeaders = (customHeaders: Record<string, string> = {}): Record<string, string> => {
  const token = typeof window !== 'undefined' ? localStorage.getItem('voxforg_api_key') : null;
  const headers: Record<string, string> = { ...customHeaders };
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  return headers;
};

export const api = {
  async getHealth(): Promise<{ status: string; version: string }> {
    const res = await fetch(`${BASE_URL}/health`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch health');
    return res.json();
  },

  async getReadiness(): Promise<HardwareInfo> {
    const res = await fetch(`${BASE_URL}/health/ready`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch readiness');
    return res.json();
  },

  async getVoices(language?: string, engine?: string): Promise<Voice[]> {
    const params = new URLSearchParams();
    if (language) params.append('language', language);
    if (engine) params.append('engine', engine);

    const res = await fetch(`${BASE_URL}/v1/voices?${params.toString()}`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch voices');
    const data = await res.json();
    return data.voices;
  },

  async synthesizeDirect(options: {
    input: string;
    voice: string;
    model?: string;
    speed?: number;
    pitch?: number;
  }): Promise<Blob> {
    const res = await fetch(`${BASE_URL}/v1/audio/speech`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        model: options.model || 'edge-tts',
        input: options.input,
        voice: options.voice,
        speed: options.speed ?? 1.0,
        pitch: options.pitch ?? 0.0,
        response_format: 'wav',
      }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Synthesis failed' }));
      throw new Error(err.detail || 'Synthesis failed');
    }

    return res.blob();
  },

  async executePipeline(pipeline: PipelineDefinition, text?: string): Promise<Blob> {
    const res = await fetch(`${BASE_URL}/v1/pipeline/execute`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        pipeline,
        input_text: text,
      }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Pipeline execution failed' }));
      throw new Error(err.detail || 'Pipeline execution failed');
    }

    return res.blob();
  },

  async runAbTest(scenario: AbTestScenario): Promise<AbTestComparison> {
    const res = await fetch(`${BASE_URL}/v1/qa/ab-test`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        name: scenario.name,
        text: scenario.text,
        variant_a: {
          text: '',
          voice_id: scenario.variant_a.voice_id,
          speed: scenario.variant_a.speed ?? 1.0,
          pitch: scenario.variant_a.pitch ?? 0.0,
          format: 'wav',
        },
        variant_b: {
          text: '',
          voice_id: scenario.variant_b.voice_id,
          speed: scenario.variant_b.speed ?? 1.0,
          pitch: scenario.variant_b.pitch ?? 0.0,
          format: 'wav',
        },
      }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'A/B Test failed' }));
      throw new Error(err.detail || 'A/B Test failed');
    }

    return res.json();
  },

  async getCatalogModels(type?: string, installedOnly?: boolean): Promise<CatalogItem[]> {
    const params = new URLSearchParams();
    if (type) params.append('type', type);
    if (installedOnly) params.append('installed_only', 'true');

    const res = await fetch(`${BASE_URL}/v1/catalog/models?${params.toString()}`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch catalog models');
    const data = await res.json();
    return data.models;
  },

  async installModel(id: string): Promise<CatalogItem> {
    const res = await fetch(`${BASE_URL}/v1/catalog/models/${encodeURIComponent(id)}/install`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Model install failed' }));
      throw new Error(err.detail || 'Model install failed');
    }
    return res.json();
  },

  async cloneVoice(payload: CloneVoicePayload): Promise<any> {
    const res = await fetch(`${BASE_URL}/v1/voices/clone`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Voice cloning failed' }));
      throw new Error(err.detail || 'Voice cloning failed');
    }
    return res.json();
  },
};
