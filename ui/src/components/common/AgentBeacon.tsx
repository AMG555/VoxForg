import React, { useState, useEffect } from 'react';
import { Bot, Radio, CheckCircle, ExternalLink, X } from 'lucide-react';

interface AgentBeaconProps {
  serverUrl?: string;
}

export const AgentBeacon: React.FC<AgentBeaconProps> = ({ serverUrl = 'http://localhost:8080' }) => {
  const [isSpeaking, setIsSpeaking] = useState<boolean>(false);
  const [activeClient, setActiveClient] = useState<string>('Claude Desktop');
  const [showFlyout, setShowFlyout] = useState<boolean>(false);
  const [mcpHealth, setMcpHealth] = useState<'connected' | 'idle' | 'offline'>('idle');

  // Listen to custom window events for MCP/agent activity or simulate active pulses
  useEffect(() => {
    const handleAgentStart = (e: any) => {
      setIsSpeaking(true);
      if (e.detail?.client) setActiveClient(e.detail.client);
    };

    const handleAgentEnd = () => {
      setIsSpeaking(false);
    };

    window.addEventListener('voxforg:agent-start', handleAgentStart);
    window.addEventListener('voxforg:agent-end', handleAgentEnd);

    // Initial MCP readiness ping
    fetch(`${serverUrl}/mcp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', method: 'ping', id: 1 }),
    })
      .then((res) => {
        if (res.ok) setMcpHealth('connected');
        else setMcpHealth('idle');
      })
      .catch(() => setMcpHealth('idle'));

    return () => {
      window.removeEventListener('voxforg:agent-start', handleAgentStart);
      window.removeEventListener('voxforg:agent-end', handleAgentEnd);
    };
  }, [serverUrl]);

  return (
    <div className="relative">
      <button
        onClick={() => setShowFlyout(!showFlyout)}
        className={`flex items-center space-x-2 px-2.5 py-1 rounded-full text-xs font-mono transition-all duration-300 border ${
          isSpeaking
            ? 'bg-amber-500/20 border-amber-500/50 text-amber-300 shadow-[0_0_12px_rgba(245,158,11,0.3)] animate-pulse'
            : 'bg-[#1A222D]/60 hover:bg-[#1A222D] border-[#242E3D] text-[#94A3B8] hover:text-white'
        }`}
        title="Model Context Protocol (MCP) Streamable Agent Status"
      >
        <span className="relative flex h-2 w-2">
          {isSpeaking && (
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-amber-400 opacity-75" />
          )}
          <span
            className={`relative inline-flex rounded-full h-2 w-2 ${
              isSpeaking
                ? 'bg-amber-400'
                : mcpHealth === 'connected'
                ? 'bg-emerald-400'
                : 'bg-[#64748B]'
            }`}
          />
        </span>

        <Bot className={`w-3.5 h-3.5 ${isSpeaking ? 'text-amber-400' : 'text-[#94A3B8]'}`} />

        <span className="text-[11px] font-medium">
          {isSpeaking ? (
            <span className="flex items-center space-x-1.5">
              <span>{activeClient}: Speaking</span>
              <span className="inline-flex space-x-0.5 items-end h-2.5">
                <span className="w-0.5 h-1.5 bg-amber-400 animate-pulse" />
                <span className="w-0.5 h-2.5 bg-amber-400 animate-bounce" />
                <span className="w-0.5 h-2 bg-amber-400 animate-pulse" />
              </span>
            </span>
          ) : (
            <span>MCP Agent Beacon</span>
          )}
        </span>
      </button>

      {/* Flyout Details Drawer */}
      {showFlyout && (
        <div className="absolute right-0 mt-2 w-80 bg-[#121820] border border-[#242E3D] rounded-xl shadow-2xl p-4 z-50 text-white animate-in fade-in zoom-in-95 duration-150">
          <div className="flex items-center justify-between pb-3 border-b border-[#242E3D]">
            <div className="flex items-center space-x-2">
              <Radio className="w-4 h-4 text-amber-400" />
              <span className="text-xs font-bold tracking-wide uppercase">Agent Voice Bridge</span>
            </div>
            <button
              onClick={() => setShowFlyout(false)}
              className="text-[#94A3B8] hover:text-white p-1 rounded hover:bg-[#1A222D]"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          </div>

          <div className="mt-3 space-y-3 text-xs">
            <div className="bg-[#0B0E14] p-2.5 rounded-lg border border-[#242E3D]">
              <div className="text-[10px] text-[#94A3B8] font-mono uppercase mb-1">MCP Endpoint</div>
              <div className="font-mono text-amber-300 text-[11px] select-all flex items-center justify-between">
                <span>{serverUrl}/mcp</span>
                <span className="text-[9px] bg-emerald-500/20 text-emerald-400 px-1.5 py-0.5 rounded border border-emerald-500/30">
                  HTTP + SSE
                </span>
              </div>
            </div>

            <div className="space-y-1.5 font-mono text-[11px]">
              <div className="flex items-center justify-between text-[#94A3B8]">
                <span>Status:</span>
                <span className="text-white flex items-center space-x-1">
                  <CheckCircle className="w-3 h-3 text-emerald-400 inline" />
                  <span>Streamable JSON-RPC 2.0</span>
                </span>
              </div>
              <div className="flex items-center justify-between text-[#94A3B8]">
                <span>Header Binding:</span>
                <span className="text-amber-400 font-mono">X-VoxForg-Client-Id</span>
              </div>
              <div className="flex items-center justify-between text-[#94A3B8]">
                <span>Supported Clients:</span>
                <span className="text-[#E2E8F0]">Cursor, Claude, Antigravity</span>
              </div>
            </div>

            <div className="pt-2 border-t border-[#242E3D] flex items-center justify-between">
              <button
                onClick={() => {
                  setIsSpeaking(!isSpeaking);
                  if (!isSpeaking) setActiveClient('Cursor IDE');
                }}
                className="text-[11px] px-2.5 py-1 rounded bg-[#1A222D] hover:bg-[#242E3D] text-[#94A3B8] hover:text-white transition-colors"
              >
                {isSpeaking ? 'Stop Test' : 'Test Speech Beacon'}
              </button>

              <a
                href={`${serverUrl}/docs`}
                target="_blank"
                rel="noreferrer"
                className="text-[11px] text-amber-400 hover:text-amber-300 flex items-center space-x-1"
              >
                <span>Docs</span>
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
