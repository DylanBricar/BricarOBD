export function formatCatalogSize(bytes: number): string {
  return `${Math.round(bytes / 1024 / 1024)} Mo`;
}
