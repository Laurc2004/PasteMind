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
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <img src="/app-icon.svg" alt="" class="toolbar-logo" />
      <span class="toolbar-title">PasteMind</span>
      <kbd class="toolbar-hotkey">{settings?.hotkey ?? 'Cmd+Shift+V'}</kbd>
    </div>
    <div class="toolbar-right">
      <div class="lang" role="group" aria-label={t('language')}>
        <button class:active={locale === 'zh-CN'} aria-pressed={locale === 'zh-CN'} on:click={() => setLocale('zh-CN')}>中</button>
        <button class:active={locale === 'en-US'} aria-pressed={locale === 'en-US'} on:click={() => setLocale('en-US')}>EN</button>
      </div>
      <button class="toolbar-btn" on:click={openClearConfirmModal} title={t('clearAll')}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
      </button>
    </div>
  </header>

  <!-- Permission banner -->
  {#if settings && settings.auto_paste_default && !permission.accessibility_granted}
    <div class="permission-bar">
      <span>{t('needsPermission')}</span>
      <button class="accent-text" on:click={onGrantPermission}>{t('grantPermission')}</button>
    </div>
  {/if}

  <!-- Sticky search -->
  <div class="search-bar">
    <svg class="search-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
    <input
      type="search"
      bind:value={query}
      on:input={onSearchInput}
      placeholder={t('searchPlaceholder')}
      class="search-input"
    />
    <span class="search-count">{entries.length}</span>
    {#if loading}
      <span class="search-loading">{t('syncing')}</span>
    {/if}
  </div>

  <!-- Toast -->
  {#if message}
    <div class="toast" aria-live="polite" aria-label={t('statusLiveRegion')}>
      {message}
    </div>
  {/if}

  <!-- Clear confirm modal -->
  {#if showClearConfirm}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <!-- svelte-ignore a11y-no-static-element-interactions -->
    <div class="modal-backdrop" on:click={closeClearConfirmModal}>
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <div class="modal" role="dialog" aria-modal="true" tabindex="-1" aria-labelledby="clear-confirm-title" on:click|stopPropagation={() => {}}>
        <h2 id="clear-confirm-title">{t('clearConfirmTitle')}</h2>
        <p>{t('clearConfirmDesc')}</p>
        <div class="modal-actions">
          <button class="btn-ghost" on:click={closeClearConfirmModal}>{t('clearConfirmCancel')}</button>
          <button class="btn-danger" on:click={onConfirmClear}>{t('clearConfirmAccept')}</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Entry list (the main content) -->
  <section class="entries">
    {#if !entries.length && !loading}
      <div class="empty">{t('noData')}</div>
    {/if}

    {#each entries as entry (entry.id)}
      <article class="entry">
        <div class="entry-row">
          <button class="entry-content" aria-label={t('clickToPaste')} on:click={() => onPasteNow(entry)}>
            {#if entry.kind === 'image' && (entry.image_preview_data || entry.image_path)}
              <img
                src={entry.image_preview_data ?? toFileSrc(entry.image_path)}
                alt={t('imageAlt')}
                loading="lazy"
                decoding="async"
              />
            {:else}
              <span class="entry-text">{entry.text_preview ?? '-'}</span>
            {/if}
          </button>
          <div class="entry-actions">
            <button class="btn-icon" title={t('pasteNow')} on:click={() => onPasteNow(entry)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1Z"/><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2h-2"/><path d="M12 11v6"/><path d="m9 14 3-3 3 3"/></svg>
            </button>
            <button class="btn-icon" title={t('copyOnly')} on:click={() => onCopyOnly(entry)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="14" height="14" x="8" y="8" rx="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/></svg>
            </button>
            <button class="btn-icon btn-icon-danger" title={t('delete')} on:click={() => onDelete(entry)}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" x2="6" y1="6" y2="18"/><line x1="6" x2="18" y1="6" y2="18"/></svg>
            </button>
          </div>
        </div>
        <div class="entry-meta">
          <span class="meta-source">{entry.source_app || t('sourceUnknown')}</span>
          <span class="meta-dot">&middot;</span>
          <span>{formatTime(entry.created_at)}</span>
          <span class="meta-dot">&middot;</span>
          <span>{formatBytes(entry.size_bytes)}</span>
          <span class="meta-kind">{entry.kind === 'image' ? t('image') : t('text')}</span>
        </div>
      </article>
    {/each}
  </section>

  <!-- Settings (collapsed toggle at bottom) -->
  <section class="settings">
    <details>
      <summary>{t('openSettings')}</summary>
      <div class="settings-body">
        <p>{t('settingsDesc')}</p>
        <label for="hotkey-capture" class="settings-label">{t('hotkeyLabel')}</label>
        <div class="hotkey-row">
          <button
            id="hotkey-capture"
            class="hotkey-capture"
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
          <button class="btn-primary" disabled={savingHotkey} on:click={onSaveHotkey}>
            {savingHotkey ? t('saveInProgress') : t('saveHotkey')}
          </button>
        </div>
        <small>{t('hotkeyHint')}</small>
      </div>
    </details>
  </section>
</main>

<style>
  :global(:root) {
    --ink: #1a1a1a;
    --ink-secondary: #6b6b6b;
    --ink-tertiary: #9b9b9b;
    --surface: #ffffff;
    --surface-hover: #f5f5f5;
    --surface-active: #ebebeb;
    --border: #e5e5e5;
    --border-focus: #a0a0a0;
    --accent: #4f46e5;
    --accent-hover: #4338ca;
    --accent-subtle: rgb(79 70 229 / 8%);
    --danger: #dc2626;
    --danger-subtle: rgb(220 38 38 / 8%);
    --radius-sm: 6px;
    --radius-md: 8px;
    --radius-lg: 10px;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    color: var(--ink);
    font-family:
      -apple-system, 'SF Pro Text', 'Inter', 'Noto Sans SC', 'Helvetica Neue',
      sans-serif;
    background: var(--surface);
    min-height: 100vh;
    -webkit-font-smoothing: antialiased;
  }

  /* --- Shell: full viewport flex column --- */
  .shell {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* --- Toolbar --- */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
    gap: 8px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .toolbar-logo {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .toolbar-title {
    font-size: 13px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .toolbar-hotkey {
    font-size: 11px;
    font-family: 'SF Mono', ui-monospace, monospace;
    color: var(--ink-tertiary);
    background: var(--surface-hover);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 1px 5px;
    white-space: nowrap;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }

  .toolbar-btn {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-secondary);
    cursor: pointer;
    transition: background 120ms ease;
    padding: 0;
  }

  .toolbar-btn:hover {
    background: var(--surface-hover);
  }

  /* --- Language toggle --- */
  .lang {
    display: flex;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .lang :global(button) {
    border: none;
    background: transparent;
    font-size: 11px;
    font-weight: 500;
    padding: 3px 8px;
    cursor: pointer;
    color: var(--ink-secondary);
    transition: all 120ms ease;
  }

  .lang :global(.active) {
    background: var(--ink);
    color: var(--surface);
  }

  /* --- Permission bar --- */
  .permission-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 12px;
    background: #fef3c7;
    border-bottom: 1px solid #fde68a;
    font-size: 12px;
    color: #92400e;
    flex-shrink: 0;
  }

  .accent-text {
    border: none;
    background: none;
    color: var(--accent);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    padding: 0;
  }

  .accent-text:hover {
    text-decoration: underline;
  }

  /* --- Search bar (sticky) --- */
  .search-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }

  .search-icon {
    color: var(--ink-tertiary);
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 8px;
    font-size: 13px;
    background: var(--surface);
    color: var(--ink);
    outline: none;
    min-width: 0;
    transition: border-color 120ms ease;
  }

  .search-input:focus {
    border-color: var(--border-focus);
  }

  .search-input::placeholder {
    color: var(--ink-tertiary);
  }

  .search-count {
    font-size: 11px;
    color: var(--ink-tertiary);
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
    flex-shrink: 0;
  }

  .search-loading {
    font-size: 11px;
    color: var(--accent);
    flex-shrink: 0;
  }

  /* --- Toast --- */
  .toast {
    padding: 7px 12px;
    font-size: 12px;
    color: var(--accent-hover);
    background: var(--accent-subtle);
    border-bottom: 1px solid rgb(79 70 229 / 15%);
    flex-shrink: 0;
  }

  /* --- Modal --- */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgb(0 0 0 / 30%);
    display: grid;
    place-items: center;
    padding: 24px;
    z-index: 50;
  }

  .modal {
    background: var(--surface);
    border-radius: var(--radius-lg);
    padding: 20px;
    width: min(340px, 100%);
    box-shadow: 0 8px 32px rgb(0 0 0 / 12%);
  }

  .modal h2 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 600;
  }

  .modal p {
    margin: 0 0 16px;
    font-size: 13px;
    color: var(--ink-secondary);
    line-height: 1.45;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }

  /* --- Entry list (main scrollable area) --- */
  .entries {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0;
  }

  .empty {
    padding: 48px 16px;
    text-align: center;
    color: var(--ink-tertiary);
    font-size: 13px;
  }

  /* --- Entry card --- */
  .entry {
    border-bottom: 1px solid var(--border);
    padding: 8px 12px;
    transition: background 100ms ease;
  }

  .entry:hover {
    background: var(--surface-hover);
  }

  .entry:last-child {
    border-bottom: none;
  }

  .entry-row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
  }

  .entry-content {
    flex: 1;
    min-width: 0;
    border: none;
    background: none;
    text-align: left;
    cursor: pointer;
    padding: 0;
    color: var(--ink);
  }

  .entry-content:hover {
    opacity: 0.8;
  }

  .entry-text {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    font-size: 13px;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .entry img {
    max-width: 100%;
    max-height: 140px;
    object-fit: contain;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
  }

  .entry-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 120ms ease;
  }

  .entry:hover .entry-actions {
    opacity: 1;
  }

  /* On touch devices, always show actions */
  @media (hover: none) {
    .entry-actions {
      opacity: 1;
    }
  }

  .entry-meta {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-top: 3px;
    font-size: 11px;
    color: var(--ink-tertiary);
    flex-wrap: wrap;
  }

  .meta-source {
    color: var(--ink-secondary);
  }

  .meta-dot {
    opacity: 0.4;
  }

  .meta-kind {
    margin-left: auto;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 10px;
    font-weight: 500;
    color: var(--ink-tertiary);
  }

  /* --- Icon buttons --- */
  .btn-icon {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--ink-secondary);
    cursor: pointer;
    padding: 0;
    transition: background 100ms ease;
  }

  .btn-icon:hover {
    background: var(--surface-active);
  }

  .btn-icon-danger:hover {
    background: var(--danger-subtle);
    color: var(--danger);
  }

  /* --- Settings --- */
  .settings {
    border-top: 1px solid var(--border);
    flex-shrink: 0;
    font-size: 13px;
  }

  .settings summary {
    padding: 8px 12px;
    cursor: pointer;
    font-weight: 500;
    color: var(--ink-secondary);
    list-style: none;
    display: flex;
    align-items: center;
    gap: 6px;
    user-select: none;
    transition: background 100ms ease;
  }

  .settings summary:hover {
    background: var(--surface-hover);
  }

  .settings summary::before {
    content: '›';
    display: inline-block;
    transition: transform 150ms ease;
  }

  .settings details[open] summary::before {
    transform: rotate(90deg);
  }

  .settings-body {
    padding: 8px 12px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .settings-body p {
    margin: 0;
    color: var(--ink-secondary);
    font-size: 12px;
  }

  .settings-label {
    font-size: 12px;
    color: var(--ink-secondary);
  }

  .hotkey-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  .hotkey-capture {
    flex: 1;
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 8px;
    background: var(--surface);
    color: var(--ink);
    font-size: 12px;
    font-family: 'SF Mono', ui-monospace, monospace;
    text-align: left;
    cursor: pointer;
    outline: none;
    transition: border-color 120ms ease;
  }

  .hotkey-capture.capturing {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgb(79 70 229 / 15%);
  }

  .settings small {
    font-size: 11px;
    color: var(--ink-tertiary);
  }

  /* --- Button styles --- */
  .btn-primary {
    border: none;
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    background: var(--accent);
    color: #fff;
    cursor: pointer;
    white-space: nowrap;
    transition: background 120ms ease;
  }

  .btn-primary:hover {
    background: var(--accent-hover);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-ghost {
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    background: var(--surface);
    color: var(--ink);
    cursor: pointer;
    transition: background 120ms ease;
  }

  .btn-ghost:hover {
    background: var(--surface-hover);
  }

  .btn-danger {
    border: none;
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    background: var(--danger);
    color: #fff;
    cursor: pointer;
    transition: opacity 120ms ease;
  }

  .btn-danger:hover {
    opacity: 0.9;
  }

  /* --- Focus visible --- */
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  input:focus-visible {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgb(79 70 229 / 15%);
  }

  /* --- Reduced motion --- */
  @media (prefers-reduced-motion: reduce) {
    * {
      animation: none !important;
      transition: none !important;
    }
  }
</style>
