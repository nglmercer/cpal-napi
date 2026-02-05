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
