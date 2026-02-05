import { getDefaultHost } from "cpal-napi";
import { AUDIO_LOGS } from "./core/common.js";
import { testBeepStream } from "./core/beep.js";
import { testInputStream, testCustomBufferStream } from "./core/recording.js";

/**
 * Main Orchestrator for the Audio Examples
 */
async function main() {
  try {
    console.log("=== CPAL-NAPI Audio Examples ===");
    
    const host = getDefaultHost();
    console.log("Current Host:", host.name());

    const outputDevice = host.defaultOutputDevice();
    const inputDevice = host.defaultInputDevice();

    // 1. Beep Test (Output only)
    if (outputDevice) {
      await testBeepStream(outputDevice);
    } else {
      console.warn(AUDIO_LOGS.NOT_FOUND('Output'));
    }

    // 2. Full Recording-Playback Cycle (Input -> Output)
    if (inputDevice && outputDevice) {
      // Record a snippet
      const { buffer, config } = await testInputStream(inputDevice);
      
      // Play it back using the output device's default config
      const outputConfig = outputDevice.defaultOutputConfig();
      await testCustomBufferStream(outputDevice, outputConfig, buffer);
    } else {
      if (!inputDevice) console.warn(AUDIO_LOGS.NOT_FOUND('Input'));
      
      // If no input, just demonstrate custom buffer generation
      if (outputDevice) {
        const config = outputDevice.defaultOutputConfig();
        await testCustomBufferStream(outputDevice, config);
      }
    }

    console.log("\nAll tests completed successfully.");
  } catch (err) {
    console.error("\n[CRITICAL ERROR]", err instanceof Error ? err.message : err);
    process.exit(1);
  }
}

// Start the tests
main().catch((err) => {
  console.error("Unhandle exception:", err);
  process.exit(1);
});
