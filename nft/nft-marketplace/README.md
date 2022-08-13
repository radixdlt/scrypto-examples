# NFT Marketplace

This package includes a number of blueprints that implement functionality typically seen in NFT marketplaces such as [OpenSea](https://opensea.io/), [Rarible](https://rarible.com/), and so on. The blueprints implement multiple types of sales such as: a fixed price sale, Dutch auction, and English auction. The aim of this package is to showcase to developers how functionality similar to that seen in popular NFT marketplaces can be implemented with Scrypto on Radix.

## Package Features:

* Supports multiple types of sales.
* Allows NFT sales to take place through any token the seller specifies (does not have to be XRD).
* NFTs can be sold individually or as a bundle if the seller wants to sell them all together.
* No central contract/component which makes this more efficient and scalable.

## Design Considerations

![](./images/comparison.png)

This package is designed in a way that showcases the power of Radix's concept of components and their importance for the scalability of the network and how monolithic application from other blockchains can be made into multiple smaller components and blueprints in Radix. 

Consider the Ethereum-style NFT marketplace shown in the diagram above, most NFT marketplaces in other blockchains and ledgers have a central contract which all sellers and buyers need to talk to in order to perform the action that they wish to make. This approach to building a marketplace has a many problems and issues associated with it:

1. From a scalability point of view, this is terrible as it means that there is one contract which all interactions with the marketplace go through, thus, it's very easy to see that this single contract becomes a bottleneck rather quickly as the marketplace would only be able to process a single purchase at one.

1. In most cases, this quickly becomes a monolithic contract which implements all types of sales, handles multiple types of sales, and handles millions of dollars of funds all in a single contract. Monolithic contracts are more prone to bugs as they're harder to develop, harder to reason about, and capturing all of the edge cases of a monolithic system is difficult so writing unit tests for the edge cases will be far from easy. Due to the aforementioned reasons, adding features to a monolithic contract is is not easy which explains why a lot of NFT marketplaces do not support multi-token sales or NFT bundles. 

Once code gets very big, it becomes no different from a large ship. Its difficult to steer and to change it course, and the smallest of mistakes could lead to the largest of catastrophes. 

Therefore, there does exist a need for NFT marketplaces to be re-imagined and built in a different way which removes the old monolithic style to handling marketplaces, and introduces a new style to how such applications can be built, maintained, and used. 

The system implemented in this package is very different from what you might expect to see in other blockchains. Due to the reasons listed above, this NFT marketplace does not have a single blueprint or component which defines the marketplace. Instead, this package comes with multiple modular blueprints which represent the types of sales that are possible to perform in the marketplace. When a seller wishes to sell their NFTs, they can simply instantiate a new component from the blueprint representing the type of sale they wish to have, and this would be it!

This means, the marketplace exists as a collection of components belonging to a collection of blueprints which all exist on the ledger with no on-ledger component linking them. To realize the advantage of this approach, consider the two diagrams shown above which showcase how an NFT marketplace can be implemented on Ethereum and on Radix. As has been mentioned before, the Ethereum style marketplace has all of the buyers calling a single contract which does not make sense from a scalability point of view. On the other hand, the Radix approach is to allow for multiple components to exist on ledger for the multiple sales that are going on at any one time. Buyers only call the component which holds the NFTs that they're interested in instead of calling a global contract which then routes or performs the sale itself. This approach makes even more sense when you consider how the network will work when Xi'an comes where component will live in a shard, if there existed a single component which all sales had to be piped through, then this would create bottlenecks as there would exist one shard which all sales rely on and need to queue for, which creates slowdowns in sales. However, this approach removes the bottleneck and allows for parallel sales as each sale would practically happen in its own shard irrelevant of any information from other shards (not including the one which the account lives in)

Despite that, there does exist a need for something to "link" all of these components together. At the end of the day, how else do the plan on showing them on a website or user interface when there is nothing to link all of them together? This need does indeed exist, however, this burden can be offloaded to off-chain solutions which would handle the aggregation of newly created components and create a database of currently active sales, thus, reducing the stress on the on-chain components.

## Demonstration

### Fixed Price Sales

As the name suggests, a fixed price sale is a sale which happens at a fixed price determined by the seller. The price of the NFT(s) does not change throughout the period of the sale unless the seller explicitly orders the price to be changed. When making the sale, the seller is free to choose which token they wish to sell their NFTs for. As an example, they can choose to sell them for 100 XRD, 100 CERB, or perhaps 100 USDT, all of this is in the hands of the seller to choose when they're creating a new fixed price sale. 

Lets go through the process of how a fixed price sale can be setup by the seller and what the buyer needs to do in order to purchase the tokens. First of all, the `bootstrap.rs` module contains a blueprint which creates a number of test NFTs for us which we will need to use for the purposes of testing. So, right after creating the accounts, publishing the package, etc... we will need to call a specific function on this blueprint to get it to create some NFTs for us.

First, lets wipe out resim's database and clear it so that we get a clean ledger to work with. 

```sh
$ resim reset
```

For the fixed price sale, we will need to have two accounts, a seller and a buyer. So, we will create two accounts through resim

```sh
$ resim new-account # This is the seller's account
$ resim new-account # This is the buyer's account
```

With the two accounts created, we are now ready to publish the package to resim

```sh
$ resim publish . 
```

We are now ready to create the test NFTs that we will be using for this sale. These NFTs will be created by calling the `bootstrap` method on the `Bootstrap` blueprint in this package. This call will create three resources which we will be naming `cars_nft`, `phones_nft`, and `laptops_nft` respectively.

```sh
$ resim call-function $package Bootstrap bootstrap
```

We are now ready to begin selling our tokens! The previous function call crated for us three car NFTs, three phone NFTs, and three laptop NFTs. Say we wish to sell three of the car NFTs and one of the phone NFTs for a price of 1000 XRD for the full bundle.

```sh
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"3\") ResourceAddress(\"$cars_nft\"); 
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"1\") ResourceAddress(\"$phones_nft\");

TAKE_FROM_WORKTOP ResourceAddress(\"$cars_nft\") Bucket(\"bucket1\");
TAKE_FROM_WORKTOP ResourceAddress(\"$phones_nft\") Bucket(\"bucket2\");

CALL_FUNCTION 
    PackageAddress(\"$package\") 
    \"FixedPriceSale\" 
    \"instantiate_fixed_price_sale\" 
    Vec<Bucket>(Bucket(\"bucket1\"), Bucket(\"bucket2\"))
    ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\")
    Decimal(\"1000\");

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/fixed_price_sale_listing.rtm
$ resim run transactions/fixed_price_sale_listing.rtm
```

| Note   | Make sure that the transaction manifest generated above does contain the required addresses. If it does not, then please setup the required environment variables with the same names shown above and then try the above commands again. |
| ------ | :----- |

When we run the above transaction manifest, we are creating a new `FixedPriceSale` component and depositing our NFTs into it. In exchange for our NFTs, the component provides us with an ownership badge which is used to denote that we are the owners of these NFTs and that only we have the right to cancel this sale and withdraw the NFTs or withdraw the funds obtained from the sale. 

Now that the sale component has been created, we may act as the buyer and attempt to buy these NFTs through this component

```sh
$ resim set-default-account $buyer_account $buyer_private_key
$ resim call-method $component "buy" 1000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag
```

Viola, and we're done! Lets now check the balances of the buyer's account to make sure that they've received the NFTs that they bought

```sh
$ resim show $buyer_account
Resources:
├─ { amount: 999000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD" }
├─ { amount: 1, resource address: 03dc4bf94e5d9106f122712ac4b7b8a689d32d90205860f071b8e7, name: "Phones NFT", symbol: "PHONE" }
│  └─ NonFungible { id: a5d44d1a68623fdf499c1231f93fc501, immutable_data: Struct("Pixel", "Google"), mutable_data: Struct() }
└─ { amount: 3, resource address: 03fef7f50bb44ee87f94534e2f8d493e9275b3aa8141cae9151b19, name: "Cars NFT", symbol: "FAST" }
   ├─ NonFungible { id: 2eb1146fc019fd3b4afaff595fecd655, immutable_data: Struct("Raptor", "Ford"), mutable_data: Struct() }
   ├─ NonFungible { id: 47eddbc2f7c3519b7eb5aaa17629773c, immutable_data: Struct("Yukon", "GMC"), mutable_data: Struct() }
   └─ NonFungible { id: 75afab970ce713fa0b34be870c1b1a17, immutable_data: Struct("Altima", "Nissan"), mutable_data: Struct() }
```

We can now switch back to the seller's account and withdraw the amount that the buyer paid for the NFTs.

```sh
$ resim set-default-account $seller_account $seller_private_key
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"create_proof\" ResourceAddress(\"$ownership_badge\");
CALL_METHOD ComponentAddress(\"$component\") \"withdraw_payment\";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/fixed_price_sale_withdraw_payment.rtm
$ resim run transactions/fixed_price_sale_withdraw_payment.rtm
$ resim show $seller_account
Resources:
├─ { amount: 1001000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD" }
```

### Dutch Auction

The idea behind Dutch auctions is simple: you list your NFTs for a given starting price, the price of your NFTs decreases until it reaches a specified minimum at a specified epoch. This process gives a change to interested buyers to buy your NFT(s) at the price that they believe to be reasonable for what you are selling. 

As an example, say I wish to sell an NFT in a Dutch auction, the following are the details of the dutch auction we wish to hold:

| | | 
| ---- | ----- |
| Starting Price | 1,000 XRD | 
| Minimum Price | 500 XRD |  
| Period | 50 Epochs |  

Looking at the above table of details, this Dutch auction will begin at 1,000 XRD and then over a period of 50 epochs, the price will drop to a minimum of 500 XRD, meaning that the price would drop by 10 XRD per epoch. Thus, if a buyer decides to buy this NFT 25 epochs after the beginning of the auction, then they would buy it for 750 XRD.

Lets begin setting up a Dutch auction with the details described above. For this auction, we will be auctioning off the same NFTs as the pervious demo: 3 Car NFTs and 1 Phone NFTs.

First, lets wipe out resim's database and clear it so that we get a clean ledger to work with. 

```sh
$ resim reset
```

For the dutch auction, we will need to have two accounts, a seller and a buyer. So, we will create two accounts through resim

```sh
$ resim new-account # This is the seller's account
$ resim new-account # This is the buyer's account
```

With the two accounts created, we are now ready to publish the package to resim

```sh
$ resim publish . 
```

We are now ready to create the test NFTs that we will be using for this sale. These NFTs will be created by calling the `bootstrap` method on the `Bootstrap` blueprint in this package. This call will create three resources which we will be naming `cars_nft`, `phones_nft`, and `laptops_nft` respectively.

```sh
$ resim call-function $package Bootstrap bootstrap
```

We are now ready to begin selling our tokens! The previous function call crated for us three car NFTs, three phone NFTs, and three laptop NFTs. Say we wish to sell three of the car NFTs and one of the phone NFTs for a price of 1000 XRD for the full bundle.

```sh
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"3\") ResourceAddress(\"$cars_nft\"); 
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"1\") ResourceAddress(\"$phones_nft\");

TAKE_FROM_WORKTOP ResourceAddress(\"$cars_nft\") Bucket(\"bucket1\");
TAKE_FROM_WORKTOP ResourceAddress(\"$phones_nft\") Bucket(\"bucket2\");

CALL_FUNCTION 
    PackageAddress(\"$package\") 
    \"DutchAuction\" 
    \"instantiate_dutch_auction\" 
    Vec<Bucket>(Bucket(\"bucket1\"), Bucket(\"bucket2\"))
    ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\")
    Decimal(\"1000\")
    Decimal(\"500\")
    50u64;

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/dutch_auction_listing.rtm
$ resim run transactions/dutch_auction_listing.rtm
```

| Note   | Make sure that the transaction manifest generated above does contain the required addresses. If it does not, then please setup the required environment variables with the same names shown above and then try the above commands again. |
| ------ | :----- |

Now that the sale component has been created, we may act as the buyer and attempt to buy these NFTs through this component

```sh
$ resim set-default-account $buyer_account $buyer_private_key
$ resim call-method $component "buy" 750,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag
```

Viola, and we're done! Lets now check the balances of the buyer's account to make sure that they've received the NFTs that they bought

```sh
$ resim show $buyer_account
Resources:
├─ { amount: 999000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD" }
├─ { amount: 1, resource address: 03dc4bf94e5d9106f122712ac4b7b8a689d32d90205860f071b8e7, name: "Phones NFT", symbol: "PHONE" }
│  └─ NonFungible { id: a5d44d1a68623fdf499c1231f93fc501, immutable_data: Struct("Pixel", "Google"), mutable_data: Struct() }
└─ { amount: 3, resource address: 03fef7f50bb44ee87f94534e2f8d493e9275b3aa8141cae9151b19, name: "Cars NFT", symbol: "FAST" }
   ├─ NonFungible { id: 2eb1146fc019fd3b4afaff595fecd655, immutable_data: Struct("Raptor", "Ford"), mutable_data: Struct() }
   ├─ NonFungible { id: 47eddbc2f7c3519b7eb5aaa17629773c, immutable_data: Struct("Yukon", "GMC"), mutable_data: Struct() }
   └─ NonFungible { id: 75afab970ce713fa0b34be870c1b1a17, immutable_data: Struct("Altima", "Nissan"), mutable_data: Struct() }
```

We can now switch back to the seller's account and withdraw the amount that the buyer paid for the NFTs.

```sh
$ resim set-default-account $seller_account $seller_private_key
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"create_proof\" ResourceAddress(\"$ownership_badge\");
CALL_METHOD ComponentAddress(\"$component\") \"withdraw_payment\";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/dutch_auction_withdraw_payment.rtm
$ resim run transactions/dutch_auction_withdraw_payment.rtm
$ resim show $seller_account
Resources:
├─ { amount: 1000750, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD" }
```

### English Auction

The English Auction is a very interesting type of sale and is typically the type of auction that comes to mind when you hear the word "auction". English auctions are simple: a seller puts NFTs up for sale, bidders bid according to how much they're willing to pay for the NFTs, then at the end of the auction's period, the bidder with the highest bid wins the auction. Quite straightforward.

This section of the document demonstrates how to deploy the required components required for an English auction as well as how bidders can make bids in the auction.

Lets begin with the stuff that we always need to do, lets wipe out resim's database and clear it so that we get a clean ledger to work with. 

```sh
$ resim reset
```

For this auction, we will need to have multiple different accounts. One will be used for the seller and the other three will be the bidders

```sh
$ resim new-account # This is the seller's account
$ resim new-account # This is the bidder1 account
$ resim new-account # This is the bidder2 account
$ resim new-account # This is the bidder3 account
```

With the accounts created, we are now ready to publish the package to resim

```sh
$ resim publish . 
```

We are now ready to create the test NFTs that we will be using for this sale. These NFTs will be created by calling the `bootstrap` method on the `Bootstrap` blueprint in this package. This call will create three resources which we will be naming `cars_nft`, `phones_nft`, and `laptops_nft` respectively.

```sh
$ resim call-function $package Bootstrap bootstrap
```

We are now ready to put the NFTs up for auction! Putting the NFTs for auction is a simple process, all that we need to do is to instantiate a new EnglishAuction component with the parameters of the auction. For this demo, we will auction the same NFTs as the two previous demos, which are 3 Car NFTs and 1 Phone NFT. We wish for the auction to go off for period of 50 epochs before ending and the winner being selected. 

The following commands creates the transaction manifest required for the above described auction to be made:

```sh
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"3\") ResourceAddress(\"$cars_nft\");
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"1\") ResourceAddress(\"$phones_nft\");

TAKE_FROM_WORKTOP ResourceAddress(\"$cars_nft\") Bucket(\"bucket1\");
TAKE_FROM_WORKTOP ResourceAddress(\"$phones_nft\") Bucket(\"bucket2\");

CALL_FUNCTION 
    PackageAddress(\"$package\") 
    \"EnglishAuction\" 
    \"instantiate_english_auction\" 
    Vec<Bucket>(Bucket(\"bucket1\"), Bucket(\"bucket2\"))
    ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\")
    50u64;

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/english_auction_listing.rtm
$ resim run transactions/english_auction_listing.rtm
```

When you run the above transaction, make sure that you save the component address, the first resource address as `$ownership_badge` and the last resource address as `$bidders_badge`.

We have now begun the auction for the four NFTs. Lets now switch to different accounts and make bids for the NFTs

```sh
$ resim set-default-account $bidder1_account $bidder1_private_key
$ resim call-method $component "bid" 1000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

$ resim set-default-account $bidder2_account $bidder2_private_key
$ resim call-method $component "bid" 2000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

$ resim set-default-account $bidder3_account $bidder3_private_key
$ resim call-method $component "bid" 3000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag
```

With three bids now in the auction, we can now attempt to claim the NFTs being auctioned off by the account which submitted the largest bid

```sh
$ resim set-current-epoch 50
$ resim call-method $component "claim_nfts" 1,$bidders_badge
$ resim show $bidder3_account
Resources:
├─ { amount: 1, resource address: 03fa50dc9ec449ea407c54d1fb8be337369bd4b8be08c663677bb6, name: "Phones NFT", symbol: "PHONE" }
│  └─ NonFungible { id: 45646da090a22a18e7e056437b77e7d9, immutable_data: Struct("Pixel", "Google"), mutable_data: Struct() }
└─ { amount: 3, resource address: 0302092b21072c871db5ddb9525db9ea751854b8c6db65d256ffb2, name: "Cars NFT", symbol: "FAST" }
   ├─ NonFungible { id: 277ad7c938f11513be10f045a4d68375, immutable_data: Struct("Raptor", "Ford"), mutable_data: Struct() }
   ├─ NonFungible { id: 51b41e362efd5b8c4346660a9701f95b, immutable_data: Struct("Yukon", "GMC"), mutable_data: Struct() }
   └─ NonFungible { id: 5c1b9063d4917362f0b2d66a0306c2d7, immutable_data: Struct("Camry", "Toyota"), mutable_data: Struct() }
```

As we can see, the account which made the largest bid in the English auction was able to claim their NFTs after enough epochs have passed. Currently the other accounts still have their bids locked in the component, they can cancel their bids by calling the `cancel_bid` method on the English auction component. 

```sh
$ resim set-default-account $bidder1_account $bidder1_private_key
$ resim call-method $component "cancel_bid" 1,$bidders_badge

$ resim set-default-account $bidder2_account $bidder2_private_key
$ resim call-method $component "cancel_bid" 1,$bidders_badge
```

Finally, the only thing that is left is for the seller of the NFTs to withdraw the funds that they obtained from auctioning off their NFTs, they can do that by calling the `withdraw_payment` method on the auction component with their `ownership_badge` in their auth zone.

```sh
$ echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"create_proof\" ResourceAddress(\"$ownership_badge\");
CALL_METHOD ComponentAddress(\"$component\") \"withdraw_payment\";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/english_auction_withdraw_payment.rtm
$ resim run transactions/english_auction_withdraw_payment.rtm
$ resim show $seller_account
Resources:
└─ { amount: 1003000, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag, name: "Radix", symbol: "XRD" }
```

## Shortcomings

* This package does not implement royalties on token sales. However, such functionality would not be difficult to add as it would be comprised of some metadata and a way to store the funds.