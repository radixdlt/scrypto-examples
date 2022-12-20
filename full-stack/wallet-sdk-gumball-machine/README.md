# The Gumball Machine on Betanet
This example is meant to guide you through building, deploying and using the [Gumball Machine Scrypto example](https://github.com/radixdlt/scrypto-examples/tree/main/full-stack/wallet-sdk-gumball-machine) using the [Betanet Wallet SDK](https://docs.radixdlt.com/main/wallet/wallet-sdk.html).

## Prefer to watch a video tutorial?
Follow along and build a full stack Betanet Gumball Machine Example.
[Scrypto Radix Youtube](https://www.youtube.com/@scrypto_radix)

## Pre-requisites
1. Node >= 12.17.0
2. The Betanet wallet & Radix-connector browser extenstion installed. Instructions [here](https://docs.radixdlt.com/main/getting-started-developers/wallet-and-connector.html)
3. Scrypto v0.7.0. Instructions to install [here](https://docs.radixdlt.com/main/getting-started-developers/first-component/install-scrypto.html) and update [here](https://docs.radixdlt.com/main/scrypto/getting-started/updating-scrypto.html)

## Building the Scrypto code
1. Enter the scrypto directory in a terminal: `cd scrypto`
1. Build the code: `scrypto build`
1. Two important files (`gumball_machine.abi` and `gumball_machine.wasm`) will be generated in `scrypto/target/wasm32-unknown-unknown/release/`. You will need them for the next step.

## Deploy the package to Betanet
1. Go to the [Betanet Dashboard Website](https://betanet-dashboard.radixdlt.com/)
2. Connect the Wallet Via the Connect Button
3. Choose an account and badge or have one created for you if you don't have one yet using the button below.
4. Upload both `gumball_machine.abi` and `gumball_machine.wasm`
5. Click on "publish package"
6. The wallet should open up and ask you to submit the transaction
7. On the wallet click on "sign transaction"
8. The deployed package address should get displayed. **You will need it for the next step**.

## Interacting with our package
1. In a terminal go back to the root of this project (gumball-machine-example)
2. Install the npm dependencies: `npm install`
3. Start the local server with `npm start`
4. Open up your browser at the provided url if it doesn't open automatically.
5. Make sure you created an account on the wallet extension.
6. Click on the button to fetch your wallet address. You should see your address appearing
7. Fill the package address you got in the previous section and click on "instantiate component"
8. Your wallet will again open up. Click on "submit". You should now see the instantiated component address on the page.
9. Buy a gumball by clicking on "Buy 1 GUM"
10. Your wallet will open up. Click on "submit". The transaction receipt will get displayed on the page.
11. Check the number of GUM token you have by clicking on the "check balance" button.