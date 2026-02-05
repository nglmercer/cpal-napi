import { availableHosts, getDefaultHost } from "cpal-napi";

export function getlistAll() {
  console.log("=== Audio Diagnostics ===");
  
  const hosts = availableHosts();
  console.log(`\nAvailable Hosts`,hosts);
  const defaultHost = getDefaultHost();
  console.log(`\nDefault Host`,defaultHost.name());

  const outputDevices = defaultHost.devices();
  console.log(`\nDevices for ${defaultHost.name()} (${outputDevices.length}):`);
  
  console.log(outputDevices);
  return {outputDevices,hosts,defaultHost};
}

getlistAll();
