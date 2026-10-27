import { readFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const updaterFormats = [
  ["windows-", /\.(exe|msi)$/i],
  ["linux-", /\.(AppImage|deb|rpm)$/],
  ["darwin-", /\.app\.tar\.gz$/],
];

export function verifyUpdaterArtifacts(manifest, release) {
  if (!manifest.version || manifest.version !== release.tag_name?.replace(/^v/, "")) {
    throw new Error("Updater version does not match its release tag");
  }
  const assets = new Map();
  const names = new Set();
  for (const asset of release.assets ?? []) {
    names.add(asset.name);
    for (const url of [asset.url, asset.browser_download_url]) {
      if (url) assets.set(url, asset.name);
    }
  }
  const entries = Object.entries(manifest.platforms ?? {});
  if (!entries.length) throw new Error("Updater manifest has no platforms");
  for (const [platform, entry] of entries) {
    const format = updaterFormats.find(([prefix]) => platform.startsWith(prefix))?.[1];
    const name = assets.get(entry.url);
    if (!format || !name || !format.test(name)) {
      throw new Error(`Unsupported Tauri v2 updater artifact for ${platform}: ${name ?? entry.url}`);
    }
    if (new URL(entry.url).protocol !== "https:") {
      throw new Error(`Updater artifact must use HTTPS: ${platform}`);
    }
    if (typeof entry.signature !== "string" || !entry.signature.trim() || !names.has(`${name}.sig`)) {
      throw new Error(`Missing updater signature or signature asset: ${platform}`);
    }
  }
  return entries.length;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [manifestPath, releasePath] = process.argv.slice(2);
  if (!manifestPath || !releasePath) {
    throw new Error("Usage: node verify-updater-artifacts.mjs <latest.json> <release.json>");
  }
  const [manifest, release] = await Promise.all(
    [manifestPath, releasePath].map(async (path) => JSON.parse(await readFile(path, "utf8"))),
  );
  const count = verifyUpdaterArtifacts(manifest, release);
  console.log(`Validated ${count} signed Tauri v2 updater entries for ${manifest.version}`);
}
