use crate::{
    crypto::PublicKey,
    types::{Block, Transaction, TransactionOutput},
};
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, Read, Write};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Enum of all the possible messages that could be passed within the network
/// including a wallet, node and miner.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    /// Fetch all the UTXOs that belong a public key
    FetchUTXOs(PublicKey),

    /// UTXOs that belong to a public key, marked by a boolean value
    UTXOs(Vec<(TransactionOutput, bool)>),

    /// Send a Transaction to the network
    SubmitTransaction(Transaction),

    /// Broadcast a new transaction to the network
    NewTransaction(Transaction),

    /// Ask the node to prepare the optimal block template
    /// with the coinbase transaction paying the specified public key
    FetchTemplate(PublicKey),

    /// A proposed block template
    Template(Block),

    /// Ask the node to validate a block template
    ValidateTemplate(Block),

    /// The boolean state of a block's validity
    TemplateValidity(bool),

    /// Submit a mined block to a node
    SubmitTemplate(Block),

    /// Ask a node to report the nodes that it knows about
    DiscoverNodes,

    /// A response to DiscoverNodes Message
    NodeList(Vec<String>),

    /// Ask a block to compare the highest block it knows about to the local blockchain
    AskDifference(u32),

    /// The Response to AskDifference Message
    Difference(i32),

    /// Ask a node to send a block with the specified height
    FetchBlock(usize),

    /// Broadcast a new block to other nodes
    NewBlock(Block),
}

impl Message {
    pub fn encode(&self) -> Result<Vec<u8>, ciborium::ser::Error<IoError>> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes)?;
        Ok(bytes)
    }

    pub fn decode(data: &[u8]) -> Result<Self, ciborium::de::Error<IoError>> {
        ciborium::from_reader(data)
    }

    pub fn send(&self, stream: &mut impl Write) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len() as u64;
        stream.write_all(&len.to_be_bytes())?;
        stream.write_all(&bytes)?;
        Ok(())
    }

    pub fn receive(stream: &mut impl Read) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];
        stream.read_exact(&mut len_bytes)?;

        let len = u64::from_be_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];

        stream.read_exact(&mut data)?;
        Self::decode(&data)
    }

    pub async fn send_async(
        &self,
        stream: &mut (impl AsyncWrite + Unpin),
    ) -> Result<(), ciborium::ser::Error<IoError>> {
        let bytes = self.encode()?;
        let len = bytes.len() as u64;

        stream.write_all(&len.to_be_bytes()).await?;
        stream.write_all(&bytes).await?;
        Ok(())
    }

    pub async fn receive_async(
        stream: &mut (impl AsyncRead + Unpin),
    ) -> Result<Self, ciborium::de::Error<IoError>> {
        let mut len_bytes = [0u8; 8];

        stream.read_exact(&mut len_bytes).await?;

        let len = u64::from_be_bytes(len_bytes) as usize;

        let mut data = vec![0u8; len];

        stream.read_exact(&mut data).await?;
        Self::decode(&data)
    }
}
