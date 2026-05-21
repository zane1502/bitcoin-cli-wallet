# rsbtc

A mini Bitcoin implementation in Rust — a learning project featuring a full node, miner, wallet, and core library.

## Architecture

Workspace with 5 crates:

| Crate | Description |
|---|---|
| **`lib/`** (`btclib`) | Core library — block/transaction/blockchain types, SHA-256 hashing, ECDSA secp256k1 crypto, CBOR-serialized P2P network messages, U256 arithmetic, mempool management, difficulty adjustment |
| **`node/`** | Full node — TCP server that maintains the blockchain, validates blocks/transactions, serves block templates to miners, relays transactions, and syncs with peer nodes |
| **`miner/`** | Mining node — connects to a node, fetches block templates, performs proof-of-work (nonce brute-force), submits mined blocks |
| **`wallet/`** | Wallet app — key generation/loading, transaction building and signing, UTXO tracking, terminal UI (cursive), background sync tasks |
| **`encode_decode/`** | Standalone utility experimenting with serialization formats (JSON, TOML, YAML, CBOR) |

## Key Parameters

| Constant | Value |
|---|---|
| Initial block reward | 50 BTC (5,000,000,000 satoshis) |
| Halving interval | 210 blocks |
| Ideal block time | 10 seconds |
| Difficulty adjustment | Every 50 blocks |
| Min target | `0x0000FFFF...FFFF` |
| Max mempool TX age | 600 seconds |
| Max TX per block | 20 |

## Usage

```sh
# Start a seed node
cargo run --bin node

# Start a node connected to peers
cargo run --bin node -- 9000 ./blockchain.cbor 127.0.0.1:9000

# Generate keys
cargo run --bin key_gen

# Run the wallet (terminal UI)
cargo run --bin good-wallet -- --node 127.0.0.1:9000

# Run the miner
cargo run --bin miner -- --address 127.0.0.1:9000 --public-key-file ./pub.pem
```

## License

MIT
