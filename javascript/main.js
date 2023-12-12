"use strict";
// replace this with the name of your module
const MODULE_NAME = "screeps_starter_rust_bg";
// TextEncoder/Decoder polyfill for UTF-8 conversion
import 'fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js';
import * as my_screeps_bot from '../pkg/screeps_starter_rust.js';
let wasm_instance;

// track whether the wasm instance has panicked, which triggers us halting the instance on
// the following tick
let halt_next_tick = false;

// This provides the function `console.error` that wasm_bindgen sometimes expects to exist,
// especially with type checks in debug mode. An alternative is to have this be `function () {}`
// and let the exception handler log the thrown JS exceptions, but there is some additional
// information that wasm_bindgen only passes here.
//
// There is nothing special about this function and it may also be used by any JS/Rust code as a convenience.
function console_error() {
    const processedArgs = _.map(arguments, (arg) => {
        if (arg instanceof Error) {
            // On this version of Node, the `stack` property of errors contains
            // the message as well.
            return arg.stack;
        } else {
            return arg;
        }
    }).join(' ');
    console.log("ERROR:", processedArgs);
    Game.notify(processedArgs);
}

module.exports.loop = function () {
    // need to freshly override the fake console object each tick
    console.error = console_error;
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
                my_screeps_bot.wasm_loop();
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
                // compile wasm module from bytes
                let wasm_module = new WebAssembly.Module(wasm_bytes);
                // initialize the module instance with its imports
                wasm_instance = my_screeps_bot.initSync(wasm_module);
                // keep going into the normal game wasm_loop after setup, it should handle the case of
                // realizing if the init took a lot of CPU time to make sure we don't use enough in
                // the first tick to risk an out-of-CPU crash and send us back to reloading
                my_screeps_bot.wasm_loop();
            }
        } catch (error) {
            // if we call `Game.cpu.halt();` this tick, console output from the tick (including the
            // stack trace) is not shown due to those contents being copied post-tick (and the halt
            // function destroying the environment immediately)
            halt_next_tick = true;
            // we've already logged the stack trace from rust via the panic hook, just write one
            // last log making the plan to destroy the next tick abundantly clear
            console.log("caught exception, will reset VM next tick: ", error);
            // not logging stack since the one from rust is generally better and this just adds noise,
            // but if we need it to debug, uncomment:
            // if (error.stack) {
            //    console.log("js stack:", error.stack);
            // }
        }
    }
}
