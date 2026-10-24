import { useCallback, useEffect, useRef, useState } from 'react';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { desktop } from './api';
import { version } from '../package.json';

type UpdateStatus = 'idle' | 'checking' | 'current' | 'available' | 'downloading' | 'installing' | 'error';

export function useUpdater() {
  const [status, setStatus] = useState<UpdateStatus>('idle');
  const [update, setUpdate] = useState<Update | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [progress, setProgress] = useState<number | null>(null);
  const resource = useRef<Update | null>(null);
  const locked = useRef(false);

  const checkNow = useCallback(async () => {
    if (!desktop || locked.current) return;
    locked.current = true;
    setStatus('checking'); setError(null);
    try {
      if (resource.current) await resource.current.close();
      resource.current = null; setUpdate(null);
      const available = await check({ timeout: 15000 });
      resource.current = available; setUpdate(available);
      setStatus(available ? 'available' : 'current');
    } catch (cause) {
      setError(`检查更新失败，请确认能够访问 GitHub 后重试。${String(cause)}`);
      setStatus('error');
    } finally { locked.current = false; }
  }, []);

  async function install() {
    const available = resource.current;
    if (!available || locked.current) return;
    locked.current = true;
    setError(null); setProgress(null); setStatus('downloading');
    let downloaded = 0;
    let total = 0;
    try {
      // 官方插件在 download 返回前校验签名；只有校验通过才启动安装器。
      await available.download(event => {
        if (event.event === 'Started') total = event.data.contentLength ?? 0;
        if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          setProgress(total ? Math.min(100, Math.round(downloaded * 100 / total)) : null);
        }
      }, { timeout: 120000 });
      setProgress(100); setStatus('installing');
      // Windows 安装器接管进程，更新后自动启动应用。
      await available.install({ restartAfterInstall: true });
    } catch (cause) {
      setError(`更新未完成，可重试下载。${String(cause)}`);
      setStatus('error');
    } finally { locked.current = false; }
  }

  useEffect(() => {
    const timer = window.setTimeout(() => { void checkNow(); }, 1500);
    return () => window.clearTimeout(timer);
  }, [checkNow]);

  return { version, status, update, error, progress, checkNow, install,
    busy: status === 'checking' || status === 'downloading' || status === 'installing' };
}
