"use strict";
let wasm_module;

// replace this with the name of your module
const MODULE_NAME = "screeps-starter-rust";

function console_error(...args) {
    console.log(...args);
    Game.notify(args.join(' '));
}

let log_setup_done = false;
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
            if (wasm_module && wasm_module.__wasm) {
                // deal with the case of the wasm init having completed but not having gotten
                // as far as running the logging setup due to running out of CPU mid-setup prior
                if (log_setup_done === false) {
                    wasm_module.log_setup();
                    log_setup_done = true;
                }
                wasm_module.loop();
            } else {
                // attempt to load the wasm only if there's lots of bucket
                if (Game.cpu.bucket < 1000) {
                    console.log("low CPU, waiting" + JSON.stringify(Game.cpu));
                    return;
                }
                // load the module, which we do here instead of at the top of the filebecause that
                // can potentially cause the module to be unable to load if it's too heavy and trap
                // the load cycle with no bucket to recover
                wasm_module = require(MODULE_NAME);
                // setup wasm instance, which attaches at wasm_module.__wasm
                wasm_module.initialize_instance();
                // run logging setup - then mark it as done only after it returns, since running out
                // of CPU time is possible at any time here
                wasm_module.log_setup();
                log_setup_done = true;
                // keep going into the normal game loop after setup, it should handle the case of
                // realizing if the init took a lot of CPU time to make sure we don't use enough in
                // the first tick to risk an out-of-CPU crash and send us back to reloading
                wasm_module.loop();
            }
        } catch (error) {
            console_error("caught exception:", error);
            if (error.stack) {
                console_error("stack trace:", error.stack);
            }
            console_error("resetting VM next tick.");
            // if we call `Game.cpu.halt();` this tick, console output from the tick (including the
            // stack trace) is not shown due to those contents being copied post-tick (and the halt
            // function destroying the environment immediately)
            halt_next_tick = true;
        }
    }
}
