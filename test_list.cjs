const { getDefaultHost } = require("./index.js");

const host = getDefaultHost();
console.log("Host:", host.name());
const inputs = host.inputDevices();
console.log("Inputs:", inputs.length);
inputs.forEach((d, i) => {
    try {
        console.log(`${i+1}: ${d.name()} [${d.isInput() ? 'In' : ''}${d.isOutput() ? 'Out' : ''}]`);
    } catch (e) {
        console.log(`${i+1}: <error getting name>`);
    }
});
