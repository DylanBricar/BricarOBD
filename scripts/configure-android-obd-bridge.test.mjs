import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./configure-android-obd-bridge.mjs", import.meta.url));

for (const nested of [false, true]) {
  test(`patches a ${nested ? "nested " : ""}generated Tauri Android project idempotently`, async () => {
    const root = await mkdtemp(join(tmpdir(), "bricarobd-android-bridge-"));
    const projectRoot = nested ? join(root, "bricarobd") : root;
    try {
      const sourceDir = join(projectRoot, "app", "src", "main");
      const packageDir = join(sourceDir, "java", "com", "bricarobd", "app");
      await mkdir(packageDir, { recursive: true });
      await writeFile(
        join(packageDir, "MainActivity.kt"),
        "package com.bricarobd.app\n\nclass MainActivity : TauriActivity()\n",
      );
      await writeFile(
        join(projectRoot, "settings.gradle.kts"),
        "dependencyResolutionManagement {\n  repositories {\n    google()\n  }\n}\n",
      );
      await writeFile(join(projectRoot, "app", "build.gradle.kts"), "dependencies {\n}\n");
      await writeFile(
        join(sourceDir, "AndroidManifest.xml"),
        '<manifest xmlns:android="http://schemas.android.com/apk/res/android">\n</manifest>\n',
      );

      for (let run = 0; run < 2; run += 1) {
        const result = spawnSync(process.execPath, [scriptPath, root], { encoding: "utf8" });
        assert.equal(result.status, 0, result.stderr);
      }

      const activity = await readFile(join(packageDir, "MainActivity.kt"), "utf8");
      const bridge = await readFile(join(packageDir, "AndroidUsbBridge.kt"), "utf8");
      const settings = await readFile(join(projectRoot, "settings.gradle.kts"), "utf8");
      const gradle = await readFile(join(projectRoot, "app", "build.gradle.kts"), "utf8");
      const manifest = await readFile(join(sourceDir, "AndroidManifest.xml"), "utf8");

      assert.equal(activity.match(/addJavascriptInterface/g)?.length, 1);
      assert.match(bridge, /InetAddress\.getLoopbackAddress\(\)/);
      assert.match(bridge, /UsbSerialProber\.getDefaultProber/);
      assert.match(bridge, /SecureRandom/);
      assert.match(bridge, /BRICAROBD-AUTH/);
      assert.equal(settings.match(/https:\/\/jitpack\.io/g)?.length, 1);
      assert.equal(gradle.match(/usb-serial-for-android:3\.11\.0/g)?.length, 1);
      assert.equal(manifest.match(/android\.hardware\.usb\.host/g)?.length, 1);
    } finally {
      await rm(root, { recursive: true, force: true });
    }
  });
}
