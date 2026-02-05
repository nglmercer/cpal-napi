import { AudioDevice, AudioBuffer, AudioStream, StreamConfig } from "../../index.js";

export class AudioSplitter {
  private inputDevice: AudioDevice;
  private outputDevices: AudioDevice[] = [];
  private buffers: AudioBuffer[] = [];
  private inputStream: AudioStream | null = null;
  private outputStreams: AudioStream[] = [];
  private config: StreamConfig;

  constructor(input: AudioDevice, outputList: AudioDevice[], config?: Partial<StreamConfig>) {
    this.inputDevice = input;
    this.outputDevices = outputList;
    
    const defaultConfig = input.isInput() ? input.defaultInputConfig() : outputList[0].defaultOutputConfig();
    
    this.config = {
      channels: config?.channels || defaultConfig.channels,
      sampleRate: config?.sampleRate || defaultConfig.sampleRate,
      bufferSize: config?.bufferSize ? (typeof config.bufferSize === 'number' ? { type: 'Fixed', field0: config.bufferSize } : config.bufferSize) : { type: 'Default' },
      sampleFormat: config?.sampleFormat || defaultConfig.sampleFormat
    };

    // Create a buffer for each output
    for (let i = 0; i < outputList.length; i++) {
      this.buffers.push(new AudioBuffer());
    }
  }

  public start(): void {
    if (this.inputStream) return;

    // The first output uses the main buffer
    const mainBuffer = new (require("../../index.js").AudioBuffer)();
    this.inputStream = this.inputDevice.createInputStream(this.config, mainBuffer);

    // Each subsequent output gets a linked buffer
    for (let i = 0; i < this.outputDevices.length; i++) {
        const out = this.outputDevices[i];
        const buf = i === 0 ? mainBuffer : mainBuffer.cloneLink();
        const stream = out.createOutputStream(this.config, buf);
        this.outputStreams.push(stream);
        stream.play();
    }

    this.inputStream.play();
  }

  public stop(): void {
    this.inputStream?.pause();
    this.inputStream?.close();
    this.outputStreams.forEach(s => {
        s.pause();
        s.close();
    });
    this.inputStream = null;
    this.outputStreams = [];
  }
}
