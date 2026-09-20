import React, { useState, useEffect } from 'react';
import { Navigation } from './components/Navigation';
import { WorkflowsDashboard } from './components/workflows/WorkflowsDashboard';
import { PipelineCanvas } from './components/canvas/PipelineCanvas';
import { VoiceLab } from './components/voices/VoiceLab';
import { HardwareInspector } from './components/telemetry/HardwareInspector';
import { AbTestLab } from './components/qa/AbTestLab';
import { ModelCatalog } from './components/catalog/ModelCatalog';
import { HardwareInfo, Voice } from './types';
import { api } from './services/api';
import { WorkflowStorage } from './services/workflowStorage';

export const App: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'workflows' | 'canvas' | 'voices' | 'catalog' | 'hardware' | 'qa'>('workflows');
  const [activeWorkflowId, setActiveWorkflowId] = useState<string>(() => WorkflowStorage.getActiveWorkflowId());
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

    const getCustomVoices = (): Voice[] => {
      try {
        const stored = localStorage.getItem('voxforg_custom_voices');
        return stored ? JSON.parse(stored) : [];
      } catch {
        return [];
      }
    };

    // Fetch voice catalog and merge persistent local cloned voices
    api
      .getVoices()
      .then((fetched) => {
        const custom = getCustomVoices();
        const customIds = new Set(custom.map((c) => c.id));
        setVoices([...custom, ...fetched.filter((f) => !customIds.has(f.id))]);
      })
      .catch(() => {
        const custom = getCustomVoices();
        const fallback: Voice[] = [
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
            id: 'ml-IN-SobhanaNeural',
            name: 'Sobhana (Neural - മലയാളം)',
            engine_id: 'edge-tts',
            language: 'ml-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['malayalam', 'kerala', 'expressive'],
          },
          {
            id: 'ml-IN-MidhunNeural',
            name: 'Midhun (Neural - മലയാളം)',
            engine_id: 'edge-tts',
            language: 'ml-IN',
            gender: 'male',
            sample_rate_hz: 24000,
            tags: ['malayalam', 'kerala', 'broadcast'],
          },
          {
            id: 'hi-IN-SwaraNeural',
            name: 'Swara (Neural - हिन्दी)',
            engine_id: 'edge-tts',
            language: 'hi-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['hindi', 'conversational'],
          },
          {
            id: 'hi-IN-MadhurNeural',
            name: 'Madhur (Neural - हिन्दी)',
            engine_id: 'edge-tts',
            language: 'hi-IN',
            gender: 'male',
            sample_rate_hz: 24000,
            tags: ['hindi', 'storytelling'],
          },
          {
            id: 'ta-IN-PallaviNeural',
            name: 'Pallavi (Neural - தமிழ்)',
            engine_id: 'edge-tts',
            language: 'ta-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['tamil', 'expressive'],
          },
          {
            id: 'te-IN-ShrutiNeural',
            name: 'Shruti (Neural - తెలుగు)',
            engine_id: 'edge-tts',
            language: 'te-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['telugu', 'melodic'],
          },
          {
            id: 'kn-IN-SapnaNeural',
            name: 'Sapna (Neural - ಕನ್ನಡ)',
            engine_id: 'edge-tts',
            language: 'kn-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['kannada', 'conversational'],
          },
          {
            id: 'bn-IN-TanishaaNeural',
            name: 'Tanishaa (Neural - বাংলা)',
            engine_id: 'edge-tts',
            language: 'bn-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['bengali', 'expressive'],
          },
          {
            id: 'mr-IN-AarohiNeural',
            name: 'Aarohi (Neural - मराठी)',
            engine_id: 'edge-tts',
            language: 'mr-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['marathi', 'conversational'],
          },
          {
            id: 'gu-IN-DhwaniNeural',
            name: 'Dhwani (Neural - ગુજરાતી)',
            engine_id: 'edge-tts',
            language: 'gu-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['gujarati', 'expressive'],
          },
          {
            id: 'ur-IN-GulNeural',
            name: 'Gul (Neural - اردو)',
            engine_id: 'edge-tts',
            language: 'ur-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['urdu', 'poetic'],
          },
          {
            id: 'pa-IN-VaaniNeural',
            name: 'Vaani (Neural - ਪੰਜਾਬੀ)',
            engine_id: 'edge-tts',
            language: 'pa-IN',
            gender: 'female',
            sample_rate_hz: 24000,
            tags: ['punjabi', 'energetic'],
          },
        ];
        const customIds = new Set(custom.map((c) => c.id));
        setVoices([...custom, ...fallback.filter((f) => !customIds.has(f.id))]);
      });
  }, []);

  const handleVoiceCreated = (newVoice: Voice) => {
    setVoices((prev) => {
      const updated = [newVoice, ...prev.filter((v) => v.id !== newVoice.id)];
      try {
        const cloned = updated.filter((v) => v.tags?.includes('cloned'));
        localStorage.setItem('voxforg_custom_voices', JSON.stringify(cloned));
      } catch {}
      return updated;
    });
  };

  const handleOpenWorkflow = (id: string) => {
    setActiveWorkflowId(id);
    WorkflowStorage.setActiveWorkflowId(id);
    setActiveTab('canvas');
  };

  const handleCreateWorkflow = () => {
    const newWorkflow = WorkflowStorage.createWorkflow('Untitled Studio Workflow');
    setActiveWorkflowId(newWorkflow.id);
    setActiveTab('canvas');
  };

  return (
    <div className="flex flex-col h-screen w-screen overflow-hidden bg-[#0B0E14]">
      <Navigation
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        hardware={hardware}
      />

      <main className="flex-1 flex overflow-hidden">
        {activeTab === 'workflows' && (
          <WorkflowsDashboard
            onOpenWorkflow={handleOpenWorkflow}
            onCreateWorkflow={handleCreateWorkflow}
          />
        )}
        {activeTab === 'canvas' && (
          <PipelineCanvas
            voices={voices}
            initialWorkflowId={activeWorkflowId}
            onBackToWorkflows={() => setActiveTab('workflows')}
          />
        )}
        {activeTab === 'voices' && <VoiceLab voices={voices} onVoiceCreated={handleVoiceCreated} />}
        {activeTab === 'catalog' && <ModelCatalog />}
        {activeTab === 'hardware' && <HardwareInspector hardware={hardware} onNavigateTab={setActiveTab} />}
        {activeTab === 'qa' && <AbTestLab voices={voices} />}
      </main>
    </div>
  );
};

export default App;
