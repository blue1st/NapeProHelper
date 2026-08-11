import React from 'react';
import { DeviceProfile } from '../types';

interface AngleAndSensitivityProps {
  device: DeviceProfile;
  activeLayer: number;
  onUpdateDpi: (dpi: number) => void;
  onToggleScrollMode: (enabled: boolean) => void;
  onToggleGestureMode: (enabled: boolean) => void;
}

export const AngleAndSensitivity: React.FC<AngleAndSensitivityProps> = ({
  device,
  onUpdateDpi,
  onToggleScrollMode,
  onToggleGestureMode,
}) => {
  const dpiPresets = [400, 800, 1800, 3200, 4000];
  const isScrollMode = device.trackball_scroll_mode ?? false;
  const isGestureMode = device.trackball_gesture_mode ?? false;

  return (
    <div className="space-y-6">
      {/* 1. TRACKBALL ADVANCED MODES (SCROLL MODE & GESTURE MODE) PARALLEL GRID */}
      <div className="space-y-2">
        {/* Trackball Scroll Mode Toggle Card */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
          <span className="text-sm font-medium text-slate-200">スクロールモード</span>
          <button
            onClick={() => onToggleScrollMode(!isScrollMode)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
              isScrollMode ? 'bg-indigo-500' : 'bg-slate-700'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                isScrollMode ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>

        {/* Trackball Gesture Mode Toggle Card */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
          <span className="text-sm font-medium text-slate-200">ジェスチャーモード</span>
          <button
            onClick={() => onToggleGestureMode(!isGestureMode)}
            className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none ${
              isGestureMode ? 'bg-indigo-500' : 'bg-slate-700'
            }`}
          >
            <span
              className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                isGestureMode ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          </button>
        </div>
      </div>

      {/* 2. POINTER SENSITIVITY (DPI) CONTROLLER */}
      <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex flex-col justify-between">
        <div>
          <h3 className="text-sm font-medium text-slate-200 mb-4">ポインター感度 (DPI)</h3>

          {/* DPI Slider */}
          <div className="space-y-4 px-2">
            <div className="flex justify-between text-xs text-slate-400 font-mono">
              <span>400 DPI</span>
              <span>4000 DPI</span>
            </div>
            <div className="flex items-center gap-3">
              <input
                type="range"
                min="400"
                max="4000"
                step="100"
                value={device.pointer_dpi}
                onChange={(e) => onUpdateDpi(Number(e.target.value))}
                className="flex-1 h-2.5 bg-slate-800 rounded-lg appearance-none cursor-pointer accent-indigo-500"
              />
              <span className="text-sm font-mono font-semibold text-indigo-400 min-w-[60px] text-right">{device.pointer_dpi}</span>
            </div>
          </div>
        </div>

        {/* DPI Quick Presets */}
        <div className="pt-4 border-t border-slate-800 mt-6">
          <div className="grid grid-cols-5 gap-2">
            {dpiPresets.map((dpi) => (
              <button
                key={dpi}
                onClick={() => onUpdateDpi(dpi)}
                className={`py-2 rounded-lg text-xs font-mono font-bold border transition-all text-center ${
                  device.pointer_dpi === dpi
                    ? 'bg-indigo-600 text-white border-indigo-400 shadow-md ring-1 ring-indigo-300'
                    : 'bg-slate-900 border-slate-800 text-slate-300 hover:bg-slate-800'
                }`}
              >
                {dpi}
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
