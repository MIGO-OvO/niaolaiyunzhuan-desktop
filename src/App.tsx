import { useEffect, useRef, useState } from 'react';
import type { ComponentType, ReactNode } from 'react';
import { AlertCircle, ArchiveRestore, ArrowLeft, ArrowUpRight, Check, CheckCheck, ChevronRight, CircleHelp, Clock3, Copy, Dices, Download, FileText, Gamepad2, Info, LoaderCircle, Map, MapPin, MessageCircle, Minus, PanelRightClose, RotateCcw, Search, Settings2, ShieldCheck, Square, UsersRound, X } from 'lucide-react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { api, desktop } from './api';
import { useUpdater } from './useUpdater';
import { countdown, dateLabel, matchingAtoms, periodLabel, statsFor } from './model';
import type { ItemView, ModuleId, RestorePreview, Snapshot, Stats, StatusFilter } from './types';

const modules: { id: ModuleId; title: string; description: string; icon: ComponentType<{ size?: number }> }[] = [
  { id: 'region', title: '区域探索', description: '行走山河，拾音入册', icon: Map },
  { id: 'gameplay', title: '玩法获得', description: '江湖百业，各有所获', icon: Gamepad2 },
  { id: 'limited', title: '限时获得', description: '记下每一期的江湖之约', icon: Clock3 },
  { id: 'chance', title: '概率获得', description: '偶有所获，也是好光景', icon: Dices },
];

type Dialog = { kind: 'settings' } | { kind: 'help' } | { kind: 'restore'; preview: RestorePreview } | { kind: 'reset'; module: ModuleId | null };

function CheckBox({ checked, mixed = false, label, disabled, onChange }: { checked: boolean; mixed?: boolean; label: string; disabled?: boolean; onChange: () => void }) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => { if (ref.current) ref.current.indeterminate = mixed; }, [mixed]);
  return <span className={`check-box ${mixed ? 'is-mixed' : ''}`}>
    <input ref={ref} type="checkbox" aria-label={label} checked={checked} disabled={disabled} onChange={onChange} />
    <span aria-hidden="true">{mixed ? <Minus size={15} /> : checked ? <Check size={16} /> : null}</span>
  </span>;
}

function SourceIcon({ item, size = 22 }: { item: ItemView; size?: number }) {
  const Icon = item.type === '故人' ? UsersRound : item.displayTitle.includes('奇遇') ? MessageCircle : item.displayTitle.includes('探索') ? MapPin : item.module === 'chance' ? Dices : FileText;
  return <Icon size={size} strokeWidth={1.7} aria-hidden="true" />;
}

function ProgressTrack({ stats }: { stats: Stats }) {
  return <span className="progress-track" aria-hidden="true">
    {Array.from({ length: stats.totalTasks }, (_, index) => <i key={index} className={index < stats.doneTasks ? 'filled' : ''} />)}
  </span>;
}

function Modal({ title, children, onClose, error }: { title: string; children: ReactNode; onClose: () => void; error?: string | null }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const dialog = ref.current!;
    dialog.showModal();
    return () => dialog.close();
  }, []);
  return <dialog ref={ref} className="modal" onCancel={onClose} onClick={event => { if (event.target === event.currentTarget) onClose(); }}>
    <div className="modal-heading"><h2>{title}</h2><button className="icon-button" aria-label="关闭对话框" onClick={onClose}><X size={20} /></button></div>
    {error && <p className="modal-error" role="alert">{error}</p>}
    {children}
  </dialog>;
}

export default function App() {
  const updater = useUpdater();
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [module, setModule] = useState<ModuleId>('region');
  const [region, setRegion] = useState('清河');
  const [selectedId, setSelectedId] = useState('src-005');
  const [query, setQuery] = useState('');
  const [filter, setFilter] = useState<StatusFilter>('all');
  const [busy, setBusy] = useState(false);
  const [initialLoading, setInitialLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [dialog, setDialog] = useState<Dialog | null>(null);
  const [now, setNow] = useState(Date.now());
  const [detailOpen, setDetailOpen] = useState(false);
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);
  const pending = useRef(false);
  const searchRef = useRef<HTMLInputElement>(null);

  function accept(value: Snapshot) {
    setSnapshot(previous => !previous || value.savedAt >= previous.savedAt ? value : previous);
    setNow(value.now);
  }

  async function refresh() {
    if (pending.current) return;
    pending.current = true;
    try { accept(await api.snapshot()); setError(null); }
    catch (cause) { setError(String(cause)); }
    finally { pending.current = false; setInitialLoading(false); }
  }

  useEffect(() => {
    void refresh();
    const timer = window.setInterval(() => { setNow(Date.now()); void refresh(); }, 60000);
    const focus = () => { void refresh(); };
    const shortcut = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault(); searchRef.current?.focus();
      }
    };
    window.addEventListener('focus', focus);
    window.addEventListener('keydown', shortcut);
    return () => { window.clearInterval(timer); window.removeEventListener('focus', focus); window.removeEventListener('keydown', shortcut); };
  }, []);

  useEffect(() => {
    if (!desktop) return;
    const window = getCurrentWindow();
    let disposed = false;
    const subscriptions = Promise.all([
      window.onResized(() => { void window.isMaximized().then(setMaximized); }),
      window.onFocusChanged(event => setFocused(event.payload)),
    ]);
    void window.isMaximized().then(setMaximized);
    void subscriptions.then(unlisten => { if (disposed) unlisten.forEach(stop => stop()); }).catch(cause => setError(String(cause)));
    return () => { disposed = true; void subscriptions.then(unlisten => unlisten.forEach(stop => stop())); };
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timer = window.setTimeout(() => setNotice(null), 5500);
    return () => window.clearTimeout(timer);
  }, [notice]);

  async function perform(action: () => Promise<void>) {
    if (pending.current) return;
    pending.current = true; setBusy(true); setError(null);
    try { await action(); }
    catch (cause) { setError(String(cause)); }
    finally { pending.current = false; setBusy(false); }
  }

  function check(atomIds: string[], checked: boolean) {
    void perform(async () => { accept(await api.check(atomIds, checked)); });
  }

  function changeModule(value: ModuleId) {
    setModule(value); setQuery(''); setFilter('all'); setDetailOpen(false);
    const first = snapshot?.items.find(item => item.module === value && (value !== 'region' || item.region === region));
    if (first) setSelectedId(first.id);
  }

  const allItems = snapshot?.items ?? [];
  const regions = [...new Set(allItems.filter(item => item.module === 'region').map(item => item.region))];
  const scoped = allItems.filter(item => query.trim() ? true : item.module === module && (module !== 'region' || item.region === region));
  const sourceItems = scoped.filter(item => matchingAtoms(item, query, 'all').length > 0);
  const visibleItems = sourceItems.filter(item => matchingAtoms(item, query, filter).length > 0)
    .sort((a, b) => a.regionTypeRank - b.regionTypeRank || a.sourceSerial - b.sourceSerial);
  const stats = statsFor(sourceItems.map(item => ({ ...item, atoms: matchingAtoms(item, query, 'all') })));
  const selected = visibleItems.find(item => item.id === selectedId) ?? visibleItems[0];
  const selectedDone = selected?.atoms.filter(atom => atom.checked).length ?? 0;
  const selectedAtoms = selected ? matchingAtoms(selected, query, filter) : [];
  const moduleInfo = modules.find(value => value.id === module)!;
  const pageTitle = query.trim() ? '搜索结果' : module === 'region' ? region : moduleInfo.title;
  const nearestEvent = allItems.filter(item => item.module === 'limited' && item.countdownEnd && new Date(item.countdownEnd).getTime() > now)
    .sort((a, b) => new Date(a.countdownEnd!).getTime() - new Date(b.countdownEnd!).getTime())[0];

  async function windowAction(action: 'minimize' | 'toggleMaximize' | 'close') {
    try { await getCurrentWindow()[action](); } catch (cause) { setError(String(cause)); }
  }

  return <div className="app-shell" data-window-focused={focused}>
    <aside className="sidebar" aria-label="来源分类与区域">
      <div className="brand" data-tauri-drag-region="deep">
        <img src="/assets/swallow-mark.png" alt="" width="53" height="53" draggable="false" />
        <div><strong>袅来运转</strong><span>Windows 桌面版</span></div>
      </div>
      <nav className="module-nav" aria-label="来源分类">
        {modules.map(value => <button key={value.id} className={`module-button ${module === value.id && !query.trim() ? 'is-active' : ''}`} onClick={() => changeModule(value.id)} aria-current={module === value.id && !query.trim() ? 'page' : undefined}>
          <value.icon size={23} /><span>{value.title}</span>
          {!!snapshot?.modules[value.id].newCount && <span className="nav-count">{snapshot.modules[value.id].newCount}</span>}
        </button>)}
      </nav>
      {nearestEvent && <div className="sidebar-event"><span className="event-dot" /><span>{nearestEvent.displayTitle} · {dateLabel(nearestEvent.countdownEnd!, true)} 结束</span></div>}
      <div className="sidebar-rule" />
      <div className="region-scroll">
        <div className="region-label">江湖行迹<span>已获 / 全部</span></div>
        <nav className="region-nav" aria-label="区域选择">
          {regions.map(value => {
            const valueStats = statsFor(allItems.filter(item => item.module === 'region' && item.region === value));
            return <button key={value} className={`region-button ${module === 'region' && region === value && !query.trim() ? 'is-active' : ''}`} onClick={() => {
              setModule('region'); setRegion(value); setQuery(''); setFilter('all'); setDetailOpen(false);
              setSelectedId(allItems.find(item => item.module === 'region' && item.region === value)?.id ?? '');
            }} aria-current={module === 'region' && region === value && !query.trim() ? 'page' : undefined}>
              <span className="region-name">{value}</span><span className="region-number">{valueStats.doneTasks}<span>/{valueStats.totalTasks}</span></span>
              <ProgressTrack stats={valueStats} />
            </button>;
          })}
        </nav>
      </div>
      <div className="sidebar-bottom"><span>山河有迹，余音入册</span><div><button className="icon-button" title="使用说明" aria-label="使用说明" onClick={() => setDialog({ kind: 'help' })}><CircleHelp size={18} /></button><button className="icon-button" title="设置与更新" aria-label="设置与更新" onClick={() => setDialog({ kind: 'settings' })}><Settings2 size={18} /></button></div></div>
    </aside>

    <header className="toolbar" data-tauri-drag-region="deep" aria-label="自定义标题栏">
      <label className="search-field"><Search size={20} /><input ref={searchRef} aria-label="搜索任务、人物或商店" placeholder="搜索任务、人物或商店" value={query} onChange={event => { setQuery(event.target.value); setDetailOpen(false); }} />
        {query ? <button className="icon-button" onClick={() => { setQuery(''); searchRef.current?.focus(); }} aria-label="清除搜索"><X size={16} /></button> : <kbd>Ctrl K</kbd>}
      </label>
      <div className="toolbar-actions">
        <button className="outline-button" disabled={busy || !snapshot} title={snapshot?.backupCurrent ? '当前进度已备份' : '导出兼容网页版的 PNG 备份'} onClick={() => void perform(async () => {
          const result = await api.export();
          if (result) { accept(result.snapshot); setNotice(`备份已保存：${result.path}`); }
        })}><Download size={18} /><span>备份记录</span>{snapshot?.backupCurrent && <i className="backup-dot" />}</button>
        <button className="outline-button" disabled={busy || !snapshot} onClick={() => void perform(async () => {
          const preview = await api.preview(); if (preview) setDialog({ kind: 'restore', preview });
        })}><ArchiveRestore size={18} /><span>恢复记录</span></button>
      </div>
      <span className={`save-state ${error ? 'has-error' : ''}`} role="status">{busy ? <LoaderCircle className="spinning" size={19} /> : error ? <AlertCircle size={19} /> : <ShieldCheck size={21} />}<span>{updater.status === 'downloading' ? '正在下载更新' : updater.status === 'installing' ? '正在安装更新' : busy ? '正在保存' : error ? '操作未完成' : snapshot ? '本地存档 · 离线可用' : '正在读取存档'}</span></span>
      <div className="window-controls" aria-label="窗口控制"><button aria-label="最小化窗口" title="最小化" disabled={!desktop} onClick={() => void windowAction('minimize')}><Minus size={17} /></button><button aria-label={maximized ? '还原窗口' : '最大化窗口'} title={maximized ? '还原' : '最大化'} disabled={!desktop} onClick={() => void windowAction('toggleMaximize')}>{maximized ? <Copy size={14} /> : <Square size={14} />}</button><button className="window-close" aria-label="关闭窗口" title="关闭" disabled={!desktop} onClick={() => void windowAction('close')}><X size={19} /></button></div>
    </header>

    <main className={`workspace ${detailOpen ? 'detail-is-open' : ''}`}>
      {(error || snapshot?.warning) && <div className={`message-banner ${error ? 'error-banner' : ''}`} role="alert"><AlertCircle size={18} /><span>{error ?? snapshot?.warning}</span>{error && <button onClick={() => setError(null)} className="icon-button" aria-label="关闭错误提示"><X size={17} /></button>}</div>}
      <section className="source-panel" aria-label={`${pageTitle}来源列表`}>
        <div className="page-heading">
          <div className="page-title-line"><h1>{pageTitle}</h1><div className="scope-stats"><span><strong>{stats.doneTasks}</strong>/{stats.totalTasks} 项已完成</span><span className="stat-separator">·</span><span><strong>{stats.doneSounds}</strong>/{stats.totalSounds} 枚</span></div></div>
          <div className="page-subline"><span>{query.trim() ? `在全部来源中查找「${query.trim()}」` : moduleInfo.description}</span><span className="overall-stats">确定来源 <b>{snapshot?.overall.doneSounds ?? 0}</b><span> / {snapshot?.overall.totalSounds ?? 0} 枚</span></span></div>
        </div>
        <div className="list-tools"><div className="filter-tabs" role="group" aria-label="完成状态筛选">
          {([{ id: 'all', label: '全部', count: stats.totalTasks }, { id: 'todo', label: '未获得', count: stats.totalTasks - stats.doneTasks }, { id: 'done', label: '已获得', count: stats.doneTasks }] as const).map(value => <button key={value.id} aria-pressed={filter === value.id} className={filter === value.id ? 'is-active' : ''} onClick={() => setFilter(value.id)}>{value.label}<span>{value.count}</span></button>)}
        </div>{(snapshot?.modules[module].newCount ?? 0) > 0 && <button className="text-button" disabled={busy} onClick={() => void perform(async () => { accept(await api.seen(module)); })}><CheckCheck size={15} />已阅新增</button>}</div>
        <div className="list-column-head"><span>状态</span><span>任务名称</span><span>相关提示</span></div>
        <div className="source-list">
          {initialLoading ? <div className="loading-list" aria-label="正在读取来源" aria-busy="true">{Array.from({ length: 6 }, (_, index) => <div className="skeleton-row" key={index}><i /><span /><span /></div>)}</div> : !snapshot ? <div className="empty-state"><AlertCircle size={30} /><h2>存档未能打开</h2><p>检查上方提示，处理后重新读取。</p><button className="outline-button" onClick={() => void refresh()}><RotateCcw size={16} />重新读取</button></div> : visibleItems.length === 0 ? <div className="empty-state"><Search size={30} /><h2>{filter === 'todo' && sourceItems.length ? '这一页，已尽数入册' : filter === 'done' && sourceItems.length ? '从第一枚开始' : '没有找到相关来源'}</h2><p>{sourceItems.length ? '切换到“全部”可以查看这里的所有来源。' : '试试任务名称、人物姓名，或换一个关键词。'}</p><button className="text-button" onClick={() => { setFilter('all'); setQuery(''); }}>查看全部来源<ArrowUpRight size={16} /></button></div> : visibleItems.map(item => {
            const done = item.atoms.filter(atom => atom.checked).length;
            const complete = done === item.atoms.length;
            const active = item.id === selected?.id;
            return <div className={`source-row ${active ? 'is-selected' : ''} ${complete ? 'is-complete' : ''}`} key={item.id}>
              <span className="row-check"><CheckBox checked={complete} mixed={done > 0 && !complete} disabled={busy} label={`${complete ? '取消完成' : '标记完成'}：${item.displayTitle}`} onChange={() => check(item.atoms.map(atom => atom.id), !complete)} /></span>
              <button className="source-select" onClick={() => { setSelectedId(item.id); setDetailOpen(true); }} aria-label={`查看${item.displayTitle}详情`} aria-pressed={active}>
                <span className="source-name"><SourceIcon item={item} /><span>{item.displayTitle}</span>{item.atoms.some(atom => atom.unseen) && <span className="new-mark">新</span>}{item.children.length > 0 && <span className="child-count">{done}/{item.atoms.length}</span>}</span>
                <span className="source-note">{query.trim() ? `${item.module === 'region' ? item.region : modules.find(value => value.id === item.module)!.title} · ` : ''}{item.note || `${item.type} · ${periodLabel(item.period)}`}</span>
                <ChevronRight className="row-chevron" size={17} />
              </button>
            </div>;
          })}
          {!!visibleItems.length && <div className="list-end"><span />{visibleItems.length} 个来源 · {visibleItems.reduce((total, item) => total + matchingAtoms(item, query, filter).length, 0)} 个独立目标<span /></div>}
        </div>
        {snapshot && <div className="panel-summary"><span><MapPin size={15} />{query.trim() ? '全部来源' : module === 'region' ? `${region} · 区域探索` : moduleInfo.title}</span><span>确定来源 {snapshot.overall.doneTasks}/{snapshot.overall.totalTasks} 项</span></div>}
      </section>

      <aside className="detail-panel" aria-label="来源详情">
        <div className="detail-topbar"><button className="text-button" onClick={() => setDetailOpen(false)}><ArrowLeft size={17} />返回列表</button><button className="icon-button" aria-label="收起详情" onClick={() => setDetailOpen(false)}><PanelRightClose size={18} /></button></div>
        {selected ? <>
          <div className="detail-heading"><span className="detail-source-icon"><SourceIcon item={selected} size={29} /></span><div><h2>{selected.displayTitle}</h2><p>{selected.type}<span>·</span>{periodLabel(selected.period)}</p><div className="detail-progress"><strong>{selectedDone}</strong>/{selected.atoms.length}<span>已完成</span>{selectedDone === selected.atoms.length && <CheckCheck size={18} />}</div></div></div>
          {(selected.countdownEnd || selected.refreshAt || selected.period === 'weekly' || selected.period === 'monthly') && <div className="period-notice"><Clock3 size={16} /><div>{selected.countdownEnd ? <><strong>{countdown(selected.countdownEnd, now)}</strong><span>{dateLabel(selected.countdownEnd, true)} 结束</span></> : selected.refreshAt ? <><strong>{periodLabel(selected.period)}来源</strong><span>{dateLabel(selected.refreshAt, true)} 本期结束</span></> : <span>{periodLabel(selected.period)} · 以北京时间计算</span>}</div></div>}
          <div className="atom-list">{selectedAtoms.map(atom => <label className={`atom-row ${atom.checked ? 'is-complete' : ''}`} key={atom.id}>
            <CheckBox checked={atom.checked} label={`${atom.checked ? '取消完成' : '标记完成'}：${atom.label}`} disabled={busy} onChange={() => check([atom.id], !atom.checked)} />
            <span className="atom-label">{selected.children.length ? atom.label : selected.displayTitle}{atom.unseen && <span className="new-mark">新</span>}</span><span className="atom-quantity">{atom.quantity === null ? (selected.children.length ? '1 枚' : '待确认') : `${atom.quantity} 枚`}</span>
          </label>)}</div>
          {selected.note && <div className="source-memo"><h3><Info size={15} />获取提示</h3><p>{selected.note}</p></div>}
          {selected.children.length > 0 && <div className="detail-actions"><button className="text-button" disabled={busy} onClick={() => check(selected.atoms.map(atom => atom.id), selectedDone !== selected.atoms.length)}><CheckCheck size={16} />{selectedDone === selected.atoms.length ? '取消整组完成' : '整组标记完成'}</button><span>共 {statsFor([selected]).totalSounds} 枚</span></div>}
          <div className="detail-footnote"><ShieldCheck size={15} />勾选后自动保存到本机</div>
        </> : <div className="empty-detail"><img src="/assets/swallow-mark.png" alt="" width="68" height="68" /><h2>寻一缕江湖余音</h2><p>选择左侧来源，查看获取条件与进度。</p></div>}
      </aside>
    </main>

    <footer className="statusbar"><span><span className={`status-dot ${error ? 'is-error' : ''}`} />{error ? '本次操作未完成' : updater.status === 'downloading' ? '正在下载并校验更新' : updater.status === 'installing' ? '更新安装后将自动启动' : busy ? '正在写入本机存档' : desktop ? '已保存到本机 · 离线可用' : '浏览器预览 · 独立存档'}</span>{updater.update && <button className="text-button" onClick={() => setDialog({ kind: 'settings' })}><Download size={14} />发现 v{updater.update.version}</button>}<span className="data-version">数据更新 {snapshot?.meta.updatedAt ?? '2026-10-24'}</span><span className="author">原作者 <b>薄荷小鹿</b>（燕云十六声）</span></footer>
    {notice && <div className="toast" role="status"><Check size={18} /><span>{notice}</span><button className="icon-button" aria-label="关闭通知" onClick={() => setNotice(null)}><X size={16} /></button></div>}

    {dialog?.kind === 'restore' && <Modal error={error} title="恢复备份记录" onClose={() => setDialog(null)}><p className="modal-intro">读取到 {dateLabel(dialog.preview.createdAt, true)} 的备份。</p><div className="restore-summary"><div><strong>{dialog.preview.completedCount}</strong><span>已完成目标</span></div><div><strong>{dialog.preview.addedCount}</strong><span>可合并的目标</span></div></div><p>恢复会合并备份里的已完成项，保留本机已有勾选。周期来源合并到当前周期。</p>{dialog.preview.unknownIds.length > 0 && <p className="warning-text">其中 {dialog.preview.unknownIds.length} 个旧目标已不在当前数据中，将跳过。</p>}<div className="modal-actions"><button className="outline-button" disabled={busy} onClick={() => setDialog(null)}>取消</button><button className="primary-button" disabled={busy} onClick={() => void perform(async () => { accept(await api.restore(dialog.preview.payload)); setDialog(null); setNotice('备份记录已合并到本机存档'); })}>{busy ? '正在合并' : '合并记录'}</button></div></Modal>}
    {dialog?.kind === 'settings' && <Modal error={error} title="设置与更新" onClose={() => setDialog(null)}>
      <div className="settings-section"><h3>程序更新 · v{updater.version}</h3>
        <p role="status">{!desktop ? '浏览器预览不安装更新，请在桌面程序中检查。' : updater.status === 'checking' ? '正在检查新版本…' : updater.status === 'downloading' ? `正在下载并校验签名${updater.progress === null ? '…' : ` · ${updater.progress}%`}` : updater.status === 'installing' ? '正在安装，完成后将自动启动程序。' : updater.update ? `发现新版本 v${updater.update.version}` : updater.status === 'current' ? '当前已是最新版本。' : '从 GitHub Release 获取新版本。'}</p>
        {updater.update?.body && <p className="update-notes">{updater.update.body}</p>}
        {updater.status === 'downloading' && <progress className="update-progress" aria-label="更新下载进度" value={updater.progress ?? undefined} max={100} />}
        {updater.error && <p className="modal-error" role="alert">{updater.error}</p>}
        <div className="update-actions"><button className="outline-button" disabled={!desktop || updater.busy || busy} onClick={() => void updater.checkNow()}>{updater.status === 'checking' ? <LoaderCircle className="spinning" size={16} /> : <RotateCcw size={16} />}检查更新</button>
          {updater.update && <button className="primary-button" disabled={updater.busy || busy} onClick={() => void perform(updater.install)}><Download size={16} />{updater.status === 'downloading' ? '正在下载' : updater.status === 'installing' ? '正在安装' : '下载并安装'}</button>}</div>
      </div>
      <div className="settings-section"><h3>存档位置</h3><p className="storage-path">{snapshot?.storagePath}</p><p>每次勾选都会保存，程序会保留上一份有效存档。</p></div><div className="settings-section"><h3>PNG 备份</h3><p>{snapshot?.backup.lastAt ? `上次备份：${dateLabel(snapshot.backup.lastAt, true)}${snapshot.backupCurrent ? '，已包含当前进度。' : '，之后的修改尚未备份。'}` : '尚未导出备份。点击顶部“备份记录”保存一张二维码图片。'}</p><p>支持袅来运转网页版的 NYNZ1、NYNZ2 备份。</p></div><div className="settings-section reset-section"><h3>重置进度</h3><div><button className="outline-button" onClick={() => setDialog({ kind: 'reset', module })}>重置{moduleInfo.title}</button><button className="text-button danger-text" onClick={() => setDialog({ kind: 'reset', module: null })}>重置全部记录</button></div></div></Modal>}
    {dialog?.kind === 'reset' && <Modal error={error} title={`重置${dialog.module ? modules.find(value => value.id === dialog.module)!.title : '全部'}记录？`} onClose={() => setDialog(null)}><p>这会清除{dialog.module ? '此分类' : '全部分类'}的完成勾选。导出的 PNG 备份仍可用于恢复。</p><div className="modal-actions"><button className="outline-button" disabled={busy} onClick={() => setDialog(null)}>保留记录</button><button className="danger-button" disabled={busy} onClick={() => void perform(async () => { accept(await api.reset(dialog.module)); setDialog(null); setNotice('所选记录已重置'); })}>确认重置</button></div></Modal>}
    {dialog?.kind === 'help' && <Modal error={error} title="把江湖余音，记在这里" onClose={() => setDialog(null)}><div className="help-step"><Map size={21} /><div><h3>按来源查漏</h3><p>从区域、玩法、限时或概率来源开始，也可以按 Ctrl K 搜索全部任务、人物和商店。</p></div></div><div className="help-step"><CheckCheck size={21} /><div><h3>勾选已获得的目标</h3><p>点击来源查看子目标；部分完成的分组显示短横线。左侧分段进度随勾选更新。</p></div></div><div className="help-step"><Download size={21} /><div><h3>用一张图片备份</h3><p>导出 PNG 后妥善保存。换电脑或从网页版迁移时，选择“恢复记录”并合并备份。</p></div></div><p className="help-credit">原作者：薄荷小鹿（燕云十六声）<br />数据 v{snapshot?.meta.releaseVersion ?? '1.5.0'} · 修订 {snapshot?.meta.sourceRevision ?? 73}</p></Modal>}
  </div>;
}
