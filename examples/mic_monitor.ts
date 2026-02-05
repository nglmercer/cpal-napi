import { AudioEngine } from "../lib/core/AudioEngine.js";
import { AudioCable } from "../lib/core/Cable.js";
import { DeviceScanner } from "../lib/utils/Scanner.js";
import { VolumeMeter } from "../lib/utils/Meter.js";
import { prepareShutdown, setSuppressAlsaLogs } from "../index.js";

async function main() {
  // Suppress ALSA stderr messages (enabled by default, but can be disabled for debugging)
  setSuppressAlsaLogs(false); // Uncomment this to see ALSA debug messages
  
  const engine = AudioEngine.getInstance();
  console.log(`Using Host: ${engine.getHost().name()}`);

  // 1. Find a microphone
  const inputs = DeviceScanner.getInputs();
  const mic = inputs.find(d => d.name().toLowerCase().includes("usb") || d.name().toLowerCase().includes("mic")) 
            || engine.getDefaultInputDevice();

  // 2. Find speakers
  const speakers = engine.getDefaultOutputDevice();

  if (!mic || !speakers) {
    console.error("Could not find input or output device.");
    process.exit(1);
  }

  console.log(`\n--- Monitoring ---`);
  console.log(`Input:  ${mic.name()}`);
  console.log(`Output: ${speakers.name()}`);
  console.log(`------------------\n`);

  // 3. Create a cable
  const cable = new AudioCable(mic, speakers, {
    bufferSize: 256, // Low latency
  });

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
    
    // Stop the cable first
    cable.stop();
    
    // Prepare for shutdown - this gives ALSA time to cleanup
    console.log("Cleaning up audio resources...");
    prepareShutdown();
    
    console.log("Cleanup complete. Exiting...");
    process.exit(0);
  });
}

main().catch(console.error);

