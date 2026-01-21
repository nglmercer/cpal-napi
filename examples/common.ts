export const AUDIO_CONFIG = {
  BEEP_FREQ: 440,
  BUFFER_FREQ: 880,
  BEEP_DURATION_MS: 1000,
  BUFFER_DURATION_MS: 2000,
};

export const LOG_MESSAGES = {
  START_BEEP: "\n--- Test 1: Beep Stream ---",
  START_BUFFER: "\n--- Test 2: AudioBuffer Stream ---",
  START_INPUT: "\n--- Test 3: Input (Recording) Stream ---",
  FINISHED: "Finished audio tests.",
  NO_DEVICE: "No default output device found.",
  NO_INPUT_DEVICE: "No default input device found.",
};

export const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));
