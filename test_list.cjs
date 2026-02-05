const { getDefaultHost, HostId, DeviceDirection } = require("./index.js");

const host = getDefaultHost();
console.log(`\n\x1b[1m\x1b[34mHost:\x1b[0m ${host.name()}`);

const allDevices = host.devices();
console.log(`\x1b[1m\x1b[32mDevices Found:\x1b[0m ${allDevices.length}\n`);

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

let deviceCount = 0;
allDevices.forEach((d) => {
    try {
        const desc = d.description();
        if (!desc.available) {
            return;
        }
        deviceCount++;

        const hostStr = getHostName(desc.hostId);
        const direction = `[${d.isInput() ? 'IN' : ''}${d.isOutput() ? 'OUT' : ''}]`;
        const channels = `${desc.maxInputChannels}in/${desc.maxOutputChannels}out`;
        
        console.log(`${(deviceCount).toString().padStart(2)}: \x1b[33m${desc.name}\x1b[0m`);
        console.log(`    \x1b[90mHost:\x1b[0m ${hostStr} | \x1b[90mCaps:\x1b[0m ${direction} | \x1b[90mCh:\x1b[0m ${channels}`);
    } catch (e) {
        console.log(`??: <error getting device details: ${e.message}>`);
    }
});

console.log("\n\x1b[1m\x1b[36mDone listing devices.\x1b[0m\n");
