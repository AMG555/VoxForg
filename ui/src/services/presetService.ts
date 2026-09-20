import { StudioMasteringConfig } from './audioProcessor';

export interface CustomVoicePreset {
  id: string;
  name: string;
  description?: string;
  author?: string;
  version: string;
  createdAt: string;
  // Voice selection
  voiceId: string;
  voiceName?: string;
  language?: string;
  engineId?: string;
  // Acoustic parameters
  speed: number;
  pitch: number;
  sampleText?: string;
  humanizeCadence: boolean;
  // Studio Mastering DSP configuration
  studioConfig: StudioMasteringConfig;
  // Optional audio clone reference / embedding
  referenceAudioBase64?: string;
  tags?: string[];
}

export const STARTER_PRESETS: CustomVoicePreset[] = [
  {
    id: 'preset-broadcast-master',
    name: '🎙️ Studio Broadcaster Master',
    description: 'Deep analog warmth with +3.5dB chest resonance and 3:1 broadcast limiter.',
    author: 'VoxForg Audio Lab',
    version: '1.0',
    createdAt: new Date().toISOString(),
    voiceId: 'en-US-AriaNeural',
    voiceName: 'Aria (Neural)',
    language: 'en-US',
    engineId: 'edge-tts',
    speed: 1.02,
    pitch: 0.0,
    sampleText: 'Good evening. You are tuned in to VoxForg Studio. Today we review acoustic workflows and neural speech synthesis.',
    humanizeCadence: true,
    studioConfig: {
      enabled: true,
      highPassRumbleCut: true,
      lowWarmthGainDb: 3.5,
      midPresenceGainDb: 1.8,
      highAirGainDb: 2.2,
      tubeDrive: 35,
      roomReverb: 'podcast',
      reverbMix: 12,
      deEsserStrength: 40,
      compressorRatio: 3,
      makeupGainDb: 2.5,
      humanizeCadence: true,
    },
    tags: ['broadcast', 'warmth', 'podcast'],
  },
  {
    id: 'preset-crisp-narrator',
    name: '✨ Crisp Audiobook Narrator',
    description: 'High air sheen, 50% de-esser, and subtle vocal booth acoustic profile.',
    author: 'VoxForg Audio Lab',
    version: '1.0',
    createdAt: new Date().toISOString(),
    voiceId: 'en-GB-SoniaNeural',
    voiceName: 'Sonia (Neural)',
    language: 'en-GB',
    engineId: 'edge-tts',
    speed: 0.98,
    pitch: 0.5,
    sampleText: 'The ancient archive hummed with low frequency energy, each crystal storing centuries of unspoken dialogue.',
    humanizeCadence: true,
    studioConfig: {
      enabled: true,
      highPassRumbleCut: true,
      lowWarmthGainDb: 0.5,
      midPresenceGainDb: 2.8,
      highAirGainDb: 4.5,
      tubeDrive: 15,
      roomReverb: 'booth',
      reverbMix: 8,
      deEsserStrength: 50,
      compressorRatio: 2,
      makeupGainDb: 1.0,
      humanizeCadence: true,
    },
    tags: ['audiobook', 'crisp', 'narrator'],
  },
  {
    id: 'preset-tokyo-cyberpunk',
    name: '⚡ Cyberpunk Radio Voice',
    description: 'Punchy 1.2kHz vocal presence with tape drive and live broadcast room acoustic.',
    author: 'VoxForg Audio Lab',
    version: '1.0',
    createdAt: new Date().toISOString(),
    voiceId: 'ja-JP-NanamiNeural',
    voiceName: 'Nanami (Neural)',
    language: 'ja-JP',
    engineId: 'edge-tts',
    speed: 1.05,
    pitch: -0.5,
    sampleText: 'マジでヤバい！ニューラル音声パイプラインの低遅延ストリーミングが完全に同期しました。',
    humanizeCadence: true,
    studioConfig: {
      enabled: true,
      highPassRumbleCut: true,
      lowWarmthGainDb: 4.0,
      midPresenceGainDb: 3.5,
      highAirGainDb: 3.0,
      tubeDrive: 55,
      roomReverb: 'broadcast',
      reverbMix: 16,
      deEsserStrength: 35,
      compressorRatio: 4.5,
      makeupGainDb: 3.0,
      humanizeCadence: true,
    },
    tags: ['cyberpunk', 'punchy', 'japanese'],
  },
  {
    id: 'preset-malayalam-storyteller',
    name: '🎙️ Kerala Malayalam Storyteller (മലയാളം)',
    description: 'Rich Malayalam neural delivery with warm chest resonance and smooth conversational cadence.',
    author: 'VoxForg Audio Lab',
    version: '1.0',
    createdAt: new Date().toISOString(),
    voiceId: 'ml-IN-SobhanaNeural',
    voiceName: 'Sobhana (Neural - മലയാളം)',
    language: 'ml-IN',
    engineId: 'edge-tts',
    speed: 0.98,
    pitch: 0.0,
    sampleText: 'നമസ്കാരം, വോക്സ്ഫോർഗ് സ്റ്റുഡിയോയിലേക്ക് ഏവർക്കും സ്വാഗതം. ഏറ്റവും പുതിയ ന്യൂറൽ ശബ്ദ സാങ്കേതികവിദ്യയിലൂടെ മലയാളം ഇപ്പോൾ കൂടുതൽ സ്വാഭാവികമായി സംസാരിക്കുന്നു.',
    humanizeCadence: true,
    studioConfig: {
      enabled: true,
      highPassRumbleCut: true,
      lowWarmthGainDb: 2.5,
      midPresenceGainDb: 2.0,
      highAirGainDb: 1.8,
      tubeDrive: 25,
      roomReverb: 'booth',
      reverbMix: 10,
      deEsserStrength: 30,
      compressorRatio: 2.8,
      makeupGainDb: 1.8,
      humanizeCadence: true,
    },
    tags: ['malayalam', 'kerala', 'storyteller'],
  },
];

const STORAGE_KEY = 'voxforg_custom_voice_presets';

export class PresetService {
  static loadCustomPresets(): CustomVoicePreset[] {
    if (typeof window === 'undefined') return STARTER_PRESETS;
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(STARTER_PRESETS));
        return STARTER_PRESETS;
      }
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed;
      }
      return STARTER_PRESETS;
    } catch (e) {
      console.error('Failed to load custom presets from localStorage:', e);
      return STARTER_PRESETS;
    }
  }

  static saveCustomPreset(preset: CustomVoicePreset): CustomVoicePreset[] {
    const presets = this.loadCustomPresets();
    const existingIndex = presets.findIndex((p) => p.id === preset.id);
    let updated: CustomVoicePreset[];
    if (existingIndex >= 0) {
      updated = [...presets];
      updated[existingIndex] = preset;
    } else {
      updated = [preset, ...presets];
    }
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
    }
    return updated;
  }

  static deleteCustomPreset(id: string): CustomVoicePreset[] {
    const presets = this.loadCustomPresets();
    const updated = presets.filter((p) => p.id !== id);
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(updated));
    }
    return updated;
  }

  static exportPresetFile(preset: CustomVoicePreset): void {
    const dataStr =
      'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(preset, null, 2));
    const downloadAnchor = document.createElement('a');
    const safeName = preset.name
      .toLowerCase()
      .replace(/[^a-z0-9]/g, '-')
      .replace(/-+/g, '-');
    downloadAnchor.setAttribute('href', dataStr);
    downloadAnchor.setAttribute('download', `voxforg-preset-${safeName || 'custom'}.json`);
    document.body.appendChild(downloadAnchor);
    downloadAnchor.click();
    downloadAnchor.remove();
  }

  static async parsePresetFile(file: File): Promise<CustomVoicePreset> {
    const text = await file.text();
    let json: any;
    try {
      json = JSON.parse(text);
    } catch (err: any) {
      throw new Error(`Invalid JSON file format: ${err.message}`);
    }

    const data = Array.isArray(json) ? json[0] : json;

    if (!data || typeof data !== 'object') {
      throw new Error('Preset file is empty or not a valid JSON object');
    }

    const preset: CustomVoicePreset = {
      id: data.id || `preset-${Date.now()}-${Math.random().toString(36).substring(2, 7)}`,
      name: data.name || file.name.replace(/\.(json|voxpreset)$/i, ''),
      description: data.description || 'Imported custom voice preset',
      author: data.author || 'User Upload',
      version: data.version || '1.0',
      createdAt: data.createdAt || new Date().toISOString(),
      voiceId: data.voiceId || 'en-US-AriaNeural',
      voiceName: data.voiceName,
      language: data.language || 'en-US',
      engineId: data.engineId || 'edge-tts',
      speed: typeof data.speed === 'number' ? data.speed : 1.0,
      pitch: typeof data.pitch === 'number' ? data.pitch : 0.0,
      sampleText: data.sampleText || data.text,
      humanizeCadence: typeof data.humanizeCadence === 'boolean' ? data.humanizeCadence : true,
      studioConfig: {
        enabled: data.studioConfig?.enabled !== false,
        highPassRumbleCut: data.studioConfig?.highPassRumbleCut !== false,
        lowWarmthGainDb:
          typeof data.studioConfig?.lowWarmthGainDb === 'number'
            ? data.studioConfig.lowWarmthGainDb
            : 3.0,
        midPresenceGainDb:
          typeof data.studioConfig?.midPresenceGainDb === 'number'
            ? data.studioConfig.midPresenceGainDb
            : 1.5,
        highAirGainDb:
          typeof data.studioConfig?.highAirGainDb === 'number'
            ? data.studioConfig.highAirGainDb
            : 2.5,
        tubeDrive:
          typeof data.studioConfig?.tubeDrive === 'number' ? data.studioConfig.tubeDrive : 35,
        roomReverb: data.studioConfig?.roomReverb || 'booth',
        reverbMix:
          typeof data.studioConfig?.reverbMix === 'number' ? data.studioConfig.reverbMix : 12,
        deEsserStrength:
          typeof data.studioConfig?.deEsserStrength === 'number'
            ? data.studioConfig.deEsserStrength
            : 35,
        compressorRatio:
          typeof data.studioConfig?.compressorRatio === 'number'
            ? data.studioConfig.compressorRatio
            : 3,
        makeupGainDb:
          typeof data.studioConfig?.makeupGainDb === 'number'
            ? data.studioConfig.makeupGainDb
            : 2.0,
        humanizeCadence: data.studioConfig?.humanizeCadence !== false,
      },
      referenceAudioBase64: data.referenceAudioBase64,
      tags: Array.isArray(data.tags) ? data.tags : ['custom-import'],
    };

    return preset;
  }
}
