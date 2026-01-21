import {
  getDefaultHost,
  availableHosts,
  AudioBuffer,
  AudioDevice,
  StreamConfig,
} from "../index.js";

// --- Configuration & Constants ---
const AUDIO_CONFIG = {
  BEEP_FREQ: 440,
  BUFFER_FREQ: 880,
  BEEP_DURATION_MS: 1000,
  BUFFER_DURATION_MS: 2000,
};

const LOG_MESSAGES = {
  START_BEEP: "\n--- Test 1: Beep Stream ---",
  START_BUFFER: "\n--- Test 2: AudioBuffer Stream ---",
  START_INPUT: "\n--- Test 3: Input (Recording) Stream ---",
  FINISHED: "Finished audio tests.",
  NO_DEVICE: "No default output device found.",
  NO_INPUT_DEVICE: "No default input device found.",
};

// --- Helpers ---
const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Test the built-in beep stream functionality.
 */
async function testBeepStream(device: AudioDevice) {
  console.log(LOG_MESSAGES.START_BEEP);
  const stream = device.createBeepStream();
  
  console.log(`Playing ${AUDIO_CONFIG.BEEP_FREQ}Hz beep for ${AUDIO_CONFIG.BEEP_DURATION_MS}ms...`);
  stream.play();
  await delay(AUDIO_CONFIG.BEEP_DURATION_MS);
  stream.pause();
}

/**
 * Test recording audio from default input device.
 */
async function testInputStream(device: AudioDevice) {
  console.log(LOG_MESSAGES.START_INPUT);
  
  const config = device.defaultInputConfig();
  console.log("Default Input Config:", config);
  
  const buffer = new AudioBuffer();
  const stream = device.createInputStream(config, buffer);
  
  console.log(`Recording for ${AUDIO_CONFIG.BUFFER_DURATION_MS}ms...`);
  stream.play();
  await delay(AUDIO_CONFIG.BUFFER_DURATION_MS);
  stream.pause();
  
  console.log(`Recorded ${buffer.length()} samples.`);
  
  // Optional: Play back the recorded audio if an output device is available
  return { buffer, config };
}

/**
 * Test custom audio buffer playback.
 */
async function testCustomBufferStream(device: AudioDevice, config: StreamConfig, buffer?: AudioBuffer) {
  console.log(LOG_MESSAGES.START_BUFFER);
  
  const targetBuffer = buffer || new AudioBuffer();
  
  if (!buffer) {
    const sampleRate = config.sampleRate;
    const durationSec = AUDIO_CONFIG.BUFFER_DURATION_MS / 1000;
    const numSamples = Math.floor(sampleRate * durationSec);
    const samples = new Float32Array(numSamples);

    // Generate sine wave samples
    const angleStep = (2 * Math.PI * AUDIO_CONFIG.BUFFER_FREQ) / sampleRate;
    for (let i = 0; i < numSamples; i++) {
      samples[i] = Math.sin(angleStep * i);
    }
    
    targetBuffer.push(samples);
  }
  
  console.log(`Initialized buffer with ${targetBuffer.length()} samples.`);

  const stream = device.createOutputStream(config, targetBuffer);
  console.log(`Playing ${buffer ? "recorded audio" : `custom ${AUDIO_CONFIG.BUFFER_FREQ}Hz buffer`} for ${AUDIO_CONFIG.BUFFER_DURATION_MS}ms...`);
  
  stream.play();
  await delay(AUDIO_CONFIG.BUFFER_DURATION_MS);
  stream.pause();
}

// --- Main execution ---
async function runDevExample() {
  try {
    console.log("Detecting audio hosts...");
    console.log("Available Hosts:", availableHosts());
    
    const host = getDefaultHost();
    console.log("Current Host:", host.name());

    const output = host.defaultOutputDevice();
    const input = host.defaultInputDevice();

    if (output) {
      console.log("Using Output Device:", output.name());
      const config = output.defaultOutputConfig();
      console.log("Default Output Configuration:", config);

      // Execute output tests
      await testBeepStream(output);
      await testCustomBufferStream(output, config);
    } else {
      console.warn(LOG_MESSAGES.NO_DEVICE);
    }

    if (input && output) {
      const { buffer } = await testInputStream(input);
      const outConfig = output.defaultOutputConfig();
      await testCustomBufferStream(output, outConfig, buffer);
    } else if (!input) {
      console.warn(LOG_MESSAGES.NO_INPUT_DEVICE);
    }
    
    console.log(LOG_MESSAGES.FINISHED);
  } catch (err) {
    console.error("Critical error during audio test:", err);
  }
}

runDevExample().catch(console.error);

