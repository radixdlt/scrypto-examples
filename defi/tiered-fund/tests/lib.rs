use scrypto::prelude::*;

#[test]
pub fn test() {
    assert_ne!(WithdrawLimit::Finite(dec!("20")), WithdrawLimit::Infinite);

    assert!(WithdrawLimit::Infinite == WithdrawLimit::Infinite);
    assert!(WithdrawLimit::Infinite > WithdrawLimit::Finite(dec!("2")));
    assert!(WithdrawLimit::Finite(dec!("20")) > WithdrawLimit::Finite(dec!("2")));
}
