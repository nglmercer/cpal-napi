import { AudioDevice, AudioBuffer, AudioStream, StreamConfig, SupportedStreamConfig } from "../../index.js";

export interface CableOptions {
  bufferSize?: number;
  sampleRate?: number;
  channels?: number; // Desired output channel count (optional)
}

export class AudioCable {
  private inputDevice: AudioDevice;
  private outputDevice: AudioDevice;
  private buffer: AudioBuffer;
  private inputStream: AudioStream | null = null;
  private outputStream: AudioStream | null = null;
  private inputConfig: StreamConfig;
  private outputConfig: StreamConfig;

  constructor(input: AudioDevice, output: AudioDevice, options: CableOptions = {}) {
    this.inputDevice = input;
    this.outputDevice = output;
    this.buffer = new AudioBuffer();

    const configs = this.findCompatibleConfigs(input, output, options);
    this.inputConfig = configs.input;
    this.outputConfig = configs.output;
  }

  private findCompatibleConfigs(input: AudioDevice, output: AudioDevice, options: CableOptions): { input: StreamConfig; output: StreamConfig } {
    // Helper to check if a sample rate is within a supported config's range
    const rateInRange = (rate: number, cfg: SupportedStreamConfig) => {
      return rate >= cfg.minSampleRate && rate <= cfg.maxSampleRate;
    };

    // Get supported configs from both devices
    let inputConfigs: SupportedStreamConfig[];
    let outputConfigs: SupportedStreamConfig[];
    try {
      inputConfigs = input.supportedInputConfigs();
    } catch (e) {
      throw new Error(`Failed to get supported input configs: ${e}`);
    }
    try {
      outputConfigs = output.supportedOutputConfigs();
    } catch (e) {
      throw new Error(`Failed to get supported output configs: ${e}`);
    }

    if (inputConfigs.length === 0) {
      throw new Error(`Input device '${input.name()}' reports no supported input configurations.`);
    }
    if (outputConfigs.length === 0) {
      throw new Error(`Output device '${output.name()}' reports no supported output configurations.`);
    }

    // Determine desired sample rate and sample format
    // First, try to use input's default config as a base
    let baseInputConfig: StreamConfig;
    try {
      baseInputConfig = input.defaultInputConfig();
    } catch (e) {
      // Fallback to first input config
      const firstIn = inputConfigs[0];
      baseInputConfig = {
        channels: firstIn.channels,
        sampleRate: firstIn.minSampleRate, // use min as a fallback
        bufferSize: { type: 'Default' },
        sampleFormat: firstIn.sampleFormat
      };
    }

    // If options specify a sampleRate, we'll try to use that if supported
    // We'll search for a matching output config that shares the same sampleRate (or can use that rate) and sampleFormat
    // We'll allow output channel count to differ.

    // Collect all possible (inputConfig, outputConfig) pairs that have matching sampleRate and sampleFormat.
    // We'll pick the highest sampleRate possible.
    let bestPair: { inCfg: SupportedStreamConfig; outCfg: SupportedStreamConfig; sampleRate: number } | null = null;

    // If options.sampleRate is provided, try to find a pair that exactly matches that rate.
    if (options.sampleRate) {
      for (const inCfg of inputConfigs) {
        for (const outCfg of outputConfigs) {
          if (inCfg.sampleFormat === outCfg.sampleFormat &&
              options.sampleRate >= inCfg.minSampleRate && options.sampleRate <= inCfg.maxSampleRate &&
              options.sampleRate >= outCfg.minSampleRate && options.sampleRate <= outCfg.maxSampleRate) {
            // Found a pair supporting the requested sample rate
            bestPair = { inCfg, outCfg, sampleRate: options.sampleRate };
            break;
          }
        }
        if (bestPair) break;
      }
      if (bestPair) {
        // Use the input's channel count from the selected inCfg, and output's channel count from outCfg.
        return {
          input: {
            channels: bestPair.inCfg.channels,
            sampleRate: bestPair.sampleRate,
            bufferSize: options.bufferSize ? { type: 'Fixed', field0: options.bufferSize } : { type: 'Default' },
            sampleFormat: bestPair.inCfg.sampleFormat
          },
          output: {
            channels: bestPair.outCfg.channels,
            sampleRate: bestPair.sampleRate,
            bufferSize: options.bufferSize ? { type: 'Fixed', field0: options.bufferSize } : { type: 'Default' },
            sampleFormat: bestPair.outCfg.sampleFormat
          }
        };
      }
      // If not found, we'll continue to search for best available
    }

    // Search for any compatible pair with overlapping sample rate ranges
    for (const inCfg of inputConfigs) {
      for (const outCfg of outputConfigs) {
        if (inCfg.sampleFormat === outCfg.sampleFormat) {
          const overlapMin = Math.max(inCfg.minSampleRate, outCfg.minSampleRate);
          const overlapMax = Math.min(inCfg.maxSampleRate, outCfg.maxSampleRate);
          if (overlapMin <= overlapMax) {
            // Choose a sample rate within the overlap: prefer highest possible
            const chosenRate = options.sampleRate && options.sampleRate >= overlapMin && options.sampleRate <= overlapMax
              ? options.sampleRate
              : overlapMax; // highest common
            if (!bestPair || chosenRate > bestPair.sampleRate) {
              bestPair = { inCfg, outCfg, sampleRate: chosenRate };
            }
          }
        }
      }
    }

    if (!bestPair) {
      throw new Error(`No common sample rate and sample format found between input '${input.name()}' and output '${output.name()}'.`);
    }

    // If options.channels is provided, interpret it as the desired output channel count.
    // We'll try to select an output config with that channel count if possible, otherwise use the one from bestPair.
    let outputChannels = bestPair.outCfg.channels;
    if (options.channels !== undefined) {
      // Check if there exists an output config with the requested channels and same sampleRate and sampleFormat
      const outCfgWithChannels = outputConfigs.find(oc => 
        oc.channels === options.channels &&
        oc.sampleFormat === bestPair.inCfg.sampleFormat &&
        bestPair.sampleRate >= oc.minSampleRate && bestPair.sampleRate <= oc.maxSampleRate
      );
      if (outCfgWithChannels) {
        outputChannels = options.channels;
      } else {
        // Requested channel count not supported; we'll ignore and use the bestPair's output channels.
        console.warn(`Requested output channel count ${options.channels} not supported; using ${bestPair.outCfg.channels} channels instead.`);
      }
    }

    return {
      input: {
        channels: bestPair.inCfg.channels,
        sampleRate: bestPair.sampleRate,
        bufferSize: options.bufferSize ? { type: 'Fixed', field0: options.bufferSize } : { type: 'Default' },
        sampleFormat: bestPair.inCfg.sampleFormat
      },
      output: {
        channels: outputChannels,
        sampleRate: bestPair.sampleRate,
        bufferSize: options.bufferSize ? { type: 'Fixed', field0: options.bufferSize } : { type: 'Default' },
        sampleFormat: bestPair.outCfg.sampleFormat
      }
    };
  }

  public start(): void {
    if (this.inputStream || this.outputStream) return;

    this.inputStream = this.inputDevice.createInputStream(this.inputConfig, this.buffer);
    this.outputStream = this.outputDevice.createOutputStream(this.outputConfig, this.buffer);

    this.inputStream.play();
    this.outputStream.play();
  }

  public stop(): void {
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
