import { readdir, readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

const androidRoot = resolve(process.argv[2] ?? "src-tauri/gen/android");
const javaRoot = join(androidRoot, "app", "src", "main", "java");
const manifestPath = join(androidRoot, "app", "src", "main", "AndroidManifest.xml");
const appGradlePath = join(androidRoot, "app", "build.gradle.kts");
const settingsPath = join(androidRoot, "settings.gradle.kts");

async function findFile(directory, fileName) {
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await findFile(path, fileName);
      if (nested) return nested;
    } else if (entry.name === fileName) {
      return path;
    }
  }
  return null;
}

function requireMarker(contents, marker, file) {
  if (!contents.includes(marker)) {
    throw new Error(`Unsupported Android template: missing ${marker} in ${file}`);
  }
}

const mainActivityPath = await findFile(javaRoot, "MainActivity.kt");
if (!mainActivityPath) throw new Error("Generated MainActivity.kt was not found");

let mainActivity = await readFile(mainActivityPath, "utf8");
const packageMatch = mainActivity.match(/^package\s+([\w.]+)/m);
if (!packageMatch) throw new Error("Generated MainActivity.kt has no package declaration");
const packageName = packageMatch[1];

if (!mainActivity.includes('addJavascriptInterface(AndroidUsbBridge(this), "AndroidUsb")')) {
  requireMarker(mainActivity, "class MainActivity : TauriActivity()", mainActivityPath);
  mainActivity = mainActivity.replace(packageMatch[0], `${packageMatch[0]}\n\nimport android.webkit.WebView`);
  mainActivity = mainActivity.replace(
    "class MainActivity : TauriActivity()",
    `class MainActivity : TauriActivity() {
  override fun onWebViewCreate(webView: WebView) {
    webView.addJavascriptInterface(AndroidUsbBridge(this), "AndroidUsb")
  }
}`,
  );
  await writeFile(mainActivityPath, mainActivity, "utf8");
}

const bridgePath = join(mainActivityPath, "..", "AndroidUsbBridge.kt");
const bridgeSource = `package ${packageName}

import android.app.Activity
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.hardware.usb.UsbManager
import android.os.Build
import android.webkit.JavascriptInterface
import com.hoho.android.usbserial.driver.UsbSerialPort
import com.hoho.android.usbserial.driver.UsbSerialProber
import org.json.JSONArray
import org.json.JSONObject
import java.io.ByteArrayOutputStream
import java.io.InputStream
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketTimeoutException
import java.security.MessageDigest
import java.security.SecureRandom
import kotlin.concurrent.thread

class AndroidUsbBridge(private val activity: Activity) {
  private val usbManager = activity.getSystemService(Context.USB_SERVICE) as UsbManager
  private var bridge: UsbSerialTcpBridge? = null

  @JavascriptInterface
  fun listDevices(): String {
    val devices = JSONArray()
    UsbSerialProber.getDefaultProber().findAllDrivers(usbManager).forEach { driver ->
      val device = driver.device
      devices.put(
        JSONObject()
          .put("name", device.productName ?: "USB Serial")
          .put("deviceId", device.deviceId.toString())
          .put("vendorId", String.format("%04X", device.vendorId))
          .put("productId", String.format("%04X", device.productId))
      )
    }
    return devices.toString()
  }

  @JavascriptInterface
  fun requestPermission(deviceId: Int): Boolean {
    val device = usbManager.deviceList.values.firstOrNull { it.deviceId == deviceId }
      ?: return false
    if (usbManager.hasPermission(device)) return true
    val intent = Intent(ACTION_USB_PERMISSION).setPackage(activity.packageName)
    val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      PendingIntent.FLAG_MUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
    } else {
      PendingIntent.FLAG_UPDATE_CURRENT
    }
    usbManager.requestPermission(device, PendingIntent.getBroadcast(activity, deviceId, intent, flags))
    return false
  }

  @JavascriptInterface
  @Synchronized
  fun startBridge(deviceId: Int, baudRate: Int): String {
    if (baudRate !in 1_200..2_000_000) return errorJson("Unsupported baud rate")
    return try {
      val driver = UsbSerialProber.getDefaultProber().findAllDrivers(usbManager)
        .firstOrNull { it.device.deviceId == deviceId }
        ?: return errorJson("No supported USB serial driver")
      if (!usbManager.hasPermission(driver.device)) return errorJson("USB permission is required")
      val connection = usbManager.openDevice(driver.device)
        ?: return errorJson("Unable to open USB device")
      val port = driver.ports.firstOrNull()
        ?: run {
          connection.close()
          return errorJson("USB serial device has no port")
        }
      stopBridge()
      val token = ByteArray(32).also(SecureRandom()::nextBytes)
        .joinToString("") { String.format("%02x", it) }
      val nextBridge = UsbSerialTcpBridge(port, connection, token)
      bridge = nextBridge
      port.open(connection)
      port.setParameters(baudRate, 8, UsbSerialPort.STOPBITS_1, UsbSerialPort.PARITY_NONE)
      runCatching { port.dtr = true }
      runCatching { port.rts = true }
      val localPort = nextBridge.start()
      JSONObject().put("ok", true).put("port", localPort).put("token", token).toString()
    } catch (error: Exception) {
      stopBridge()
      errorJson(error.message ?: error.javaClass.simpleName)
    }
  }

  @JavascriptInterface
  @Synchronized
  fun stopBridge() {
    bridge?.close()
    bridge = null
  }

  @JavascriptInterface
  fun isRunning(): Boolean = bridge?.isRunning() == true

  private fun errorJson(message: String): String =
    JSONObject().put("ok", false).put("error", message).toString()

  companion object {
    private const val ACTION_USB_PERMISSION = "${packageName}.USB_PERMISSION"
  }
}

private class UsbSerialTcpBridge(
  private val serialPort: UsbSerialPort,
  private val usbConnection: android.hardware.usb.UsbDeviceConnection,
  private val authenticationToken: String,
) {
  @Volatile private var running = false
  private var serverSocket: ServerSocket? = null
  private var clientSocket: Socket? = null

  fun start(): Int {
    val server = ServerSocket(0, 1, InetAddress.getLoopbackAddress())
    server.soTimeout = 500
    serverSocket = server
    running = true
    thread(start = true, isDaemon = true, name = "bricarobd-usb-bridge") {
      runBridge(server)
    }
    return server.localPort
  }

  private fun runBridge(server: ServerSocket) {
    try {
      while (running && clientSocket == null) {
        try {
          clientSocket = server.accept()
        } catch (_: SocketTimeoutException) {
          // Re-check the running flag.
        }
      }
      val socket = clientSocket ?: return
      socket.soTimeout = 2_000
      socket.tcpNoDelay = true
      val input = socket.getInputStream()
      if (!authenticate(input)) return
      socket.soTimeout = 20
      val output = socket.getOutputStream()
      val tcpBuffer = ByteArray(4096)
      val serialBuffer = ByteArray(4096)
      while (running) {
        try {
          val count = input.read(tcpBuffer)
          if (count < 0) break
          if (count > 0) serialPort.write(tcpBuffer.copyOf(count), 2_000)
        } catch (_: SocketTimeoutException) {
          // Serial reads still need to be serviced when TCP is idle.
        }
        val serialCount = serialPort.read(serialBuffer, 50)
        if (serialCount > 0) {
          output.write(serialBuffer, 0, serialCount)
          output.flush()
        }
      }
    } catch (_: Exception) {
      // The frontend observes the closed loopback connection and reports the disconnect.
    } finally {
      close()
    }
  }

  private fun authenticate(input: InputStream): Boolean {
    val received = ByteArrayOutputStream()
    while (received.size() <= 128) {
      val byte = input.read()
      if (byte < 0) return false
      if (byte == '\n'.code) break
      if (byte != '\r'.code) received.write(byte)
    }
    val expected = "BRICAROBD-AUTH $authenticationToken".toByteArray(Charsets.UTF_8)
    return MessageDigest.isEqual(received.toByteArray(), expected)
  }

  @Synchronized
  fun close() {
    if (!running && serverSocket == null && clientSocket == null) return
    running = false
    runCatching { clientSocket?.close() }
    runCatching { serverSocket?.close() }
    runCatching { serialPort.close() }
    runCatching { usbConnection.close() }
    clientSocket = null
    serverSocket = null
  }

  fun isRunning(): Boolean = running
}
`;
await writeFile(bridgePath, bridgeSource, "utf8");

let settings = await readFile(settingsPath, "utf8");
if (!settings.includes('maven(url = "https://jitpack.io")')) {
  const dependencyRepositories = /dependencyResolutionManagement\s*\{[\s\S]*?repositories\s*\{/;
  const match = settings.match(dependencyRepositories);
  if (!match) throw new Error("Unsupported Android template: dependency repositories not found");
  settings = settings.replace(match[0], `${match[0]}\n        maven(url = "https://jitpack.io")`);
  await writeFile(settingsPath, settings, "utf8");
}

let appGradle = await readFile(appGradlePath, "utf8");
const serialDependency = 'implementation("com.github.mik3y:usb-serial-for-android:3.11.0")';
if (!appGradle.includes(serialDependency)) {
  requireMarker(appGradle, "dependencies {", appGradlePath);
  appGradle = appGradle.replace("dependencies {", `dependencies {\n    ${serialDependency}`);
  await writeFile(appGradlePath, appGradle, "utf8");
}

let manifest = await readFile(manifestPath, "utf8");
const usbFeature = '<uses-feature android:name="android.hardware.usb.host" android:required="false" />';
if (!manifest.includes(usbFeature)) {
  const manifestTag = manifest.match(/<manifest[^>]*>/)?.[0];
  if (!manifestTag) throw new Error("Unsupported Android template: manifest tag not found");
  manifest = manifest.replace(manifestTag, `${manifestTag}\n    ${usbFeature}`);
  await writeFile(manifestPath, manifest, "utf8");
}

console.log(`Configured Android USB OBD bridge for ${packageName}`);
