import { AudioDevice } from "../../index.js";
import { AudioEngine } from "../core/AudioEngine.js";

export class DeviceScanner {
  public static findByName(pattern: string | RegExp): AudioDevice[] {
    const engine = AudioEngine.getInstance();
    const devices = engine.listDevices();
    
    return devices.filter(d => {
      const name = d.name();
      if (typeof pattern === 'string') {
        return name.toLowerCase().includes(pattern.toLowerCase());
      }
      return pattern.test(name);
    });
  }

  public static getInputs(): AudioDevice[] {
    const engine = AudioEngine.getInstance();
    return engine.listDevices().filter(d => d.isInput());
  }

  public static getOutputs(): AudioDevice[] {
    const engine = AudioEngine.getInstance();
    return engine.listDevices().filter(d => d.isOutput());
  }
}
