# cpal-napi

N-API (Node-API) wrapper for [cpal](https://github.com/RustAudio/cpal), a Cross-Platform Audio Library in pure Rust.

## Features

- **Enumerate Hosts**: List available audio hosts (ALSA, WASAPI, CoreAudio, etc.)
- **Device Management**: List input and output devices and get defaults.
- **Audio Output**:
  - Easy beep stream for testing.
  - High-performance audio output via `AudioBuffer` (ring buffer) pushed from JavaScript/TypeScript.
- **TypeScript Support**: Full type definitions automatically generated.
- **Cross-platform**: Support for Linux, Windows, and macOS.

## Installation

```bash
bun install cpal-napi
```

## Usage

```typescript
import { getDefaultHost, AudioBuffer } from "cpal-napi";

const host = getDefaultHost();
const device = host.defaultOutputDevice();

if (device) {
  const config = device.defaultOutputConfig();
  const buffer = new AudioBuffer();

  // Fill buffer with samples (-1.0 to 1.0)
  const samples = new Float32Array(44100);
  for (let i = 0; i < samples.length; i++) {
    samples[i] = Math.sin((2 * Math.PI * 440 * i) / 44100);
  }
  buffer.push(samples);

  const stream = device.createOutputStream(config, buffer);
  stream.play();

  // Audio plays in background
  setTimeout(() => stream.pause(), 1000);
}
```

## Development

### Building

```bash
# Debug build
bun run build:debug

# Release build
bun run build
```

### Running Examples

```bash
bun run examples/dev.ts
```

## API

### `getDefaultHost(): AudioHost`

Returns the default audio host for the current system.

### `availableHosts(): string[]`

Returns a list of available audio host names.

### `AudioHost`

- `name(): string`
- `devices(): AudioDevice[]`
- `defaultOutputDevice(): AudioDevice | null`
- `defaultInputDevice(): AudioDevice | null`

### `AudioDevice`

- `name(): string`
- `defaultOutputConfig(): AudioStreamConfig`
- `defaultInputConfig(): AudioStreamConfig`
- `supportedOutputConfigs(): SupportedAudioStreamConfig[]`
- `createBeepStream(): AudioStream`
- `createOutputStream(config: AudioStreamConfig, buffer: AudioBuffer): AudioStream`

### `AudioBuffer`

- `new AudioBuffer()`
- `push(data: Float32Array): void`
- `clear(): void`
- `length(): number`

### `AudioStream`

- `play(): void`
- `pause(): void`

## License

MIT
