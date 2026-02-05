import { expect, test, describe } from "bun:test";
import {
  getDefaultHost,
  AudioBuffer,
  hostFromId,
  HostId,
  I24,
  U24,
} from "../index";

const IS_CI = process.env.GITHUB_ACTIONS === "true";

describe("Audio Library Tests", () => {

  test("getDefaultHost should return a host with a name", () => {
    const host = getDefaultHost();
    expect(host).toBeDefined();
    expect(typeof host.name()).toBe("string");
  });

  test("AudioBuffer should push and report length", () => {
    const buffer = new AudioBuffer();
    expect(buffer.length()).toBe(0);
    
    const samples = new Float32Array([0.1, 0.2, 0.3]);
    buffer.push(samples);
    expect(buffer.length()).toBe(3);
    
    buffer.clear();
    expect(buffer.length()).toBe(0);
  });

  test("AudioBuffer.beep should generate samples", () => {
    const buffer = new AudioBuffer();
    const frequency = 440;
    const durationMs = 100; // short for testing
    const sampleRate = 44100;
    const expectedSamples = (sampleRate * durationMs) / 1000;

    buffer.beep(frequency, durationMs, sampleRate);
    expect(buffer.length()).toBe(expectedSamples);
    
    buffer.clear();
    expect(buffer.length()).toBe(0);
  });

  test("I24 and U24 types should work", () => {
    const i24 = new I24(0x12345678);
    expect(i24.toI32()).toBe(0x12345678 & 0xFFFFFF);

    const u24 = new U24(0x12345678);
    expect(u24.toU32()).toBe(0x12345678 & 0xFFFFFF);
  });

  test("Host should have devices", () => {
    const host = getDefaultHost();
    const devices = host.devices();
    expect(Array.isArray(devices)).toBe(true);
  });

  test("Default output device should have properties", () => {
    if (IS_CI) return;

    const host = getDefaultHost();
    const output = host.defaultOutputDevice();
    if (output) {
      expect(typeof output.name()).toBe("string");
      const id = output.id();
      expect(typeof id.id).toBe("string");
      
      const config = output.defaultOutputConfig();
      expect(config.channels).toBeGreaterThan(0);
      expect(config.sampleRate).toBeGreaterThan(0);
    }
  });

  test("OutputStream creation via AudioBuffer", () => {
    if (IS_CI) return;

    const host = getDefaultHost();
    const output = host.defaultOutputDevice();
    if (output) {
      try {
        const buffer = new AudioBuffer();
        const config = output.defaultOutputConfig();
        
        // Generate a tiny bit of audio
        buffer.beep(440, 50, config.sampleRate);
        
        const stream = output.createOutputStream(config, buffer);
        expect(stream).toBeDefined();
        stream.play();
        stream.pause();
      } catch (e) {
        console.warn("Could not create output stream:", e);
      }
    }
  });

  test("InputStream creation via AudioBuffer", () => {
    if (IS_CI) return;

    const host = getDefaultHost();
    const input = host.defaultInputDevice();
    if (input) {
      try {
        const buffer = new AudioBuffer();
        const config = input.defaultInputConfig();
        
        const stream = input.createInputStream(config, buffer);
        expect(stream).toBeDefined();
        stream.play();
        stream.pause();
      } catch (e) {
        console.warn("Could not create input stream:", e);
      }
    }
  });
});
