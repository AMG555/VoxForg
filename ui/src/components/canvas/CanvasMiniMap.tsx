import React, { useRef } from 'react';
import { PipelineNode } from '../../types';
import { MapPin, Minus } from 'lucide-react';

interface CanvasMiniMapProps {
  nodes: PipelineNode[];
  pan: { x: number; y: number };
  zoom: number;
  containerWidth: number;
  containerHeight: number;
  onNavigate: (newPan: { x: number; y: number }) => void;
  isOpen: boolean;
  onToggleOpen: () => void;
}

export const CanvasMiniMap: React.FC<CanvasMiniMapProps> = ({
  nodes,
  pan,
  zoom,
  containerWidth,
  containerHeight,
  onNavigate,
  isOpen,
  onToggleOpen,
}) => {
  const mapRef = useRef<HTMLDivElement>(null);

  if (!isOpen) {
    return (
      <button
        onClick={onToggleOpen}
        className="p-2 rounded-xl bg-[#121820]/90 backdrop-blur-md border border-[#242E3D] hover:border-amber-500/50 text-[#94A3B8] hover:text-white shadow-xl transition-all"
        title="Show Canvas Mini-Map"
      >
        <MapPin className="w-4 h-4 text-amber-400" />
      </button>
    );
  }

  // Calculate bounding box of all nodes
  const mapW = 180;
  const mapH = 110;
  const padding = 200;

  let minX = 0;
  let maxX = 2000;
  let minY = 0;
  let maxY = 1200;

  if (nodes.length > 0) {
    const xs = nodes.map((n) => n.position?.x || 100);
    const ys = nodes.map((n) => n.position?.y || 100);
    minX = Math.min(...xs) - padding;
    maxX = Math.max(...xs) + 260 + padding;
    minY = Math.min(...ys) - padding;
    maxY = Math.max(...ys) + 140 + padding;
  }

  const worldWidth = Math.max(maxX - minX, 1000);
  const worldHeight = Math.max(maxY - minY, 700);

  const scaleX = mapW / worldWidth;
  const scaleY = mapH / worldHeight;
  const mapScale = Math.min(scaleX, scaleY);

  // Viewport bounds in world space
  const viewWorldX = -pan.x / zoom;
  const viewWorldY = -pan.y / zoom;
  const viewWorldW = containerWidth / zoom;
  const viewWorldH = containerHeight / zoom;

  // Viewport in mini-map space
  const vpLeft = Math.max(0, (viewWorldX - minX) * mapScale);
  const vpTop = Math.max(0, (viewWorldY - minY) * mapScale);
  const vpWidth = Math.min(mapW, viewWorldW * mapScale);
  const vpHeight = Math.min(mapH, viewWorldH * mapScale);

  const handleMapPointerDown = (e: React.PointerEvent) => {
    e.stopPropagation();
    if (!mapRef.current) return;
    const rect = mapRef.current.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    // Convert to world coordinates
    const targetWorldX = minX + clickX / mapScale;
    const targetWorldY = minY + clickY / mapScale;

    // Center viewport at target
    const newPanX = containerWidth / 2 - targetWorldX * zoom;
    const newPanY = containerHeight / 2 - targetWorldY * zoom;

    onNavigate({ x: Math.round(newPanX), y: Math.round(newPanY) });
  };

  return (
    <div className="bg-[#121820]/95 backdrop-blur-md border border-[#242E3D] rounded-xl shadow-2xl p-2 select-none text-xs font-mono">
      <div className="flex items-center justify-between pb-1.5 mb-1.5 border-b border-[#242E3D]/80 text-[10px] text-[#64748B]">
        <div className="flex items-center space-x-1.5 text-amber-400 font-semibold">
          <MapPin className="w-3 h-3" />
          <span>Mini-Map</span>
        </div>
        <button
          onClick={onToggleOpen}
          className="text-[#64748B] hover:text-white p-0.5"
          title="Hide Mini-map"
        >
          <Minus className="w-3 h-3" />
        </button>
      </div>

      <div
        ref={mapRef}
        onPointerDown={handleMapPointerDown}
        className="relative bg-[#0B0E14] border border-[#242E3D] rounded-lg overflow-hidden cursor-crosshair"
        style={{ width: `${mapW}px`, height: `${mapH}px` }}
      >
        {/* Render node dots */}
        {nodes.map((n) => {
          const nx = ((n.position?.x || 100) - minX) * mapScale;
          const ny = ((n.position?.y || 100) - minY) * mapScale;
          const nw = Math.max(8, 256 * mapScale);
          const nh = Math.max(5, 120 * mapScale);

          return (
            <div
              key={n.id}
              className="absolute rounded bg-amber-500/70 border border-amber-400/90 pointer-events-none"
              style={{
                left: `${nx}px`,
                top: `${ny}px`,
                width: `${nw}px`,
                height: `${nh}px`,
              }}
            />
          );
        })}

        {/* Viewport rectangle */}
        <div
          className="absolute border border-sky-400 bg-sky-400/20 rounded pointer-events-none transition-all duration-75"
          style={{
            left: `${vpLeft}px`,
            top: `${vpTop}px`,
            width: `${Math.max(12, vpWidth)}px`,
            height: `${Math.max(8, vpHeight)}px`,
          }}
        />
      </div>
    </div>
  );
};
