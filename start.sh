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
