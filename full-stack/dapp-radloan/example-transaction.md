What happens in transactions? 

Transactions are a way to group operations together. If any operation within the transaction fails, the entire transaction fails, and the ledger state is left unchanged. Otherwise, the changes from all operations in the transaction are applied at once.

We write transaction manifest files in RTM. Here's an example of a transaction manifest:

```rs
CALL_METHOD
    Address("${account}")
    "lock_fee"
    Decimal("100");
CALL_METHOD
    Address("${component}")
    "take_loan"
    Decimal("500");
CALL_METHOD
    Address("${account}")
    "withdraw"
    Address("${xrd}")
    Decimal("0.5");
TAKE_FROM_WORKTOP_BY_AMOUNT 
    Decimal("500.5")
    Address("${xrd}")
    Bucket("xrd_bucket");
TAKE_FROM_WORKTOP
    Address("${transient_token}")
    Bucket("loan_terms");
CALL_METHOD
    Address("${component}")
    "repay_loan"
    Bucket("xrd_bucket")
    Bucket("loan_terms");
```
The manifest above is a transaction that takes a loan from a component, withdraws XRD from the account to pay the loan fee, and then repays the loan. The transaction will fail if any of the operations fail.

Transactions are executed in the order they are written in the manifest. The first operation is executed first, and the last operation is executed last. 

This simple example shows how we can compose all the operations we need to take a loan and repay it. We can also use transaction manifests to do more complex things, like perform swaps at multiple exchanges, using the borrowed funds and repay the loan with the proceeds.