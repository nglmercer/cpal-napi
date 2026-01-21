import { AudioDevice } from "cpal-napi";
import { AUDIO_CONFIG, LOG_MESSAGES, delay } from "./common.js";

/**
 * Test the built-in beep stream functionality.
 */
export async function testBeepStream(device: AudioDevice) {
  console.log(LOG_MESSAGES.START_BEEP);
  const stream = device.createBeepStream();
  
  console.log(`Playing ${AUDIO_CONFIG.BEEP_FREQ}Hz beep for ${AUDIO_CONFIG.BEEP_DURATION_MS}ms...`);
  stream.play();
  await delay(AUDIO_CONFIG.BEEP_DURATION_MS);
  stream.pause();
}
