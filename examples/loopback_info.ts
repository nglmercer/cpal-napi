import { getDefaultHost } from "../index.js";

async function main() {
    const host = getDefaultHost();
    
    console.log("--- Combined Devices ---");
    for (const d of host.devices()) {
        const desc = d.description();
        console.log(`[Combined] ${desc.name} | Input: ${desc.maxInputChannels} | Output: ${desc.maxOutputChannels} | LP: ${desc.isLoopback}`);
    }

    console.log("\n--- Input Devices ---");
    for (const d of host.inputDevices()) {
        const desc = d.description();
        console.log(`[Input] ${desc.name} | Input: ${desc.maxInputChannels} | Output: ${desc.maxOutputChannels} | LP: ${desc.isLoopback}`);
    }

    console.log("\n--- Output Devices ---");
    for (const d of host.outputDevices()) {
        const desc = d.description();
        console.log(`[Output] ${desc.name} | Input: ${desc.maxInputChannels} | Output: ${desc.maxOutputChannels} | LP: ${desc.isLoopback}`);
    }
}

main().catch(console.error);
