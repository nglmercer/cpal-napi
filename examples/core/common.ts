import { AudioDevice, AudioStream } from "cpal-napi";

/**
 * Audio Test Settings
 */
export const AUDIO_SETTINGS = {
  BEEP_FREQUENCY: 440,
  BUFFER_FREQUENCY: 880,
  DEFAULT_DURATION_MS: 2000,
  BEEP_DURATION_MS: 1000,
} as const;

/**
 * Standard logs to avoid magic strings throughout the examples
 */
export const AUDIO_LOGS = {
  START_TEST: (name: string) => `\n>>> Starting Test: ${name}`,
  DEVICE_INFO: (name: string, type: string) => `Using ${type} Device: ${name}`,
  NOT_FOUND: (type: string) => `Error: No ${type} device found.`,
  TEST_COMPLETE: (name: string) => `<<< Test Complete: ${name}`,
} as const;

/**
 * Utility to wait for a certain amount of time
 */
export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

/**
 * Reusable helper to run an audio stream for a specific duration
 */
export async function runStream(stream: AudioStream, durationMs: number) {
  try {
    stream.play();
    await sleep(durationMs);
    stream.pause();
  } catch (error) {
    console.error("Stream execution error:", error);
  }
}

/**
 * Get device information for summary logs
 */
export function logDeviceInfo(device: AudioDevice, type: 'Input' | 'Output') {
  console.log(AUDIO_LOGS.DEVICE_INFO(device.name(), type));
}

/**
 * Encodes Float32 audio samples into a 16-bit PCM WAV file
 */
export function writeWavFile(filename: string, samples: Float32Array, sampleRate: number, channels: number) {
  const bytesPerSample = 2; // 16-bit PCM
  const blockAlign = channels * bytesPerSample;
  const byteRate = sampleRate * blockAlign;
  const dataSize = samples.length * bytesPerSample;
  
  const buffer = Buffer.alloc(44 + dataSize);
  
  // RIFF Header
  buffer.write("RIFF", 0);
  buffer.writeUInt32LE(36 + dataSize, 4);
  buffer.write("WAVE", 8);
  
  // fmt Subchunk
  buffer.write("fmt ", 12);
  buffer.writeUInt32LE(16, 16); // Subchunk1Size
  buffer.writeUInt16LE(1, 20); // AudioFormat (1 = PCM)
  buffer.writeUInt16LE(channels, 22);
  buffer.writeUInt32LE(sampleRate, 24);
  buffer.writeUInt32LE(byteRate, 28);
  buffer.writeUInt16LE(blockAlign, 32);
  buffer.writeUInt16LE(16, 34); // BitsPerSample
  
  // data Subchunk
  buffer.write("data", 36);
  buffer.writeUInt32LE(dataSize, 40);
  
  // PCM Data
  for (let i = 0; i < samples.length; i++) {
    // Clamp to [-1, 1] and convert to Int16
    const s = Math.max(-1, Math.min(1, samples[i]));
    const val = s < 0 ? s * 0x8000 : s * 0x7FFF;
    buffer.writeInt16LE(Math.floor(val), 44 + i * bytesPerSample);
  }
  
  import("node:fs").then(fs => {
    fs.default.writeFileSync(filename, buffer);
  });
}
