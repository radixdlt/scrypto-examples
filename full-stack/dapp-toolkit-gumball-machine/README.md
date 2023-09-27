# The Gumball Machine on Stokenet
This example is meant to guide you through building, deploying and using the [Gumball Machine Scrypto example](https://github.com/radixdlt/scrypto-examples/tree/main/full-stack/dapp-toolkit-gumball-machine) using the [Radix dApp Toolkit](https://github.com/radixdlt/radix-dapp-toolkit#readme)

## Pre-requisites
1. Node >= 12.17.0
2. The Radix Wallet [more info here](https://docs.radixdlt.com/docs/radix-wallet-overview)
3. The Radix connector-extension with dev tools [download from github](https://github.com/radixdlt/connector-extension/releases) and load unpacked manually for interacting on localhost installed.
4. Scrypto v1.0.0. Instructions to install [here](https://docs.radixdlt.com/docs/getting-rust-scrypto) and update [here](https://docs.radixdlt.com/docs/updating-scrypto)

## Building the Scrypto code
1. Enter the scrypto directory in a terminal: `cd scrypto`
1. Build the code: `scrypto build`
1. Two important files (`gumball_machine.schema` and `gumball_machine.wasm`) will be generated in `scrypto/target/wasm32-unknown-unknown/release/`. You will need them for the next step.

## Deploy the package to RCnet
1. Go to the [Stokenet Developer Console Website](https://stokenet-console.radixdlt.com/deploy-package)
2. Connect the Wallet Via the Connect Button
3. Navigate to Deploy Package & choose an account and badge or have one created for you if you don't have one yet using the link below. (Which appears once you have selected an account)
4. Upload both `gumball_machine.schema` and `gumball_machine.wasm`
5. Click on "publish package"
6. The wallet should open up and ask you to approve the transaction
7. On the wallet click on "sign transaction"
8. The deployed package address should get displayed. **You will need it for the next step**.

## Interacting with our package
1. In a terminal go back to the from root of this project `cd client`(dapp-toolkit-gumball-machine)
2. Install the npm dependencies: `npm install`
3. Start the local server with `npm run dev`
4. Open up your browser at the provided url if it doesn't open automatically.
5. Make sure you created an account on the wallet and added funds via the faucet by clicking on account name and then the three dots > Dev Preferences >  a button to Get XRD Test Tokens from the faucet should open.
6. Click on the connect button to fetch your wallet address. You should see your address appearing 
7. Fill the package address you got in the previous section and enter a symbol name for your gumball to display in the wallet then click on "instantiate gumball machine"
8. Your wallet will again open up. Click on "sign transaction". You should now see the instantiated component address and Gumball resource address on the page.
9. Buy a gumball by clicking on "Buy 1 GUM"
10. Your wallet will open up. Click on "sign transaction". The transaction receipt will get displayed on the page.
11. Check the number of GUM tokens you have by clicking on the account name in your wallet and viewing the tokens tab.

## Interacting with our Gumball Machine Locally
First lets start with a fresh clean ledger.
`resim reset`

Next we need to create a default account `resim new-account`

Store the account address in the account environment variable
`export account=<account_address>`

Now we can publish our package, to do this locally run
`resim publish .`

Store the returned package address in the package environment variable `export package=<package_address>`

At this point we can instantiate our Gumball Machine locally `resim run instantiate.rtm`

Store the returned component addres in the component environment variable `export component=<component_address>`

Run `resim show $account` and find the admin badge resource address and store it in the admin_badge environment variable `export admin_badge=<resource_address>` and the owner_badge environment variable `export owner_badge=<resource_address>`

Let's also set the xrd address as an environment variable `export xrd=resource_sim1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxakj8n3`

You can run `resim run get_price.rtm` to fetch the current gumball price

You can also run the set_price.rtm transaction manifest to change the price of a gumball `resim run set_price.rtm`

To mint an NFT Staff Member Badge run `resim run mint_staff_badge.rtm`

and of course you can buy a gumball by running `resim run buy_gum.rtm`

As the holder of the admin badge you can run `resim run withdraw_earnings.rtm` to collect your riches.

And when the Gumball Machine needs to be refilled either a holder of an admin_badge or staff_badge can mint more gumball tokens with the `refill_gumball_machine` method. Can you compose your own transaction manifest to make this method call?