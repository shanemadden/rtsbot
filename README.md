# rtsbot

Example Rust AI for [Screeps: World][screeps], the JavaScript-based MMO game.

This bot aims to provide a relatively capable Screeps bot, but with an emphasis on RTS-style
manual unit control, enabled via scripts injected into the Screeps client via log messages.

This uses the [`screeps-game-api`] bindings from the [rustyscreeps] organization.

Instead of `cargo-screeps`, this example uses `wasm-pack`, `rollup`, and the
[`screeps-api`] Node.js package for building and deploying the code.

```sh
# Install rustup: https://rustup.rs/

# Install wasm-pack
cargo install wasm-pack

# Install nvm: https://github.com/nvm-sh/nvm
# (Windows: https://github.com/coreybutler/nvm-windows)

# Install node at version 20
nvm install 20
nvm use 20

# Install deps
npm install

# Set up for upload
cp .example-screeps.yaml .screeps.yaml
# (edit file, add API key etc)

# deploy to a configured server
npm run deploy -- --server mmo
```

[screeps]: https://screeps.com/
[`screeps-game-api`]: https://github.com/rustyscreeps/screeps-game-api/
[rustyscreeps]: https://github.com/rustyscreeps/
[`screeps-api`]: https://github.com/screepers/node-screeps-api
