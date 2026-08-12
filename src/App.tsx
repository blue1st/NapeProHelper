import { useEffect, useState } from 'react';
import { AppConfig, DeviceProfile } from './types';
import { Header } from './components/Header';
import { NapeVisualizer } from './components/NapeVisualizer';
import { AngleAndSensitivity } from './components/AngleAndSensitivity';
import { AppSettings } from './components/AppSettings';
import { AboutApp } from './components/AboutApp';
import { invoke } from '@tauri-apps/api/core';
import { Info, CheckCircle2, Cpu, Globe, ExternalLink } from 'lucide-react';

const defaultDevice: DeviceProfile = {
  id: 'dev-nape-01',
  name: 'Keychron Nape Pro',
  interface_type: 'USB / 2.4GHz',
  serial_number: '',
  is_connected: false,
  active_layer: 0,
  octashift_angle: 0,
  layer_octashift_angles: { 0: 0, 1: 0, 2: 0, 3: 0, 4: 0, 5: 0, 6: 0, 7: 0 },
  pointer_dpi: 1600,
  trackball_scroll_mode: false,
  trackball_gesture_mode: false,
  layer_names: {},
  button_mappings: {},
};

const initialEmptyConfig: AppConfig = {
  autostart: true,
  minimize_to_tray: true,
  show_notifications: true,
  device: defaultDevice,
};

export function App() {
  const [config, setConfig] = useState<AppConfig>(initialEmptyConfig);
  const [activeTab, setActiveTab] = useState<'visualizer' | 'hardware' | 'settings' | 'about'>('visualizer');
  const [showTrayHelp, setShowTrayHelp] = useState<boolean>(false);
  const [isReloading, setIsReloading] = useState<boolean>(false);
  const [hasUpdate, setHasUpdate] = useState<boolean>(false);

  const handleReloadDeviceConfig = async () => {
    setIsReloading(true);
    try {
      const res = await invoke<AppConfig>('refresh_from_hardware');
      if (res) setConfig(res);
    } catch (err) {
      console.log('Refresh from hardware skipped:', err);
    } finally {
      setTimeout(() => setIsReloading(false), 800);
    }
  };

  // Fetch Rust backend config on load & setup periodic connection polling
  useEffect(() => {
    async function loadConfig() {
      try {
        const res = await invoke<AppConfig>('get_config');
        if (res && res.device) {
          setConfig(res);
        }
      } catch (err) {
        console.log('Running in browser or fallback preview mode:', err);
      }
    }
    loadConfig();

    const interval = setInterval(async () => {
      try {
        const res = await invoke<AppConfig>('check_connection');
        if (res && res.device) {
          setConfig(res);
        }
      } catch {
        // Ignored in browser preview
      }
    }, 2500);

    // Listen to real-time config changes from tray menu layer switching
    let unlistenFn: (() => void) | null = null;
    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<AppConfig>('config-updated', (event) => {
        if (event.payload) {
          setConfig(event.payload);
        }
      }).then((unlisten) => {
        unlistenFn = unlisten;
      });
    });

    return () => {
      clearInterval(interval);
      if (unlistenFn) unlistenFn();
    };
  }, []);

  const device = config.device || defaultDevice;
  const activeLayer = device.active_layer ?? 0;

  // Handlers invoking Rust commands or updating local config
  const handleSelectLayer = async (layerId: number) => {
    setConfig((prev) => {
      const targetAngle = prev.device.layer_octashift_angles?.[layerId] ?? 0;
      return {
        ...prev,
        device: {
          ...prev.device,
          active_layer: layerId,
          octashift_angle: targetAngle,
        },
      };
    });
    try {
      const res = await invoke<AppConfig>('set_active_layer', { layerId });
      if (res) setConfig(res);
    } catch (err) {
      console.log('Backend sync skipped:', err);
    }
  };

  const handleUpdateAngle = async (layerId: number, angle: number) => {
    setConfig((prev) => {
      const layerAngles = { ...(prev.device.layer_octashift_angles || {}), [layerId]: angle };
      const mainAngle = layerId === prev.device.active_layer ? angle : prev.device.octashift_angle;
      return {
        ...prev,
        device: {
          ...prev.device,
          layer_octashift_angles: layerAngles,
          octashift_angle: mainAngle,
        },
      };
    });
    try {
      const res = await invoke<AppConfig>('set_octashift_angle', {
        layerId,
        angle,
      });
      if (res) setConfig(res);
    } catch (err) {
      console.log('Backend sync skipped:', err);
    }
  };

  const handleUpdateDpi = async (dpi: number) => {
    setConfig((prev) => ({
      ...prev,
      device: { ...prev.device, pointer_dpi: dpi },
    }));
    try {
      const res = await invoke<AppConfig>('set_pointer_dpi', { dpi });
      if (res) setConfig(res);
    } catch (err) {
      console.log('Backend sync skipped:', err);
    }
  };

  const handleToggleScrollMode = async (enabled: boolean) => {
    setConfig((prev) => ({
      ...prev,
      device: { ...prev.device, trackball_scroll_mode: enabled },
    }));
    try {
      const res = await invoke<AppConfig>('set_trackball_scroll_mode', { enabled });
      if (res) setConfig(res);
    } catch (err) {
      console.log('Backend sync skipped:', err);
    }
  };

  const handleToggleGestureMode = async (enabled: boolean) => {
    setConfig((prev) => ({
      ...prev,
      device: { ...prev.device, trackball_gesture_mode: enabled },
    }));
    try {
      const res = await invoke<AppConfig>('set_trackball_gesture_mode', { enabled });
      if (res) setConfig(res);
    } catch (err) {
      console.log('Backend sync skipped:', err);
    }
  };

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans select-none">
      {/* Top Navigation & Header */}
      <Header
        device={device}
        onToggleTrayHelp={() => setShowTrayHelp(true)}
        onReloadDeviceConfig={handleReloadDeviceConfig}
        isReloading={isReloading}
      />

      {/* Primary Tab Navigation & Keychron Launcher Link */}
      <div className="border-b border-slate-800 px-4 flex items-center justify-between">
        <div className="flex items-center gap-6">
          {[
            { id: 'visualizer', label: 'プレビュー' },
            { id: 'hardware', label: 'トラックボール' },
            { id: 'settings', label: '設定' },
            { id: 'about', label: 'アプリ情報', hasBadge: hasUpdate },
          ].map((tab) => {
            const isActive = activeTab === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={
                  (isActive
                    ? 'text-indigo-400 border-b-2 border-indigo-400 py-2.5 text-xs font-semibold'
                    : 'text-slate-500 hover:text-slate-300 py-2.5 border-b-2 border-transparent text-xs font-medium transition-colors') +
                  ' relative flex items-center gap-1.5'
                }
              >
                <span>{tab.label}</span>
                {tab.hasBadge && (
                  <span className="flex h-2 w-2 relative">
                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                    <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
                  </span>
                )}
              </button>
            );
          })}
        </div>

        <button
          onClick={async () => {
            try {
              await invoke('open_keychron_launcher');
            } catch {
              window.open('https://launcher.keychron.com/', '_blank');
            }
          }}
          className="flex items-center gap-1.5 py-1 px-3 my-1 rounded-lg text-xs font-medium text-slate-300 hover:text-white bg-slate-900 hover:bg-slate-800 border border-slate-800 hover:border-slate-700 transition-all shadow-sm group"
          title="公式 Keychron Web Launcher を開いて全キーマップ・マクロ等を編集"
        >
          <Globe className="w-3.5 h-3.5 text-indigo-400 group-hover:scale-110 transition-transform" />
          <span>Keychron Launcher (公式Web設定)</span>
          <ExternalLink className="w-3 h-3 text-slate-400 ml-0.5" />
        </button>
      </div>

      {/* Main Content Viewport */}
      <main className="flex-1 p-4 overflow-y-auto max-w-6xl w-full mx-auto">
        {activeTab === 'about' ? (
          <AboutApp onUpdateDetected={() => setHasUpdate(true)} />
        ) : activeTab === 'settings' ? (
          <AppSettings
            config={config}
            onUpdateConfig={(newCfg) => setConfig((prev) => ({ ...prev, ...newCfg }))}
            onNavigateToAbout={() => setActiveTab('about')}
          />
        ) : !device.is_connected ? (
          <div className="bg-slate-900/80 border border-slate-800 rounded-3xl p-10 text-center max-w-xl mx-auto my-12 space-y-6 shadow-2xl backdrop-blur-md">
            <div className="w-20 h-20 bg-indigo-500/10 border border-indigo-500/30 rounded-full flex items-center justify-center mx-auto text-indigo-400 animate-pulse">
              <Cpu className="w-10 h-10" />
            </div>
            <div className="space-y-2">
              <h2 className="text-xl font-bold text-slate-100">Keychron Nape Pro の接続待ち</h2>
              <p className="text-xs text-slate-400 leading-relaxed">
                USB-C ケーブルまたは 2.4GHz レシーバーで Keychron Nape Pro を PC に接続してください。<br />
                接続が検知されると、自動的に実機と同期し、ビジュアル操作画面が開きます。
              </p>
            </div>
            <button
              onClick={() => invoke('open_keychron_launcher')}
              className="inline-flex items-center gap-2 px-5 py-2.5 bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white rounded-xl text-xs font-bold transition-all shadow-lg shadow-indigo-500/25"
            >
              🌐 Keychron Launcher (公式Web設定) を開く
            </button>
          </div>
        ) : (
          <>
            {activeTab === 'visualizer' && (
              <NapeVisualizer
                device={device}
                activeLayer={activeLayer}
                onSelectLayer={handleSelectLayer}
                onUpdateAngle={handleUpdateAngle}
                onRefreshFromHardware={handleReloadDeviceConfig}
              />
            )}

            {activeTab === 'hardware' && (
              <AngleAndSensitivity
                device={device}
                activeLayer={activeLayer}
                onUpdateDpi={handleUpdateDpi}
                onToggleScrollMode={handleToggleScrollMode}
                onToggleGestureMode={handleToggleGestureMode}
              />
            )}
          </>
        )}
      </main>

      {/* Tray Help Modal */}
      {showTrayHelp && (
        <div className="fixed inset-0 bg-black/75 backdrop-blur-md z-50 flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-700 rounded-2xl max-w-lg w-full p-6 shadow-2xl space-y-4">
            <div className="flex items-center gap-3 border-b border-slate-800 pb-3">
              <div className="p-2 bg-indigo-500/20 text-indigo-400 rounded-xl">
                <Info className="w-6 h-6" />
              </div>
              <div>
                <h3 className="font-bold text-slate-100 text-base">システムトレイ常駐 &amp; 動作仕様</h3>
                <p className="text-xs text-slate-400">Nape Pro Helper の挙動について</p>
              </div>
            </div>

            <ul className="space-y-3 text-xs text-slate-300">
              <li className="flex items-start gap-2.5">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span>
                  <strong>システムトレイ常駐:</strong> ウィンドウ右上の × ボタンを押してもアプリはバックグラウンドで起動し続けます。
                </span>
              </li>
              <li className="flex items-start gap-2.5">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span>
                  <strong>タスクバー非表示:</strong> ウィンドウを閉じている間、WindowsのタスクバーやMacのDockからアイコンが消え、システムトレイ (タスクトレイ) のみ表示されます。
                </span>
              </li>
              <li className="flex items-start gap-2.5">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span>
                  <strong>トレイアイコン右クリック:</strong> トレイアイコンの右クリックメニューからアクティブレイヤーのクイック切替やウィンドウ再表示が可能です。
                </span>
              </li>
              <li className="flex items-start gap-2.5">
                <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                <span>
                  <strong>完全終了:</strong> アプリを完全に終了するには、トレイアイコン右クリックメニューの「終了 (Quit)」を選択してください。
                </span>
              </li>
            </ul>

            <div className="pt-4 border-t border-slate-800 flex justify-end">
              <button
                onClick={() => setShowTrayHelp(false)}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl text-xs font-semibold transition-all"
              >
                理解しました
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;

