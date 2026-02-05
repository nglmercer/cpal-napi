import { AudioDevice, AudioBuffer } from "cpal-napi";
import { AUDIO_SETTINGS, AUDIO_LOGS, runStream, logDeviceInfo } from "./common.js";

const TEST_NAME = "Beep Generation";

/**
 * Demonstrates how to generate a simple sine wave using AudioBuffer.beep
 */
export async function testBeepStream(device: AudioDevice) {
  console.log(AUDIO_LOGS.START_TEST(TEST_NAME));
  logDeviceInfo(device, 'Output');

  const buffer = new AudioBuffer();
  const config = device.defaultOutputConfig();

  // Populate the buffer with a sine wave tone
  // frequency, durationMs, sampleRate
  buffer.beep(
    AUDIO_SETTINGS.BEEP_FREQUENCY, 
    AUDIO_SETTINGS.BEEP_DURATION_MS, 
    config.sampleRate
  );

  console.log(`- Buffer prepared with ${AUDIO_SETTINGS.BEEP_FREQUENCY}Hz tone`);
  
  const stream = device.createOutputStream(config, buffer);
  
  console.log(`- Playing for ${AUDIO_SETTINGS.BEEP_DURATION_MS}ms...`);
  await runStream(stream, AUDIO_SETTINGS.BEEP_DURATION_MS);

  console.log(AUDIO_LOGS.TEST_COMPLETE(TEST_NAME));
}
