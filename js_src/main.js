"use strict";
// TextEncoder/Decoder polyfill for UTF-8 conversion
import 'fastestsmallesttextencoderdecoder-encodeinto/EncoderDecoderTogether.min.js';

import './client_scripts.js';
import * as screeps_bot from '../pkg/rtsbot.js';

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
    console.log('<font color="#DC5257">[ERROR]</font>', processedArgs);
    Game.notify(processedArgs);
}

// track whether running wasm loop for each tick completes, to detect errors or aborted execution
let running = false;

// replacement for the default Memory object which can be written into by game functions; will be
// forgotten instead of persisted on global reset, and RawMemory used from within wasm
let js_memory = {};

function loaded_loop() {
    if (running) {
        // we've had an error on the last tick; skip execution during the current tick, asking to
        // have our IVM immediately destroyed so we get a fresh environment next tick;
        // workaround for https://github.com/rustwasm/wasm-bindgen/issues/3130
        Game.cpu.halt();
    } else {
        // Replace the Memory object (which gets populated into our global each tick) with our local
        // heap object, so that js won't try to parse RawMemory.
        delete global.Memory;
        global.Memory = js_memory;
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
            console.log(`<font color="#DC5257">[ERROR]</font> rtsbot: caught exception, will halt next tick: ${error}`);
            // not logging stack since we've already logged the stack trace from rust via the panic
            // hook and that one is generally better, but if we need it, uncomment:

            // if (error.stack) {
            //     console.log("js stack:", error.stack);
            // }
        }
    }
}

// cache for each step of the wasm module's initialization
let wasm_bytes, wasm_module, wasm_instance;

module.exports.loop = function() {
    // attempt to load the wasm only if there's lots of bucket
    if (Game.cpu.bucket < 1250) {
        console.log(`<font color="#DC5257">[WARN]</font> rtsbot: startup deferred; ${Game.cpu.bucket} / 1250 required bucket`);
        return;
    }
    
    // run each step of the load process, saving each result so that this can happen over multiple ticks
    if (!wasm_bytes) wasm_bytes = require('rtsbot');
    if (!wasm_module) wasm_module = new WebAssembly.Module(wasm_bytes);
    if (!wasm_instance) wasm_instance = screeps_bot.initSync(wasm_module);

    // remove the bytes from the heap and require cache, we don't need 'em anymore
    wasm_bytes = null;
    delete require.cache['rtsbot'];
    // replace this function with the post-load loop for next tick
    module.exports.loop = loaded_loop;
    console.log(`<font color="#F3C87B">[INFO]</font> rtsbot: loading complete, CPU used: ${Game.cpu.getUsed()}`)
}
