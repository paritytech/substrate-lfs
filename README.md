# substrate-lfs
[Substrate Large File Storage](https://github.com/paritytech/substrate-lfs/)

A substrate extension to allow storing static data (like assets) along side your chain.

TODO: license

## Demo

There is an avatar demo runtime and node available in `demo`. You can try our default setup by running `cargo run --release -- --dev` from the root directory.

Once you have it running, add `Alice` as an authority to LFS by submitting `lfs.AddAuthority` via sudo to the chain (`Alice` is sudo). Once that is submitted, you can run the example `rpc-demo` to see the avatar/lfs live in action with `cargo run --release -p lfs-demo-rpc-client`.

The client will upload an included PNG via the RPC and create a transaction, submitting the `LFSReference` as the avatar for `Bob`. This will internally trigger an offchain worker event for the given `LFSReference` to be looked up. Once the lookup (in the local cache) suceeds, our Node will submit the confirmation back on chain, which will trigger setting the `LFSReference` for `Bob`. You can now fetch the `LFSReference`-content via the RPC provided by the node.