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