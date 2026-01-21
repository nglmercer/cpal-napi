import {
  getDefaultHost,
  availableHosts,
  AudioBuffer,
} from "../index.js";

async function main() {
  console.log("Available Hosts:", availableHosts());
  
  const host = getDefaultHost();
  console.log("Default Host:", host.name());

  const defaultOutput = host.defaultOutputDevice();
  if (defaultOutput) {
    console.log("Default Output Device:", defaultOutput.name());
    
    try {
      const config = defaultOutput.defaultOutputConfig();
      console.log("Default Output Config:", config);

      // --- Test 1: Beep Stream ---
      console.log("\n--- Test 1: Beep Stream ---");
      const beepStream = defaultOutput.createBeepStream();
      console.log("Playing beep for 1 second...");
      beepStream.play();
      await new Promise(resolve => setTimeout(resolve, 1000));
      beepStream.pause();

      // --- Test 2: AudioBuffer Stream ---
      console.log("\n--- Test 2: AudioBuffer Stream ---");
      const audioBuffer = new AudioBuffer();
      
      // Fill buffer with a different tone (880Hz)
      const sampleRate = config.sampleRate;
      const duration = 2; // seconds
      const numSamples = sampleRate * duration;
      const samples = new Float32Array(numSamples);
      for (let i = 0; i < numSamples; i++) {
        samples[i] = Math.sin(2 * Math.PI * 880 * i / sampleRate);
      }
      audioBuffer.push(samples);
      console.log("Buffer length:", audioBuffer.length());

      const customStream = defaultOutput.createOutputStream(config, audioBuffer);
      console.log("Playing custom buffer for 2 seconds...");
      customStream.play();
      
      await new Promise(resolve => setTimeout(resolve, 2000));
      customStream.pause();
      console.log("Finished.");

    } catch (e) {
      console.error("Error with stream:", e);
    }
  } else {
    console.log("No default output device found");
  }
}

main().catch(console.error);
