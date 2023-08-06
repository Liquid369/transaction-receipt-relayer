use crate::{TransactionReceipt, ValidationError, H256};
use alloy_rlp::Encodable;
use cita_trie::MemoryDB;
use cita_trie::{PatriciaTrie, Trie};
use hasher::HasherKeccak;
use std::sync::Arc;

/// A Merkle proof that a transaction receipt has been included in a block.
///
/// Merkle proofs for transaction receipts use Ethereum's [Patricia Merkle Trie][1] data structure.
/// The `receipt_root` field in a block is the root of the trie.
///
/// Requires a [`TransactionReceipt`] to generate a leaf node, and the rest of the proof proceeds
/// from the leaf node.
///
/// [1]: https://ethereum.org/se/developers/docs/data-structures-and-encoding/patricia-merkle-trie/
pub struct ReceiptMerkleProof {
    pub proof: PatriciaTrie<MemoryDB, HasherKeccak>,
}

impl ReceiptMerkleProof {
    pub fn new() -> Self {
        let proof = PatriciaTrie::new(Arc::new(MemoryDB::new(true)), Arc::new(HasherKeccak::new()));
        Self { proof }
    }

    /// Given a transaction receipt, compute the Merkle root of the Patricia Merkle Trie using the
    /// rest of the Merkle proof.
    pub fn merkle_root(&mut self, leaf: &TransactionReceipt) -> Result<H256, ValidationError> {
        let mut value = vec![];
        leaf.encode(&mut value);
        let key = H256::hash(value.clone()).0.to_vec();
        self.proof
            .insert(key, value)
            .map_err(|_| ValidationError::IntenalPatriciaTrieError)?;
        let root = self
            .proof
            .root()
            .map_err(|_| ValidationError::IntenalPatriciaTrieError)?;
        if root.len() != 32 {
            return Err(ValidationError::InternalError);
        }

        Ok(H256(
            root[..]
                .try_into()
                .map_err(|_| ValidationError::InternalError)?,
        ))
    }
}
