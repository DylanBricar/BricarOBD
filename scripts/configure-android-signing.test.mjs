import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { mkdtemp } from "node:fs/promises";
import test from "node:test";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./configure-android-signing.mjs", import.meta.url));

const gradleTemplate = `import java.util.Properties

android {
    buildTypes {
        getByName("debug") {
            isDebuggable = true
        }
        getByName("release") {
            isMinifyEnabled = true
        }
    }
}
`;

test("configures release signing idempotently and safely escapes properties", async () => {
  const directory = await mkdtemp(join(tmpdir(), "bricarobd-signing-"));
  const gradlePath = join(directory, "build.gradle.kts");
  const propertiesPath = join(directory, "keystore.properties");
  const keyStorePath = join(directory, "release key.jks");
  await writeFile(gradlePath, gradleTemplate, "utf8");

  const environment = {
    ...process.env,
    ANDROID_KEY_ALIAS: "release:key",
    ANDROID_KEY_PASSWORD: "safe pass=word",
    ANDROID_KEYSTORE_PATH: keyStorePath,
  };

  for (let run = 0; run < 2; run += 1) {
    const result = spawnSync(process.execPath, [scriptPath, gradlePath, propertiesPath], {
      encoding: "utf8",
      env: environment,
    });
    assert.equal(result.status, 0, result.stderr);
  }

  const gradle = await readFile(gradlePath, "utf8");
  assert.equal(gradle.match(/import java\.io\.FileInputStream/g)?.length, 1);
  assert.equal(gradle.match(/signingConfigs \{/g)?.length, 1);
  assert.equal(gradle.match(/signingConfig = signingConfigs\.getByName\("release"\)/g)?.length, 1);

  const properties = await readFile(propertiesPath, "utf8");
  assert.match(properties, /^keyAlias=release\\:key$/m);
  assert.match(properties, /^password=safe\\ pass\\=word$/m);
  assert.match(properties, /^storeFile=.*release\\ key\.jks$/m);
});

test("fails closed when the generated Gradle template is unexpected", async () => {
  const directory = await mkdtemp(join(tmpdir(), "bricarobd-signing-"));
  const gradlePath = join(directory, "build.gradle.kts");
  const propertiesPath = join(directory, "keystore.properties");
  await writeFile(gradlePath, "android {}\n", "utf8");

  const result = spawnSync(process.execPath, [scriptPath, gradlePath, propertiesPath], {
    encoding: "utf8",
    env: {
      ...process.env,
      ANDROID_KEY_ALIAS: "release",
      ANDROID_KEY_PASSWORD: "password",
      ANDROID_KEYSTORE_PATH: join(directory, "release.jks"),
    },
  });

  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Unsupported Gradle template/);
});
