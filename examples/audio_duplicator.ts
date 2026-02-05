import { AudioEngine } from "../lib/core/AudioEngine.js";
import { DeviceScanner } from "../lib/utils/Scanner.js";

async function main() {
  const engine = AudioEngine.getInstance();
  
  // 1. Get input
  const mic = engine.getDefaultInputDevice();
  if (!mic) throw new Error("No default input found");

  // 2. Get two outputs
  const outputs = DeviceScanner.getOutputs();
  if (outputs.length < 1) throw new Error("Need at least 1 output");

  const out1 = engine.getDefaultOutputDevice();
  // Find another output that isn't the default, or just use the same one for testing
  const out2 = outputs.find(d => d.name() !== out1?.name()) || out1;

  if (!out1 || !out2) throw new Error("Outputs not found");

  console.log(`Duplicating: ${mic.name()}`);
  console.log(`To Output 1: ${out1.name()}`);
  console.log(`To Output 2: ${out2.name()}`);

  const config = out1.defaultOutputConfig();
  // Use a slightly larger buffer to avoid underruns on some systems
  config.bufferSize = { type: 'Fixed', field0: 1024 };

  // 3. Setup cloning
  const { AudioBuffer } = await import("../index.js");
  const buffer1 = new AudioBuffer();
  const buffer2 = buffer1.cloneLink(); // buffer1 will now broadcast to buffer2

  // 4. Create streams
  const inputStream = mic.createInputStream(config, buffer1);
  const outputStream1 = out1.createOutputStream(config, buffer1);
  const outputStream2 = out2.createOutputStream(config, buffer2);

  console.log("\nStarting streams...");
  inputStream.play();
  outputStream1.play();
  outputStream2.play();

  console.log("Running. Press Ctrl+C to stop.");
  
  process.on('SIGINT', () => {
    inputStream.pause();
    outputStream1.pause();
    outputStream2.pause();
    process.exit(0);
  });

  // Keep alive
  setInterval(() => {}, 1000);
}

main().catch(console.error);
