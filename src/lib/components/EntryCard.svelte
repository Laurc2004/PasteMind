<script lang="ts">
  /* eslint-disable no-unused-vars */
  import type { ClipboardEntry } from '$lib/types';
  import type { Locale } from '$lib/i18n';
  import { t } from '$lib/i18n';
  import { formatTime, formatBytes } from '$lib/format';
  import { toFileSrc } from '$lib/file';

  let {
    entry,
    locale,
    onPaste,
    onCopy,
    onDelete
  }: {
    entry: ClipboardEntry;
    locale: Locale;
    onPaste: (e: ClipboardEntry) => void;
    onCopy: (e: ClipboardEntry) => void;
    onDelete: (e: ClipboardEntry) => void;
  } = $props();
</script>

<article class="entry">
  <div class="entry-row">
    <button
      class="entry-content"
      aria-label={t(locale, 'clickToPaste')}
      onclick={() => onPaste(entry)}
    >
      {#if entry.kind === 'image' && (entry.image_preview_data || entry.image_path)}
        <img
          src={entry.image_preview_data ?? toFileSrc(entry.image_path)}
          alt={t(locale, 'imageAlt')}
          loading="lazy"
          decoding="async"
        />
      {:else}
        <span class="entry-text">{entry.text_preview ?? '-'}</span>
      {/if}
    </button>
    <div class="entry-actions">
      <button class="btn-icon" title={t(locale, 'pasteNow')} onclick={() => onPaste(entry)}>
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><path d="M15 2H9a1 1 0 0 0-1 1v2a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3a1 1 0 0 0-1-1Z" /><path
            d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2h-2"
          /><path d="M12 11v6" /><path d="m9 14 3-3 3 3" /></svg
        >
      </button>
      <button class="btn-icon" title={t(locale, 'copyOnly')} onclick={() => onCopy(entry)}>
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><rect width="14" height="14" x="8" y="8" rx="2" /><path
            d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"
          /></svg
        >
      </button>
      <button
        class="btn-icon btn-icon-danger"
        title={t(locale, 'delete')}
        onclick={() => onDelete(entry)}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><line x1="18" x2="6" y1="6" y2="18" /><line x1="6" x2="18" y1="6" y2="18" /></svg
        >
      </button>
    </div>
  </div>
  <div class="entry-meta">
    <span class="meta-source">{entry.source_app || t(locale, 'sourceUnknown')}</span>
    <span class="meta-dot">&middot;</span>
    <span>{formatTime(entry.created_at, locale)}</span>
    <span class="meta-dot">&middot;</span>
    <span>{formatBytes(entry.size_bytes)}</span>
    <span class="meta-kind">{entry.kind === 'image' ? t(locale, 'image') : t(locale, 'text')}</span>
  </div>
</article>

<style>
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
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    font-size: 13px;
    line-height: 1.4;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .entry :global(img) {
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

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
