// Intended use:
// 1. Create an empty struct with the name of your Stage.
// 2. Implement `StageType` trait on your struct.
// 3. Create a `Stage` struct using Stage::new()
// 4. Use `verify_box()` to create verified `StageBox<T:StageType>`s. These represent boxes that are guaranteed to valid boxes at a given stage, and thus can be used for performing Actions without any further checks.
// 5. Write functions that represent Actions in your protocol using `StageBox<t>`s for the inputs and output types to guarantee that your Action(state transition) logic is valid.

use crate::predicated_boxes::StageBox;
pub use ergo_lib::ast::Constant;
use ergo_lib::chain::address::{Address, AddressEncoder, NetworkPrefix};
pub use ergo_lib::chain::ergo_box::ErgoBox;
pub use ergo_lib::chain::token::TokenAmount;
use ergo_lib::serialization::serializable::SigmaSerializable;
use ergo_offchain_utilities::P2SAddressString;
use std::collections::HashMap;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BoxVerificationError>;

#[derive(Error, Debug)]
pub enum BoxVerificationError {
    #[error("The P2S address of the box does not match the `StageChecker` P2S address.")]
    InvalidP2SAddress,
    #[error("The number of Ergs held within the box is invalid: {0}")]
    InvalidErgsValue(String),
    #[error("The provided `ErgoBox` did not pass the verification predicate because of a problem with the tokens held in the box: {0}")]
    InvalidTokens(String),
    #[error("The provided `ErgoBox` did not pass the verification predicate because of a problem with the values within the registers of the box: {0}")]
    InvalidRegisters(String),
    #[error("{0}")]
    OtherError(String),
}

/// A trait for defining the datatype (effectively the name
/// on the type level) of your `Stage` in your off-chain code.
pub trait StageType {
    /// Create a new `StageType`
    fn new() -> Self;
}

// A struct which represents a `Stage` in a
// multi-stage smart contract protocol. This struct defines all of the key
// essentials and thus provides an interface for performing
// validation checks that a given `ErgoBox` is indeed at said stage.
#[derive(Clone)]
pub struct Stage<ST: StageType> {
    /// Hardcoded values within the `Stage` contract
    pub hardcoded_values: HashMap<String, Constant>,
    /// The P2S Address of the `Stage` as a base58 `String`
    pub p2s_address: P2SAddressString,
    /// A predicate that an `ErgoBox` must pass in order to be classified
    /// as being at the current `Stage`. This predicate can check
    /// any data within the ErgoBox matches given requirements.
    pub verification_predicate: fn(&ErgoBox) -> Result<()>,
    /// The `Stage` data type that this `StageChecker` is created for.
    /// Only used for carrying the type forward into this struct and
    /// for any `StageBox<T>`s created.
    stage_type: ST,
}

impl<ST: StageType> Stage<ST> {
    /// Create a new Stage<ST>
    pub fn new(
        hardcoded_values: HashMap<String, Constant>,
        p2s_address: &P2SAddressString,
        verification_predicate: fn(&ErgoBox) -> Result<()>)
        -> Stage<ST> {
            Stage {
                hardcoded_values: hardcoded_values,
                p2s_address: p2s_address.clone(),
                verification_predicate: verification_predicate,
                stage_type: ST::new(),
            }
    }

    /// Verify that a provided `ErgoBox` is indeed at the given `StageChecker`.
    /// In other words, check that the box is at the right P2S address,
    /// holds Ergs within the correct range, hold tokens which succeed
    /// all provided predicates, and has values in its registers which
    /// pass all of the register predicates.
    pub fn verify_box(&self, b: &ErgoBox) -> Result<StageBox<ST>> {
        // Verify box P2S Address
        let address = Address::P2S(b.ergo_tree.sigma_serialise_bytes());
        let encoder = AddressEncoder::new(NetworkPrefix::Mainnet);
        match self.p2s_address == encoder.address_to_str(&address) {
            true => Ok(()),
            false => Err(BoxVerificationError::InvalidP2SAddress),
        }?;

        // Apply verification predicate to the `ErgoBox`. If it returns
        // an error, then the `?` will prevent the function from proceeding
        (self.verification_predicate)(b)?;
        let stage_box = StageBox {
            stage: ST::new(),
            predicate: self.verification_predicate,
            ergo_box: b.clone(),
        };

        Ok(stage_box)
    }
}