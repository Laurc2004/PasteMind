<script lang="ts">
  import type { Locale } from '$lib/i18n';
  import { t } from '$lib/i18n';

  let {
    locale,
    message
  }: {
    locale: Locale;
    message: string;
  } = $props();
</script>

{#if message}
  <div class="toast" aria-live="polite" aria-label={t(locale, 'statusLiveRegion')}>
    {message}
  </div>
{/if}

<style>
  .toast {
    padding: 7px 12px;
    font-size: 12px;
    color: var(--accent-hover);
    background: var(--accent-subtle);
    border-bottom: 1px solid rgb(79 70 229 / 15%);
    flex-shrink: 0;
  }
</style>
