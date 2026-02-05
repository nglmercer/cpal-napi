import { AudioDevice, AudioBuffer, AudioStream, StreamConfig } from "../../index.js";

export interface CableOptions {
  bufferSize?: number;
  sampleRate?: number;
  channels?: number;
}

export class AudioCable {
  private inputDevice: AudioDevice;
  private outputDevice: AudioDevice;
  private buffer: AudioBuffer;
  private inputStream: AudioStream | null = null;
  private outputStream: AudioStream | null = null;
  private config: StreamConfig;

  constructor(input: AudioDevice, output: AudioDevice, options: CableOptions = {}) {
    this.inputDevice = input;
    this.outputDevice = output;
    this.buffer = new AudioBuffer();

    // Use default configs if not specified
    const defaultConfig = output.defaultOutputConfig();
    
    this.config = {
      channels: options.channels || defaultConfig.channels,
      sampleRate: options.sampleRate || defaultConfig.sampleRate,
      bufferSize: options.bufferSize ? { type: 'Fixed', field0: options.bufferSize } : { type: 'Default' },
      sampleFormat: defaultConfig.sampleFormat
    };
  }

  public start(): void {
    if (this.inputStream || this.outputStream) return;

    this.inputStream = this.inputDevice.createInputStream(this.config, this.buffer);
    this.outputStream = this.outputDevice.createOutputStream(this.config, this.buffer);

    this.inputStream.play();
    this.outputStream.play();
  }

  public stop(): void {
    this.inputStream?.pause();
    this.outputStream?.pause();
    this.inputStream?.close();
    this.outputStream?.close();
    this.inputStream = null;
    this.outputStream = null;
    this.buffer.clear();
  }

  public getBuffer(): AudioBuffer {
    return this.buffer;
  }

  public isRunning(): boolean {
    return !!(this.inputStream && this.outputStream);
  }
}
