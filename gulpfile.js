const { series, parallel } = require('gulp');
const spawn = require('child_process').spawn;
const del = require('del');
const fs = require('fs');
const yaml = require('yaml');
const argv = require('yargs').argv;

const rollup = require('rollup');
const copy = require('rollup-plugin-copy');
const node_resolve = require('@rollup/plugin-node-resolve');
const screeps = require('rollup-plugin-screeps');
const terser = require('@rollup/plugin-terser');

// config object for rollup-plugin-screeps 
let screeps_config;
// whether to run terser
let use_terser = false;
// any additional flags to pass to wasm-pack
let extra_options = [];

async function load_config() {
    // read and parse the config
    const config = yaml.parse(fs.readFileSync('.screeps.yaml', { encoding: 'utf8' }));

    // regardless of whether we have a destination or not, load the 'global' configs
    // for terser and wasm-pack extra options
    let configs = config.configs || {};

    let terser_configs = configs.terser || {};
    if (terser_configs["*"] !== undefined) {
        use_terser = terser_configs["*"];
    }

    let wasm_pack_options = configs["wasm-pack-options"] || {};
    if (wasm_pack_options["*"]) {
        extra_options = extra_options.concat(wasm_pack_options["*"])
    }

    if (!argv.dest) {
        console.log('No --dest specified - code will be compiled but not uploaded!');
    } else {
        // check for a per-server terser config and override default
        // (or global config)
        if (terser_configs[argv.dest] !== undefined) {
            use_terser = terser_configs[argv.dest];
        }

        // check for per-server wasm-pack options array and add them on
        if (wasm_pack_options[argv.dest]) {
            extra_options = extra_options.concat(wasm_pack_options[argv.dest])
        }

        // modify the server config from unified format
        // (https://github.com/screepers/screepers-standards/blob/master/SS3-Unified_Credentials_File.md)
        // to the config expected by rollup-plugin-screeps
        screeps_config = (config.servers || {})[argv.dest];
        if (screeps_config == null) throw new Error('Missing config for --dest');
        screeps_config.hostname = screeps_config.host;
        screeps_config.port = screeps_config.port || (screeps_config.secure ? 443 : 21025);
        screeps_config.host = `${screeps_config.host}:${screeps_config.port}`;
        screeps_config.email = screeps_config.username;
        screeps_config.protocol = screeps_config.secure ? 'https' : 'http';
        screeps_config.path = screeps_config.path || '/';
        // 'auto' will cause rollup plugin to use the local git branch's name
        screeps_config.branch = screeps_config.branch || 'auto';
    }
}

function wasm_pack() {
    let args = ['run', 'nightly', 'wasm-pack', 'build', '--target', 'web', '--release', ...extra_options];
    return spawn('rustup', args, { stdio: 'inherit' });
}

async function run_rollup() {
    const bundle = await rollup.rollup({
        input: './javascript/main.js',
        plugins: [
            node_resolve.nodeResolve(),
            copy({
              targets: [
                { src: 'pkg/*.wasm', dest: 'dist' }
              ]
            }),
        ]
    });
    await bundle.write({
        format: 'cjs',
        file: 'dist/main.js',
        plugins: [
            use_terser && terser(),
            screeps({ config: screeps_config, dryRun: !screeps_config })
        ]
    });
}

function clean() {
    return del(['dist', 'pkg']);
}

exports.clean = clean;

exports.default = series(parallel(clean, load_config), wasm_pack, run_rollup);
