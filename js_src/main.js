"use strict";
// replace this with the name of your module
const MODULE_NAME = "screeps_starter_rust_bg";

// TextEncoder/Decoder polyfill for UTF-8 conversion
import 'fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js';

import './client_scripts.js';
import * as screeps_bot from '../pkg/screeps_starter_rust.js';

// glue functions for client scripts
global.update_selected_object = screeps_bot.update_selected_object;
global.right_click_position = screeps_bot.right_click_position;

// This provides the function `console.error` that wasm_bindgen sometimes expects to exist,
// especially with type checks in debug mode. An alternative is to have this be `function () {}`
// and let the exception handler log the thrown JS exceptions, but there is some additional
// information that wasm_bindgen only passes here.
//
// There is nothing special about this function and it may also be used by any JS/Rust code as a
// convenience.
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

// track whether running wasm loop for each tick completes, to detect errors or aborted execution
let running = false;

function main_loop() {
    if (running) {
        // we've had an error on the last tick; skip execution during the current tick, asking to
        // have our IVM immediately destroyed so we get a fresh environment next tick;
        // workaround for https://github.com/rustwasm/wasm-bindgen/issues/3130
        Game.cpu.halt();
    } else {
        // Replace the Memory object (which gets populated into our global each tick) with an empty
        // object, so that accesses to it from within the driver that we can't prevent (such as
        // when a creep is spawned) won't trigger an attempt to parse RawMemory. Replace the object
        // with one unattached to memory magic - game functions will access the `Memory` object and
        // can throw data in here, and it'll go away at the end of tick.
        delete global.Memory;
        global.Memory = {};
        // also override the console.error, if we're in dev mode bindgen might use it (and it, too,
        // gets overwritten every tick)
        console.error = console_error;

        try {
            running = true;
            screeps_bot.wasm_loop();
            // if execution doesn't get to this point for any reason (error or out-of-CPU
            // cancellation), setting to false won't happen which will cause a halt() next tick
            running = false;
        } catch (error) {
            console.log("caught exception, will reset next tick: ", error);
            // not logging stack since we've already logged the stack trace from rust via the panic
            // hook and that one is generally better, but if we need it, uncomment:

            // if (error.stack) {
            //     console.log("js stack:", error.stack);
            // }
        }
    }
}

// cache for each step of the wasm module's initialization
let wasm_bytes;
let wasm_module;
let wasm_instance;

module.exports.loop = function() {
    // attempt to load the wasm only if there's lots of bucket
    if (Game.cpu.bucket < 1000) {
        console.log("low bucket for wasm compile, waiting" + JSON.stringify(Game.cpu));
        return;
    }
    // load the module, which we do here instead of at the top of the file, because that can
    // potentially cause the module to be unable to load if it's too heavy, and trap the load cycle
    // with no bucket to recover
    if (!wasm_bytes) {
        wasm_bytes = require(MODULE_NAME);
    }
    // compile wasm module from bytes
    if (!wasm_module) {
        wasm_module = new WebAssembly.Module(wasm_bytes);
    }
    // initialize the module instance with its imports
    if (!wasm_instance) {
        wasm_instance = screeps_bot.initSync(wasm_module);
    }
    // remove the bytes from the heap and require cache, we don't need 'em anymore
    wasm_bytes = null;
    delete require.cache[MODULE_NAME];
    // replace this function with the post-load loop for next tick
    module.exports.loop = main_loop;
    console.log("loading complete, CPU used: " + Game.cpu.getUsed())
}
