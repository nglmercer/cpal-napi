const { getDefaultHost, HostId, DeviceType, DeviceDirection } = require("./index.js");

const host = getDefaultHost();
console.log(`\n\x1b[1m\x1b[34mHost:\x1b[0m ${host.name()}`);

const inputs = host.inputDevices();
console.log(`\x1b[1m\x1b[32mInput Devices:\x1b[0m ${inputs.length}\n`);

const getDeviceTypeName = (type) => {
    switch (type) {
        case DeviceType.Internal: return "Internal";
        case DeviceType.Usb: return "USB";
        case DeviceType.Bluetooth: return "Bluetooth";
        case DeviceType.Network: return "Network";
        case DeviceType.Firewire: return "Firewire";
        case DeviceType.Virtual: return "Virtual";
        default: return "Other";
    }
};

const getHostName = (id) => {
    switch(id) {
        case HostId.Alsa: return "ALSA";
        case HostId.Jack: return "JACK";
        case HostId.Wasapi: return "WASAPI";
        case HostId.Asio: return "ASIO";
        case HostId.CoreAudio: return "CoreAudio";
        default: return "Other";
    }
};

inputs.forEach((d, i) => {
    try {
        const desc = d.description();
        const typeStr = getDeviceTypeName(desc.deviceType);
        const hostStr = getHostName(desc.hostId);
        const direction = `[${d.isInput() ? 'IN' : ''}${d.isOutput() ? 'OUT' : ''}]`;
        const channels = `${desc.maxInputChannels}in/${desc.maxOutputChannels}out`;
        
        console.log(`${(i+1).toString().padStart(2)}: \x1b[33m${desc.name}\x1b[0m`);
        console.log(`    \x1b[90mType:\x1b[0m ${typeStr} | \x1b[90mHost:\x1b[0m ${hostStr} | \x1b[90mCaps:\x1b[0m ${direction} | \x1b[90mCh:\x1b[0m ${channels}`);
    } catch (e) {
        console.log(`${i+1}: <error getting device details: ${e.message}>`);
    }
});

console.log("\n\x1b[1m\x1b[36mDone listing devices.\x1b[0m\n");
