import assert from "node:assert/strict";
import test from "node:test";
import { verifyUpdaterArtifacts } from "./verify-updater-artifacts.mjs";

function fixture(platform = "windows-x86_64", name = "BricarOBD_2.0.27_x64-setup.exe") {
  const apiUrl = "https://api.github.com/repos/example/BricarOBD/releases/assets/42";
  const browserUrl = `https://github.com/example/BricarOBD/releases/download/v2.0.27/${name}`;
  return {
    manifest: { version: "2.0.27", platforms: { [platform]: { url: apiUrl, signature: "fixture-signature" } } },
    release: {
      tag_name: "v2.0.27",
      assets: [{ name, url: apiUrl, browser_download_url: browserUrl }, { name: `${name}.sig` }],
    },
  };
}

for (const [platform, name] of [
  ["windows-x86_64", "BricarOBD_2.0.27_x64-setup.exe"],
  ["windows-x86_64-msi", "BricarOBD_2.0.27_x64_en-US.msi"],
  ["linux-x86_64", "BricarOBD_2.0.27_amd64.AppImage"],
  ["linux-x86_64-deb", "BricarOBD_2.0.27_amd64.deb"],
  ["darwin-aarch64", "BricarOBD_2.0.27_aarch64.app.tar.gz"],
]) {
  test(`accepts the signed direct updater artifact for ${platform}`, () => {
    const { manifest, release } = fixture(platform, name);
    assert.equal(verifyUpdaterArtifacts(manifest, release), 1);
    manifest.platforms[platform].url = release.assets[0].browser_download_url;
    assert.equal(verifyUpdaterArtifacts(manifest, release), 1);
  });
}

for (const [platform, name] of [
  ["windows-x86_64", "BricarOBD_2.0.27_x64-setup.exe.zip"],
  ["linux-x86_64", "BricarOBD_2.0.27_amd64.AppImage.tar.gz"],
  ["darwin-aarch64", "BricarOBD_2.0.27_aarch64.dmg"],
]) {
  test(`rejects an incompatible updater bundle for ${platform}`, () => {
    const { manifest, release } = fixture(platform, name);
    assert.throws(() => verifyUpdaterArtifacts(manifest, release), /Unsupported/);
  });
}

test("rejects missing signatures and assets from another release", () => {
  const { manifest, release } = fixture();
  release.assets.pop();
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /signature/);
  release.assets.push({ name: `${release.assets[0].name}.sig` });
  manifest.platforms["windows-x86_64"].signature = "";
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /signature/);
  manifest.platforms["windows-x86_64"].signature = "fixture-signature";
  manifest.platforms["windows-x86_64"].url = "https://example.invalid/unrelated.exe";
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /Unsupported/);
});

test("rejects a mismatched version or empty platform manifest", () => {
  const { manifest, release } = fixture();
  release.tag_name = "v2.0.26";
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /version/);
  release.tag_name = "v2.0.27";
  manifest.platforms = {};
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /no platforms/);
});

test("rejects plaintext artifact URLs even when they belong to the release", () => {
  const { manifest, release } = fixture();
  const url = "http://example.invalid/fixture.exe";
  manifest.platforms["windows-x86_64"].url = url;
  release.assets[0].url = url;
  assert.throws(() => verifyUpdaterArtifacts(manifest, release), /HTTPS/);
});
