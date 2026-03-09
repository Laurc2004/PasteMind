<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { onMount, tick } from 'svelte';
  import {
    clearHistory,
    deleteEntry,
    getHistory,
    getPermissionState,
    getSettings,
    openAccessibilitySettings,
    requestAccessibilityPermission,
    setHotkeyRecording,
    selectEntry,
    updateSettings
  } from '$lib/api';
  import { toFileSrc } from '$lib/file';
  import type { ClipboardEntry, PermissionState, Settings } from '$lib/types';

  type Locale = 'zh-CN' | 'en-US';
  const LOCALE_KEY = 'PasteMind.locale';

  const COPY = {
    'zh-CN': {
      title: '本地剪贴板中枢',
      subtitle: '轻量常驻，不上云。为文本与图片提供可搜索历史。',
      clearAll: '清空全部',
      needsPermission: '自动粘贴需要 macOS 辅助功能权限。',
      grantPermission: '前往授权',
      searchPlaceholder: '搜索文本或来源应用',
      syncing: '同步中...',
      noData: '当前没有可展示的剪贴板记录。',
      image: '图片',
      text: '文本',
      pasteNow: '立即粘贴',
      copyOnly: '仅复制',
      delete: '删除',
      loadingHistoryFailed: '加载历史失败',
      permissionStateFailed: '读取权限状态失败',
      loadingSettingsFailed: '读取设置失败',
      languageSaveFailed: '语言设置保存失败',
      selectFailed: '选择失败',
      deleteFailed: '删除失败',
      clearFailed: '清空失败',
      permissionGuideFailed: '权限引导失败',
      hotkeySaveFailed: '快捷键保存失败',
      hotkeySaved: '快捷键已更新并生效。',
      autoPasted: '已自动粘贴。',
      copiedOnly: '已写回剪贴板。',
      copiedOnlyWithPermissionHint: '已写回剪贴板。请授权辅助功能后使用自动粘贴。',
      openSettings: '快捷键设置',
      closeSettings: '关闭设置',
      hotkeyLabel: '全局呼出快捷键',
      hotkeyHint: '支持 Cmd+Shift+V，也支持 F1-F24（可单独设置或配合修饰键）。',
      saveHotkey: '保存快捷键',
      captureHotkey: '录制快捷键',
      captureHotkeyPending: '请按下快捷键组合...',
      captureCanceled: '已取消快捷键录制。',
      captureNeedsModifier: '快捷键至少需要一个修饰键（Cmd / Control / Option / Shift）。',
      captureUnsupportedKey: '当前按键暂不支持，请换一个按键。',
      captureSuccess: '快捷键录制完成',
      language: '语言',
      records: '条记录',
      sourceUnknown: '未知来源',
      imageAlt: '剪贴板图片预览',
      copiedAt: '复制时间',
      clickToPaste: '点击内容立即粘贴',
      statusLiveRegion: '状态消息',
      usingHotkey: '当前快捷键',
      settingsTitle: '交互设置',
      settingsDesc: '修改后立即生效，用于呼出粘贴板窗口。',
      saveInProgress: '保存中...',
      clearConfirmTitle: '确认清空全部记录？',
      clearConfirmDesc: '文本和图片历史会被永久删除，且无法恢复。',
      clearConfirmCancel: '取消',
      clearConfirmAccept: '确认清空'
    },
    'en-US': {
      title: 'Local Clipboard Hub',
      subtitle: 'Lightweight and local-first. Searchable history for text and images.',
      clearAll: 'Clear all',
      needsPermission: 'Auto paste needs macOS Accessibility permission.',
      grantPermission: 'Grant access',
      searchPlaceholder: 'Search text or source app',
      syncing: 'Syncing...',
      noData: 'No clipboard records yet.',
      image: 'Image',
      text: 'Text',
      pasteNow: 'Paste now',
      copyOnly: 'Copy only',
      delete: 'Delete',
      loadingHistoryFailed: 'Failed to load history',
      permissionStateFailed: 'Failed to load permission state',
      loadingSettingsFailed: 'Failed to load settings',
      languageSaveFailed: 'Failed to save language setting',
      selectFailed: 'Failed to select entry',
      deleteFailed: 'Failed to delete entry',
      clearFailed: 'Failed to clear history',
      permissionGuideFailed: 'Failed to guide permission setup',
      hotkeySaveFailed: 'Failed to save hotkey',
      hotkeySaved: 'Hotkey updated and active.',
      autoPasted: 'Pasted automatically.',
      copiedOnly: 'Copied back to clipboard.',
      copiedOnlyWithPermissionHint:
        'Copied back to clipboard. Grant accessibility permission to enable auto paste.',
      openSettings: 'Hotkey settings',
      closeSettings: 'Close settings',
      hotkeyLabel: 'Global summon hotkey',
      hotkeyHint: 'Supports Cmd+Shift+V and F1-F24 (alone or with modifiers).',
      saveHotkey: 'Save hotkey',
      captureHotkey: 'Record hotkey',
      captureHotkeyPending: 'Press your shortcut now...',
      captureCanceled: 'Hotkey recording canceled.',
      captureNeedsModifier: 'Use at least one modifier key (Cmd / Control / Option / Shift).',
      captureUnsupportedKey: 'This key is not supported yet. Try another key.',
      captureSuccess: 'Hotkey recorded',
      language: 'Language',
      records: 'records',
      sourceUnknown: 'Unknown app',
      imageAlt: 'Clipboard image preview',
      copiedAt: 'Copied at',
      clickToPaste: 'Click content to paste',
      statusLiveRegion: 'Status messages',
      usingHotkey: 'Active hotkey',
      settingsTitle: 'Interaction settings',
      settingsDesc: 'Changes are applied immediately to summon the clipboard frame.',
      saveInProgress: 'Saving...',
      clearConfirmTitle: 'Clear all records?',
      clearConfirmDesc: 'All text and image history will be deleted permanently.',
      clearConfirmCancel: 'Cancel',
      clearConfirmAccept: 'Confirm clear'
    }
  } as const;

  type CopyKey = keyof (typeof COPY)['zh-CN'];

  let entries: ClipboardEntry[] = [];
  let query = '';
  let loading = false;
  let message = '';
  let settings: Settings | null = null;
  let hotkeyDraft = '';
  let savingHotkey = false;
  let isCapturingHotkey = false;
  let showClearConfirm = false;
  let hotkeyCaptureButton: HTMLButtonElement | null = null;
  let locale: Locale = 'zh-CN';

  let permission: PermissionState = {
    accessibility_granted: false,
    can_auto_paste: false
  };

  let searchTimer: ReturnType<typeof setTimeout> | undefined;

  const t = (key: CopyKey): string => COPY[locale][key];

  function toLocale(value: string | null | undefined): Locale | null {
    if (value === 'zh-CN' || value === 'en-US') {
      return value;
    }
    return null;
  }

  function detectInitialLocale(): Locale {
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem(LOCALE_KEY);
      if (saved === 'zh-CN' || saved === 'en-US') {
        return saved;
      }
    }

    if (typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('zh')) {
      return 'zh-CN';
    }

    return 'en-US';
  }

  async function setLocale(next: Locale) {
    locale = next;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(LOCALE_KEY, next);
    }

    if (!settings || settings.ui_locale === next) {
      return;
    }

    try {
      settings = await updateSettings({ ui_locale: next });
    } catch (error) {
      message = `${t('languageSaveFailed')}: ${String(error)}`;
    }
  }

  async function reloadEntries() {
    loading = true;
    try {
      entries = await getHistory({
        query,
        limit: 120,
        offset: 0
      });
    } catch (error) {
      message = `${t('loadingHistoryFailed')}: ${String(error)}`;
    } finally {
      loading = false;
    }
  }

  async function reloadPermission() {
    try {
      permission = await getPermissionState();
    } catch (error) {
      message = `${t('permissionStateFailed')}: ${String(error)}`;
    }
  }

  async function bootstrap() {
    const fallbackLocale = detectInitialLocale();

    try {
      settings = await getSettings();
      hotkeyDraft = settings.hotkey;
      locale = toLocale(settings.ui_locale) ?? fallbackLocale;
    } catch (error) {
      message = `${t('loadingSettingsFailed')}: ${String(error)}`;
      locale = fallbackLocale;
    }

    await Promise.all([reloadEntries(), reloadPermission()]);
  }

  function onSearchInput() {
    if (searchTimer) {
      clearTimeout(searchTimer);
    }
    searchTimer = setTimeout(() => {
      void reloadEntries();
    }, 160);
  }

  async function onPasteNow(entry: ClipboardEntry) {
    message = '';
    try {
      await selectEntry(entry.id, true);
      message = t('autoPasted');
    } catch (error) {
      const text = String(error);
      if (text.includes('PERMISSION_DENIED')) {
        await selectEntry(entry.id, false);
        message = t('copiedOnlyWithPermissionHint');
      } else {
        message = `${t('selectFailed')}: ${text}`;
      }
    }

    await reloadPermission();
  }

  async function onCopyOnly(entry: ClipboardEntry) {
    message = '';
    try {
      await selectEntry(entry.id, false);
      message = t('copiedOnly');
    } catch (error) {
      message = `${t('selectFailed')}: ${String(error)}`;
    }
  }

  async function onDelete(entry: ClipboardEntry) {
    message = '';
    try {
      await deleteEntry(entry.id);
      await reloadEntries();
    } catch (error) {
      message = `${t('deleteFailed')}: ${String(error)}`;
    }
  }

  function openClearConfirmModal() {
    showClearConfirm = true;
  }

  function closeClearConfirmModal() {
    showClearConfirm = false;
  }

  async function onConfirmClear() {
    showClearConfirm = false;
    message = '';
    try {
      await clearHistory();
      await reloadEntries();
    } catch (error) {
      message = `${t('clearFailed')}: ${String(error)}`;
    }
  }

  async function onGrantPermission() {
    try {
      const granted = await requestAccessibilityPermission();
      if (!granted) {
        await openAccessibilitySettings();
      }
      await reloadPermission();
    } catch (error) {
      message = `${t('permissionGuideFailed')}: ${String(error)}`;
    }
  }

  function hasModifierToken(hotkey: string): boolean {
    const modifierTokens = new Set([
      'Cmd',
      'Command',
      'Control',
      'Ctrl',
      'Option',
      'Alt',
      'Shift',
      'Meta',
      'Super'
    ]);
    return hotkey
      .split('+')
      .map((token) => token.trim())
      .some((token) => modifierTokens.has(token));
  }

  function isFunctionKeyToken(token: string): boolean {
    return /^F([1-9]|1[0-9]|2[0-4])$/i.test(token.trim());
  }

  function allowsHotkeyWithoutModifier(hotkey: string): boolean {
    const tokens = hotkey
      .split('+')
      .map((token) => token.trim())
      .filter((token) => token.length > 0);
    return tokens.length === 1 && isFunctionKeyToken(tokens[0]);
  }

  function isModifierOnlyKey(key: string): boolean {
    return key === 'Meta' || key === 'Control' || key === 'Shift' || key === 'Alt';
  }

  function normalizeMainHotkeyKey(event: KeyboardEvent): string | null {
    if (isModifierOnlyKey(event.key)) {
      return null;
    }

    if (/^[a-z]$/i.test(event.key)) {
      return event.key.toUpperCase();
    }

    if (/^[0-9]$/.test(event.key)) {
      return event.key;
    }

    if (/^F([1-9]|1[0-9]|2[0-4])$/i.test(event.key)) {
      return event.key.toUpperCase();
    }

    switch (event.key) {
      case 'Space':
      case ' ':
      case 'Spacebar':
        return 'Space';
      case 'ArrowUp':
        return 'Up';
      case 'ArrowDown':
        return 'Down';
      case 'ArrowLeft':
        return 'Left';
      case 'ArrowRight':
        return 'Right';
      case 'Enter':
        return 'Enter';
      case 'Tab':
        return 'Tab';
      case 'Backspace':
        return 'Backspace';
      case 'Delete':
        return 'Delete';
      case 'Home':
        return 'Home';
      case 'End':
        return 'End';
      case 'PageUp':
        return 'PageUp';
      case 'PageDown':
        return 'PageDown';
      case 'Escape':
        return 'Escape';
      default:
        break;
    }

    if (event.code.startsWith('Key') && event.code.length === 4) {
      return event.code.slice(3).toUpperCase();
    }

    if (event.code.startsWith('Digit') && event.code.length === 6) {
      return event.code.slice(5);
    }

    const codeMap: Record<string, string> = {
      Backquote: 'Backquote',
      Minus: 'Minus',
      Equal: 'Equal',
      BracketLeft: 'BracketLeft',
      BracketRight: 'BracketRight',
      Backslash: 'Backslash',
      Semicolon: 'Semicolon',
      Quote: 'Quote',
      Comma: 'Comma',
      Period: 'Period',
      Slash: 'Slash',
      Insert: 'Insert',
      PrintScreen: 'PrintScreen',
      Numpad0: 'Numpad0',
      Numpad1: 'Numpad1',
      Numpad2: 'Numpad2',
      Numpad3: 'Numpad3',
      Numpad4: 'Numpad4',
      Numpad5: 'Numpad5',
      Numpad6: 'Numpad6',
      Numpad7: 'Numpad7',
      Numpad8: 'Numpad8',
      Numpad9: 'Numpad9',
      NumpadAdd: 'NumpadAdd',
      NumpadSubtract: 'NumpadSubtract',
      NumpadMultiply: 'NumpadMultiply',
      NumpadDivide: 'NumpadDivide',
      NumpadDecimal: 'NumpadDecimal',
      NumpadEnter: 'NumpadEnter'
    };
    if (codeMap[event.code]) {
      return codeMap[event.code];
    }

    return null;
  }

  async function setHotkeyCaptureMode(active: boolean): Promise<boolean> {
    try {
      await setHotkeyRecording(active);
      return true;
    } catch (error) {
      message = `${t('hotkeySaveFailed')}: ${String(error)}`;
      return false;
    }
  }

  function stopHotkeyCapture(notice: CopyKey | null = null) {
    if (!isCapturingHotkey) {
      return;
    }

    isCapturingHotkey = false;
    void setHotkeyCaptureMode(false);
    if (notice) {
      message = t(notice);
    }
  }

  async function onToggleHotkeyCapture() {
    if (isCapturingHotkey) {
      stopHotkeyCapture('captureCanceled');
      return;
    }

    message = '';
    const enabled = await setHotkeyCaptureMode(true);
    if (!enabled) {
      return;
    }
    isCapturingHotkey = true;
    await tick();
    hotkeyCaptureButton?.focus();
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (showClearConfirm && event.key === 'Escape') {
      event.preventDefault();
      closeClearConfirmModal();
      return;
    }

    if (!isCapturingHotkey) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    if (event.key === 'Escape') {
      stopHotkeyCapture('captureCanceled');
      return;
    }

    const mainKey = normalizeMainHotkeyKey(event);
    if (!mainKey) {
      if (!isModifierOnlyKey(event.key)) {
        message = t('captureUnsupportedKey');
      }
      return;
    }

    const modifiers: string[] = [];
    if (event.metaKey) {
      modifiers.push('Cmd');
    }
    if (event.ctrlKey) {
      modifiers.push('Control');
    }
    if (event.altKey) {
      modifiers.push('Option');
    }
    if (event.shiftKey) {
      modifiers.push('Shift');
    }

    if (modifiers.length === 0 && !isFunctionKeyToken(mainKey)) {
      message = t('captureNeedsModifier');
      return;
    }

    const hotkey = [...modifiers, mainKey].join('+');
    hotkeyDraft = hotkey;
    stopHotkeyCapture(null);
    message = `${t('captureSuccess')}: ${hotkey}`;
  }

  async function onSaveHotkey() {
    if (!settings) {
      return;
    }

    if (!hasModifierToken(hotkeyDraft) && !allowsHotkeyWithoutModifier(hotkeyDraft)) {
      message = t('captureNeedsModifier');
      return;
    }

    stopHotkeyCapture(null);

    savingHotkey = true;
    message = '';

    try {
      const next = await updateSettings({ hotkey: hotkeyDraft });
      settings = next;
      hotkeyDraft = next.hotkey;
      message = t('hotkeySaved');
    } catch (error) {
      message = `${t('hotkeySaveFailed')}: ${String(error)}`;
    } finally {
      savingHotkey = false;
    }
  }

  function formatTime(ts: string): string {
    const value = new Date(ts);
    return new Intl.DateTimeFormat(locale, {
      hour: '2-digit',
      minute: '2-digit',
      month: '2-digit',
      day: '2-digit'
    }).format(value);
  }

  function formatBytes(value: number): string {
    if (value < 1024) {
      return `${value}B`;
    }
    if (value < 1024 * 1024) {
      return `${(value / 1024).toFixed(1)}KB`;
    }
    return `${(value / (1024 * 1024)).toFixed(1)}MB`;
  }

  onMount(() => {
    void bootstrap();

    let unlistenHistory: (() => void) | undefined;
    let unlistenPermission: (() => void) | undefined;

    void listen('history:updated', () => {
      void reloadEntries();
    }).then((off) => {
      unlistenHistory = off;
    });

    void listen('permission:changed', () => {
      void reloadPermission();
    }).then((off) => {
      unlistenPermission = off;
    });

    return () => {
      stopHotkeyCapture(null);
      void setHotkeyCaptureMode(false);
      if (searchTimer) {
        clearTimeout(searchTimer);
      }
      unlistenHistory?.();
      unlistenPermission?.();
    };
  });
</script>

<svelte:head>
  <title>PasteMind</title>
</svelte:head>

<svelte:window on:keydown={onWindowKeydown} />

<main class="shell">
  <header class="panel hero reveal">
    <div class="brand">
      <div class="logo-wrap">
        <img src="/app-icon.svg" alt="PasteMind logo" />
      </div>
      <div>
        <p class="kicker">PasteMind</p>
        <h1>{t('title')}</h1>
        <p class="sub">{t('subtitle')}</p>
      </div>
    </div>

    <div class="hero-actions">
      <div class="lang" role="group" aria-label={t('language')}>
        <button
          class:active={locale === 'zh-CN'}
          aria-pressed={locale === 'zh-CN'}
          on:click={() => setLocale('zh-CN')}
        >
          中
        </button>
        <button
          class:active={locale === 'en-US'}
          aria-pressed={locale === 'en-US'}
          on:click={() => setLocale('en-US')}
        >
          EN
        </button>
      </div>
      <button class="danger" on:click={openClearConfirmModal}>{t('clearAll')}</button>
    </div>
  </header>

  <section class="panel metric reveal delay-1">
    <div class="metric-item">
      <span>{t('usingHotkey')}</span>
      <strong>{settings?.hotkey ?? 'Cmd+Shift+V'}</strong>
    </div>
    <div class="metric-item">
      <span>History</span>
      <strong>{entries.length} {t('records')}</strong>
    </div>
  </section>

  {#if settings && settings.auto_paste_default && !permission.accessibility_granted}
    <section class="panel permission reveal delay-2">
      <p>{t('needsPermission')}</p>
      <button class="primary" on:click={onGrantPermission}>{t('grantPermission')}</button>
    </section>
  {/if}

  <section class="panel controls reveal delay-2">
    <div class="search-wrap">
      <input
        type="search"
        bind:value={query}
        on:input={onSearchInput}
        placeholder={t('searchPlaceholder')}
      />
    </div>
    <div class="meta">
      <span>{entries.length} {t('records')}</span>
      {#if loading}
        <span>{t('syncing')}</span>
      {/if}
    </div>
  </section>

  {#if message}
    <section class="panel toast" aria-live="polite" aria-label={t('statusLiveRegion')}>
      {message}
    </section>
  {/if}

  {#if showClearConfirm}
    <div class="modal-backdrop">
      <div
        class="panel modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="clear-confirm-title"
      >
        <h2 id="clear-confirm-title">{t('clearConfirmTitle')}</h2>
        <p>{t('clearConfirmDesc')}</p>
        <div class="modal-actions">
          <button class="ghost" on:click={closeClearConfirmModal}>{t('clearConfirmCancel')}</button>
          <button class="danger" on:click={onConfirmClear}>{t('clearConfirmAccept')}</button>
        </div>
      </div>
    </div>
  {/if}

  <section class="panel list reveal delay-4">
    {#if !entries.length && !loading}
      <div class="empty">{t('noData')}</div>
    {/if}

    {#each entries as entry (entry.id)}
      <article class="entry">
        <div class="entry-top">
          <span class="pill">{entry.kind === 'image' ? t('image') : t('text')}</span>
          <span class="time">{t('copiedAt')}: {formatTime(entry.created_at)}</span>
        </div>

        <button
          class="entry-preview"
          aria-label={t('clickToPaste')}
          on:click={() => onPasteNow(entry)}
        >
          {#if entry.kind === 'image' && (entry.image_preview_data || entry.image_path)}
            <img
              src={entry.image_preview_data ?? toFileSrc(entry.image_path)}
              alt={t('imageAlt')}
              loading="lazy"
              decoding="async"
            />
          {:else}
            <p>{entry.text_preview ?? '-'}</p>
          {/if}
        </button>

        <div class="entry-foot">
          <span>{entry.source_app || t('sourceUnknown')}</span>
          <span>{formatBytes(entry.size_bytes)}</span>
        </div>

        <div class="entry-actions">
          <button class="primary" on:click={() => onPasteNow(entry)}>{t('pasteNow')}</button>
          <button class="secondary" on:click={() => onCopyOnly(entry)}>{t('copyOnly')}</button>
          <button class="ghost" on:click={() => onDelete(entry)}>{t('delete')}</button>
        </div>
      </article>
    {/each}
  </section>

  <section class="panel settings reveal delay-3">
    <div>
      <h2>{t('settingsTitle')}</h2>
      <p>{t('settingsDesc')}</p>
    </div>
    <label for="hotkey-capture">{t('hotkeyLabel')}</label>
    <div class="hotkey-row">
      <button
        id="hotkey-capture"
        class="capture"
        class:capturing={isCapturingHotkey}
        bind:this={hotkeyCaptureButton}
        aria-pressed={isCapturingHotkey}
        on:click={onToggleHotkeyCapture}
      >
        {#if isCapturingHotkey}
          {t('captureHotkeyPending')}
        {:else}
          {t('captureHotkey')}: {hotkeyDraft}
        {/if}
      </button>
      <button class="primary" disabled={savingHotkey} on:click={onSaveHotkey}>
        {savingHotkey ? t('saveInProgress') : t('saveHotkey')}
      </button>
    </div>
    <small>{t('hotkeyHint')}</small>
  </section>
</main>

<style>
  :global(:root) {
    --bg-1: #eef4ff;
    --bg-2: #dbeafe;
    --panel: rgb(255 255 255 / 50%);
    --panel-strong: rgb(255 255 255 / 62%);
    --ink: #0f172a;
    --muted: #334155;
    --line: rgb(255 255 255 / 58%);
    --line-soft: rgb(148 163 184 / 35%);
    --accent: #f97316;
    --accent-strong: #ea580c;
    --danger-bg: rgb(248 113 113 / 18%);
    --danger-ink: #b91c1c;
  }

  :global(body) {
    margin: 0;
    color: var(--ink);
    font-family: 'Inter', 'SF Pro Text', 'Noto Sans SC', sans-serif;
    background:
      radial-gradient(circle at 8% 4%, rgb(59 130 246 / 26%), transparent 24%),
      radial-gradient(circle at 87% 10%, rgb(14 165 233 / 20%), transparent 30%),
      radial-gradient(circle at 50% 92%, rgb(249 115 22 / 18%), transparent 34%),
      linear-gradient(155deg, var(--bg-1), var(--bg-2));
    min-height: 100vh;
  }

  .shell {
    max-width: 780px;
    margin: 0 auto;
    padding: 18px 16px 28px;
    display: grid;
    gap: 12px;
    position: relative;
  }

  .panel {
    background: linear-gradient(140deg, var(--panel), rgb(255 255 255 / 36%));
    border: 1px solid var(--line);
    border-radius: 14px;
    box-shadow:
      0 14px 28px rgb(15 23 42 / 14%),
      inset 0 1px 0 rgb(255 255 255 / 58%);
    backdrop-filter: blur(18px) saturate(145%);
    -webkit-backdrop-filter: blur(18px) saturate(145%);
  }

  .hero {
    display: flex;
    justify-content: space-between;
    gap: 14px;
    padding: 14px;
  }

  .brand {
    display: flex;
    gap: 12px;
  }

  .logo-wrap {
    width: 48px;
    height: 48px;
    border-radius: 13px;
    display: grid;
    place-items: center;
    background: linear-gradient(155deg, rgb(255 255 255 / 48%), rgb(255 255 255 / 24%));
    border: 1px solid var(--line);
    box-shadow:
      0 8px 20px rgb(15 23 42 / 16%),
      inset 0 1px 0 rgb(255 255 255 / 68%);
  }

  .logo-wrap img {
    width: 32px;
    height: 32px;
    object-fit: contain;
  }

  .kicker {
    margin: 0;
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    color: #475569;
  }

  h1 {
    margin: 4px 0;
    font-size: 25px;
    line-height: 1.08;
  }

  h2 {
    margin: 0;
    font-size: 15px;
  }

  .sub,
  .settings p {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .hero-actions {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .lang {
    display: flex;
    border: 1px solid var(--line);
    border-radius: 10px;
    overflow: hidden;
    background: rgb(255 255 255 / 28%);
  }

  .lang button {
    border-radius: 0;
    min-width: 44px;
  }

  .lang .active {
    background: rgb(37 99 235 / 85%);
    color: white;
  }

  .metric {
    padding: 12px 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    background: linear-gradient(140deg, var(--panel-strong), rgb(255 255 255 / 30%));
  }

  .metric-item {
    display: grid;
    gap: 2px;
  }

  .metric-item span {
    font-size: 11px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.09em;
  }

  .metric-item strong {
    font-size: 14px;
  }

  .permission {
    padding: 12px 14px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    background: linear-gradient(140deg, rgb(255 247 237 / 74%), rgb(255 255 255 / 42%));
  }

  .permission p {
    margin: 0;
    color: #6b4d06;
    font-size: 13px;
  }

  .controls {
    padding: 12px 14px;
    background: linear-gradient(145deg, rgb(255 255 255 / 55%), rgb(255 255 255 / 34%));
  }

  .search-wrap {
    display: grid;
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .meta {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
    font-size: 12px;
    color: var(--muted);
  }

  .settings {
    padding: 12px 14px;
    display: grid;
    gap: 8px;
    background: linear-gradient(145deg, rgb(255 255 255 / 58%), rgb(255 255 255 / 35%));
  }

  .hotkey-row {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 8px;
  }

  .capture {
    border: 1px solid var(--line-soft);
    border-radius: 10px;
    padding: 9px 11px;
    background: rgb(255 255 255 / 62%);
    color: #0f172a;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    text-align: left;
    font-family: 'SF Mono', 'Fira Code', 'JetBrains Mono', ui-monospace, monospace;
  }

  .capture.capturing {
    border-color: rgb(59 130 246 / 70%);
    box-shadow: 0 0 0 3px rgb(59 130 246 / 20%);
    background: rgb(219 234 254 / 70%);
  }

  .settings label,
  .settings small {
    font-size: 12px;
    color: var(--muted);
  }

  .toast {
    padding: 10px 12px;
    border-color: rgb(249 115 22 / 35%);
    background: linear-gradient(135deg, rgb(255 237 213 / 72%), rgb(255 255 255 / 46%));
    color: #9a3412;
    font-size: 13px;
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgb(15 23 42 / 28%);
    display: grid;
    place-items: center;
    padding: 16px;
    z-index: 30;
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
  }

  .modal {
    width: min(420px, 100%);
    padding: 14px;
    display: grid;
    gap: 10px;
    background: linear-gradient(145deg, rgb(255 255 255 / 72%), rgb(255 255 255 / 42%));
  }

  .modal p {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }

  .list {
    padding: 10px;
    display: grid;
    gap: 10px;
    max-height: 56vh;
    overflow: auto;
  }

  .entry {
    border: 1px solid var(--line-soft);
    border-radius: 12px;
    padding: 11px;
    background: linear-gradient(145deg, rgb(255 255 255 / 62%), rgb(255 255 255 / 38%));
    display: grid;
    gap: 8px;
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 55%);
  }

  .entry-top,
  .entry-foot {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
  }

  .entry p {
    margin: 0;
    font-size: 14px;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  .entry-preview {
    border: 1px solid transparent;
    border-radius: 10px;
    padding: 0;
    background: transparent;
    cursor: pointer;
    display: block;
    text-align: left;
  }

  .entry-preview:hover {
    transform: none;
    border-color: rgb(59 130 246 / 25%);
    background: rgb(255 255 255 / 22%);
  }

  .entry-preview:focus-visible {
    outline: 2px solid rgb(37 99 235 / 80%);
    outline-offset: 1px;
  }

  .entry img {
    width: 100%;
    max-height: 180px;
    object-fit: contain;
    border-radius: 10px;
    border: 1px solid var(--line-soft);
    background: rgb(255 255 255 / 45%);
  }

  .entry-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pill {
    border-radius: 999px;
    padding: 2px 9px;
    font-size: 11px;
    background: rgb(59 130 246 / 12%);
    color: #1e40af;
  }

  input {
    border: 1px solid var(--line-soft);
    border-radius: 10px;
    padding: 9px 11px;
    background: rgb(255 255 255 / 62%);
    font-size: 14px;
    outline: none;
    color: #0f172a;
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
  }

  input:focus {
    border-color: rgb(59 130 246 / 60%);
    box-shadow: 0 0 0 3px rgb(59 130 246 / 20%);
  }

  button {
    border: 0;
    border-radius: 10px;
    padding: 8px 12px;
    font-size: 13px;
    font-weight: 550;
    cursor: pointer;
    transition:
      color 180ms ease,
      background-color 180ms ease,
      box-shadow 180ms ease,
      transform 180ms ease;
  }

  button:hover {
    transform: translateY(-1px);
  }

  button:focus-visible {
    outline: 2px solid #101010;
    outline-offset: 2px;
  }

  button:disabled {
    opacity: 0.62;
    cursor: not-allowed;
    transform: none;
  }

  .primary {
    color: #fff;
    background: linear-gradient(135deg, #2563eb, #3b82f6);
  }

  .secondary {
    color: #0f172a;
    background: rgb(255 255 255 / 55%);
    border: 1px solid var(--line-soft);
  }

  .ghost {
    color: #0f172a;
    background: rgb(255 255 255 / 42%);
    border: 1px solid var(--line-soft);
  }

  .danger {
    background: var(--danger-bg);
    color: var(--danger-ink);
  }

  .empty {
    padding: 22px 12px;
    text-align: center;
    color: var(--muted);
    font-size: 13px;
  }

  .reveal {
    opacity: 0;
    transform: translateY(8px);
    animation: reveal 340ms ease forwards;
  }

  .delay-1 {
    animation-delay: 60ms;
  }

  .delay-2 {
    animation-delay: 120ms;
  }

  .delay-3 {
    animation-delay: 180ms;
  }

  .delay-4 {
    animation-delay: 240ms;
  }

  @keyframes reveal {
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 760px) {
    .hero {
      flex-direction: column;
    }

    .hero-actions {
      justify-content: space-between;
      align-items: center;
    }

    h1 {
      font-size: 22px;
    }

    .search-wrap,
    .hotkey-row {
      grid-template-columns: 1fr;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    * {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
