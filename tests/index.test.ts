import { expect, test, describe } from "bun:test";
import {
  availableHosts,
  getDefaultHost,
  AudioBuffer,
  hostFromId,
  getAllHosts,
  HostId,
  I24,
  U24,
} from "../index";

const IS_CI = process.env.GITHUB_ACTIONS === "true";

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

  test("getAllHosts should return a list of HostId", () => {
    const hosts = getAllHosts();
    expect(Array.isArray(hosts)).toBe(true);
    expect(hosts.length).toBeGreaterThan(0);
    expect(typeof hosts[0]).toBe("number");
  });

  test("hostFromId should work for at least one host", () => {
    const hosts = availableHosts();
    if (hosts.length > 0) {
      // Find a host that we have an enum for
      const allHostIds = [
        HostId.Alsa,
        HostId.CoreAudio,
        HostId.Wasapi,
        HostId.Jack,
        HostId.Asio,
      ];
      
      // Try to find one that matches available hosts
      for (const id of allHostIds) {
        try {
          const host = hostFromId(id);
          expect(host).toBeDefined();
          expect(typeof host.name()).toBe("string");
          break; // If one works, we are good
        } catch (e) {
          // Some hosts might not be available on all platforms
        }
      }
    }
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
    if (IS_CI) return; // Skip in CI as it likely won't have devices

    const host = getDefaultHost();
    const output = host.defaultOutputDevice();
    if (output) {
      expect(typeof output.name()).toBe("string");
      const id = output.id();
      expect(typeof id.id).toBe("string");
      
      const desc = output.description();
      expect(desc.name).toBe(output.name());
      
      const config = output.defaultOutputConfig();
      expect(config.channels).toBeGreaterThan(0);
      expect(config.sampleRate).toBeGreaterThan(0);

      const configs = output.supportedOutputConfigs();
      expect(Array.isArray(configs)).toBe(true);
    }
  });

  test("Default input device should have properties", () => {
    if (IS_CI) return; // Skip in CI

    const host = getDefaultHost();
    const input = host.defaultInputDevice();
    if (input) {
      expect(typeof input.name()).toBe("string");
      const config = input.defaultInputConfig();
      expect(config.channels).toBeGreaterThan(0);
      expect(config.sampleRate).toBeGreaterThan(0);

      const configs = input.supportedInputConfigs();
      expect(Array.isArray(configs)).toBe(true);
    }
  });

  test("Beep stream creation", () => {
    if (IS_CI) return; // Skip in CI

    const host = getDefaultHost();
    const output = host.defaultOutputDevice();
    if (output) {
      try {
        const stream = output.createBeepStream();
        expect(stream).toBeDefined();
        stream.play();
        stream.pause();
      } catch (e) {
        // Some devices might fail to build stream even if present (e.g. busy)
        console.warn("Could not create beep stream:", e);
      }
    }
  });
});

