import { getDefaultHost, AudioBuffer } from "../index.js";

async function main() {
    const host = getDefaultHost();
    console.log(`Using Host: ${host.name()}`);

    const allDevices = host.devices();
    
    // VB-Audio Cable Strategy: Play to 'Input', Record from 'Output'
    let cableInput = allDevices.find(d => d.name().includes("CABLE Input"));
    let cableOutput = allDevices.find(d => d.name().includes("CABLE Output"));

    if (cableInput && cableOutput) {
        console.log("VB-Audio Cable detected. Using specialized test...");
        await runTest(cableInput, cableOutput);
    } else {
        // Standard Loopback Strategy: Find a device and its (Loopback) variant
        console.log("No VB-Cable found. Searching for WASAPI Loopback pair...");
        let render = allDevices.find(d => d.isOutput() && !d.name().includes("Loopback"));
        let loopback = allDevices.find(d => d.isInput() && d.name().includes(render?.description().name || "---") && d.name().includes("Loopback"));

        if (render && loopback) {
            await runTest(render, loopback);
        } else {
            console.error("Could not find any suitable Output/Input or Loopback pair.");
        }
    }
}

async function runTest(outputDevice: any, inputDevice: any) {
    console.log(`\n--- Test Config ---`);
    console.log(`Output: ${outputDevice.name()}`);
    console.log(`Input:  ${inputDevice.name()}`);

    const outConfig = outputDevice.defaultOutputConfig();
    const inConfig = inputDevice.defaultInputConfig();

    const outBuffer = new AudioBuffer();
    const inBuffer = new AudioBuffer();

    // Generate 440Hz Tone
    const tone = new Float32Array(outConfig.sampleRate * 2);
    for (let i = 0; i < tone.length; i++) {
        tone[i] = Math.sin(2 * Math.PI * 440 * (i / outConfig.sampleRate)) * 0.5;
    }
    outBuffer.push(tone);

    const outStream = outputDevice.createOutputStream(outConfig, outBuffer);
    const inStream = inputDevice.createInputStream(inConfig, inBuffer);

    console.log("\nStreams created. Playing tone & Monitoring levels...");
    outStream.play();
    inStream.play();

    for (let i = 0; i < 40; i++) {
        await new Promise(r => setTimeout(r, 100));
        let peak = inBuffer.getPeakLevel();
        const bar = "█".repeat(Math.floor(peak * 50)).padEnd(50, "░");
        process.stdout.write(`\rCapture Level: [${bar}] ${(peak * 100).toFixed(1)}%   `);
        if (i % 5 === 0) inBuffer.clear();
    }

    console.log("\n\nCleaning up...");
    outStream.close();
    inStream.close();
    console.log("Done.");
}

main().catch(console.error);
