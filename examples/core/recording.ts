import { AudioDevice, AudioBuffer, StreamConfig } from "cpal-napi";
import { AUDIO_SETTINGS, AUDIO_LOGS, runStream, logDeviceInfo } from "./common.js";

const RECORD_TEST = "Audio Recording";
const PLAYBACK_TEST = "Buffer Playback";

/**
 * Captures audio from an input device into a buffer
 */
export async function testInputStream(device: AudioDevice) {
  console.log(AUDIO_LOGS.START_TEST(RECORD_TEST));
  logDeviceInfo(device, 'Input');
  
  const config = device.defaultInputConfig();
    
  const buffer = new AudioBuffer();
  const stream = device.createInputStream(config, buffer);
  
  console.log(`- Recording into buffer for ${AUDIO_SETTINGS.DEFAULT_DURATION_MS}ms...`);
  await runStream(stream, AUDIO_SETTINGS.DEFAULT_DURATION_MS);
  
  console.log(`- Captured ${buffer.length()} samples.`);
  console.log(AUDIO_LOGS.TEST_COMPLETE(RECORD_TEST));
  
  return { buffer, config };
}

/**
 * Plays back content from an AudioBuffer
 * If no buffer is provided, it generates a custom tone
 */
export async function testCustomBufferStream(
  device: AudioDevice, 
  config: StreamConfig, 
  buffer?: AudioBuffer
) {
  console.log(AUDIO_LOGS.START_TEST(PLAYBACK_TEST));
  logDeviceInfo(device, 'Output');

  const playbackBuffer = buffer || new AudioBuffer();
  
  if (!buffer) {
    console.log(`- No buffer provided, generating ${AUDIO_SETTINGS.BUFFER_FREQUENCY}Hz reference tone`);
    playbackBuffer.beep(
      AUDIO_SETTINGS.BUFFER_FREQUENCY,
      AUDIO_SETTINGS.DEFAULT_DURATION_MS,
      config.sampleRate
    );
  } else {
    console.log("- Playing back recorded data");
  }

  const stream = device.createOutputStream(config, playbackBuffer);
  
  await runStream(stream, AUDIO_SETTINGS.DEFAULT_DURATION_MS);
  console.log(AUDIO_LOGS.TEST_COMPLETE(PLAYBACK_TEST));
}
