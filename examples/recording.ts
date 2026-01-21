import { AudioDevice, AudioBuffer, StreamConfig } from "cpal-napi";
import { AUDIO_CONFIG, LOG_MESSAGES, delay } from "./common.js";

/**
 * Test recording audio from default input device.
 */
export async function testInputStream(device: AudioDevice) {
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
  
  return { buffer, config };
}

/**
 * Test custom audio buffer playback.
 */
export async function testCustomBufferStream(device: AudioDevice, config: StreamConfig, buffer?: AudioBuffer) {
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
