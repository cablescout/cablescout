export function to_ini<T>(obj: T): string {
  return Object.entries(obj).map(
    ([section, items]) => {
      const items_formatted = Object.entries(items).map(([key, value]) => `${key}=${value}`).join('\n')
      return `[${section}]\n${items_formatted}`
    }
  ).join('\n\n')
}
