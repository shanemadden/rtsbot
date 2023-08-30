const gulp = require('gulp');
const screeps = require('gulp-screeps');

const rollup = require('rollup');
const copy = require('rollup-plugin-copy');
const node_resolve = require('@rollup/plugin-node-resolve');
const terser = require('@rollup/plugin-terser');

const spawn = require('child_process').spawn;
const del = require('del');
const fs = require('fs');
const yaml = require('yaml');
const argv = require('yargs')
    .option('upload', {
        alias: 'u',
    })
    .argv;

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

    if (argv.upload) {
        // check for a per-server terser config and override default
        // (or global config)
        if (terser_configs[argv.upload] !== undefined) {
            use_terser = terser_configs[argv.upload];
        }

        // check for per-server wasm-pack options array and add them on
        if (wasm_pack_options[argv.upload]) {
            extra_options = extra_options.concat(wasm_pack_options[argv.upload])
        }

        // modify the server config from unified format
        // (https://github.com/screepers/screepers-standards/blob/master/SS3-Unified_Credentials_File.md)
        // to the config expected by gulp-screeps: set `email` to `username`
        screeps_config = (config.servers || {})[argv.upload];
        if (screeps_config == null) throw new Error('Missing config section for specified upload destination');
        screeps_config.email = screeps_config.username;
    }
}

function clear_output() {
    return del(['dist', 'pkg']);
}

function compile_rs() {
    let args = ['run', 'nightly', 'wasm-pack', 'build', '--target', 'web', '--release', ...extra_options];
    return spawn('rustup', args, { stdio: 'inherit' });
}

async function compile_js() {
    const bundle = await rollup.rollup({
        input: './javascript/main.js',
        plugins: [
            node_resolve.nodeResolve(),
            copy({
              targets: [{ src: 'pkg/*.wasm', dest: 'dist' }]
            }),
        ]
    });
    await bundle.write({
        format: 'cjs',
        file: 'dist/main.js',
        plugins: [use_terser && terser()]
    });
}

function upload(done) {
    if (screeps_config) {
        return gulp.src('dist/*').pipe(screeps(screeps_config));
    } else {
        console.log('No --upload destination specified - not uploading!');
        done()
    }
}

exports.clean = clear_output;

exports.default = gulp.series(gulp.parallel(clear_output, load_config), compile_rs, compile_js, upload);
