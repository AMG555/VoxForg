import { PipelineDefinition } from '../types';

export interface StickyNoteData {
  id: string;
  title: string;
  content: string;
  color: 'amber' | 'sky' | 'emerald' | 'purple' | 'rose';
  position: { x: number; y: number };
  width?: number;
  height?: number;
}

export interface StoredWorkflow extends PipelineDefinition {
  tags?: string[];
  category?: 'Podcast' | 'Dialogue' | 'Comms' | 'Radio' | 'Video Dubbing' | 'General';
  sticky_notes?: StickyNoteData[];
  is_active?: boolean;
}

const STORAGE_KEY = 'voxforg_workflows_v2';
const ACTIVE_ID_KEY = 'voxforg_active_workflow_id';
const OPEN_TABS_KEY = 'voxforg_open_tabs_v2';

const DEFAULT_PRESETS: StoredWorkflow[] = [
  {
    id: 'preset-narrative',
    name: 'Narrative Dialogue Pipeline',
    category: 'Dialogue',
    tags: ['fiction', 'dialogue', 'audiobook'],
    nodes: [
      {
        id: 'node-1',
        name: 'Input Script',
        node_type: 'text_input',
        params: {
          text: "Captain: We have cleared asteroid belt alpha.\nNav: Coordinates aligned, sir.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'node-2',
        name: 'Speaker Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'node-3',
        name: 'Voice Allocation',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-AriaNeural',
          speaker_map: {
            Captain: 'en-US-GuyNeural',
            Nav: 'en-US-AriaNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'node-4',
        name: 'Speech Synthesizer',
        node_type: 'synthesizer',
        params: {},
        position: { x: 1020, y: 120 },
      },
      {
        id: 'node-5',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 180 },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'node-6',
        name: 'Master Output',
        node_type: 'output_sink',
        params: {},
        position: { x: 1660, y: 120 },
      },
    ],
    edges: [
      { id: 'e1', from_node: 'node-1', to_node: 'node-2' },
      { id: 'e2', from_node: 'node-2', to_node: 'node-3' },
      { id: 'e3', from_node: 'node-3', to_node: 'node-4' },
      { id: 'e4', from_node: 'node-4', to_node: 'node-5' },
      { id: 'e5', from_node: 'node-5', to_node: 'node-6' },
    ],
    sticky_notes: [
      {
        id: 'sn-1',
        title: 'Dialogue Flow',
        content: 'Parses speaker names automatically and crossfades speech segments seamlessly.',
        color: 'sky',
        position: { x: 60, y: 320 },
        width: 280,
      },
    ],
    created_at: new Date('2026-01-01').toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'preset-podcast',
    name: 'Multi-Voice Studio Podcast',
    category: 'Podcast',
    tags: ['podcast', 'interview', 'mastering'],
    nodes: [
      {
        id: 'p-1',
        name: 'Podcast Interview Script',
        node_type: 'text_input',
        params: {
          text: "Host: Welcome back to Quantum Wave! Today we dive into multi-voice neural speech synthesis in high-concurrency environments.\nGuest: Thanks for having me, Alex! Combining distinct voices on a single directed canvas is a massive leap for studio production.\nHost: Absolutely. With inter-speaker crossfades and dynamic audio mastering, creators can build full podcast episodes in seconds.",
        },
        position: { x: 60, y: 120 },
      },
      {
        id: 'p-2',
        name: 'Speaker Script Parser',
        node_type: 'speaker_parser',
        params: {},
        position: { x: 380, y: 120 },
      },
      {
        id: 'p-3',
        name: 'Podcast Voice Assigner',
        node_type: 'voice_assigner',
        params: {
          default_voice: 'en-US-JennyNeural',
          speaker_map: {
            Host: 'en-US-JennyNeural',
            Guest: 'en-US-GuyNeural',
          },
        },
        position: { x: 700, y: 120 },
      },
      {
        id: 'p-4',
        name: 'Neural Synthesizer',
        node_type: 'synthesizer',
        params: { speed: 1.0, pitch: 0.0 },
        position: { x: 1020, y: 120 },
      },
      {
        id: 'p-5',
        name: 'Studio DSP Mastering',
        node_type: 'audio_filter',
        params: {
          trim_silence: true,
          enable_eq: true,
          eq_low_gain_db: 1.5,
          eq_mid_gain_db: 2.0,
          eq_high_gain_db: 1.5,
          enable_compressor: true,
          compressor_threshold_db: -16.0,
          compressor_ratio: 3.0,
          normalize: true,
        },
        position: { x: 1340, y: 120 },
      },
      {
        id: 'p-6',
        name: 'Master Audio Merge',
        node_type: 'audio_merge',
        params: { pause_ms: 160 },
        position: { x: 1660, y: 120 },
      },
      {
        id: 'p-7',
        name: 'Broadcast Master Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 1980, y: 120 },
      },
    ],
    edges: [
      { id: 'pe1', from_node: 'p-1', to_node: 'p-2' },
      { id: 'pe2', from_node: 'p-2', to_node: 'p-3' },
      { id: 'pe3', from_node: 'p-3', to_node: 'p-4' },
      { id: 'pe4', from_node: 'p-4', to_node: 'p-5' },
      { id: 'pe5', from_node: 'p-5', to_node: 'p-6' },
      { id: 'pe6', from_node: 'p-6', to_node: 'p-7' },
    ],
    sticky_notes: [
      {
        id: 'sn-2',
        title: 'Studio Mastering Node',
        content: 'Applies parametric EQ, multi-band compression and true-peak brickwall limiting before final audio merge.',
        color: 'emerald',
        position: { x: 1340, y: 320 },
        width: 300,
      },
    ],
    created_at: new Date('2026-01-01').toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'preset-character-dialogue',
    name: 'Multi-Character Voice DAG',
    category: 'Dialogue',
    tags: ['gaming', 'animation', 'character'],
    nodes: [
      {
        id: 'char-1',
        name: 'Captain Miller',
        node_type: 'character_voice',
        params: {
          character_name: 'Captain Miller',
          voice_id: 'en-US-GuyNeural',
          text: 'All flight decks, initiate pre-burn diagnostics. We break orbit in forty seconds.',
          speed: 0.98,
          pitch: 0,
        },
        position: { x: 60, y: 60 },
      },
      {
        id: 'char-2',
        name: 'Science Officer Nova',
        node_type: 'character_voice',
        params: {
          character_name: 'Science Officer Nova',
          voice_id: 'en-US-AriaNeural',
          text: 'Telemetry confirmed, Captain. Gravitational sensors are nominal and corridor is clear.',
          speed: 1.05,
          pitch: 1,
        },
        position: { x: 60, y: 260 },
      },
      {
        id: 'char-3',
        name: 'AI Co-pilot Vector',
        node_type: 'character_voice',
        params: {
          character_name: 'AI Co-pilot Vector',
          voice_id: 'en-US-JennyNeural',
          text: 'Neural jump drive primed. Synchronizing audio channels to master broadcast.',
          speed: 1.1,
          pitch: 2,
        },
        position: { x: 60, y: 460 },
      },
      {
        id: 'char-synth',
        name: 'Neural Synthesizer',
        node_type: 'synthesizer',
        params: { speed: 1.0, pitch: 0.0 },
        position: { x: 420, y: 260 },
      },
      {
        id: 'char-filter',
        name: 'Studio DSP Mastering',
        node_type: 'audio_filter',
        params: {
          trim_silence: true,
          enable_eq: true,
          eq_low_gain_db: 1.0,
          eq_high_gain_db: 2.0,
          enable_compressor: true,
          compressor_threshold_db: -16.0,
          normalize: true,
        },
        position: { x: 740, y: 260 },
      },
      {
        id: 'char-merge',
        name: 'Audio Merge & Crossfade',
        node_type: 'audio_merge',
        params: { pause_ms: 220 },
        position: { x: 1060, y: 260 },
      },
      {
        id: 'char-sink',
        name: 'Master Broadcast Sink',
        node_type: 'output_sink',
        params: {},
        position: { x: 1380, y: 260 },
      },
    ],
    edges: [
      { id: 'che-1', from_node: 'char-1', to_node: 'char-synth' },
      { id: 'che-2', from_node: 'char-2', to_node: 'char-synth' },
      { id: 'che-3', from_node: 'char-3', to_node: 'char-synth' },
      { id: 'che-4', from_node: 'char-synth', to_node: 'char-filter' },
      { id: 'che-5', from_node: 'char-filter', to_node: 'char-merge' },
      { id: 'che-6', from_node: 'char-merge', to_node: 'char-sink' },
    ],
    sticky_notes: [
      {
        id: 'sn-3',
        title: 'Voice Character Cast',
        content: 'Individual character lines converge into the neural synthesizer and dynamic DSP chain.',
        color: 'purple',
        position: { x: 60, y: 640 },
        width: 320,
      },
    ],
    created_at: new Date('2026-01-01').toISOString(),
    updated_at: new Date().toISOString(),
  },
];

export class WorkflowStorage {
  static getWorkflows(): StoredWorkflow[] {
    if (typeof window === 'undefined') return DEFAULT_PRESETS;
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        this.saveWorkflows(DEFAULT_PRESETS);
        return DEFAULT_PRESETS;
      }
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
      return DEFAULT_PRESETS;
    } catch {
      return DEFAULT_PRESETS;
    }
  }

  static saveWorkflows(workflows: StoredWorkflow[]) {
    if (typeof window === 'undefined') return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(workflows));
    } catch (e) {
      console.error('Failed saving workflows to localStorage', e);
    }
  }

  static getActiveWorkflowId(): string {
    if (typeof window === 'undefined') return DEFAULT_PRESETS[0].id;
    return localStorage.getItem(ACTIVE_ID_KEY) || DEFAULT_PRESETS[0].id;
  }

  static setActiveWorkflowId(id: string) {
    if (typeof window === 'undefined') return;
    localStorage.setItem(ACTIVE_ID_KEY, id);
    this.addOpenTab(id);
  }

  static getActiveWorkflow(): StoredWorkflow {
    const list = this.getWorkflows();
    const activeId = this.getActiveWorkflowId();
    return list.find((w) => w.id === activeId) || list[0] || DEFAULT_PRESETS[0];
  }

  static saveWorkflow(workflow: StoredWorkflow): StoredWorkflow {
    const list = this.getWorkflows();
    const updated = {
      ...workflow,
      updated_at: new Date().toISOString(),
    };
    const index = list.findIndex((w) => w.id === workflow.id);
    if (index >= 0) {
      list[index] = updated;
    } else {
      list.unshift(updated);
    }
    this.saveWorkflows(list);
    return updated;
  }

  static createWorkflow(name = 'Untitled Workflow', category: StoredWorkflow['category'] = 'General'): StoredWorkflow {
    const freshId = `wf-${Date.now()}`;
    const initialNodeId = `node-${Date.now()}`;
    const newWorkflow: StoredWorkflow = {
      id: freshId,
      name,
      category,
      tags: ['custom'],
      nodes: [
        {
          id: initialNodeId,
          name: 'Script Input',
          node_type: 'text_input',
          params: { text: 'Host: Welcome to the studio!\nGuest: Great to be here.' },
          position: { x: 80, y: 120 },
        },
      ],
      edges: [],
      sticky_notes: [],
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    const list = this.getWorkflows();
    list.unshift(newWorkflow);
    this.saveWorkflows(list);
    this.setActiveWorkflowId(freshId);
    return newWorkflow;
  }

  static duplicateWorkflow(id: string): StoredWorkflow {
    const list = this.getWorkflows();
    const source = list.find((w) => w.id === id) || this.getActiveWorkflow();
    const freshId = `wf-${Date.now()}`;
    const duplicated: StoredWorkflow = {
      ...JSON.parse(JSON.stringify(source)),
      id: freshId,
      name: `${source.name} (Copy)`,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    list.unshift(duplicated);
    this.saveWorkflows(list);
    this.setActiveWorkflowId(freshId);
    return duplicated;
  }

  static renameWorkflow(id: string, name: string): StoredWorkflow | null {
    const list = this.getWorkflows();
    const target = list.find((w) => w.id === id);
    if (!target) return null;
    target.name = name.trim();
    target.updated_at = new Date().toISOString();
    this.saveWorkflows(list);
    return target;
  }

  static deleteWorkflow(id: string): boolean {
    let list = this.getWorkflows();
    if (list.length <= 1) return false; // Prevent deleting the only workflow
    list = list.filter((w) => w.id !== id);
    this.saveWorkflows(list);
    this.removeOpenTab(id);
    if (this.getActiveWorkflowId() === id) {
      this.setActiveWorkflowId(list[0].id);
    }
    return true;
  }

  // Open Tabs State (like in n8n / VSCode)
  static getOpenTabs(): string[] {
    if (typeof window === 'undefined') return [DEFAULT_PRESETS[0].id];
    try {
      const raw = localStorage.getItem(OPEN_TABS_KEY);
      if (!raw) return [this.getActiveWorkflowId()];
      const tabs = JSON.parse(raw);
      return Array.isArray(tabs) && tabs.length > 0 ? tabs : [this.getActiveWorkflowId()];
    } catch {
      return [this.getActiveWorkflowId()];
    }
  }

  static saveOpenTabs(tabs: string[]) {
    if (typeof window === 'undefined') return;
    localStorage.setItem(OPEN_TABS_KEY, JSON.stringify(tabs));
  }

  static addOpenTab(id: string) {
    const tabs = this.getOpenTabs();
    if (!tabs.includes(id)) {
      tabs.push(id);
      this.saveOpenTabs(tabs);
    }
  }

  static removeOpenTab(id: string) {
    let tabs = this.getOpenTabs().filter((t) => t !== id);
    if (tabs.length === 0) {
      tabs = [this.getWorkflows()[0]?.id || DEFAULT_PRESETS[0].id];
    }
    this.saveOpenTabs(tabs);
  }

  static exportWorkflowJson(workflow: StoredWorkflow) {
    const dataStr = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(workflow, null, 2));
    const downloadAnchor = document.createElement('a');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `${workflow.name.toLowerCase().replace(/\s+/g, '-')}.voxforg.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  }
}
