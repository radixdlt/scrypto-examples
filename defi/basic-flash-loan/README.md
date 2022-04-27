# Basic Flash Loan

This example demonstrates the core function of a flash loan, implemented in an asset-oriented way, stripped down to its minimum form.

It shows how a "transient" token which can never be deposited (only burnt) can be returned to a method's caller in order to force them to later call some other method which has the authority to burn it.  The two methods need not be in the same component.

Because the Radix Engine forbids a "dangling" token from existing at the end of a transaction, by returning a transient token, you are effectively creating an obligation that some later action you desire be taken, else you won't burn the token and the transaction can not complete successfully.

A flash loan is an obvious example of this (loan the money, let the caller do whatever they like, but they must pay you back plus a fee before the transaction is done), but this pattern is also useful for operations where you have an ecosystem of components, and you wish to incentivize using them together.

For example, you might have an oracle which receives regular off-ledger price feeds and costs money to keep updated, and costs a fee to fetch price information from.  You also control some kind of token selling component.  You might set up a specialized entry point on your oracle which will return a price for free, but also give out a transient token.  The only way to burn the transient token is to then pass it to your token selling component, which will burn it as long as a purchase of high enough value is performed.  In other words, you can make your oracle free to use for people who use that price information to then make a trade on your other component.