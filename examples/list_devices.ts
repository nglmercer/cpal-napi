import { availableHosts, getDefaultHost } from "cpal-napi";

function listAll() {
  console.log("=== Audio Diagnostics ===");
  
  const hosts = availableHosts();
  console.log(`\nAvailable Hosts (${hosts.length}):`);
  hosts.forEach(name => console.log(` - ${name}`));

  const defaultHost = getDefaultHost();
  console.log(`\nDefault Host: ${defaultHost.name()}`);

  const outputDevices = defaultHost.devices();
  console.log(`\nDevices for ${defaultHost.name()} (${outputDevices.length}):`);
  
  outputDevices.forEach((device, index) => {
    try {
      const name = device.name();
      const id = device.id().id;
      const desc = device.description();
      
      console.log(`\n[${index}] ${name}`);
      console.log(`    ID: ${id}`);
      console.log(`    Type: ${desc.deviceType}`);
      console.log(`    Interface: ${desc.interfaceType}`);
      
      try {
        const outConfig = device.defaultOutputConfig();
        console.log(`    Default Output: ${outConfig.channels} channels, ${outConfig.sampleRate}Hz`);
      } catch (e) {
        console.log("    Default Output: Not available");
      }

      try {
        const inConfig = device.defaultInputConfig();
        console.log(`    Default Input: ${inConfig.channels} channels, ${inConfig.sampleRate}Hz`);
      } catch (e) {
        console.log("    Default Input: Not available");
      }
    } catch (err) {
      console.log(`\n[${index}] Error accessing device: ${err}`);
    }
  });
}

listAll();
