// SPDX-License-Identifier: CC0-1.0

//! Blockdata constants.
//!
//! This module provides various constants relating to the blockchain and
//! consensus code. In particular, it defines the genesis block and its
//! single transaction.
//!

use core::default::Default;

use hashes::{sha256d, Hash};
use hex_lit::hex;
use internals::impl_array_newtype;

use crate::blockdata::block::{self, Block};
use crate::blockdata::locktime::absolute;
use crate::blockdata::opcodes::all::*;
use crate::blockdata::script;
use crate::blockdata::transaction::{self, OutPoint, Sequence, Transaction, TxIn, TxOut};
use crate::blockdata::witness::Witness;
use crate::internal_macros::impl_bytes_newtype;
use crate::network::Network;
use crate::pow::CompactTarget;
use crate::Amount;

/// How many seconds between blocks we expect on average.
pub const TARGET_BLOCK_SPACING: u32 = 60;
/// How many blocks between diffchanges.
pub const DIFFCHANGE_INTERVAL: u32 = 2016;
/// How much time on average should occur between diffchanges.
pub const DIFFCHANGE_TIMESPAN: u32 = 24 * 60 * 60;

#[deprecated(since = "0.31.0", note = "Use Weight::MAX_BLOCK instead")]
/// The maximum allowed weight for a block, see BIP 141 (network rule).
pub const MAX_BLOCK_WEIGHT: u32 = 4_000_000;

#[deprecated(since = "0.31.0", note = "Use Weight::MIN_TRANSACTION instead")]
/// The minimum transaction weight for a valid serialized transaction.
pub const MIN_TRANSACTION_WEIGHT: u32 = 4 * 60;

/// The factor that non-witness serialization data is multiplied by during weight calculation.
pub const WITNESS_SCALE_FACTOR: usize = 4;
/// The maximum allowed number of signature check operations in a block.
pub const MAX_BLOCK_SIGOPS_COST: i64 = 80_000;
/// Mainnet pubkey address prefix (Bit mainnet 'B' => 25 / 0x19).
pub const PUBKEY_ADDRESS_PREFIX_MAIN: u8 = 25; // 0x19
/// Mainnet script address prefix (Bit mainnet => 22 / 0x16).
pub const SCRIPT_ADDRESS_PREFIX_MAIN: u8 = 22; // 0x16
/// Testnet/signet pubkey address prefix (Bit testnet 'T' => 65 / 0x41).
pub const PUBKEY_ADDRESS_PREFIX_TEST: u8 = 65; // 0x41
/// Testnet/signet script address prefix (Bit testnet => 196 / 0xc4).
pub const SCRIPT_ADDRESS_PREFIX_TEST: u8 = 196; // 0xc4
// Regtest pubkey address prefix (keep existing if unspecified by Bit params).
pub const PUBKEY_ADDRESS_PREFIX_REGTEST: u8 = 47; // 0x2f
/// The maximum allowed script size.
pub const MAX_SCRIPT_ELEMENT_SIZE: usize = 520;
/// How may blocks between halvings.
pub const SUBSIDY_HALVING_INTERVAL: u32 = 100_000;
/// Maximum allowed value for an integer in Script.
pub const MAX_SCRIPTNUM_VALUE: u32 = 0x80000000; // 2^31
/// Number of blocks needed for an output from a coinbase transaction to be spendable.
pub const COINBASE_MATURITY: u32 = 70;

/// Constructs and returns the coinbase (and only) transaction of the Bit genesis block.
fn bitcoin_genesis_tx() -> Transaction {
    // Base
    let mut ret = Transaction {
        version: transaction::Version::ONE,
        lock_time: absolute::LockTime::ZERO,
        input: vec![],
        output: vec![],
    };

    // Inputs
    let in_script = script::Builder::new()
        .push_int(486604799)
        .push_int_non_minimal(4)
        .push_slice(b"Follow The White Rabbit")
        .into_script();
    ret.input.push(TxIn {
        previous_output: OutPoint::null(),
        script_sig: in_script,
        sequence: Sequence::MAX,
        witness: Witness::default(),
    });

    // Outputs
    // Bit mainnet genesis pubkey (from Bit chainparams).
    let script_bytes = hex!("042e8ae07eee20bacb42b873bb1e9f7c507089d1826de4eaed5109a238a1f329df87c5dc06d3fe1c7cb4f6d8325ea333f3a2519cdcd4327ce240da348a257f6585");
    let out_script =
        script::Builder::new().push_slice(script_bytes).push_opcode(OP_CHECKSIG).into_script();
    // Bit genesis reward: 5 * COIN
    ret.output.push(TxOut { value: Amount::from_sat(5 * 100_000_000), script_pubkey: out_script });

    // end
    ret
}

/// Constructs and returns the genesis block.
pub fn genesis_block(network: Network) -> Block {
    let txdata = vec![bitcoin_genesis_tx()];
    // Bit genesis coinbase merkle root (from chainparams).
    let hash: sha256d::Hash = sha256d::Hash::from_slice(&hex!("a97aa9a0d3e21626a97d73a02d2afe352e9dcdd211a7427f30525498e6706748")).unwrap();
    let merkle_root = hash.into();
    match network {
        Network::Bitcoin => Block {
            header: block::Header {
                version: block::Version::ONE,
                prev_blockhash: Hash::all_zeros(),
                merkle_root,
                time: 1751109927,
                bits: CompactTarget::from_consensus(0x1e0ffff0),
                nonce: 411785,
                aux_data: None,
            },
            txdata,
        },
        Network::Testnet => Block {
            header: block::Header {
                version: block::Version::ONE,
                prev_blockhash: Hash::all_zeros(),
                merkle_root,
                time: 1751110035,
                bits: CompactTarget::from_consensus(0x1e0ffff0),
                nonce: 916278,
                aux_data: None,
            },
            txdata,
        },
        Network::Signet => Block {
            header: block::Header {
                version: block::Version::ONE,
                prev_blockhash: Hash::all_zeros(),
                merkle_root,
                // Bit does not define signet separately; mirror testnet values.
                time: 1751110035,
                bits: CompactTarget::from_consensus(0x1e0ffff0),
                nonce: 916278,
                aux_data: None,
            },
            txdata,
        },
        Network::Regtest => Block {
            header: block::Header {
                version: block::Version::ONE,
                prev_blockhash: Hash::all_zeros(),
                merkle_root,
                time: 1751110269,
                bits: CompactTarget::from_consensus(0x207fffff),
                nonce: 1,
                aux_data: None,
            },
            txdata,
        },
    }
}

/// The uniquely identifying hash of the target blockchain.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainHash([u8; 32]);
impl_array_newtype!(ChainHash, u8, 32);
impl_bytes_newtype!(ChainHash, 32);

impl ChainHash {
    // Mainnet value can be verified at https://github.com/lightning/bolts/blob/master/00-introduction.md
    //https://bitcoin.stackexchange.com/questions/74358/what-is-bitcoins-genesis-hash
    /// `ChainHash` for mainnet bitcoin.

    // Bit mainnet/testnet/signet/regtest chain hashes (genesis block hashes).
    pub const BITCOIN: Self = Self([
        134, 135, 245, 229, 95, 156, 176, 229, 246, 74, 135, 55, 174, 195, 166, 115,
        94, 23, 5, 86, 94, 254, 57, 24, 51, 63, 219, 150, 228, 63, 119, 22,
    ]);
    /// `ChainHash` for testnet (Bit testnet).
    pub const TESTNET: Self = Self([
        1, 178, 115, 242, 174, 246, 201, 209, 46, 159, 196, 98, 129, 227, 62, 205,
        30, 74, 132, 212, 137, 168, 102, 88, 150, 161, 249, 16, 104, 144, 177, 170,
    ]);
    /// `ChainHash` for signet (mirror testnet for Bit).
    pub const SIGNET: Self = Self([
        1, 178, 115, 242, 174, 246, 201, 209, 46, 159, 196, 98, 129, 227, 62, 205,
        30, 74, 132, 212, 137, 168, 102, 88, 150, 161, 249, 16, 104, 144, 177, 170,
    ]);
    /// `ChainHash` for regtest (Bit regtest).
    pub const REGTEST: Self = Self([
        0, 237, 82, 108, 34, 217, 180, 205, 18, 215, 184, 249, 211, 10, 216, 158,
        180, 162, 151, 80, 5, 179, 19, 12, 142, 140, 112, 184, 5, 16, 194, 214,
    ]);

    /// Returns the hash of the `network` genesis block for use as a chain hash.
    ///
    /// See [BOLT 0](https://github.com/lightning/bolts/blob/ffeece3dab1c52efdb9b53ae476539320fa44938/00-introduction.md#chain_hash)
    /// for specification.
    pub fn using_genesis_block(network: Network) -> Self {
        let bh = genesis_block(network).block_hash();
        Self::from_genesis_block_hash(bh)
    }

    /// Converts genesis block hash into `ChainHash`.
    pub fn from_genesis_block_hash(block_hash: crate::BlockHash) -> Self {
        ChainHash(block_hash.to_byte_array())
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;

    use hex::test_hex_unwrap as hex;

    use super::*;
    use crate::blockdata::locktime::absolute;
    use crate::blockdata::transaction;
    use crate::consensus::encode::serialize;
    use crate::network::Network;

    #[test]
    #[ignore]
    fn bitcoin_genesis_first_transaction() {
        let gen = bitcoin_genesis_tx();

        assert_eq!(gen.version, transaction::Version::ONE);
        assert_eq!(gen.input.len(), 1);
        assert_eq!(gen.input[0].previous_output.txid, Hash::all_zeros());
        assert_eq!(gen.input[0].previous_output.vout, 0xFFFFFFFF);
        assert_eq!(serialize(&gen.input[0].script_sig),
                   hex!("4d04ffff001d0104455468652054696d65732030332f4a616e2f32303039204368616e63656c6c6f72206f6e206272696e6b206f66207365636f6e64206261696c6f757420666f722062616e6b73"));

        assert_eq!(gen.input[0].sequence, Sequence::MAX);
        assert_eq!(gen.output.len(), 1);
        assert_eq!(serialize(&gen.output[0].script_pubkey),
                   hex!("434104678afdb0fe5548271967f1a67130b7105cd6a828e03909a67962e0ea1f61deb649f6bc3f4cef38c4f35504e51ec112de5c384df7ba0b8d578a4c702b6bf11d5fac"));
        assert_eq!(gen.output[0].value, Amount::from_str("50 BTC").unwrap());
        assert_eq!(gen.lock_time, absolute::LockTime::ZERO);

        assert_eq!(
            gen.wtxid().to_string(),
            "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
        );
    }

    #[test]
    fn bitcoin_genesis_full_block() {
        let gen = genesis_block(Network::Bitcoin);

        assert_eq!(gen.header.version, block::Version::ONE);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(
            gen.header.merkle_root.to_string(),
            "486770e6985452307f42a711d2cd9d2e35fe2a2da0737da92616e2d3a0a97aa9"
        );

        assert_eq!(gen.header.time, 1751109927);
        assert_eq!(gen.header.bits, CompactTarget::from_consensus(0x1e0ffff0));
        assert_eq!(gen.header.nonce, 411785);
        assert_eq!(
            gen.header.block_hash().to_string(),
            "16773fe496db3f331839fe5e5605175e73a6c3ae37874af6e5b09c5fe5f58786"
        );
    }

    #[test]
    fn testnet_genesis_full_block() {
        let gen = genesis_block(Network::Testnet);
        assert_eq!(gen.header.version, block::Version::ONE);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(
            gen.header.merkle_root.to_string(),
            "486770e6985452307f42a711d2cd9d2e35fe2a2da0737da92616e2d3a0a97aa9"
        );
        assert_eq!(gen.header.time, 1751110035);
        assert_eq!(gen.header.bits, CompactTarget::from_consensus(0x1e0ffff0));
        assert_eq!(gen.header.nonce, 916278);
        assert_eq!(
            gen.header.block_hash().to_string(),
            "aab1906810f9a1965866a889d4844a1ecd3ee38162c49f2ed1c9f6aef273b201"
        );
    }

    #[test]
    fn signet_genesis_full_block() {
        let gen = genesis_block(Network::Signet);
        assert_eq!(gen.header.version, block::Version::ONE);
        assert_eq!(gen.header.prev_blockhash, Hash::all_zeros());
        assert_eq!(
            gen.header.merkle_root.to_string(),
            "486770e6985452307f42a711d2cd9d2e35fe2a2da0737da92616e2d3a0a97aa9"
        );
        assert_eq!(gen.header.time, 1751110035);
        assert_eq!(gen.header.bits, CompactTarget::from_consensus(0x1e0ffff0));
        assert_eq!(gen.header.nonce, 916278);
        assert_eq!(
            gen.header.block_hash().to_string(),
            "aab1906810f9a1965866a889d4844a1ecd3ee38162c49f2ed1c9f6aef273b201"
        );
    }

    // The *_chain_hash tests are sanity/regression tests, they verify that the const byte array
    // representing the genesis block is the same as that created by hashing the genesis block.
    fn chain_hash_and_genesis_block(network: Network) {
        use hashes::sha256;

        // The genesis block hash is a double-sha256 and it is displayed backwards.
        let genesis_hash = genesis_block(network).block_hash();
        // We abuse the sha256 hash here so we get a LowerHex impl that does not print the hex backwards.
        let hash = sha256::Hash::from_slice(genesis_hash.as_byte_array()).unwrap();
        let want = format!("{:02x}", hash);

        let chain_hash = ChainHash::using_genesis_block(network);
        let got = format!("{:02x}", chain_hash);

        // Compare strings because the spec specifically states how the chain hash must encode to hex.
        assert_eq!(got, want);

        #[allow(unreachable_patterns)] // This is specifically trying to catch later added variants.
        match network {
            Network::Bitcoin => {},
            Network::Testnet => {},
            Network::Signet => {},
            Network::Regtest => {},
            _ => panic!("Update ChainHash::using_genesis_block and chain_hash_genesis_block with new variants"),
        }
    }

    macro_rules! chain_hash_genesis_block {
        ($($test_name:ident, $network:expr);* $(;)*) => {
            $(
                #[test]
                fn $test_name() {
                    chain_hash_and_genesis_block($network);
                }
            )*
        }
    }

    chain_hash_genesis_block! {
        mainnet_chain_hash_genesis_block, Network::Bitcoin;
        testnet_chain_hash_genesis_block, Network::Testnet;
        signet_chain_hash_genesis_block, Network::Signet;
        regtest_chain_hash_genesis_block, Network::Regtest;
    }

    // Test vector taken from: https://github.com/lightning/bolts/blob/master/00-introduction.md
    #[test]
    fn mainnet_chain_hash_test_vector() {
        let got = ChainHash::using_genesis_block(Network::Bitcoin).to_string();
        let want = "8687f5e55f9cb0e5f64a8737aec3a6735e1705565efe3918333fdb96e43f7716";
        assert_eq!(got, want);
    }
}
