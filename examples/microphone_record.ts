import { getDefaultHost, AudioBuffer } from "cpal-napi";
import { AUDIO_SETTINGS, AUDIO_LOGS, runStream, logDeviceInfo, writeWavFile } from "./core/common.js";
import { createInterface } from "readline/promises";
import fs from "node:fs";

/**
 * Microphone Selection and Recording Example
 * Demonstrates:
 * 1. Listing specific input devices
 * 2. Manual device selection logic
 * 3. Recording from a specific device
 * 4. Automatic playback of the recording
 * 5. Saving the recorded data to a file for verification
 */

async function main() {
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    console.log("=== Microphone Selection & Recording Example ===");
    
    const host = getDefaultHost();
    const inputDevices = host.inputDevices();
    
    console.log("\nAvailable Input Devices:");
    inputDevices.forEach((device, i) => {
      try {
        const name = device.name();
        const isInput = device.isInput();
        const isOutput = device.isOutput();
        const typeStr = [isInput ? "Input" : "", isOutput ? "Output" : ""].filter(Boolean).join("/");
        console.log(`${i + 1}. ${name} [${typeStr}]`);
      } catch {
        console.log(`${i + 1}. [Unnamed Device]`);
      }
    });

    const answer = await rl.question("\nSelect microphone number (or press Enter for default): ");
    let index = parseInt(answer) - 1;
    
    let selectedDevice: any;
    if (isNaN(index)) {
        selectedDevice = host.defaultInputDevice();
    } else {
        selectedDevice = inputDevices[index];
    }
    
    if (!selectedDevice) {
      throw new Error(AUDIO_LOGS.NOT_FOUND('Input'));
    }

    logDeviceInfo(selectedDevice, 'Input');

    const buffer = new AudioBuffer();
    const config = selectedDevice.defaultInputConfig();
    
    console.log(`- Sample Rate: ${config.sampleRate}Hz`);
    console.log(`- Channels: ${config.channels}`);
    
    const inputStream = selectedDevice.createInputStream(config, buffer);
    
    console.log(`\nRecording for ${AUDIO_SETTINGS.DEFAULT_DURATION_MS}ms...`);
    await runStream(inputStream, AUDIO_SETTINGS.DEFAULT_DURATION_MS);
    console.log(`- Captured ${buffer.length()} samples.`);

    // --- SAVE DATA TO FILE ---
    const data = buffer.getData();
    if (data.length > 0) {
      const filename = "recording.wav";
      console.log(`\n- Saving ${data.length} samples to '${filename}'...`);
      
      writeWavFile(filename, data, config.sampleRate, config.channels);
      
      // Save metadata for reference
      fs.writeFileSync("recording.json", JSON.stringify({
        filename,
        sampleRate: config.sampleRate,
        channels: config.channels,
        sampleCount: data.length,
        durationMs: AUDIO_SETTINGS.DEFAULT_DURATION_MS
      }, null, 2));
      console.log("- Audio saved as standard WAV.");
    }

    // Playback
    const outputDevice = host.defaultOutputDevice();
    if (outputDevice) {
      console.log("\nPlaying back recording...");
      logDeviceInfo(outputDevice, 'Output');
      
      const outConfig = outputDevice.defaultOutputConfig();
      const outputStream = outputDevice.createOutputStream(outConfig, buffer);
      
      await runStream(outputStream, AUDIO_SETTINGS.DEFAULT_DURATION_MS);
    } else {
      console.warn(AUDIO_LOGS.NOT_FOUND('Output'));
    }

    console.log("\nExample finished successfully.");

  } catch (err) {
    console.error("\n[Error]", err instanceof Error ? err.message : err);
  } finally {
    rl.close();
  }
}

main().catch(console.error);
