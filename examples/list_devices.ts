import { getAvailableHostNames, getDefaultHost } from "cpal-napi";

/**
 * Diagnostic utility to list all available audio hosts and devices
 */
export function listAllDevices() {
  console.log("=== CPAL Audio Diagnostics ===");
  
  const hosts = getAvailableHostNames();
  console.log(`\n[Available Audio Hosts] (${hosts.length})`);
  hosts.forEach((host, i) => console.log(`  ${i + 1}. ${host}`));

  const defaultHost = getDefaultHost();
  console.log(`\n[Default Host]: ${defaultHost.name()}`);

  const devices = defaultHost.devices();
  console.log(`\n[Devices for ${defaultHost.name()}] (${devices.length}):`);
  
  devices.forEach((device, index) => {
    try {
      const name = device.name();
      console.log(`  ${index + 1}. ${name}`, device.description());
      // Only log brief info to avoid excessive output
    } catch (e) {
      console.log(`  ${index + 1}. [Error getting device name]`);
    }
  });

  return { devices, hosts, defaultHost };
}

// Execute if run directly
if (import.meta.url.endsWith(process.argv[1]?.replace(/\\/g, '/'))) {
  listAllDevices();
} else if (process.argv[1]?.includes('list_devices')) {
    // Fallback for some execution environments
    listAllDevices();
}
