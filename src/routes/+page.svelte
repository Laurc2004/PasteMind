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
  import type { ClipboardEntry, PermissionState, Settings } from '$lib/types';
  import { t, DEFAULT_LOCALE, LOCALE_KEY } from '$lib/i18n';
  import type { Locale, CopyKey } from '$lib/i18n';
  import {
    hasModifierToken,
    allowsHotkeyWithoutModifier,
    isModifierOnlyKey,
    normalizeMainHotkeyKey
  } from '$lib/hotkey';
  import EntryCard from '$lib/components/EntryCard.svelte';
  import ConfirmModal from '$lib/components/ConfirmModal.svelte';
  import Toast from '$lib/components/Toast.svelte';

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
  let reloadGeneration = 0;

  let permission: PermissionState = {
    accessibility_granted: false,
    can_auto_paste: false
  };

  let searchTimer: ReturnType<typeof setTimeout> | undefined;

  function toLocale(value: string | null | undefined): Locale | null {
    if (value === 'zh-CN' || value === 'en-US') {
      return value;
    }
    return null;
  }

  function detectInitialLocale(): Locale {
    try {
      if (typeof localStorage !== 'undefined') {
        const saved = localStorage.getItem(LOCALE_KEY);
        if (saved === 'zh-CN' || saved === 'en-US') {
          return saved;
        }
      }
    } catch {
      // localStorage unavailable (e.g. Safari private browsing)
    }

    if (typeof navigator !== 'undefined' && navigator.language.toLowerCase().startsWith('zh')) {
      return 'zh-CN';
    }

    return DEFAULT_LOCALE;
  }

  async function setLocale(next: Locale) {
    locale = next;
    try {
      if (typeof localStorage !== 'undefined') {
        localStorage.setItem(LOCALE_KEY, next);
      }
    } catch {
      // ignore storage errors
    }

    if (!settings || settings.ui_locale === next) {
      return;
    }

    try {
      settings = await updateSettings({ ui_locale: next });
    } catch (error) {
      message = `${t(locale, 'languageSaveFailed')}: ${String(error)}`;
    }
  }

  async function reloadEntries() {
    const gen = ++reloadGeneration;
    loading = true;
    try {
      const result = await getHistory({
        query,
        limit: 120,
        offset: 0
      });
      if (gen === reloadGeneration) {
        entries = result;
      }
    } catch (error) {
      if (gen === reloadGeneration) {
        message = `${t(locale, 'loadingHistoryFailed')}: ${String(error)}`;
      }
    } finally {
      if (gen === reloadGeneration) {
        loading = false;
      }
    }
  }

  async function reloadPermission() {
    try {
      permission = await getPermissionState();
    } catch (error) {
      message = `${t(locale, 'permissionStateFailed')}: ${String(error)}`;
    }
  }

  async function bootstrap() {
    const fallbackLocale = detectInitialLocale();

    try {
      settings = await getSettings();
      hotkeyDraft = settings.hotkey;
      locale = toLocale(settings.ui_locale) ?? fallbackLocale;
    } catch (error) {
      message = `${t(locale, 'loadingSettingsFailed')}: ${String(error)}`;
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
      message = t(locale, 'autoPasted');
    } catch (error) {
      const text = String(error);
      if (text.includes('PERMISSION_DENIED')) {
        await selectEntry(entry.id, false);
        message = t(locale, 'copiedOnlyWithPermissionHint');
      } else {
        message = `${t(locale, 'selectFailed')}: ${text}`;
      }
    }

    try {
      await reloadPermission();
    } catch {
      // non-critical — paste already succeeded
    }
  }

  async function onCopyOnly(entry: ClipboardEntry) {
    message = '';
    try {
      await selectEntry(entry.id, false);
      message = t(locale, 'copiedOnly');
    } catch (error) {
      message = `${t(locale, 'selectFailed')}: ${String(error)}`;
    }
  }

  async function onDelete(entry: ClipboardEntry) {
    message = '';
    try {
      await deleteEntry(entry.id);
      await reloadEntries();
    } catch (error) {
      message = `${t(locale, 'deleteFailed')}: ${String(error)}`;
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
      message = `${t(locale, 'clearFailed')}: ${String(error)}`;
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
      message = `${t(locale, 'permissionGuideFailed')}: ${String(error)}`;
    }
  }

  async function setHotkeyCaptureMode(active: boolean): Promise<boolean> {
    try {
      await setHotkeyRecording(active);
      return true;
    } catch (error) {
      message = `${t(locale, 'hotkeySaveFailed')}: ${String(error)}`;
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
      message = t(locale, notice);
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
        message = t(locale, 'captureUnsupportedKey');
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

    if (modifiers.length === 0 && !allowsHotkeyWithoutModifier(mainKey)) {
      message = t(locale, 'captureNeedsModifier');
      return;
    }

    const hotkey = [...modifiers, mainKey].join('+');
    hotkeyDraft = hotkey;
    stopHotkeyCapture(null);
    message = `${t(locale, 'captureSuccess')}: ${hotkey}`;
  }

  async function onSaveHotkey() {
    if (!settings) {
      return;
    }

    if (!hasModifierToken(hotkeyDraft) && !allowsHotkeyWithoutModifier(hotkeyDraft)) {
      message = t(locale, 'captureNeedsModifier');
      return;
    }

    stopHotkeyCapture(null);

    savingHotkey = true;
    message = '';

    try {
      const next = await updateSettings({ hotkey: hotkeyDraft });
      settings = next;
      hotkeyDraft = next.hotkey;
      message = t(locale, 'hotkeySaved');
    } catch (error) {
      message = `${t(locale, 'hotkeySaveFailed')}: ${String(error)}`;
    } finally {
      savingHotkey = false;
    }
  }

  onMount(() => {
    void bootstrap();

    let unlistenHistory: (() => void) | undefined;
    let unlistenPermission: (() => void) | undefined;

    const historyPromise = listen('history:updated', () => {
      void reloadEntries();
    })
      .then((off) => {
        unlistenHistory = off;
      })
      .catch((err) => {
        console.error('Failed to listen history:updated:', err);
      });

    const permissionPromise = listen('permission:changed', () => {
      void reloadPermission();
    })
      .then((off) => {
        unlistenPermission = off;
      })
      .catch((err) => {
        console.error('Failed to listen permission:changed:', err);
      });

    return async () => {
      stopHotkeyCapture(null);
      void setHotkeyCaptureMode(false);
      if (searchTimer) {
        clearTimeout(searchTimer);
      }
      await Promise.all([historyPromise, permissionPromise]);
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
      <div class="lang" role="group" aria-label={t(locale, 'language')}>
        <button
          class:active={locale === 'zh-CN'}
          aria-pressed={locale === 'zh-CN'}
          on:click={() => setLocale('zh-CN')}>中</button
        >
        <button
          class:active={locale === 'en-US'}
          aria-pressed={locale === 'en-US'}
          on:click={() => setLocale('en-US')}>EN</button
        >
      </div>
      <button class="toolbar-btn" on:click={openClearConfirmModal} title={t(locale, 'clearAll')}>
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><polyline points="3 6 5 6 21 6" /><path
            d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
          /></svg
        >
      </button>
    </div>
  </header>

  <!-- Permission banner -->
  {#if settings && settings.auto_paste_default && !permission.accessibility_granted}
    <div class="permission-bar">
      <span>{t(locale, 'needsPermission')}</span>
      <button class="accent-text" on:click={onGrantPermission}
        >{t(locale, 'grantPermission')}</button
      >
    </div>
  {/if}

  <!-- Sticky search -->
  <div class="search-bar">
    <svg
      class="search-icon"
      width="15"
      height="15"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" /></svg
    >
    <input
      type="search"
      bind:value={query}
      on:input={onSearchInput}
      placeholder={t(locale, 'searchPlaceholder')}
      class="search-input"
    />
    <span class="search-count">{entries.length}</span>
    {#if loading}
      <span class="search-loading">{t(locale, 'syncing')}</span>
    {/if}
  </div>

  <!-- Toast -->
  <Toast {locale} {message} />

  <!-- Clear confirm modal -->
  {#if showClearConfirm}
    <ConfirmModal onConfirm={onConfirmClear} onCancel={closeClearConfirmModal} {locale} />
  {/if}

  <!-- Entry list (the main content) -->
  <section class="entries">
    {#if !entries.length && !loading}
      <div class="empty">{t(locale, 'noData')}</div>
    {/if}

    {#each entries as entry (entry.id)}
      <EntryCard {entry} {locale} onPaste={onPasteNow} onCopy={onCopyOnly} {onDelete} />
    {/each}
  </section>

  <!-- Settings (collapsed toggle at bottom) -->
  <section class="settings">
    <details>
      <summary>{t(locale, 'openSettings')}</summary>
      <div class="settings-body">
        <p>{t(locale, 'settingsDesc')}</p>
        <label for="hotkey-capture" class="settings-label">{t(locale, 'hotkeyLabel')}</label>
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
              {t(locale, 'captureHotkeyPending')}
            {:else}
              {t(locale, 'captureHotkey')}: {hotkeyDraft}
            {/if}
          </button>
          <button class="btn-primary" disabled={savingHotkey} on:click={onSaveHotkey}>
            {savingHotkey ? t(locale, 'saveInProgress') : t(locale, 'saveHotkey')}
          </button>
        </div>
        <small>{t(locale, 'hotkeyHint')}</small>
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
      -apple-system, 'SF Pro Text', 'Inter', 'Noto Sans SC', 'Helvetica Neue', sans-serif;
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
