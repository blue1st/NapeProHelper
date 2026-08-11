import React from 'react';
import { AppConfig } from '../types';
import { isEnabled, enable, disable } from '@tauri-apps/plugin-autostart';

interface AppSettingsProps {
  config: AppConfig;
  onUpdateConfig: (newConfig: Partial<AppConfig>) => void;
}

export const AppSettings: React.FC<AppSettingsProps> = ({ config, onUpdateConfig }) => {
  const [autostartActive, setAutostartActive] = React.useState(config.autostart);

  React.useEffect(() => {
    async function checkAutostart() {
      try {
        const active = await isEnabled();
        setAutostartActive(active);
      } catch {
        // Fallback for dev mode
      }
    }
    checkAutostart();
  }, []);

  const handleToggleAutostart = async () => {
    try {
      if (autostartActive) {
        await disable();
        setAutostartActive(false);
        onUpdateConfig({ autostart: false });
      } else {
        await enable();
        setAutostartActive(true);
        onUpdateConfig({ autostart: true });
      }
    } catch {
      // Toggle state locally if running in web preview
      const next = !autostartActive;
      setAutostartActive(next);
      onUpdateConfig({ autostart: next });
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3 mb-2">
        <h2 className="text-sm font-semibold text-slate-200">設定</h2>
      </div>

      <div className="space-y-3">
        {/* Startup Boot Launch */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
          <div>
            <h4 className="text-sm font-semibold text-white">ログイン時に自動起動</h4>
            <p className="text-xs text-slate-400 mt-0.5">PC起動時にバックグラウンドで起動します</p>
          </div>
          <button
            onClick={handleToggleAutostart}
            className={`w-12 h-6 rounded-full p-1 transition-colors duration-200 ease-in-out ${
              autostartActive ? 'bg-indigo-600' : 'bg-slate-700'
            }`}
          >
            <div
              className={`w-4 h-4 rounded-full bg-white transition-transform duration-200 ease-in-out ${
                autostartActive ? 'translate-x-6' : 'translate-x-0'
              }`}
            />
          </button>
        </div>

        {/* Notification toggle */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
          <div>
            <h4 className="text-sm font-semibold text-white">レイヤー切替通知</h4>
            <p className="text-xs text-slate-400 mt-0.5">レイヤーが切り替わった際にOS通知を表示します</p>
          </div>
          <button
            onClick={() => onUpdateConfig({ show_notifications: !config.show_notifications })}
            className={`w-12 h-6 rounded-full p-1 transition-colors duration-200 ease-in-out ${
              config.show_notifications ? 'bg-indigo-600' : 'bg-slate-700'
            }`}
          >
            <div
              className={`w-4 h-4 rounded-full bg-white transition-transform duration-200 ease-in-out ${
                config.show_notifications ? 'translate-x-6' : 'translate-x-0'
              }`}
            />
          </button>
        </div>
      </div>
    </div>
  );
};
