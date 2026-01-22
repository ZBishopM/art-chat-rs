export function load<T>(key: string, defaultValue: T): T {
  if (typeof localStorage === 'undefined') return defaultValue;
  const val = localStorage.getItem(key);
  return val ? JSON.parse(val) : defaultValue;
}

export function save(key: string, value: unknown): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(key, JSON.stringify(value));
}
