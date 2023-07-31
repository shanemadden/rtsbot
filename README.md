# screeps-starter-rust

Example Rust AI for [Screeps: World][screeps], the JavaScript-based MMO game.

This uses the [`screeps-game-api`] bindings from the [rustyscreeps] organization.

Instead of `cargo-screeps`, this example uses `gulp`, `wasm-pack`, `rollup`, and
`rollup-plugin-screeps` for building and deploying the code.

```sh
# Install rustup: https://rustup.rs/

# Install wasm-pack
cargo install wasm-pack

# Install nvm: https://github.com/nvm-sh/nvm
# (Windows: https://github.com/coreybutler/nvm-windows)

# Install node at version 16 (broken at 20, todo figure out exactly what breaks)
nvm install 16
nvm use 16

# Install deps
npm install

# Install gulp
npm install --global gulp-cli

# Set up for upload
cp .example-screeps.yaml .screeps.yaml
# (edit file, add API key etc)

# build to `dist` directory
gulp

# deploy to a configured server
gulp --dest mmo
```

[screeps]: https://screeps.com/
[`screeps-game-api`]: https://github.com/rustyscreeps/screeps-game-api/
[rustyscreeps]: https://github.com/rustyscreeps/
