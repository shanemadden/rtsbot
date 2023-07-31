"use strict";
// replace this with the name of your module
const MODULE_NAME = "screeps_starter_rust_bg";
// TextEncoder/Decoder polyfill for UTF-8 conversion
import 'fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js';
import { initSync } from '../pkg/screeps_starter_rust.js';
let wasm_instance;

// track whether the logging setup has been done (we only want to run it once per wasm instance)
let log_setup_done = false;
// as well as whether the wasm instance has panicked, which triggers us halting the instance on
// the following tick
let halt_next_tick = false;

module.exports.loop = function () {
    if (halt_next_tick === true) {
        // we've had an error on the last tick; skip execution during the current tick, asking the
        // IVM to dispose of our instance to give us a fresh environment next tick
        // (see comment in error catch below)
        Game.cpu.halt();
    } else {
        // Replace the Memory object (which gets populated into our global each tick) with an empty
        // object, so that accesses to it from within the driver that we can't prevent (such as
        // when a creep is spawned) won't trigger an attempt to parse RawMemory. Replace the object
        // with one unattached to memory magic - game functions will access the `Memory` object and
        // can throw data in here, and it'll go away at the end of tick.

        // Because it's in place, RawMemory's string won't be thrown to JSON.parse to deserialize -
        // and because that didn't happen, RawMemory._parsed isn't set and won't trigger a
        // post-tick serialize.
        delete global.Memory;
        global.Memory = {};

        try {
            if (wasm_instance) {
                // deal with the case of the wasm init having completed but not having gotten
                // as far as running the logging setup due to running out of CPU mid-setup prior
                if (log_setup_done === false) {
                    wasm_instance.log_setup();
                    log_setup_done = true;
                }
                wasm_instance.wasm_loop();
            } else {
                // attempt to load the wasm only if there's lots of bucket
                if (Game.cpu.bucket < 1000) {
                    console.log("low bucket for wasm compile, waiting" + JSON.stringify(Game.cpu));
                    return;
                }
                // load the module, which we do here instead of at the top of the file, because 
                // that can potentially cause the module to be unable to load if it's too heavy,
                // and trap the load cycle with no bucket to recover
                let wasm_bytes = require(MODULE_NAME);
                // setup wasm instance
                let wasm_module = new WebAssembly.Module(wasm_bytes);
                // initialize the module
                wasm_instance = initSync(wasm_module);
                // run logging setup - then mark it as done only after it returns, since running out
                // of CPU time is possible at any time here
                wasm_instance.log_setup();
                log_setup_done = true;
                // keep going into the normal game wasm_loop after setup, it should handle the case of
                // realizing if the init took a lot of CPU time to make sure we don't use enough in
                // the first tick to risk an out-of-CPU crash and send us back to reloading
                wasm_instance.wasm_loop();
            }
        } catch (error) {
            // if we call `Game.cpu.halt();` this tick, console output from the tick (including the
            // stack trace) is not shown due to those contents being copied post-tick (and the halt
            // function destroying the environment immediately)
            halt_next_tick = true;
            // we've already logged the stack trace from rust via the panic hook, just write one
            // last log making the plan to destroy the next tick abundantly clear
            console.log("resetting VM next tick");
        }
    }
}
