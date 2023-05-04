### The Builder Stage Compiles the code
FROM parity/rust-builder AS builder

# Clone the template code and checkout the right commit
RUN https://github.com/SaaS3-Foundation/saas3-dao.git
WORKDIR /builds/saas3-dao
RUN git checkout main

# Build the Parachain Collator node
RUN cargo +nightly build --release

### The final stage just copies binary and chainspecs

# Choose the base image. Same on used in main Polkadot repo
FROM debian:stretch-slim

# Copy the node into the image
COPY --from=builder /builds/saas3-dao/target/release/saas3-dao-node .

## Copy chainspecs into the image
COPY chainspecs/raw-local-chainspec.json .

# Open default ports. User is responsible for re-mapping these
# or using host or overlay networking.
EXPOSE 30333 9933 9944

ENTRYPOINT ["./saas3-dao-node"]