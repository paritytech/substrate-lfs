# substrate-lfs
[Substrate Large File Storage](https://github.com/paritytech/substrate-lfs/)

A substrate extension to allow storing static data (like assets) along side your chain.

TODO: license

## Demo

There is an avatar demo runtime and node available in `demo`. You can try our default setup by running `cargo run --release -- --dev` from the root directory.

The demo has support for `UserData` and hosting of homepages through it. Once the server is running, you can the homepage for alice by running: `cargo run --release -p lfs-demo-rpc-client -- upload-dir --prefix "" --replace-index demo/example_data/personal_site/`. This demo client will read the directory and all its files, uploads each one via rpc to the `node` and then submits them as a batch as the home page for `Alice`. Once the offchain worker confirm the availability of the data, you can browse the website with the http-server included in the demo-node under `http://localhost:8080/5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY/` .

The rpc-client as further features, you can read all about them by passing `--help`. Among others, the uploader can be used to set the global hompage via the `--root` flag. If you, for example, run the `cargo run --release -p lfs-demo-rpc-client -- --root upload-dir --prefix "" --replace-index demo/example_data/website`, you can surf the example website on `http://localhost:8080` \o/ .