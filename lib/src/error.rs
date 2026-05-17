use thiserror::Error;

pub type Result<G> = std::result::Result<G, BtcLibError>;

#[derive(Debug, Error)]
pub enum BtcLibError {
    #[error("Invalid transaction")]
    InvalidTransaction,

    #[error("Invalid block")]
    InvalidBlock,

    #[error("Invalid block header")]
    InvalidBlockHeader,

    #[error("Invalid transaction input")]
    InvalidTransactionInput,

    #[error("Invalid transaction output")]
    InvalidTransactionOutput,

    #[error("Invalid Merkle root")]
    InvalidMerkleRoot,

    #[error("Invalid hash")]
    InvalidHash,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid public key")]
    InvalidPublicKey,

    #[error("Invalid private key")]
    InvalidPrivateKey,
}
