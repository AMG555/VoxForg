import { HardwareInfo, PipelineDefinition, Voice } from '../types';

const BASE_URL = '';

export const api = {
  async getHealth(): Promise<{ status: string; version: string }> {
    const res = await fetch(`${BASE_URL}/health`);
    if (!res.ok) throw new Error('Failed to fetch health');
    return res.json();
  },

  async getReadiness(): Promise<HardwareInfo> {
    const res = await fetch(`${BASE_URL}/health/ready`);
    if (!res.ok) throw new Error('Failed to fetch readiness');
    return res.json();
  },

  async getVoices(language?: string, engine?: string): Promise<Voice[]> {
    const params = new URLSearchParams();
    if (language) params.append('language', language);
    if (engine) params.append('engine', engine);

    const res = await fetch(`${BASE_URL}/v1/voices?${params.toString()}`);
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
      headers: { 'Content-Type': 'application/json' },
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
      headers: { 'Content-Type': 'application/json' },
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
};
