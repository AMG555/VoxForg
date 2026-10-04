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
    instruct?: string;
    crossfade_ms?: number;
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
        instruct: options.instruct,
        crossfade_ms: options.crossfade_ms,
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

  async uninstallModel(id: string): Promise<CatalogItem> {
    const res = await fetch(`${BASE_URL}/v1/catalog/models/${encodeURIComponent(id)}`, {
      method: 'DELETE',
      headers: getHeaders(),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Model uninstall failed' }));
      throw new Error(err.detail || 'Model uninstall failed');
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

  async transcribeAudio(audioBase64: string, model = 'whisper-base', language?: string): Promise<{ text: string; duration_seconds?: number }> {
    const res = await fetch(`${BASE_URL}/v1/audio/transcriptions`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        audio_base64: audioBase64,
        model,
        language: language || undefined,
        response_format: 'json',
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Audio transcription failed' }));
      throw new Error(err.detail || 'Audio transcription failed');
    }
    return res.json();
  },

  async refineText(
    text: string,
    options: { remove_fillers?: boolean; fix_repetitions?: boolean; punctuation_pass?: boolean } = {}
  ): Promise<{ refined_text: string; removed_fillers: number; cleaned_repetitions: number }> {
    const res = await fetch(`${BASE_URL}/v1/audio/refine`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        text,
        remove_fillers: options.remove_fillers ?? true,
        fix_repetitions: options.fix_repetitions ?? true,
        punctuation_pass: options.punctuation_pass ?? true,
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Text refinement failed' }));
      throw new Error(err.detail || 'Text refinement failed');
    }
    return res.json();
  },

  async speechToSpeech(payload: {
    audio_base64: string;
    target_voice: string;
    speed?: number;
    preserve_tempo?: boolean;
  }): Promise<Blob> {
    const res = await fetch(`${BASE_URL}/v1/audio/speech-to-speech`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({
        audio_base64: payload.audio_base64,
        target_voice: payload.target_voice,
        speed: payload.speed ?? 1.0,
        preserve_tempo: payload.preserve_tempo ?? true,
        response_format: 'wav',
      }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Speech-to-Speech conversion failed' }));
      throw new Error(err.detail || 'Speech-to-Speech conversion failed');
    }
    return res.blob();
  },

  async assessAudio(audio_base64: string): Promise<{
    duration_seconds: number;
    sample_rate: number;
    channels: number;
    peak_dbfs: number;
    rms_dbfs: number;
    snr_estimate_db: number;
    clipping_detected: boolean;
    is_silent: boolean;
    clarity_rating: 'excellent' | 'good' | 'fair' | 'noisy';
    recommendations: string[];
  }> {
    const res = await fetch(`${BASE_URL}/v1/voices/assess`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ audio_base64 }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Audio assessment failed' }));
      throw new Error(err.detail || 'Audio assessment failed');
    }
    return res.json();
  },

  async designVoice(payload: {
    prompt: string;
    name?: string;
    gender?: 'male' | 'female' | 'neutral';
    language?: string;
    director?: {
      energy: number;
      emotion: number;
      pace: number;
      intimacy: number;
      formality: number;
    };
  }): Promise<Voice> {
    const res = await fetch(`${BASE_URL}/v1/voices/design`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify(payload),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Voice design failed' }));
      throw new Error(err.detail || 'Voice design failed');
    }
    return res.json();
  },

  async exportPersona(voiceId: string): Promise<any> {
    const res = await fetch(`${BASE_URL}/v1/voices/${encodeURIComponent(voiceId)}/export`, {
      headers: getHeaders(),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Failed to export persona bundle' }));
      throw new Error(err.detail || 'Failed to export persona bundle');
    }
    return res.json();
  },

  async importPersona(bundle: any): Promise<Voice> {
    const res = await fetch(`${BASE_URL}/v1/voices/import`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify(bundle),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ detail: 'Failed to import persona bundle' }));
      throw new Error(err.detail || 'Failed to import persona bundle');
    }
    return res.json();
  },

  async getDictionary(): Promise<{ count: number; entries: Array<{ term: string; replacement: string; note?: string }> }> {
    const res = await fetch(`${BASE_URL}/v1/pronunciation/dictionary`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch pronunciation dictionary');
    return res.json();
  },

  async upsertDictionary(entry: { term: string; replacement: string; note?: string }): Promise<any> {
    const res = await fetch(`${BASE_URL}/v1/pronunciation/dictionary`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify(entry),
    });
    if (!res.ok) throw new Error('Failed to save pronunciation entry');
    return res.json();
  },

  async deleteDictionary(term: string): Promise<void> {
    const res = await fetch(`${BASE_URL}/v1/pronunciation/dictionary/${encodeURIComponent(term)}`, {
      method: 'DELETE',
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to delete pronunciation entry');
  },

  async applyPronunciation(text: string, language?: string): Promise<{ original: string; processed: string; replacements_count: number }> {
    const res = await fetch(`${BASE_URL}/v1/pronunciation/apply`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ text, language }),
    });
    if (!res.ok) throw new Error('Failed to apply pronunciation rules');
    return res.json();
  },

  async embedWatermark(audioBase64: string, payload?: number, strength?: number): Promise<{ audio_base64: string; payload: number; repetitions: number; duration_seconds: number }> {
    const res = await fetch(`${BASE_URL}/v1/audio/watermark`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ audio_base64: audioBase64, payload, strength }),
    });
    if (!res.ok) throw new Error('Failed to embed audio watermark');
    return res.json();
  },

  async verifyWatermark(audioBase64: string): Promise<{ is_detected: boolean; confidence: number; payload?: number; signature_match: boolean; sample_rate: number; duration_seconds: number }> {
    const res = await fetch(`${BASE_URL}/v1/audio/verify-watermark`, {
      method: 'POST',
      headers: getHeaders({ 'Content-Type': 'application/json' }),
      body: JSON.stringify({ audio_base64: audioBase64 }),
    });
    if (!res.ok) throw new Error('Failed to verify audio watermark');
    return res.json();
  },

  async getJobCheckpoints(): Promise<{ jobs: any[] }> {
    const res = await fetch(`${BASE_URL}/v1/jobs/checkpoints`, {
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to fetch job checkpoints');
    return res.json();
  },

  async resumeJob(jobId: string): Promise<{ manifest: any; remaining_indices: number[]; is_finished: boolean }> {
    const res = await fetch(`${BASE_URL}/v1/jobs/${encodeURIComponent(jobId)}/resume`, {
      method: 'POST',
      headers: getHeaders(),
    });
    if (!res.ok) throw new Error('Failed to resume job checkpoint');
    return res.json();
  },
};
