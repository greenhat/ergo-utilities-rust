use ergo_lib::chain::ergo_box::ErgoBox;
use ergo_lib::chain::transaction::unsigned::UnsignedTransaction;
use ergo_protocol_framework::*;

#[derive(Debug, Clone, WrapBox)]
pub struct MathBountyBox {
    ergo_box: ErgoBox,
}

impl SpecifiedBox for MathBountyBox {
    fn box_spec() -> BoxSpec {
        let address = Some("94hWSMqgxHtRNEWoKrJFGVNQEYX34zfX68FNxWr".to_string());
        BoxSpec::new(address, None, vec![], vec![])
    }
}

impl MathBountyBox {
    pub fn new(ergo_box: &ErgoBox) -> Option<MathBountyBox> {
        // Using the automatically implemented `verify_box` method
        // from the `BoxSpec` to verify the `ErgoBox` is a valid
        // `MathBountyBox`.
        Self::box_spec().verify_box(ergo_box).ok()?;

        // Creating the `MathBountyBox`
        let math_bounty_box = MathBountyBox {
            ergo_box: ergo_box.clone(),
        };

        // Returning the `MathBountyBox`
        Some(math_bounty_box)
    }
}

pub struct MathBountyProtocol {}

impl MathBountyProtocol {
    /// A bootstrap action which allows a user to create a `MathBountyBox`
    /// with funds locked inside as a bounty for solving the math problem.
    pub fn action_bootstrap_math_bounty_box(
        bounty_amount_in_nano_ergs: u64,
        ergs_box_for_bounty: ErgsBox,
        current_height: u64,
        transaction_fee: u64,
        ergs_box_for_fee: ErgsBox,
        user_address: String,
    ) -> UnsignedTransaction {
        let tx_inputs = vec![
            ergs_box_for_bounty.as_unsigned_input(),
            ergs_box_for_fee.as_unsigned_input(),
        ];

        // Calculating left over change nanoErgs
        let total_nano_ergs = ergs_box_for_bounty.nano_ergs() + ergs_box_for_fee.nano_ergs();
        let total_change = total_nano_ergs - bounty_amount_in_nano_ergs - transaction_fee;

        // Creating our Math Bounty Box output candidate
        let math_bounty_candidate = create_candidate(
            bounty_amount_in_nano_ergs,
            &"94hWSMqgxHtRNEWoKrJFGVNQEYX34zfX68FNxWr".to_string(),
            &vec![],
            &vec![],
            current_height,
        )
        .unwrap();

        // Create the Transaction Fee box candidate
        let transaction_fee_candidate =
            TxFeeBox::output_candidate(transaction_fee, current_height).unwrap();

        // Create the Change box candidate
        let change_box_candidate =
            ChangeBox::output_candidate(&vec![], total_change, &user_address, current_height)
                .unwrap();

        // Our output candidates list, specifically with the Math Bounty box
        // candidate being the first, meaning Output #0.
        let output_candidates = vec![
            math_bounty_candidate,
            transaction_fee_candidate,
            change_box_candidate,
        ];

        UnsignedTransaction::new(tx_inputs, vec![], output_candidates)
    }
}
