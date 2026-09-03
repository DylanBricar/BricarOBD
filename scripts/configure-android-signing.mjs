import { chmod, readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";

const gradlePath = resolve(process.argv[2] ?? "src-tauri/gen/android/app/build.gradle.kts");
const propertiesPath = resolve(process.argv[3] ?? "src-tauri/gen/android/keystore.properties");

function requireEnvironment(name) {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}

function escapeJavaProperty(value) {
  let escaped = "";
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    const character = value[index];
    if (character === "\\") escaped += "\\\\";
    else if (character === "\n") escaped += "\\n";
    else if (character === "\r") escaped += "\\r";
    else if (character === "\t") escaped += "\\t";
    else if ("=:# !".includes(character)) escaped += `\\${character}`;
    else if (code < 0x20 || code > 0x7e) {
      escaped += `\\u${code.toString(16).padStart(4, "0")}`;
    } else escaped += character;
  }
  return escaped;
}

const keyAlias = requireEnvironment("ANDROID_KEY_ALIAS");
const keyPassword = requireEnvironment("ANDROID_KEY_PASSWORD");
const keyStorePath = resolve(requireEnvironment("ANDROID_KEYSTORE_PATH"));

let gradle = await readFile(gradlePath, "utf8");

if (!gradle.includes('signingConfig = signingConfigs.getByName("release")')) {
  const importMarker = "import java.util.Properties\n";
  const buildTypesMarker = "    buildTypes {\n";
  const releaseMarker = '        getByName("release") {\n';

  if (!gradle.includes(importMarker)) {
    throw new Error(`Unsupported Gradle template: missing ${importMarker.trim()}`);
  }
  if (!gradle.includes(buildTypesMarker) || !gradle.includes(releaseMarker)) {
    throw new Error("Unsupported Gradle template: build type markers not found");
  }

  gradle = gradle.replace(importMarker, `${importMarker}import java.io.FileInputStream\n`);
  gradle = gradle.replace(
    buildTypesMarker,
    `    signingConfigs {
        create("release") {
            val keystorePropertiesFile = rootProject.file("keystore.properties")
            val keystoreProperties = Properties()
            require(keystorePropertiesFile.exists()) {
                "Missing Android release signing configuration"
            }
            keystoreProperties.load(FileInputStream(keystorePropertiesFile))
            keyAlias = keystoreProperties.getProperty("keyAlias")
            keyPassword = keystoreProperties.getProperty("password")
            storeFile = file(keystoreProperties.getProperty("storeFile"))
            storePassword = keystoreProperties.getProperty("password")
        }
    }
    buildTypes {
`,
  );
  gradle = gradle.replace(
    releaseMarker,
    `${releaseMarker}            signingConfig = signingConfigs.getByName("release")\n`,
  );

  await writeFile(gradlePath, gradle, "utf8");
}

const properties = [
  `keyAlias=${escapeJavaProperty(keyAlias)}`,
  `password=${escapeJavaProperty(keyPassword)}`,
  `storeFile=${escapeJavaProperty(keyStorePath)}`,
  "",
].join("\n");

await writeFile(propertiesPath, properties, { encoding: "utf8", mode: 0o600 });
await chmod(propertiesPath, 0o600);
