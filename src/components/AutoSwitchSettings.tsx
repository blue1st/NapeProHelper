import React, { useState } from 'react';
import { AppConfig, AutoSwitchRule, ActiveAppInfo } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { RefreshCw, Plus, Trash2, Layers, Monitor, CheckCircle2, AlertCircle, Clock } from 'lucide-react';

interface AutoSwitchSettingsProps {
  config: AppConfig;
  onUpdateConfig: (updatedFields: Partial<AppConfig>) => void;
}

export const AutoSwitchSettings: React.FC<AutoSwitchSettingsProps> = ({ config, onUpdateConfig }) => {
  const isEnabled = config.auto_switch_enabled ?? false;
  const defaultLayer = config.auto_switch_default_layer !== undefined ? config.auto_switch_default_layer : 0;
  const rules = config.auto_switch_rules ?? [];

  const [isAdding, setIsAdding] = useState(false);
  const [newRuleName, setNewRuleName] = useState('');
  const [newAppName, setNewAppName] = useState('');
  const [newTargetLayer, setNewTargetLayer] = useState<number>(0);
  const [fetchingAppInfo, setFetchingAppInfo] = useState(false);
  const [countdown, setCountdown] = useState<number | null>(null);
  const [fetchedApp, setFetchedApp] = useState<ActiveAppInfo | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  const saveAutoSwitchConfig = async (
    newEnabled: boolean,
    newDefaultLayer: number | null,
    newRules: AutoSwitchRule[]
  ) => {
    try {
      const res = await invoke<AppConfig>('update_auto_switch_config', {
        enabled: newEnabled,
        defaultLayer: newDefaultLayer,
        rules: newRules,
      });
      if (res) {
        onUpdateConfig(res);
      }
    } catch {
      // Browser fallback / fallback mode
      onUpdateConfig({
        auto_switch_enabled: newEnabled,
        auto_switch_default_layer: newDefaultLayer,
        auto_switch_rules: newRules,
      });
    }
  };

  const handleToggleEnabled = () => {
    saveAutoSwitchConfig(!isEnabled, defaultLayer, rules);
  };

  const handleChangeDefaultLayer = (newVal: number | null) => {
    saveAutoSwitchConfig(isEnabled, newVal, rules);
  };

  const handleToggleRule = (ruleId: string) => {
    const updated = rules.map((r) => (r.id === ruleId ? { ...r, enabled: !r.enabled } : r));
    saveAutoSwitchConfig(isEnabled, defaultLayer, updated);
  };

  const handleDeleteRule = (ruleId: string) => {
    const updated = rules.filter((r) => r.id !== ruleId);
    saveAutoSwitchConfig(isEnabled, defaultLayer, updated);
  };

  const handleFetchActiveApp = async () => {
    setFetchingAppInfo(true);
    setErrorMsg(null);
    try {
      const info = await invoke<ActiveAppInfo>('get_active_app_info');
      setFetchedApp(info);
      if (info.app_name) {
        setNewRuleName((prev) => prev || info.app_name);
        setNewAppName((prev) => prev || info.app_name);
      }
    } catch (err: unknown) {
      setErrorMsg(typeof err === 'string' ? err : '直前のアクティブアプリ情報を取得できませんでした');
    } finally {
      setFetchingAppInfo(false);
    }
  };

  const handleFetchDelayedApp = async () => {
    setFetchingAppInfo(true);
    setErrorMsg(null);
    setCountdown(3);

    const timer = setInterval(() => {
      setCountdown((prev) => (prev && prev > 1 ? prev - 1 : null));
    }, 1000);

    try {
      const info = await invoke<ActiveAppInfo>('get_active_app_info_delayed', { delaySeconds: 3 });
      setFetchedApp(info);
      if (info.app_name) {
        setNewRuleName((prev) => prev || info.app_name);
        setNewAppName((prev) => prev || info.app_name);
      }
    } catch (err: unknown) {
      setErrorMsg(typeof err === 'string' ? err : 'アプリ情報の取得に失敗しました');
    } finally {
      clearInterval(timer);
      setCountdown(null);
      setFetchingAppInfo(false);
    }
  };

  const handleAddRule = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newRuleName.trim() || !newAppName.trim()) {
      setErrorMsg('ルール名と対象アプリ識別子を入力してください');
      return;
    }

    const newRule: AutoSwitchRule = {
      id: Date.now().toString(),
      name: newRuleName.trim(),
      app_name: newAppName.trim(),
      target_layer: newTargetLayer,
      enabled: true,
    };

    const updated = [...rules, newRule];
    saveAutoSwitchConfig(isEnabled, defaultLayer, updated);

    // Reset form
    setNewRuleName('');
    setNewAppName('');
    setNewTargetLayer(0);
    setFetchedApp(null);
    setIsAdding(false);
    setErrorMsg(null);
  };

  const layerNames = config.device?.layer_names || {};

  const getLayerLabel = (layerId: number) => {
    const customName = layerNames[layerId];
    if (customName && customName !== `Layer ${layerId}` && customName !== `L${layerId}`) {
      return `Layer ${layerId}: ${customName}`;
    }
    return `Layer ${layerId}`;
  };

  return (
    <div className="bg-slate-900/50 border border-slate-800 rounded-xl p-5 space-y-4">
      {/* Header & Main Enable Switch */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2.5">
          <div className="p-2 bg-indigo-500/10 border border-indigo-500/20 rounded-lg text-indigo-400">
            <Monitor className="w-5 h-5" />
          </div>
          <div>
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              アクティブアプリ連動・自動レイヤー切り替え
              {isEnabled && (
                <span className="text-[10px] bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 px-2 py-0.5 rounded-full font-mono">
                  ACTIVE
                </span>
              )}
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              フォーカスされたアプリ（プロセス名）に応じて自動的にレイヤーを切り替えます
            </p>
          </div>
        </div>

        <button
          onClick={handleToggleEnabled}
          className={`w-12 h-6 rounded-full p-1 transition-colors duration-200 ease-in-out shrink-0 ${
            isEnabled ? 'bg-indigo-600' : 'bg-slate-700'
          }`}
        >
          <div
            className={`w-4 h-4 rounded-full bg-white transition-transform duration-200 ease-in-out ${
              isEnabled ? 'translate-x-6' : 'translate-x-0'
            }`}
          />
        </button>
      </div>

      {/* Default Layer Setting (For rules unmatched apps) */}
      <div className="bg-slate-800/60 border border-slate-700/60 rounded-xl p-3.5 flex items-center justify-between gap-3">
        <div>
          <h4 className="text-xs font-semibold text-white">
            デフォルトレイヤー (ルール外のアプリ用)
          </h4>
          <p className="text-[11px] text-slate-400 mt-0.5">
            登録ルールに該当しないアプリがアクティブになった際の復帰先レイヤー
          </p>
        </div>

        <select
          value={defaultLayer === null ? 'none' : defaultLayer}
          onChange={(e) => {
            const val = e.target.value === 'none' ? null : Number(e.target.value);
            handleChangeDefaultLayer(val);
          }}
          className="bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500 shrink-0 font-medium"
        >
          <option value="none">直前のレイヤーを維持 (切り替えない)</option>
          {[0, 1, 2, 3, 4, 5, 6, 7].map((l) => (
            <option key={l} value={l}>
              {getLayerLabel(l)}
            </option>
          ))}
        </select>
      </div>

      {/* Rules List Section */}
      <div className="space-y-3 pt-2">
        <div className="flex items-center justify-between">
          <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
            自動切替ルール ({rules.length})
          </h4>

          {!isAdding && (
            <button
              onClick={() => setIsAdding(true)}
              className="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors shadow-sm"
            >
              <Plus className="w-3.5 h-3.5" />
              新規ルール追加
            </button>
          )}
        </div>

        {/* Add New Rule Form */}
        {isAdding && (
          <form onSubmit={handleAddRule} className="bg-slate-800/80 border border-indigo-500/40 rounded-xl p-4 space-y-3.5">
            <div className="flex items-center justify-between flex-wrap gap-2">
              <h5 className="text-xs font-bold text-indigo-300 flex items-center gap-1.5">
                <Plus className="w-4 h-4 text-indigo-400" />
                新しい自動切替ルールを作成
              </h5>

              <div className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={handleFetchActiveApp}
                  disabled={fetchingAppInfo}
                  className="px-2.5 py-1 bg-indigo-900/60 hover:bg-indigo-800/80 border border-indigo-700/60 rounded text-xs font-medium text-indigo-200 flex items-center gap-1 transition-colors disabled:opacity-50"
                  title="NapePro Helperを開く直前に操作していた外部アプリの情報を取得します"
                >
                  <RefreshCw className={`w-3 h-3 ${fetchingAppInfo && countdown === null ? 'animate-spin' : ''}`} />
                  直前のアプリを取得
                </button>

                <button
                  type="button"
                  onClick={handleFetchDelayedApp}
                  disabled={fetchingAppInfo}
                  className="px-2.5 py-1 bg-slate-700 hover:bg-slate-600 border border-slate-600 rounded text-xs font-medium text-slate-200 flex items-center gap-1 transition-colors disabled:opacity-50"
                  title="3秒後にアクティブなアプリを取得します。ボタンを押した後に目的のアプリをクリックしてください。"
                >
                  <Clock className="w-3 h-3 text-amber-400" />
                  {countdown !== null ? `${countdown}秒後に取得...` : '3秒後に取得'}
                </button>
              </div>
            </div>

            {fetchedApp && (
              <div className="bg-indigo-950/40 border border-indigo-500/30 rounded-lg p-2.5 text-xs text-indigo-200 flex items-start gap-2">
                <CheckCircle2 className="w-4 h-4 text-indigo-400 shrink-0 mt-0.5" />
                <div>
                  <span className="font-semibold text-white">検出アプリ:</span> {fetchedApp.app_name}{' '}
                  {fetchedApp.title && <span className="text-slate-400">({fetchedApp.title})</span>}
                  <p className="text-[10px] text-indigo-300/80 mt-0.5">
                    ※ 取得されたアプリ名がルールの検索キーワードにセットされました
                  </p>
                </div>
              </div>
            )}

            {errorMsg && (
              <div className="bg-rose-950/40 border border-rose-500/30 rounded-lg p-2.5 text-xs text-rose-300 flex items-center gap-2">
                <AlertCircle className="w-4 h-4 text-rose-400 shrink-0" />
                {errorMsg}
              </div>
            )}

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">ルール表示名</label>
                <input
                  type="text"
                  placeholder="例: VS Code"
                  value={newRuleName}
                  onChange={(e) => setNewRuleName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">対象アプリ名 / プロセスキーワード</label>
                <input
                  type="text"
                  placeholder="例: Code または photoshop"
                  value={newAppName}
                  onChange={(e) => setNewAppName(e.target.value)}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white font-mono placeholder-slate-500 focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label className="block text-[11px] font-medium text-slate-400 mb-1">切り替え先レイヤー</label>
                <select
                  value={newTargetLayer}
                  onChange={(e) => setNewTargetLayer(Number(e.target.value))}
                  className="w-full bg-slate-900 border border-slate-700 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-indigo-500"
                >
                  {[0, 1, 2, 3, 4, 5, 6, 7].map((l) => (
                    <option key={l} value={l}>
                      {getLayerLabel(l)}
                    </option>
                  ))}
                </select>
              </div>
            </div>

            <div className="flex justify-end gap-2 pt-1">
              <button
                type="button"
                onClick={() => {
                  setIsAdding(false);
                  setErrorMsg(null);
                }}
                className="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-slate-300 rounded-lg text-xs font-medium transition-colors"
              >
                キャンセル
              </button>
              <button
                type="submit"
                className="px-4 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold transition-colors shadow-sm"
              >
                保存して登録
              </button>
            </div>
          </form>
        )}

        {/* Existing Rules List */}
        {rules.length === 0 ? (
          <div className="bg-slate-900/30 border border-dashed border-slate-800 rounded-xl p-6 text-center">
            <Monitor className="w-8 h-8 text-slate-600 mx-auto mb-2 opacity-60" />
            <p className="text-xs text-slate-400">登録されている自動切替ルールがありません</p>
            <p className="text-[11px] text-slate-500 mt-1">「新規ルール追加」から特定のアプリとレイヤーを紐づけられます</p>
          </div>
        ) : (
          <div className="space-y-2">
            {rules.map((rule) => (
              <div
                key={rule.id}
                className={`border rounded-xl p-3.5 flex items-center justify-between gap-3 transition-all ${
                  rule.enabled
                    ? 'bg-slate-900/60 border-slate-800 hover:border-slate-700'
                    : 'bg-slate-950/40 border-slate-900 opacity-60'
                }`}
              >
                <div className="flex items-center gap-3 min-w-0">
                  <button
                    onClick={() => handleToggleRule(rule.id)}
                    className={`w-8 h-4 rounded-full p-0.5 transition-colors duration-200 ease-in-out shrink-0 ${
                      rule.enabled ? 'bg-indigo-600' : 'bg-slate-700'
                    }`}
                  >
                    <div
                      className={`w-3 h-3 rounded-full bg-white transition-transform duration-200 ease-in-out ${
                        rule.enabled ? 'translate-x-4' : 'translate-x-0'
                      }`}
                    />
                  </button>

                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-semibold text-white truncate">{rule.name}</span>
                      <span className="text-[11px] font-mono text-indigo-300 bg-indigo-950/60 border border-indigo-800/40 px-2 py-0.5 rounded truncate">
                        {rule.app_name}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-3 shrink-0">
                  <div className="flex items-center gap-1.5 bg-slate-800/80 border border-slate-700/60 px-2.5 py-1 rounded-lg text-xs text-indigo-200">
                    <Layers className="w-3.5 h-3.5 text-indigo-400" />
                    <span className="font-semibold">{getLayerLabel(rule.target_layer)}</span>
                  </div>

                  <button
                    onClick={() => handleDeleteRule(rule.id)}
                    className="p-1.5 text-slate-500 hover:text-rose-400 hover:bg-rose-500/10 rounded-lg transition-colors"
                    title="ルールを削除"
                  >
                    <Trash2 className="w-4 h-4" />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};
