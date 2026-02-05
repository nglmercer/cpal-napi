import { AudioDevice, AudioBuffer } from "cpal-napi";
import { AUDIO_CONFIG, LOG_MESSAGES, delay } from "./common.js";

/**
 * Test the beep generation via AudioBuffer.
 */
export async function testBeepStream(device: AudioDevice) {
  console.log(LOG_MESSAGES.START_BEEP);
  
  const buffer = new AudioBuffer();
  const config = device.defaultOutputConfig();
  
  // Generate beep into buffer
  buffer.beep(AUDIO_CONFIG.BEEP_FREQ, AUDIO_CONFIG.BEEP_DURATION_MS, config.sampleRate);
  
  const stream = device.createOutputStream(config, buffer);
  
  console.log(`Playing ${AUDIO_CONFIG.BEEP_FREQ}Hz beep for ${AUDIO_CONFIG.BEEP_DURATION_MS}ms...`);
  stream.play();
  await delay(AUDIO_CONFIG.BEEP_DURATION_MS);
  stream.pause();
}
