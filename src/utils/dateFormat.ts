function toValidDate(dateString?: string): Date | null {
  if (!dateString) return null;
  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return null;
  return date;
}

export function formatDateYmd(dateString?: string, fallback = '—'): string {
  const date = toValidDate(dateString);
  if (!date) return fallback;
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(
    date.getDate(),
  ).padStart(2, '0')}`;
}

export function formatDateYmdHm(dateString?: string, fallback = '—'): string {
  const date = toValidDate(dateString);
  if (!date) return fallback;
  return `${formatDateYmd(dateString, fallback)} ${String(date.getHours()).padStart(2, '0')}:${String(
    date.getMinutes(),
  ).padStart(2, '0')}`;
}

export function formatDateMonthDay(dateString?: string, fallback = '—'): string {
  const date = toValidDate(dateString);
  if (!date) return fallback;
  return `${date.getMonth() + 1}月${date.getDate()}日`;
}
