export function formatTime(ts: string, locale: string): string {
  const value = new Date(ts);
  return new Intl.DateTimeFormat(locale, {
    hour: '2-digit',
    minute: '2-digit',
    month: '2-digit',
    day: '2-digit'
  }).format(value);
}

export function formatBytes(value: number): string {
  if (value < 1024) {
    return `${value}B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)}KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)}MB`;
}
