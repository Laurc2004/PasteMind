<script lang="ts">
  import type { Locale } from '$lib/i18n';
  import { t } from '$lib/i18n';

  let {
    onConfirm,
    onCancel,
    locale
  }: { onConfirm: () => void; onCancel: () => void; locale: Locale } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="modal-backdrop" onclick={onCancel}>
  <div
    class="modal"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    aria-labelledby="clear-confirm-title"
    onclick={(e) => e.stopPropagation()}
  >
    <h2 id="clear-confirm-title">{t(locale, 'clearConfirmTitle')}</h2>
    <p>{t(locale, 'clearConfirmDesc')}</p>
    <div class="modal-actions">
      <button class="btn-ghost" onclick={onCancel}>{t(locale, 'clearConfirmCancel')}</button>
      <button class="btn-danger" onclick={onConfirm}>{t(locale, 'clearConfirmAccept')}</button>
    </div>
  </div>
</div>

<style>
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

  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
