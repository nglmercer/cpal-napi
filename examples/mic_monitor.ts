import { AudioEngine } from "../lib/core/AudioEngine.js";
import { AudioCable } from "../lib/core/Cable.js";
import { DeviceScanner } from "../lib/utils/Scanner.js";
import { VolumeMeter } from "../lib/utils/Meter.js";
import { prepareShutdown, AudioDevice } from "../index.js";

async function main() {
  // Suppress ALSA stderr messages (enabled by default, but can be disabled for debugging)
  
  const engine = AudioEngine.getInstance();
  console.log(`Using Host: ${engine.getHost().name()}`);

  // 1. Find a compatible input/output pair
  const inputs = DeviceScanner.getInputs();
  const outputs = DeviceScanner.getOutputs();

  if (inputs.length === 0) {
    console.error("No input devices found.");
    process.exit(1);
  }
  if (outputs.length === 0) {
    console.error("No output devices found.");
    process.exit(1);
  }

  // Prefer default devices if they are compatible
  const defaultInput = engine.getDefaultInputDevice();
  const defaultOutput = engine.getDefaultOutputDevice();

  let selectedInput: AudioDevice | null = null;
  let selectedOutput: AudioDevice | null = null;
  let cable: AudioCable | null = null;

  // Helper to test a pair
  const testPair = (input: AudioDevice, output: AudioDevice): AudioCable | null => {
    try {
      return new AudioCable(input, output, { bufferSize: 256 });
    } catch (e) {
      return null;
    }
  };

  // Try default pair first
  if (defaultInput && defaultOutput) {
    const testCable = testPair(defaultInput, defaultOutput);
    if (testCable) {
      selectedInput = defaultInput;
      selectedOutput = defaultOutput;
      cable = testCable;
    }
  }

  // If not found, try all combinations
  if (!cable) {
    for (const input of inputs) {
      for (const output of outputs) {
        // Skip if already tested (default pair)
        if (defaultInput && defaultOutput && input === defaultInput && output === defaultOutput) continue;
        const testCable = testPair(input, output);
        if (testCable) {
          selectedInput = input;
          selectedOutput = output;
          cable = testCable;
          break;
        }
      }
      if (cable) break;
    }
  }

  if (!cable) {
    console.error("Could not find any compatible input/output pair.");
    process.exit(1);
  }

  console.log(`\n--- Monitoring ---`);
  console.log(`Input:  ${selectedInput!.name()}`);
  console.log(`Output: ${selectedOutput!.name()}`);
  console.log(`------------------\n`);

  console.log("Starting monitor... Press Ctrl+C to stop.");
  cable.start();

  // Visualization
  const vizInterval = setInterval(() => {
    if (cable.isRunning()) {
      const level = VolumeMeter.getPeak(cable.getBuffer());
      const bars = "#".repeat(Math.floor(level * 50));
      const bufferLen = cable.getBuffer().length();
      process.stdout.write(`\rVolume: [${bars.padEnd(50)}] | Buffer: ${bufferLen} samples    `);
    }
  }, 50);

  // Keep alive
  let isStopping = false;
  process.on('SIGINT', async () => {
    if (isStopping) return;
    isStopping = true;
    
    console.log("\nStopping...");
    clearInterval(vizInterval);
    
    // First, prepare for shutdown - this signals the native code to start cleanup
    console.log("Preparing shutdown...");
    prepareShutdown();
    
    // Now stop the cable (streams will use mem::forget during shutdown to prevent double-free)
    cable.stop();
    
    console.log("Cleanup complete. Exiting...");
    
    // Use setTimeout to allow any remaining async cleanup to complete
    // This prevents the runtime from racing with ALSA's cleanup
    setTimeout(() => {
      process.exit(0);
    }, 100);
  });
}

main().catch(console.error);

