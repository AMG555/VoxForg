export type NodeType =
  | 'text_input'
  | 'speaker_parser'
  | 'voice_assigner'
  | 'synthesizer'
  | 'audio_filter'
  | 'audio_merge'
  | 'output_sink';

export interface PipelineNode {
  id: string;
  name: string;
  node_type: NodeType;
  params: Record<string, any>;
  position?: { x: number; y: number };
}

export interface PipelineEdge {
  id: string;
  from_node: string;
  to_node: string;
  from_port?: string;
  to_port?: string;
}

export interface PipelineDefinition {
  id: string;
  name: string;
  description?: string;
  nodes: PipelineNode[];
  edges: PipelineEdge[];
  created_at: string;
  updated_at: string;
}

export interface Voice {
  id: string;
  name: string;
  engine_id: string;
  language: string;
  gender: 'male' | 'female' | 'neutral';
  sample_rate_hz: number;
  tags: string[];
  description?: string;
}

export interface HardwareInfo {
  status: string;
  hardware_tier: string;
  registered_engines: string[];
  arch: string;
  os: string;
}

export interface AudioQualityMetrics {
  duration_seconds: number;
  sample_rate: number;
  channels: number;
  total_samples: number;
  peak_amplitude: number;
  peak_dbfs: number;
  rms_amplitude: number;
  rms_dbfs: number;
  clipping_samples_count: number;
  is_silent: boolean;
}

export interface VariantResult {
  engine_id: string;
  voice_id: string;
  latency_ms: number;
  audio_duration_seconds: number;
  realtime_factor: number;
  metrics: AudioQualityMetrics;
}

export interface AbTestComparison {
  scenario_name: string;
  variant_a: VariantResult;
  variant_b: VariantResult;
  latency_delta_ms: number;
  speedup_ratio: number;
  rms_delta_db: number;
  faster_variant: string;
  recommended_variant: string;
  summary: string;
}

export interface AbTestScenario {
  name: string;
  text: string;
  variant_a: {
    voice_id: string;
    speed?: number;
    pitch?: number;
  };
  variant_b: {
    voice_id: string;
    speed?: number;
    pitch?: number;
  };
}

export interface CatalogItem {
  id: string;
  name: string;
  description: string;
  model_type: 'tts' | 'asr' | 'vad' | 'diarizer';
  format: string;
  size_bytes: number;
  min_ram_mb: number;
  requires_gpu: boolean;
  supported_languages: string[];
  status: 'Available' | 'Downloading' | 'Installed' | 'Error';
  installed_at?: string;
  local_path?: string;
}

export interface CloneVoicePayload {
  name: string;
  engine_id?: string;
  reference_audio_base64?: string;
  reference_transcript?: string;
  language?: string;
  gender?: 'male' | 'female' | 'neutral';
  description?: string;
}
