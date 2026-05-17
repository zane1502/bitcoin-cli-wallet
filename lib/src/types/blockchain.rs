use super::{Block, Transaction, TransactionOutput};
use crate::MAX_MEMPOOL_TRANSACTION_AGE;
use crate::{
    U256,
    error::{BtcLibError, Result},
    sha256::Hash,
    util::{MerkleRoot, Saveable},
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    blocks: Vec<Block>,
    target: U256,
    utxos: HashMap<Hash, (bool, TransactionOutput)>,
    #[serde(default, skip_serializing)]
    mempool: Vec<(DateTime<Utc>, Transaction)>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            target: crate::MIN_TARGET,
            utxos: HashMap::new(),
            mempool: vec![],
        }
    }

    // Add a new block and return () or return an error if the block is not valid
    // to be added to the chain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Check if the block is valid
        if self.blocks.is_empty() {
            // This is the first block's hash, so check if the previous block hash
            // is all zeroes. If it is not then this block is invalid.
            if block.header.prev_block_hash != Hash::zero() {
                println!("Zero Hash");
                return Err(BtcLibError::InvalidBlock);
            }
        } else {
            // In the event that this is not the first block in the chain,
            // then get the hash of the last block and make sure that it is not
            // equal to the hash of this current block
            let last_block: &Block = self.blocks.last().unwrap();

            if block.header.prev_block_hash != last_block.hash() {
                println!("Previous hash is wrong");
                return Err(BtcLibError::InvalidBlock);
            }

            // Check if the block's hash is less than the target
            if !block.header.hash().matches_target(block.header.target) {
                println!("Does not match target");
                return Err(BtcLibError::InvalidBlock);
            }

            let calculated_merkle_root = MerkleRoot::calculate_merkle_root(&block.transactions);

            if calculated_merkle_root != block.header.merkle_root {
                return Err(BtcLibError::InvalidMerkleRoot);
            }

            // Check if this particular block's timestamp is after the last block's
            // timestamp
            if block.header.timestamp <= last_block.header.timestamp {
                return Err(BtcLibError::InvalidBlock);
            }

            block.verify_transactions(self.block_height(), self.utxos.clone())?;
        }

        let block_transactions: HashSet<_> = block
            .transactions
            .iter()
            .map(|tx: &Transaction| tx.hash())
            .collect();

        self.mempool
            .retain(|(_, tx)| !block_transactions.contains(&tx.hash()));

        self.blocks.push(block);
        self.try_adjust_target();
        Ok(())
    }

    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for transaction in &block.transactions {
                for input in &transaction.inputs {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                }

                for output in transaction.outputs.iter() {
                    self.utxos
                        .insert(transaction.hash(), (false, output.clone()));
                }
            }
        }
    }

    pub fn try_adjust_target(&mut self) {
        if self.blocks.is_empty() {
            return;
        }

        if self.blocks.len() % crate::DIFFICULTY_UPDATE_INTERVAL as usize != 0 {
            return;
        }

        // Measure time taken to mine the last blocks from DIFFICULTY_UPDATE_INTERVAL
        let start_time = self.blocks
            [self.blocks.len() - crate::DIFFICULTY_UPDATE_INTERVAL as usize]
            .header
            .timestamp;

        let end_time = self.blocks.last().unwrap().header.timestamp;

        let time_diff = end_time - start_time;

        let time_diff_in_seconds = time_diff.num_seconds();

        let target_in_seconds = crate::DIFFICULTY_UPDATE_INTERVAL * crate::IDEAL_BLOCK_TIME;

        let new_target = BigDecimal::parse_bytes(&self.target.to_string().as_bytes(), 10)
            .expect("BIG: impossible")
            * (BigDecimal::from(time_diff_in_seconds) / (BigDecimal::from(target_in_seconds)));

        let new_target_str = new_target
            .to_string()
            .split('.')
            .next()
            .expect("BUG: expected a decimal point")
            .to_owned();

        let new_target: U256 = U256::from_str_radix(&new_target_str, 10).expect("BUG: impossible");

        let new_target = if new_target < self.target / 4 {
            self.target / 4
        } else if new_target > self.target * 4 {
            self.target * 4
        } else {
            new_target
        };

        self.target = new_target.min(crate::MIN_TARGET);
    }

    pub fn utxos(&self) -> &HashMap<Hash, (bool, TransactionOutput)> {
        &self.utxos
    }

    pub fn target(&self) -> U256 {
        self.target
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.iter()
    }

    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    pub fn mempool(&self) -> &[(DateTime<Utc>, Transaction)] {
        &self.mempool
    }

    // add a transaction to the mempool
    pub fn add_to_mempool(&mut self, transaction: Transaction) -> Result<()> {
        let mut known_inputs = HashSet::new();

        for input in &transaction.inputs {
            if !self.utxos.contains_key(&input.prev_transaction_output_hash) {
                println!("UTXO not found");
                dbg!(&self.utxos);
                return Err(BtcLibError::InvalidTransaction);
            }

            if known_inputs.contains(&input.prev_transaction_output_hash) {
                println!("Duplicate Input");
                return Err(BtcLibError::InvalidTransaction);
            }
            known_inputs.insert(input.prev_transaction_output_hash);
        }

        for input in &transaction.inputs {
            if let Some((true, _)) = self.utxos.get(&input.prev_transaction_output_hash) {
                // Look for the transaction that references the UTXO
                // we want to reference
                let referencing_tx =
                    self.mempool
                        .iter()
                        .enumerate()
                        .find(|(_, (_, transaction))| {
                            transaction
                                .outputs
                                .iter()
                                .any(|output| output.hash() == input.prev_transaction_output_hash)
                        });

                // If one is found unmark all of its UTXOs
                if let Some((idx, (_, referencing_tx))) = referencing_tx {
                    // Set all UTXOs from this transaction to false
                    for input in &referencing_tx.inputs {
                        self.utxos
                            .entry(input.prev_transaction_output_hash)
                            .and_modify(|(marked, _)| {
                                *marked = false;
                            });
                    }

                    // Remove this transaction from the mempool
                    self.mempool.remove(idx);
                } else {
                    // If no transaction matches...
                    // Set this utxo to false
                    self.utxos
                        .entry(input.prev_transaction_output_hash)
                        .and_modify(|(marked, _)| {
                            *marked = false;
                        });
                }
            }
        }

        let all_inputs = transaction
            .inputs
            .iter()
            .map(|input| {
                self.utxos
                    .get(&input.prev_transaction_output_hash)
                    .expect("BUG: Impossible")
                    .1
                    .value
            })
            .sum::<u64>();

        let all_outputs = transaction.outputs.iter().map(|output| output.value).sum();

        if all_inputs < all_outputs {
            return Err(BtcLibError::InvalidTransaction);
        }

        // Add the transaction along with the time it was added into the mempool
        self.mempool.push((Utc::now(), transaction));

        // sort the mempool by the miner fees
        self.mempool.sort_by_key(|(_, transaction)| {
            let all_inputs = transaction
                .inputs
                .iter()
                .map(|input| {
                    self.utxos
                        .get(&input.prev_transaction_output_hash)
                        .expect("Bug: Impossible")
                        .1
                        .value
                })
                .sum::<u64>();

            let all_outputs: u64 = transaction.outputs.iter().map(|output| output.value).sum();

            let miner_fee = all_inputs - all_outputs;
            miner_fee
        });

        Ok(())
    }

    pub fn cleanup_mempool(&mut self) {
        let now = Utc::now();
        let mut utxo_hashes_to_unmark: Vec<Hash> = vec![];

        self.mempool.retain(|(timestamp, transaction)| {
            if now - *timestamp > chrono::Duration::seconds(MAX_MEMPOOL_TRANSACTION_AGE as i64) {
                utxo_hashes_to_unmark.extend(
                    transaction
                        .inputs
                        .iter()
                        .map(|input| input.prev_transaction_output_hash),
                );

                false
            } else {
                true
            }
        });

        for hash in utxo_hashes_to_unmark {
            self.utxos.entry(hash).and_modify(|(marked, _)| {
                *marked = false;
            });
        }
    }

    pub fn calculate_block_reward(&self) -> u64 {
        let block_height = self.block_height();
        let halvings = block_height / crate::HALVING_INTERVAL;
        (crate::INITIAL_REWARD * 10u64.pow(8)) >> halvings
    }
}

impl Saveable for Blockchain {
    fn load<I: Read>(reader: I) -> IoResult<Self> {
        ciborium::de::from_reader(reader)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to deserialize Blockchain"))
    }

    fn save<O: Write>(&self, writer: O) -> IoResult<()> {
        ciborium::ser::into_writer(self, writer)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "Failed to serialize Blockchain"))
    }
}
