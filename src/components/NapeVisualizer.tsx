import React, { useState } from 'react';
import { DeviceProfile } from '../types';
import { Layers, RotateCw } from 'lucide-react';

interface NapeVisualizerProps {
  device: DeviceProfile;
  activeLayer: number;
  onSelectLayer: (layerId: number) => void;
  onRefreshFromHardware?: () => void;
  onUpdateAngle: (layerId: number, angle: number) => void;
  showAdvancedControls?: boolean;
}

export const NapeVisualizer: React.FC<NapeVisualizerProps> = ({
  device,
  activeLayer,
  onSelectLayer,
  onUpdateAngle,
  showAdvancedControls = false,
}) => {
  const [selectedButtonId, setSelectedButtonId] = useState<number>(1);
  const [isSimulatingClick, setIsSimulatingClick] = useState<number | null>(null);
  const [viewLayer, setViewLayer] = useState<number | null>(null);

  const currentPreviewLayer = viewLayer !== null ? viewLayer : activeLayer;
  const mappings = device?.button_mappings?.[currentPreviewLayer] || [];
  const getButtonMapping = (id: number) => {
    return (
      mappings.find((m) => m.button_id === id) || {
        button_id: id,
        name: `ボタン ${id}`,
        action_type: 'key',
        key_code: '',
        description: `ボタン ${id}`,
      }
    );
  };

  const handleSimulateClick = (buttonId: number) => {
    setIsSimulatingClick(buttonId);
    setTimeout(() => setIsSimulatingClick(null), 300);
  };

  const currentAngle = device?.layer_octashift_angles?.[currentPreviewLayer] ?? device?.octashift_angle ?? 0;

  // CANVAS & HARDWARE LAYOUT GEOMETRY
  const CX = 440;
  const CY = 290;
  const rad = (currentAngle * Math.PI) / 180;
  const cos = Math.cos(rad);
  const sin = Math.sin(rad);

  // Precise physical button anchor points (relative to center) and local push vectors
  const anchorSpecs = [
    { id: 5, dx: -52, dy: -184, pushDx: -135, pushDy: 0 },   // 03 (Top-Left G3)
    { id: 6, dx: 52, dy: -184, pushDx: 135, pushDy: 0 },    // 04 (Top-Right G4)
    { id: 7, dx: 72, dy: 0, pushDx: 145, pushDy: 0 },       // Scroll Ring (Right Middle)
    { id: 3, dx: -52, dy: 154, pushDx: -135, pushDy: 0 },    // 01 (Bottom-Left G1)
    { id: 4, dx: 52, dy: 154, pushDx: 135, pushDy: 0 },     // 02 (Bottom-Right G2)
    { id: 1, dx: -35, dy: 202, pushDx: -135, pushDy: 35 },   // M1 (Bottommost Left)
    { id: 2, dx: 35, dy: 202, pushDx: 135, pushDy: 35 },    // M2 (Bottommost Right)
  ];

  const calculatedAnchors = anchorSpecs.map((item) => {
    // Rotated button anchor point on hardware body (screen coords)
    const bx = CX + item.dx * cos - item.dy * sin;
    const by = CY + item.dx * sin + item.dy * cos;

    // Rotated badge displacement vector
    const rx = item.pushDx * cos - item.pushDy * sin;
    const ry = item.pushDx * sin + item.pushDy * cos;

    // Final badge center position in screen coords
    const lx = bx + rx;
    const ly = by + ry;

    const isSelected = selectedButtonId === item.id || (item.id === 7 && (selectedButtonId === 7 || selectedButtonId === 8));

    return {
      ...item,
      bx,
      by,
      lx,
      ly,
      isSelected,
    };
  });

  return (
    <div className="space-y-4">
      {/* 1. LARGE & WIDE LAYER SWITCHER TABS */}
      <div className="bg-slate-900/60 border border-slate-800/80 rounded-2xl p-3 shadow-xl backdrop-blur-md">
        <div className="flex items-center justify-between px-1 pb-2 mb-2 border-b border-slate-800/60">
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-indigo-400" />
            <span className="text-xs font-bold text-slate-300 tracking-wide">レイヤー切り替え</span>
          </div>
          <span className="text-[11px] text-slate-500 font-medium">
            表示中: <span className="font-mono text-indigo-400 font-bold">L{currentPreviewLayer}</span>
            {currentPreviewLayer === activeLayer && (
              <span className="ml-1.5 inline-flex items-center gap-1 text-[10px] text-emerald-400 bg-emerald-950/60 border border-emerald-800/60 px-1.5 py-0.5 rounded-full font-mono">
                <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                ACTIVE
              </span>
            )}
          </span>
        </div>

        {/* 8 Wide Tabs Grid */}
        <div className="grid grid-cols-4 sm:grid-cols-8 gap-2">
          {Array.from({ length: 8 }).map((_, layerId) => {
            const isPreview = currentPreviewLayer === layerId;
            const isHardwareActive = activeLayer === layerId;
            const layerAngle = device?.layer_octashift_angles?.[layerId] ?? (layerId * 45);

            return (
              <button
                key={layerId}
                onClick={() => {
                  setViewLayer(layerId);
                  onSelectLayer(layerId);
                }}
                className={`relative flex flex-col items-center justify-center py-3 px-2 rounded-xl border transition-all duration-200 cursor-pointer group ${
                  isPreview
                    ? 'bg-gradient-to-b from-indigo-600 to-indigo-700 text-white border-indigo-400/80 shadow-lg shadow-indigo-500/25 ring-2 ring-indigo-400/40 scale-[1.02] z-10'
                    : 'bg-slate-900/90 text-slate-400 hover:text-slate-200 hover:bg-slate-800/80 border-slate-800 hover:border-slate-700'
                }`}
              >
                {/* Active Indicator Pin */}
                {isHardwareActive && (
                  <div
                    className={`absolute top-1.5 right-1.5 w-2 h-2 rounded-full ${
                      isPreview ? 'bg-emerald-300 ring-2 ring-emerald-500' : 'bg-emerald-500/80'
                    }`}
                    title="ハードウェア適用中レイヤー"
                  />
                )}

                <span className="text-base font-extrabold font-mono tracking-wider">L{layerId}</span>
                <span
                  className={`text-[11px] font-mono font-medium mt-1 px-2 py-0.5 rounded-md transition-colors ${
                    isPreview
                      ? 'bg-slate-950/40 text-indigo-100 border border-indigo-400/30'
                      : 'bg-slate-950/50 text-slate-400 group-hover:text-slate-300 border border-slate-800'
                  }`}
                >
                  {layerAngle}°
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {/* 2. OFFICIAL KEYCHRON LAUNCHER STYLE HARDWARE VISUALIZER */}
      <div className="bg-slate-900/50 border border-slate-800 rounded-2xl p-4 relative overflow-hidden flex flex-col items-center shadow-xl">
        {/* HEADER WITHIN PREVIEW BOX: OCTASHIFT ANGLE SELECTOR / BADGE */}
        <div className="w-full flex flex-wrap items-center justify-between gap-3 pb-3 mb-2 border-b border-slate-800/80 px-2 z-20">
          <div className="flex items-center gap-2">
            <RotateCw className="w-4 h-4 text-cyan-400" />
            <span className="text-xs font-semibold text-slate-200">
              認識角度 (OctaShift): <span className="font-mono text-cyan-400 font-bold ml-1">{currentAngle}°</span>
            </span>
          </div>

          {showAdvancedControls ? (
            /* OctaShift Angle Quick Selector for Current Layer */
            <div className="flex items-center gap-1 bg-slate-950/70 p-1 rounded-xl border border-slate-800/90 shadow-inner">
              {[0, 45, 90, 135, 180, 225, 270, 315].map((deg) => (
                <button
                  key={deg}
                  onClick={() => onUpdateAngle(currentPreviewLayer, deg)}
                  className={`px-2.5 py-1 rounded-lg text-xs font-mono font-bold transition-all ${
                    currentAngle === deg
                      ? 'bg-gradient-to-r from-indigo-600 to-cyan-600 text-white shadow-md shadow-indigo-500/25 ring-1 ring-cyan-400/50'
                      : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'
                  }`}
                >
                  {deg}°
                </button>
              ))}
            </div>
          ) : (
            <div className="flex items-center gap-1.5 text-xs text-slate-500">
              <span>L{currentPreviewLayer} 設定角度</span>
              <span className="px-2 py-0.5 rounded-lg bg-slate-950/80 text-cyan-300 font-mono font-bold border border-cyan-500/30">
                {currentAngle}°
              </span>
            </div>
          )}
        </div>

        {/* DYNAMIC SCREEN-SPACE DIAGRAM VISUALIZER */}
        <div className="relative w-full max-w-5xl py-4 my-2 flex items-center justify-center z-10 min-h-[580px] overflow-visible">
          {/* HARDWARE ROTATION STYLE FOR BODY */}
          <style>{`
            .rotate-hardware {
              transform: rotate(${currentAngle}deg);
              transition: transform 0.5s cubic-bezier(0.4, 0, 0.2, 1);
            }
            .smooth-canvas-item {
              transition: left 0.5s cubic-bezier(0.4, 0, 0.2, 1), top 0.5s cubic-bezier(0.4, 0, 0.2, 1), transform 0.3s ease;
            }
          `}</style>

          {/* CANVAS WRAPPER (880px x 580px) */}
          <div className="relative w-[880px] h-[580px] flex items-center justify-center">
            {/* SVG CONNECTOR LINES & BUTTON DOTS OVERLAY */}
            <svg className="absolute inset-0 w-full h-full pointer-events-none z-20">
              <defs>
                <linearGradient id="lineGlow" x1="0%" y1="0%" x2="100%" y2="100%">
                  <stop offset="0%" stopColor="#818cf8" stopOpacity="0.9" />
                  <stop offset="100%" stopColor="#06b6d4" stopOpacity="0.9" />
                </linearGradient>
              </defs>

              {calculatedAnchors.map((item) => (
                <g key={item.id} className="transition-all duration-500">
                  {/* Leader Line */}
                  <line
                    x1={item.bx}
                    y1={item.by}
                    x2={item.lx}
                    y2={item.ly}
                    stroke={item.isSelected ? 'url(#lineGlow)' : '#475569'}
                    strokeWidth={item.isSelected ? 2.5 : 1.5}
                    strokeDasharray={item.isSelected ? 'none' : '4 3'}
                    strokeOpacity={item.isSelected ? 1 : 0.6}
                  />
                  {/* Button Anchor Dot */}
                  <circle
                    cx={item.bx}
                    cy={item.by}
                    r={item.isSelected ? 5.5 : 3.5}
                    fill={item.isSelected ? '#38bdf8' : '#818cf8'}
                    className={item.isSelected ? 'animate-pulse' : ''}
                  />
                </g>
              ))}
            </svg>

            {/* AUTHENTIC KEYCHRON NAPE PRO BAR HARDWARE BODY (ROTATING IN CENTER) */}
            <div
              className="relative w-[140px] h-[460px] bg-gradient-to-b from-slate-900 via-slate-800 to-slate-900 rounded-3xl border-2 border-slate-600/80 shadow-2xl p-2.5 flex flex-col justify-between items-center rotate-hardware z-10"
              style={{
                boxShadow: '0 25px 60px rgba(0,0,0,0.8), inset 0 1px 2px rgba(255,255,255,0.2)',
              }}
            >
              {/* CNC Anodized Texture */}
              <div className="absolute inset-0 bg-slate-900/40 rounded-3xl pointer-events-none"></div>

              {/* TOP HEADER SECTION (Keychron Logo & Power/Sensor Dot) */}
              <div className="w-full flex items-center justify-between px-1 pt-1 z-10 border-b border-slate-700/40 pb-2">
                <div className="w-2.5 h-2.5 rounded-full bg-slate-700 border border-slate-500"></div>
                <span className="text-[9px] font-bold font-mono tracking-widest text-slate-400 uppercase">Keychron</span>
                <div className="w-3.5 h-3.5 rounded-full bg-slate-900 border border-slate-600 flex items-center justify-center">
                  <div className="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></div>
                </div>
              </div>

              {/* TOP BUTTON SECTION (03 Left / 04 Right) */}
              <div className="w-full relative z-10 flex items-center justify-between px-2 py-1">
                {/* 03 Button (Top-Left / G3) */}
                <button
                  onClick={() => {
                    setSelectedButtonId(5);
                    handleSimulateClick(5);
                  }}
                  className={`flex items-center gap-1 px-1.5 py-1 rounded-lg border text-left transition-all ${
                    selectedButtonId === 5 ? 'bg-indigo-600 border-indigo-400 ring-2 ring-indigo-400/50' : 'bg-slate-800/80 border-slate-700 hover:bg-slate-700'
                  }`}
                >
                  <div className="w-2 h-2 rounded-full bg-indigo-400"></div>
                  <span className="text-[10px] font-mono font-bold text-indigo-300">03</span>
                </button>

                {/* 04 Button (Top-Right / G4) */}
                <button
                  onClick={() => {
                    setSelectedButtonId(6);
                    handleSimulateClick(6);
                  }}
                  className={`flex items-center gap-1 px-1.5 py-1 rounded-lg border text-right transition-all ${
                    selectedButtonId === 6 ? 'bg-indigo-600 border-indigo-400 ring-2 ring-indigo-400/50' : 'bg-slate-800/80 border-slate-700 hover:bg-slate-700'
                  }`}
                >
                  <span className="text-[10px] font-mono font-bold text-indigo-300">04</span>
                  <div className="w-2 h-2 rounded-full bg-indigo-400"></div>
                </button>
              </div>

              {/* CENTER TRACKBALL SPHERE & SCROLL RING SECTION */}
              <div className="relative z-10 my-auto flex items-center justify-center w-full">
                {/* Bezel Ring */}
                <div className="w-28 h-28 rounded-full bg-gradient-to-b from-slate-950 to-slate-800 p-1 shadow-inner border border-slate-700 flex items-center justify-center relative">
                  {/* Scroll Ring Indicator on Trackball Right */}
                  <div className="absolute -right-2 top-1/2 transform -translate-y-1/2 w-4 h-8 rounded-full bg-slate-900 border border-slate-600 flex flex-col items-center justify-between p-0.5">
                    <div className="w-2 h-2 rounded-full bg-cyan-400"></div>
                    <div className="w-2 h-2 rounded-full bg-indigo-400"></div>
                  </div>

                  {/* Trackball Sphere (Center) */}
                  <div
                    onClick={() => handleSimulateClick(99)}
                    className={`w-24 h-24 rounded-full bg-gradient-to-tr from-slate-950 via-slate-800 to-slate-400 shadow-2xl border border-slate-500/50 cursor-pointer transition-all duration-300 relative overflow-hidden ${
                      isSimulatingClick === 99 ? 'scale-95 ring-4 ring-cyan-400' : 'hover:scale-105'
                    }`}
                    title="25mm センタートラックボール"
                  >
                    {/* Glare Reflection */}
                    <div className="absolute top-2 left-3 w-8 h-8 rounded-full bg-gradient-to-br from-white/70 to-transparent blur-[1px] pointer-events-none"></div>
                    <div className="absolute bottom-3 right-4 w-4 h-4 rounded-full bg-slate-900/60 blur-[2px] pointer-events-none"></div>
                  </div>
                </div>
              </div>

              {/* BOTTOM BUTTON SECTION (01 Left / 02 Right) */}
              <div className="w-full relative z-10 flex items-center justify-between px-2 py-1">
                {/* 01 Button (Bottom-Left / G1) */}
                <button
                  onClick={() => {
                    setSelectedButtonId(3);
                    handleSimulateClick(3);
                  }}
                  className={`flex items-center gap-1 px-1.5 py-1 rounded-lg border text-left transition-all ${
                    selectedButtonId === 3 ? 'bg-indigo-600 border-indigo-400 ring-2 ring-indigo-400/50' : 'bg-slate-800/80 border-slate-700 hover:bg-slate-700'
                  }`}
                >
                  <div className="w-2 h-2 rounded-full bg-indigo-400"></div>
                  <span className="text-[10px] font-mono font-bold text-indigo-300">01</span>
                </button>

                {/* 02 Button (Bottom-Right / G2) */}
                <button
                  onClick={() => {
                    setSelectedButtonId(4);
                    handleSimulateClick(4);
                  }}
                  className={`flex items-center gap-1 px-1.5 py-1 rounded-lg border text-right transition-all ${
                    selectedButtonId === 4 ? 'bg-indigo-600 border-indigo-400 ring-2 ring-indigo-400/50' : 'bg-slate-800/80 border-slate-700 hover:bg-slate-700'
                  }`}
                >
                  <span className="text-[10px] font-mono font-bold text-indigo-300">02</span>
                  <div className="w-2 h-2 rounded-full bg-indigo-400"></div>
                </button>
              </div>

              {/* BOTTOMMOST MAIN BUTTON CAPS (M1 Left / M2 Right) */}
              <div className="w-full z-10 flex items-center justify-between gap-1.5 border-t border-slate-700/40 pt-1.5">
                <button
                  onClick={() => {
                    setSelectedButtonId(1);
                    handleSimulateClick(1);
                  }}
                  className={`flex-1 h-10 rounded-xl border flex items-center justify-center font-bold text-xs font-mono transition-all ${
                    selectedButtonId === 1
                      ? 'bg-indigo-600 border-indigo-300 text-white ring-2 ring-indigo-400 shadow-md'
                      : 'bg-slate-800 border-slate-700 text-slate-300 hover:bg-slate-700'
                  } ${isSimulatingClick === 1 ? 'scale-95' : ''}`}
                >
                  M1
                </button>
                <button
                  onClick={() => {
                    setSelectedButtonId(2);
                    handleSimulateClick(2);
                  }}
                  className={`flex-1 h-10 rounded-xl border flex items-center justify-center font-bold text-xs font-mono transition-all ${
                    selectedButtonId === 2
                      ? 'bg-indigo-600 border-indigo-300 text-white ring-2 ring-indigo-400 shadow-md'
                      : 'bg-slate-800 border-slate-700 text-slate-300 hover:bg-slate-700'
                  } ${isSimulatingClick === 2 ? 'scale-95' : ''}`}
                >
                  M2
                </button>
              </div>
            </div>

            {/* DYNAMIC NON-OVERLAPPING LEVEL LABEL CARDS LAYER */}
            {calculatedAnchors.map((item) => {
              if (item.id === 7) {
                // SCROLL RING DUAL BADGE
                return (
                  <div
                    key={item.id}
                    className="absolute z-30 smooth-canvas-item pointer-events-auto"
                    style={{
                      left: `${item.lx}px`,
                      top: `${item.ly}px`,
                      transform: 'translate(-50%, -50%)',
                    }}
                  >
                    <div className={`bg-slate-900/95 border rounded-xl p-2 text-xs shadow-xl flex flex-col gap-1 min-w-[140px] whitespace-nowrap transition-all ${
                      item.isSelected ? 'border-cyan-400 ring-2 ring-cyan-400/40 bg-slate-900' : 'border-slate-700/80 hover:border-slate-600'
                    }`}>
                      {/* Upper Slot: Button ID 7 (Cyan dot / Clockwise ↻) */}
                      <button
                        onClick={() => {
                          setSelectedButtonId(7);
                          handleSimulateClick(7);
                        }}
                        className={`flex items-center gap-1.5 text-[11px] text-left transition-all ${
                          selectedButtonId === 7 ? 'text-cyan-300 font-bold' : 'text-slate-200 hover:text-white'
                        }`}
                      >
                        <span className="w-2 h-2 rounded-full bg-cyan-400 shrink-0"></span>
                        <span className="font-mono text-cyan-400 font-bold text-xs shrink-0" title="時計回り (CW)">↻</span>
                        <span className="font-semibold">
                          {getButtonMapping(7).description || getButtonMapping(7).key_code || '下スクロール'}
                        </span>
                      </button>
                      <div className="w-full h-[1px] bg-slate-800"></div>
                      {/* Lower Slot: Button ID 8 (Indigo dot / Counter-Clockwise ↺) */}
                      <button
                        onClick={() => {
                          setSelectedButtonId(8);
                          handleSimulateClick(8);
                        }}
                        className={`flex items-center gap-1.5 text-[11px] text-left transition-all ${
                          selectedButtonId === 8 ? 'text-indigo-300 font-bold' : 'text-slate-200 hover:text-white'
                        }`}
                      >
                        <span className="w-2 h-2 rounded-full bg-indigo-400 shrink-0"></span>
                        <span className="font-mono text-indigo-400 font-bold text-xs shrink-0" title="反時計回り (CCW)">↺</span>
                        <span className="font-semibold">
                          {getButtonMapping(8).description || getButtonMapping(8).key_code || '上にスクロール'}
                        </span>
                      </button>
                    </div>
                  </div>
                );
              }

              const mapping = getButtonMapping(item.id);
              const badgeLabel = item.id === 1 ? 'M1' : item.id === 2 ? 'M2' : `0${item.id === 5 ? 3 : item.id === 6 ? 4 : item.id === 3 ? 1 : 2}`;

              return (
                <div
                  key={item.id}
                  className="absolute z-30 smooth-canvas-item pointer-events-auto"
                  style={{
                    left: `${item.lx}px`,
                    top: `${item.ly}px`,
                    transform: 'translate(-50%, -50%)',
                  }}
                >
                  <button
                    onClick={() => {
                      setSelectedButtonId(item.id);
                      handleSimulateClick(item.id);
                    }}
                    className={`bg-slate-900/95 border rounded-xl px-3 py-1.5 text-xs shadow-xl transition-all flex items-center gap-2 whitespace-nowrap ${
                      item.isSelected
                        ? 'border-indigo-400 text-white ring-2 ring-indigo-400/50 bg-indigo-950/90 shadow-indigo-500/20 scale-105'
                        : 'border-slate-700/80 text-slate-200 hover:border-indigo-500/60 hover:text-white'
                    }`}
                  >
                    <span className={`font-mono text-[10px] font-bold px-1 py-0.5 rounded ${
                      item.id === 1 || item.id === 2 ? 'bg-slate-800 text-slate-300' : 'text-indigo-400 bg-indigo-950/60'
                    }`}>
                      {badgeLabel}
                    </span>
                    <span className="font-bold">{mapping.description || mapping.name}</span>
                  </button>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
};
