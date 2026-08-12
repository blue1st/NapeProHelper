import React, { useEffect, useState, useCallback } from 'react';
import {
  Github,
  ExternalLink,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Sparkles,
  Download,
  ShieldCheck,
  Cpu,
  Tag,
  Calendar,
  MessageSquare,
  Globe,
  Heart,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';
import packageJson from '../../package.json';

export const CURRENT_VERSION = packageJson.version;
export const REPO_URL = 'https://github.com/blue1st/NapeProHelper';

export const RELEASES_URL = 'https://github.com/blue1st/NapeProHelper/releases';
export const ISSUES_URL = 'https://github.com/blue1st/NapeProHelper/issues';
export const KEYCHRON_LAUNCHER_URL = 'https://launcher.keychron.com/';
export const GITHUB_API_LATEST_RELEASE = 'https://api.github.com/repos/blue1st/NapeProHelper/releases/latest';

export interface ReleaseInfo {
  version: string;
  tagName: string;
  name: string;
  body: string;
  htmlUrl: string;
  publishedAt: string;
}

interface AboutAppProps {
  onUpdateDetected?: (release: ReleaseInfo) => void;
}

/**
 * Compare two semver strings (e.g. "1.0.10" vs "1.0.11").
 * Returns > 0 if v2 is newer than v1, 0 if equal, < 0 if v1 is newer.
 */
export function compareSemver(v1: string, v2: string): number {
  const clean1 = v1.replace(/^v/i, '').trim();
  const clean2 = v2.replace(/^v/i, '').trim();

  const parts1 = clean1.split('.').map((n) => parseInt(n, 10) || 0);
  const parts2 = clean2.split('.').map((n) => parseInt(n, 10) || 0);

  const maxLen = Math.max(parts1.length, parts2.length);
  for (let i = 0; i < maxLen; i++) {
    const num1 = parts1[i] ?? 0;
    const num2 = parts2[i] ?? 0;
    if (num2 > num1) return 1;
    if (num1 > num2) return -1;
  }
  return 0;
}

export const AboutApp: React.FC<AboutAppProps> = ({ onUpdateDetected }) => {
  const [appVersion, setAppVersion] = useState<string>(CURRENT_VERSION);
  const [status, setStatus] = useState<'idle' | 'checking' | 'up-to-date' | 'update-available' | 'error'>('idle');
  const [latestRelease, setLatestRelease] = useState<ReleaseInfo | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [lastCheckedAt, setLastCheckedAt] = useState<Date | null>(null);

  useEffect(() => {
    getVersion()
      .then((ver) => {
        if (ver) setAppVersion(ver);
      })
      .catch(() => {
        // Fallback to package.json version
      });
  }, []);

  const openExternalUrl = async (url: string) => {
    try {
      await invoke('open_url', { url });
    } catch {
      window.open(url, '_blank');
    }
  };

  const checkForUpdates = useCallback(async () => {
    setStatus('checking');
    setErrorMsg(null);

    try {
      const res = await fetch(GITHUB_API_LATEST_RELEASE, {
        headers: {
          Accept: 'application/vnd.github.v3+json',
        },
      });

      if (!res.ok) {
        if (res.status === 404) {
          // No release published yet on GitHub
          setStatus('up-to-date');
          setLastCheckedAt(new Date());
          return;
        }
        throw new Error(`GitHub API HTTP ${res.status}`);
      }

      const data = await res.json();
      const tagName = data.tag_name || '';
      const version = tagName.replace(/^v/i, '');

      const releaseInfo: ReleaseInfo = {
        version,
        tagName,
        name: data.name || tagName,
        body: data.body || '',
        htmlUrl: data.html_url || RELEASES_URL,
        publishedAt: data.published_at ? new Date(data.published_at).toLocaleDateString('ja-JP') : '',
      };

      setLatestRelease(releaseInfo);
      setLastCheckedAt(new Date());

      const isNewer = compareSemver(appVersion, version) > 0;
      if (isNewer) {
        setStatus('update-available');
        if (onUpdateDetected) {
          onUpdateDetected(releaseInfo);
        }
      } else {
        setStatus('up-to-date');
      }
    } catch (err: any) {
      console.warn('Failed to check updates from GitHub Releases API:', err);
      setStatus('error');
      setErrorMsg(err.message || '更新の確認中にエラーが発生しました');
      setLastCheckedAt(new Date());
    }
  }, [appVersion, onUpdateDetected]);

  useEffect(() => {
    checkForUpdates();
  }, [checkForUpdates]);

  return (
    <div className="space-y-6 max-w-4xl mx-auto pb-8">
      {/* Hero / App Branding Card */}
      <div className="relative overflow-hidden bg-gradient-to-br from-slate-900 via-slate-900/90 to-indigo-950/40 border border-slate-800 rounded-3xl p-6 md:p-8 shadow-2xl backdrop-blur-md">
        <div className="absolute -right-12 -bottom-12 w-64 h-64 bg-indigo-500/10 rounded-full blur-3xl pointer-events-none" />
        <div className="relative z-10 flex flex-col md:flex-row items-start md:items-center justify-between gap-6">
          <div className="flex items-start gap-4">
            <div className="w-16 h-16 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-2xl flex items-center justify-center text-white shadow-lg shadow-indigo-500/25 shrink-0 border border-indigo-400/30">
              <Cpu className="w-9 h-9" />
            </div>
            <div className="space-y-1">
              <div className="flex items-center gap-2 flex-wrap">
                <h1 className="text-2xl font-bold text-white tracking-tight">Nape Pro Helper</h1>
                <span className="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                  v{appVersion}
                </span>
                <span className="px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-800 text-slate-400 border border-slate-700">
                  MIT License
                </span>
              </div>
              <p className="text-xs md:text-sm text-slate-300 leading-relaxed max-w-xl">
                Keychron Nape Pro キーボード・トラックボールのオクタシフト角度・感度リアルタイム視覚化＆設定管理ヘルパー
              </p>
            </div>
          </div>

          <button
            onClick={() => openExternalUrl(REPO_URL)}
            className="flex items-center gap-2 px-4 py-2.5 bg-slate-800/80 hover:bg-slate-700 text-slate-100 rounded-xl text-xs font-semibold border border-slate-700 hover:border-slate-600 transition-all shadow-md group shrink-0"
          >
            <Github className="w-4 h-4 text-indigo-400 group-hover:scale-110 transition-transform" />
            <span>GitHub リポジトリ</span>
            <ExternalLink className="w-3.5 h-3.5 text-slate-400 ml-1" />
          </button>
        </div>
      </div>

      {/* Version Status & Release Check Card */}
      <div className="bg-slate-900/60 border border-slate-800 rounded-2xl p-6 space-y-4 shadow-xl">
        <div className="flex items-center justify-between border-b border-slate-800/80 pb-4">
          <div className="flex items-center gap-2.5">
            <div className="p-2 bg-indigo-500/10 text-indigo-400 rounded-lg">
              <Sparkles className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-sm font-bold text-slate-100">バージョン &amp; アップデート確認</h2>
              <p className="text-xs text-slate-400">GitHub Releases API を使用して最新のリリースを確認します</p>
            </div>
          </div>

          <button
            onClick={checkForUpdates}
            disabled={status === 'checking'}
            className="flex items-center gap-1.5 px-3.5 py-2 bg-indigo-600/90 hover:bg-indigo-500 disabled:bg-slate-800 disabled:text-slate-500 text-white rounded-xl text-xs font-semibold transition-all shadow-md disabled:cursor-not-allowed"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${status === 'checking' ? 'animate-spin' : ''}`} />
            <span>{status === 'checking' ? '確認中...' : '更新を確認'}</span>
          </button>
        </div>

        {/* Status display logic */}
        {status === 'checking' && (
          <div className="bg-slate-950/40 border border-slate-800/60 rounded-xl p-4 flex items-center gap-3">
            <RefreshCw className="w-5 h-5 text-indigo-400 animate-spin shrink-0" />
            <div className="text-xs text-slate-300">
              <p className="font-semibold">GitHub から最新リリース情報を取得中...</p>
              <p className="text-slate-500 text-[11px]">最新のバージョン情報を照会しています</p>
            </div>
          </div>
        )}

        {status === 'up-to-date' && (
          <div className="bg-emerald-950/30 border border-emerald-500/30 rounded-xl p-4 flex items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <CheckCircle2 className="w-5 h-5 text-emerald-400 shrink-0" />
              <div className="text-xs">
                <p className="font-bold text-emerald-300">最新バージョンを使用しています</p>
                <p className="text-slate-400 text-[11px]">
                  現在お使いの v{appVersion} は最新版です。
                  {lastCheckedAt && ` (最終確認: ${lastCheckedAt.toLocaleTimeString('ja-JP')})`}
                </p>
              </div>
            </div>
            <button
              onClick={() => openExternalUrl(RELEASES_URL)}
              className="text-xs text-emerald-400 hover:text-emerald-300 font-medium flex items-center gap-1 underline underline-offset-2 shrink-0"
            >
              <span>過去のリリース一覧</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>
        )}

        {status === 'update-available' && latestRelease && (
          <div className="relative overflow-hidden bg-gradient-to-r from-indigo-950/60 via-slate-900 to-purple-950/40 border-2 border-indigo-500/50 rounded-2xl p-5 space-y-4 shadow-xl">
            <div className="flex items-start justify-between gap-4">
              <div className="flex items-start gap-3">
                <div className="p-2.5 bg-indigo-500/20 text-indigo-300 rounded-xl border border-indigo-400/30 shrink-0">
                  <Sparkles className="w-6 h-6 animate-pulse text-indigo-400" />
                </div>
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="px-2 py-0.5 rounded-full text-[10px] font-extrabold uppercase bg-emerald-500/20 text-emerald-300 border border-emerald-500/40 animate-bounce">
                      NEW
                    </span>
                    <h3 className="text-base font-bold text-white">新しいバージョンが利用可能です！</h3>
                  </div>
                  <p className="text-xs text-slate-300">
                    最新バージョン <strong className="text-indigo-300">v{latestRelease.version}</strong> がリリースされています。（現在のバージョン: v{appVersion}）
                  </p>
                </div>
              </div>

              <button
                onClick={() => openExternalUrl(latestRelease.htmlUrl)}
                className="flex items-center gap-2 px-4 py-2.5 bg-gradient-to-r from-indigo-600 to-purple-600 hover:from-indigo-500 hover:to-purple-500 text-white rounded-xl text-xs font-bold transition-all shadow-lg shadow-indigo-500/30 shrink-0"
              >
                <Download className="w-4 h-4" />
                <span>最新版をダウンロード</span>
                <ExternalLink className="w-3.5 h-3.5 ml-0.5" />
              </button>
            </div>

            {/* Release metadata & notes */}
            <div className="bg-slate-950/60 border border-slate-800 rounded-xl p-4 space-y-2 text-xs">
              <div className="flex items-center justify-between text-slate-400 border-b border-slate-800 pb-2">
                <div className="flex items-center gap-2">
                  <Tag className="w-3.5 h-3.5 text-indigo-400" />
                  <span className="font-semibold text-slate-200">{latestRelease.name}</span>
                </div>
                {latestRelease.publishedAt && (
                  <div className="flex items-center gap-1.5 text-slate-400 text-[11px]">
                    <Calendar className="w-3 h-3" />
                    <span>リリース日: {latestRelease.publishedAt}</span>
                  </div>
                )}
              </div>
              {latestRelease.body && (
                <div className="text-slate-300 text-xs leading-relaxed max-h-32 overflow-y-auto whitespace-pre-wrap pt-1 font-mono text-[11px] bg-slate-900/40 p-2.5 rounded-lg border border-slate-800/80">
                  {latestRelease.body}
                </div>
              )}
            </div>
          </div>
        )}

        {status === 'error' && (
          <div className="bg-amber-950/30 border border-amber-500/30 rounded-xl p-4 flex items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <AlertCircle className="w-5 h-5 text-amber-400 shrink-0" />
              <div className="text-xs">
                <p className="font-bold text-amber-300">更新の確認に失敗しました</p>
                <p className="text-slate-400 text-[11px]">
                  {errorMsg || 'GitHub API からの応答を取得できませんでした。'}
                </p>
              </div>
            </div>
            <button
              onClick={() => openExternalUrl(RELEASES_URL)}
              className="text-xs text-amber-400 hover:text-amber-300 font-medium flex items-center gap-1 underline underline-offset-2 shrink-0"
            >
              <span>GitHub で直接確認</span>
              <ExternalLink className="w-3 h-3" />
            </button>
          </div>
        )}
      </div>

      {/* Resource & Link Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* GitHub Repository */}
        <div
          onClick={() => openExternalUrl(REPO_URL)}
          className="bg-slate-900/50 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 cursor-pointer transition-all duration-200 group flex items-start gap-4 shadow-lg"
        >
          <div className="p-3 bg-slate-800 text-indigo-400 group-hover:text-indigo-300 group-hover:bg-indigo-500/20 rounded-xl border border-slate-700 transition-colors shrink-0">
            <Github className="w-6 h-6" />
          </div>
          <div className="space-y-1 flex-1">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-bold text-white group-hover:text-indigo-300 transition-colors">
                GitHub Repository
              </h3>
              <ExternalLink className="w-3.5 h-3.5 text-slate-500 group-hover:text-slate-300 transition-colors" />
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              ソースコードの閲覧、開発への貢献、スター登録はこちらから。
            </p>
          </div>
        </div>

        {/* Release Notes */}
        <div
          onClick={() => openExternalUrl(RELEASES_URL)}
          className="bg-slate-900/50 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 cursor-pointer transition-all duration-200 group flex items-start gap-4 shadow-lg"
        >
          <div className="p-3 bg-slate-800 text-purple-400 group-hover:text-purple-300 group-hover:bg-purple-500/20 rounded-xl border border-slate-700 transition-colors shrink-0">
            <Tag className="w-6 h-6" />
          </div>
          <div className="space-y-1 flex-1">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-bold text-white group-hover:text-purple-300 transition-colors">
                リリース一覧 &amp; 変更履歴
              </h3>
              <ExternalLink className="w-3.5 h-3.5 text-slate-500 group-hover:text-slate-300 transition-colors" />
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              過去の全バージョンの変更点やビルド済みインストーラーのダウンロード。
            </p>
          </div>
        </div>

        {/* Issue & Feedback */}
        <div
          onClick={() => openExternalUrl(ISSUES_URL)}
          className="bg-slate-900/50 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 cursor-pointer transition-all duration-200 group flex items-start gap-4 shadow-lg"
        >
          <div className="p-3 bg-slate-800 text-emerald-400 group-hover:text-emerald-300 group-hover:bg-emerald-500/20 rounded-xl border border-slate-700 transition-colors shrink-0">
            <MessageSquare className="w-6 h-6" />
          </div>
          <div className="space-y-1 flex-1">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-bold text-white group-hover:text-emerald-300 transition-colors">
                不具合報告・機能要望
              </h3>
              <ExternalLink className="w-3.5 h-3.5 text-slate-500 group-hover:text-slate-300 transition-colors" />
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              動作の不具合や新機能のご提案は GitHub Issues にお寄せください。
            </p>
          </div>
        </div>

        {/* Keychron Launcher */}
        <div
          onClick={() => openExternalUrl(KEYCHRON_LAUNCHER_URL)}
          className="bg-slate-900/50 hover:bg-slate-900 border border-slate-800 hover:border-slate-700 rounded-2xl p-5 cursor-pointer transition-all duration-200 group flex items-start gap-4 shadow-lg"
        >
          <div className="p-3 bg-slate-800 text-amber-400 group-hover:text-amber-300 group-hover:bg-amber-500/20 rounded-xl border border-slate-700 transition-colors shrink-0">
            <Globe className="w-6 h-6" />
          </div>
          <div className="space-y-1 flex-1">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-bold text-white group-hover:text-amber-300 transition-colors">
                Keychron Launcher (公式Web)
              </h3>
              <ExternalLink className="w-3.5 h-3.5 text-slate-500 group-hover:text-slate-300 transition-colors" />
            </div>
            <p className="text-xs text-slate-400 leading-relaxed">
              Keychron 公式 Web VIA ツールを開いて、全キーマップ・マクロを編集。
            </p>
          </div>
        </div>
      </div>

      {/* System Technical Specifications Info */}
      <div className="bg-slate-900/40 border border-slate-800/80 rounded-2xl p-5 space-y-3">
        <div className="flex items-center gap-2 text-slate-300 border-b border-slate-800 pb-2.5">
          <ShieldCheck className="w-4 h-4 text-indigo-400" />
          <h3 className="text-xs font-bold text-slate-200 uppercase tracking-wider">アプリ技術仕様 &amp; 環境</h3>
        </div>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 text-xs">
          <div>
            <span className="text-slate-500 text-[11px] block">アプリバージョン</span>
            <span className="font-semibold text-slate-200">v{appVersion}</span>
          </div>
          <div>
            <span className="text-slate-500 text-[11px] block">フレームワーク</span>
            <span className="font-semibold text-slate-200">Tauri v2 + React 19</span>
          </div>
          <div>
            <span className="text-slate-500 text-[11px] block">デバイス接続方式</span>
            <span className="font-semibold text-slate-200">USB / 2.4GHz (HID)</span>
          </div>
          <div>
            <span className="text-slate-500 text-[11px] block">オクタシフト対応</span>
            <span className="font-semibold text-slate-200">8方向角度リアルタイム同期</span>
          </div>
        </div>
      </div>

      {/* Footer copyright / credit */}
      <div className="text-center pt-2 text-xs text-slate-500 flex items-center justify-center gap-1.5">
        <span>Made with</span>
        <Heart className="w-3.5 h-3.5 text-rose-500 fill-rose-500/20" />
        <span>for Keychron Nape Pro users &bull; &copy; {new Date().getFullYear()} Nape Pro Helper</span>
      </div>
    </div>
  );
};

export default AboutApp;
