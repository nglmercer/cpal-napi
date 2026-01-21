import { expect, test, describe } from "bun:test";
import {
  availableHosts,
  getDefaultHost,
  AudioBuffer,
} from "../index.js";


describe("Audio Library Tests", () => {
  test("availableHosts should return an array", () => {
    const hosts = availableHosts();
    expect(Array.isArray(hosts)).toBe(true);
    expect(hosts.length).toBeGreaterThan(0);
  });

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

  test("Host should have devices", () => {
    const host = getDefaultHost();
    const devices = host.devices();
    expect(Array.isArray(devices)).toBe(true);
  });

  test("Default output device should have a name and config", () => {
    const host = getDefaultHost();
    const output = host.defaultOutputDevice();
    if (output) {
      expect(typeof output.name()).toBe("string");
      const config = output.defaultOutputConfig();
      expect(config.channels).toBeGreaterThan(0);
      expect(config.sampleRate).toBeGreaterThan(0);
    }
  });
});
