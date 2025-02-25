#![cfg(feature = "test")]

use musli::compat::Sequence;
use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct Inner;

#[derive(Debug, PartialEq, Encode, Decode)]
pub struct SequenceCompat {
    pub empty_sequence: Sequence<()>,
}

#[test]
fn sequence_compat() {
    musli::rt!(
        full,
        SequenceCompat {
            empty_sequence: Sequence(()),
        }
    );
}
