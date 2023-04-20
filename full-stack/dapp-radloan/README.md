# RADLOAN

## What we will build

Faucet - A reusable faucet component to make testing things out a bit easier.

Radiswap DEX - We will create a simple DEX that we can reuse for testing purposes. We will use the Radiswap DEX to create a series of DEXs that we will use to demonstrate a real world arbitrage trade scenario.

RADLOAN - A flashloan arbitrage trading dApp utilizing a transient token to guarantee the loan funds are returned to the lender with the appropriate loan fee.

## What you will Learn
- How to create and utilize a transient token in a real world use case.
- How to Build a complete application on the Radix network
- The basic mechanics of an AMM(Automated Market Maker) DEX(Decentralized Exchange)
- The basic mechanics of a Flash Loan
- How to use a Flash Loan to make a profit on a series of DEX arbitrage trades.

Flash loans and arbitrage trades are two popular concepts in the world of decentralized finance (DeFi). They both involve the use of smart contracts and allow users to make profits quickly, but they work in slightly different ways. In this article, we'll explain what flash loans and arbitrage trades are, and how they can be used together.

### What are Flash Loans?

Flash loans are a type of uncollateralized loan that allows users to borrow a large amount of funds without putting up any collateral. These loans were first made possible by the smart contract technology on the Ethereum blockchain. Since they are uncollateralized, flash loans must be repaid within a single transaction. If the loan is not repaid, the transaction is automatically reversed and the loan is cancelled. On many networks this makes flash loans very risky, since any error or failure in the transaction could result in the user losing all their funds. However, this risk can be mitigated by using Scrypto and the Transaction Manifest on Radix DLT Network to build safe flash loans for arbitrage trades, which we'll discuss in more detail below.

The main advantage of flash loans is that they allow users to access a large amount of funds without having to put up any collateral. This makes it possible to execute complex trades and transactions quickly, without having to wait for funds to be transferred or loans to be approved. However, flash loans are also very risky, since they must be repaid within a single transaction. If the transaction fails for any reason, the loan is cancelled and the user is left with nothing. This means that flash loans should only be used by experienced traders who understand the risks involved. 

With the right tools and knowledge, flash loans can be a powerful tool for making quick profits in the world of DeFi. The atomic composability of Radix DLT Network makes it possible to build flash loans for arbitrage trades without the inherent risks that builders face on networks like Ethereum. The transaction layer of the Radix DLT Network built with Scrypto, allows users to build smart contracts that can be executed atomically. 

This means that if any part of the transaction fails, the entire transaction is cancelled and the user has lost nothing short of a small transaction fee. This makes it possible to build flash loans that are much safer than those on Ethereum, since the user can't lose any funds if the transaction fails.

### What is Arbitrage Trading?

Arbitrage trading is a strategy that involves buying and selling the same asset on different markets in order to take advantage of price discrepancies. For example, if Bitcoin is trading for $10,000 on one exchange and $9,800 on another, an arbitrage trader can buy Bitcoin on the second exchange and immediately sell it on the first exchange, making a profit of $200 per Bitcoin.

Arbitrage trading is a popular strategy in the world of traditional finance, and it's also becoming more common in the world of DeFi. Since many DeFi protocols are decentralized and operate independently, there can often be price discrepancies between different markets. This creates opportunities for arbitrage traders to buy low on one market and sell high on another, making a profit in the process.

### Using Flash Loans for Arbitrage Trading

Flash loans can be a powerful tool for arbitrage traders. Since flash loans allow users to borrow large amounts of funds without collateral, they can be used to quickly execute arbitrage trades across multiple markets. For example, an arbitrage trader could use a flash loan to buy Bitcoin on one exchange, then sell it on another exchange where the price is higher, making a profit in the process.

Conclusion

Flash loans and arbitrage trading are two powerful tools that can be used together to make quick profits in the world of DeFi. However, it's important to remember that both of these tools are very risky, and should only be used by experienced traders who understand the risks involved. If you're interested in using flash loans or arbitrage trading, make sure to do your research and understand the risks before diving in.

The Radix tech stack provides the tools and framework needed to build flash loans for arbitrage trades on the Radix DLT Network in a way that is safe and predictable. 

## Building the Faucet
We will build a reusable faucet component to make testing things out a bit easier. The faucet will be a simple component that will allow us to mint tokens for testing purposes. We will use the faucet to mint tokens for the Radiswap DEX and the Radloan DApp.

## Building the Radiswap DEX
For a complete walkthough of the Radiswap DEX, please see the Radix Academy Scrypto 101 [Radiswap DEX Walkthrough](https://academy.radixdlt.com/path-player?courseid=scrypto101&unit=scrypto101_1674667994844_0Unit)

## Building the Radloan DApp
We will be building a flashloan arbitrage trading dApp. The dApp will allow users to take out a flashloan and use it to make a series of arbitrage trades on the Radiswap DEXs which we will create 2 instances of in order to demonstrate a real world scenario.
