import React, { useState, useEffect } from 'react';
import { Navigation } from './components/Navigation';
import { PipelineCanvas } from './components/canvas/PipelineCanvas';
import { VoiceLab } from './components/voices/VoiceLab';
import { HardwareInspector } from './components/telemetry/HardwareInspector';
import { AbTestLab } from './components/qa/AbTestLab';
import { HardwareInfo, Voice } from './types';
import { api } from './services/api';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'canvas' | 'voices' | 'hardware' | 'qa'>('canvas');
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [voices, setVoices] = useState<Voice[]>([]);

  useEffect(() => {
    // Fetch readiness / hardware profile
    api.getReadiness().then(setHardware).catch(() => {
      // Offline fallback state
      setHardware({
        status: 'ready',
        hardware_tier: 'TIER_2_STANDARD',
        registered_engines: ['edge-tts', 'mock-tts'],
        arch: 'x86_64',
        os: 'Windows',
      });
    });

    // Fetch voice catalog
    api.getVoices().then(setVoices).catch(() => {
      // Offline default voices
      setVoices([
        {
          id: 'en-US-AriaNeural',
          name: 'Aria (Neural)',
          engine_id: 'edge-tts',
          language: 'en-US',
          gender: 'female',
          sample_rate_hz: 24000,
          tags: ['conversational'],
        },
        {
          id: 'en-US-GuyNeural',
          name: 'Guy (Neural)',
          engine_id: 'edge-tts',
          language: 'en-US',
          gender: 'male',
          sample_rate_hz: 24000,
          tags: ['friendly'],
        },
        {
          id: 'mock-en-female',
          name: 'Mock Female Voice',
          engine_id: 'mock-tts',
          language: 'en-US',
          gender: 'female',
          sample_rate_hz: 24000,
          tags: ['offline'],
        },
      ]);
    });
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#0B0E14]">
      <Navigation
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        hardware={hardware}
      />

      <main className="flex-1 flex overflow-hidden">
        {activeTab === 'canvas' && <PipelineCanvas voices={voices} />}
        {activeTab === 'voices' && <VoiceLab voices={voices} />}
        {activeTab === 'hardware' && <HardwareInspector hardware={hardware} />}
        {activeTab === 'qa' && <AbTestLab voices={voices} />}
      </main>
    </div>
  );
};

export default App;
