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
