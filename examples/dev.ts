import {
  getDefaultHost,
  availableHosts,
} from "cpal-napi";
import { LOG_MESSAGES } from "./common.js";
import { testBeepStream } from "./beep.js";
import { testInputStream, testCustomBufferStream } from "./recording.js";

// --- Main execution ---
async function runDevExample() {
  try {
    console.log("Detecting audio hosts...");
    console.log("Available Hosts:", availableHosts());
    
    const host = getDefaultHost();
    console.log("Current Host:", host.name());

    const output = host.defaultOutputDevice();
    const input = host.defaultInputDevice();

    if (output) {
      console.log("Using Output Device:", output.name());
      const config = output.defaultOutputConfig();
      console.log("Default Output Configuration:", config);

      // Execute output tests
      await testBeepStream(output);
      await testCustomBufferStream(output, config);
    } else {
      console.warn(LOG_MESSAGES.NO_DEVICE);
    }

    if (input && output) {
      const { buffer } = await testInputStream(input);
      const outConfig = output.defaultOutputConfig();
      await testCustomBufferStream(output, outConfig, buffer);
    } else if (!input) {
      console.warn(LOG_MESSAGES.NO_INPUT_DEVICE);
    }
    
    console.log(LOG_MESSAGES.FINISHED);
  } catch (err) {
    console.error("Critical error during audio test:", err);
  }
}

runDevExample().catch(console.error);
