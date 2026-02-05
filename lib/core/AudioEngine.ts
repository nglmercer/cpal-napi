import { getDefaultHost, AudioHost, HostId, hostFromId, AudioDevice } from "../../index.js";

export class AudioEngine {
  private static instance: AudioEngine;
  private currentHost: AudioHost;

  private constructor() {
    this.currentHost = getDefaultHost();
  }

  public static getInstance(): AudioEngine {
    if (!AudioEngine.instance) {
      AudioEngine.instance = new AudioEngine();
    }
    return AudioEngine.instance;
  }

  public getHost(): AudioHost {
    return this.currentHost;
  }

  public setHost(id: HostId): void {
    this.currentHost = hostFromId(id);
  }

  public getDefaultInputDevice(): AudioDevice | null {
    return this.currentHost.defaultInputDevice();
  }

  public getDefaultOutputDevice(): AudioDevice | null {
    return this.currentHost.defaultOutputDevice();
  }

  public listDevices(): AudioDevice[] {
    return this.currentHost.devices().filter(d => d.description().available);
  }
}
