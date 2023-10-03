# Radiswap Svelte on Stokenet

This example is meant to guide you through building, deploying and using the
[Radiswap Svelte Scrypto example](https://github.com/radixdlt/scrypto-examples/tree/main/full-stack/)
using the
[Radix dApp Toolkit](https://github.com/radixdlt/radix-dapp-toolkit#readme)

## Pre-requisites

1. Node >= 12.17.0
2. The Radix Wallet
   [more info here](https://docs.radixdlt.com/docs/radix-wallet-overview)
3. The Radix connector-extension with dev tools
   [download from github](https://github.com/radixdlt/connector-extension/releases)
   and load unpacked manually for interacting on localhost installed.
4. Scrypto v1.0.0. Instructions to install
   [here](https://docs.radixdlt.com/docs/getting-rust-scrypto) and update
   [here](https://docs.radixdlt.com/docs/updating-scrypto)

## Building the Scrypto code

1. Enter the scrypto directory in a terminal: `cd scrypto`
2. Build the code: `scrypto build`
3. Two important files (`radiswap.rpd` and `radiswap.wasm`) will be generated in
   `scrypto/target/wasm32-unknown-unknown/release/`. You will need them for the
   next step.

## Deploy the package to Stokenet

1. Go to the
   [Stokenet Developer Console Website](https://stokenet-console.radixdlt.com/deploy-package)
2. Connect the Wallet Via the Connect Button
3. Navigate to Deploy Package
4. Upload both `radiswap.schema` and `radiswap.wasm`
5. Make sure you created an account in the wallet and added funds via the faucet
   by clicking on;
   - the account name
   - the three dots "...",
   - "Dev Preferences",
   - the "Get XRD Test Tokens" button.
6. Choose an account to receive an owner badge that will be minted when the
   package is deployed. This account will also pay the deployment transaction
   fee.
7. Click on "Send to the Radix Wallet"
8. Go to your wallet where it should be asking you to approve the transaction
9. On the wallet "Slide to Sign" the deployment transaction.
10. Once the transaction completes, the deployed package address should then be
    displayed back in the Stokenet Console. **Add this address to
    `client/src/config.json`**. e.g.
    `"packageAddress": "package_Your_New_Package_Address"`

## Create the dApp definition on Stokenet

1. Go to the
   [Stokenet Developer Console Website](https://stokenet-console.radixdlt.com/dapp-metadata)
2. Connect the Wallet via the Connect Button
3. Navigate to Manage dApp Metadata
4. Select an account to become the Radiswap dApp definition. If you need to
   create a new account, this can be done in the wallet. One done, to access the
   newly created account in the Console, you will need to;
   - click on the connect button again,
   - then "Update Data Sharing"
   - and follow the instructions in the wallet.
5. Click on "This account is a dApp Definition" checkbox
6. Add name and description details if you wish to.
7. Click on "Send Update TransactIon to the Radix Wallet"
8. Go to your wallet where it should be asking you to approve the transaction.
   "Slide to Sign" the update transaction. You may need to change the Fee Payer
   be able the sign the transaction. To do this;
   - click "Customize"
   - then "Change"
   - select the an account with XRD
   - click "Select Account"
   - "X" in the top left corner
   - "Slide to Sign" the update transaction.
9. Once the transaction completes **Add the dApp account address to
   `client/src/config.json`**. e.g.
   `"dAppDefinitionAddress": "account_Your_New_dApp_Definition_Address"`

## Interacting with our package

1. In a terminal go back to the from root of this project
   `cd client`(dapp-toolkit-gumball-machine)
2. Install the npm dependencies: `npm install`
3. Start the local server with `npm run dev`
4. Open up your browser at the provided url if it doesn't open automatically.
5. Click on the Admin link in the Navbar to navigate to `/admin`
6. Click on the connect button.
7. Radiswap requires 2 resource addressed for it's pool. You may add your own or
   create some with the "Create Resources" button. Once you have the addresses,
   **add them to `client/src/config.json`**. e.g.
   `"poolResourceAddress1": "resource_Your_First_Pool_Resource_Address", "poolResourceAddress2": "resource_Your_Second_Pool_Resource_Address"`
8. Click on the "Instantiate" button to instantiate a Radiswap component from
   the package.
9. Your wallet will again open up. "Slide to Sign" again and You should see the
   instantiated Radiswap Component Address, Pool Address and Pool Unit resource
   address on the page.
10. Add the Radiswap Component Address and Pool Address to
    `client/src/config.json`. e.g.
    `"componentAddress": "component_Your_New_Component_Address", "poolAddress": "pool_Your_New_Pool_Address"`.
    When saving these changes you should see the Pool Balance and Pool Unit
    Balance Total Supply update on the page.
11. Navigate to the Pool page and deposit some tokens by;
    - entering the amount of the each token you want to deposit,
    - clicking on the "Deposit" button
    - then "Slide to Sign" in your wallet. You will receive Pool Units. You can
      check the balance of the pool and pool unit supply by returning to the
      Admin page.
12. Once there are tokens in the pool you can navigate to the Swap page and
    perform a swap. You can do this by;
    - entering your swap amount,
    - clicking on the "Swap" button
    - then "Slide to Sign" in your wallet.
13. To Withdraw from the pool you can;
    - return to the Pool page,
    - click "Deposit ⇄ Withdraw",
    - enter the amount Pool Units you wish to send,
    - click "Deposit"
    - and then "Slide to Sign" in your wallet.
