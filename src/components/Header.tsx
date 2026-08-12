import React from 'react';
import { DeviceProfile } from '../types';
import { Wifi, Usb, Bluetooth, Info, RefreshCw } from 'lucide-react';

interface HeaderProps {
  device: DeviceProfile;
  onToggleTrayHelp: () => void;
  onReloadDeviceConfig: () => void;
  isReloading?: boolean;
}

export const Header: React.FC<HeaderProps> = ({
  device,
  onToggleTrayHelp,
  onReloadDeviceConfig,
  isReloading = false,
}) => {
  const getInterfaceIcon = (type: string) => {
    if (type.includes('USB')) return <Usb className="w-4 h-4 text-emerald-400" />;
    if (type.includes('2.4G') || type.includes('ドングル')) return <Wifi className="w-4 h-4 text-cyan-400" />;
    return <Bluetooth className="w-4 h-4 text-indigo-400" />;
  };

  return (
    <header className="h-12 px-4 bg-slate-900/80 border-b border-slate-800 flex items-center justify-between backdrop-blur-md select-none sticky top-0 z-50">
      {/* Brand & Title */}
      <div className="flex items-center gap-3">
        <img
          src="/app-icon.png"
          alt="Nape Pro Helper"
          className="w-8 h-8 rounded-xl object-cover shadow-lg shadow-indigo-500/20 ring-1 ring-white/20"
        />
        <h1 className="font-semibold text-sm text-white tracking-wide">Nape Pro Helper</h1>
      </div>

      {/* Connection Status Badge & Reload */}
      <div className="flex items-center gap-3">
        {/* Device Status Badge */}
        <div className="flex items-center bg-slate-950/80 border border-slate-800 rounded-xl px-3 py-1.5 gap-2 text-xs font-semibold">
          {getInterfaceIcon(device.interface_type || 'USB')}
          <span className="text-slate-200">{device.name || 'Keychron Nape Pro'}</span>
          {device.is_connected ? (
            <span className="inline-flex items-center gap-1 text-emerald-400 text-[11px]">
              <span className="w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
              接続中
            </span>
          ) : (
            <span className="inline-flex items-center gap-1 text-slate-500 text-[11px]">
              <span className="w-2 h-2 rounded-full bg-slate-600" />
              未接続
            </span>
          )}
        </div>

        {/* Hardware Settings Reload Button */}
        <button
          onClick={onReloadDeviceConfig}
          disabled={isReloading}
          className="p-2 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 transition-colors"
          title="Keychron Launcher や本体で変更した最新設定 (キー割り当て・DPI・角度) を実機から再読み込み"
        >
          <RefreshCw className={`w-5 h-5 ${isReloading ? 'animate-spin' : ''}`} />
        </button>

        {/* Tray Resident Info Button */}
        <button
          onClick={onToggleTrayHelp}
          className="p-2 text-slate-400 hover:text-indigo-400 hover:bg-slate-800 rounded-lg transition-colors border border-transparent hover:border-slate-700"
          title="システムトレイ常駐について"
        >
          <Info className="w-5 h-5" />
        </button>
      </div>
    </header>
  );
};

