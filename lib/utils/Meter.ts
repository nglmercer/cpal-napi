import { AudioBuffer } from "../../index.js";

export class VolumeMeter {
  public static getLevel(buffer: AudioBuffer): number {
    const data = buffer.getData();
    if (data.length === 0) return 0;
    
    let sum = 0;
    for (let i = 0; i < data.length; i++) {
      sum += data[i] * data[i];
    }
    
    // RMS level
    const rms = Math.sqrt(sum / data.length);
    return rms;
  }

  public static getPeak(buffer: AudioBuffer): number {
    const data = buffer.getData();
    if (data.length === 0) return 0;
    
    let peak = 0;
    for (let i = 0; i < data.length; i++) {
        const abs = Math.abs(data[i]);
        if (abs > peak) peak = abs;
    }
    return peak;
  }
}
