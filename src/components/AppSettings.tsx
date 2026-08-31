import React from 'react';
import { AppConfig } from '../types';
import { isEnabled, enable, disable } from '@tauri-apps/plugin-autostart';
import { invoke } from '@tauri-apps/api/core';
import { AutoSwitchSettings } from './AutoSwitchSettings';

interface AppSettingsProps {
  config: AppConfig;
  onUpdateConfig: (newConfig: Partial<AppConfig>) => void;
  onNavigateToAbout?: () => void;
}

export const AppSettings: React.FC<AppSettingsProps> = ({ config, onUpdateConfig, onNavigateToAbout }) => {
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

      <div className="space-y-4">
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
            onClick={async () => {
              const nextVal = !config.show_notifications;
              try {
                const res = await invoke<AppConfig>('update_general_config', {
                  showNotifications: nextVal,
                  showAdvancedHardwareControls: null,
                });
                if (res) onUpdateConfig(res);
              } catch {
                onUpdateConfig({ show_notifications: nextVal });
              }
            }}
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

        {/* Advanced Hardware Controls toggle */}
        <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
          <div className="max-w-[80%]">
            <h4 className="text-sm font-semibold text-white">高度なハードウェア設定を表示</h4>
            <p className="text-xs text-slate-400 mt-0.5 leading-relaxed">
              トラックボールタブ、OctaShift認識角度の手動変更セレクタ、トレイメニューのDPI・モード切替を表示します（通常は公式Keychron Launcherでの設定を推奨）。
            </p>
          </div>
          <button
            onClick={async () => {
              const nextVal = !config.show_advanced_hardware_controls;
              try {
                const res = await invoke<AppConfig>('update_general_config', {
                  showNotifications: null,
                  showAdvancedHardwareControls: nextVal,
                });
                if (res) onUpdateConfig(res);
              } catch {
                onUpdateConfig({ show_advanced_hardware_controls: nextVal });
              }
            }}
            className={`w-12 h-6 rounded-full p-1 transition-colors duration-200 ease-in-out shrink-0 ${
              config.show_advanced_hardware_controls ? 'bg-indigo-600' : 'bg-slate-700'
            }`}
          >
            <div
              className={`w-4 h-4 rounded-full bg-white transition-transform duration-200 ease-in-out ${
                config.show_advanced_hardware_controls ? 'translate-x-6' : 'translate-x-0'
              }`}
            />
          </button>
        </div>

        {/* Auto Layer Switching Section */}
        <AutoSwitchSettings config={config} onUpdateConfig={onUpdateConfig} />

        {/* About App & Version Info Shortcut */}
        {onNavigateToAbout && (
          <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-4 flex items-center justify-between">
            <div>
              <h4 className="text-sm font-semibold text-white">バージョン情報 &amp; リポジトリ</h4>
              <p className="text-xs text-slate-400 mt-0.5">アプリのバージョン確認、アップデート、GitHubリポジトリへのリンク</p>
            </div>
            <button
              onClick={onNavigateToAbout}
              className="px-3.5 py-1.5 bg-slate-800 hover:bg-slate-700 text-indigo-300 rounded-lg text-xs font-semibold border border-slate-700 hover:border-slate-600 transition-all"
            >
              アプリ情報を開く
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

