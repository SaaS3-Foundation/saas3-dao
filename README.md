# SaaS3 DAO

SaaS3 DAO refers to the decentralized autonomous organization (DAO), a governance body established with a full-fledged mechanism for token economics. The platform runs on the Substrate blockchain and uses two pallets: `pallet-treasury` and `pallet-court`. `pallet-treasury` is used for collecting funds, while `pallet-court` is used to resolve disputes between users of the SaaS3 oracle.

## pallet-treasury

The `pallet-treasury` pallet is responsible for collecting and managing funds for the SaaS3 DAO platform.

## pallet-court

The `pallet-court` pallet provides a platform for resolving disputes between users of the SaaS3 oracle. When a dispute arises, users can submit evidence to support their case. A panel of judges is then selected to review the evidence and make a ruling. If the ruling is in favor of the plaintiff, they are awarded a reward from the defendant's bond.

## Building and Testing

To build the SaaS3 DAO project, run the following command:

```
cargo +nightly build --release
```

To run tests, use the following command:

```
cargo +nightly test
```

## Run a testnet
#### Start relay chain
>Follow the [polkadot offcial doc](https://docs.substrate.io/tutorials/build-a-parachain/prepare-a-local-relay-chain/) or the following doc to run the relay chain.

Before start, you need download polkadot release or build from source yourself. You can found downloaded relay chain spec from this repo, they are in the `chainspecs` folder.

Start the first validator using the alice account by running the following command:
```
 ./target/release/polkadot \
--alice \
--validator \
--base-path /tmp/relay/alice \
--chain /path/to/saas3-dao/chainspecs/raw-local-chainspec.json \
--port 30333 \
--rpc-port 9944
```
Open a new terminal and start the second validator using the bob account.
```
./target/release/polkadot \
--bob \
--validator \
--base-path /tmp/relay-bob \
--chain ~/workhub/github/saas3-foundation/saas3-dao/chainspecs/raw-local-chainspec.json \
--port 30334 \
--rpc-port 9945
```

#### Prepare parachain node
Follow the [polkadot official doc](https://docs.substrate.io/tutorials/build-a-parachain/connect-a-local-parachain/) to reserve a parachain identifier. The spec files are in the `chainspecs` folder. After that, follow following steps to generate files for parachain node.
Export the WebAssembly runtime for the parachain.
```
./target/release/saas3-dao-node export-genesis-wasm --chain chainspecs/raw-parachain-chainspec.json para-2000-wasm
```
Generate a parachain genesis state
```
./target/release/saas3-dao-node export-genesis-state --chain chainspecs/raw-parachain-chainspec.json para-2000-genesis-state
```
#### Start parachain node
```
 ./target/release/saas3-dao-node \
--alice \
--collator \
--force-authoring \
--chain chainspecs/raw-parachain-chainspec.json \
--base-path /tmp/parachain/alice \
--port 40333 \
--ws-port 8844 \
-- \
--execution wasm \
--chain chainspecs/raw-local-chainspec.json \
--port 30343 \
--ws-port 9977
```