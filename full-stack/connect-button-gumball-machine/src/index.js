// import {
//   configure,
//   requestBuilder,
//   requestItem,
//   ManifestBuilder,
//   Decimal,
//   Bucket,
//   Expression
// } from '@radixdlt/connect-button'
import { RadixDappToolkit } from '@radixdlt/radix-dapp-toolkit'
const dAppId = 'radixdlt.dashboard.com'
const rdt = RadixDappToolkit(
  { dAppDefinitionAddress: dAppId, dAppName: 'GumballMachine' },
  // 'radixdlt.dashboard.com',
  (requestData) => {
    console.log('requesting account data')
    requestData({
      accounts: { quantifier: 'atLeast', quantity: 1 },
    }).map(({ data: { accounts } }) => {
      // add accounts to dApp application state
      console.log("account data: ", accounts)
      // setState({ connected: true, loading: false })
      // document.getElementById('accountAddress').innerText = data.accounts
      // accountAddress = data.accounts
    })
  },
  { networkId: 34 }
)
console.log("dApp Toolkit: ", rdt)

// // Configure the connect button
// const connectBtn = configure({
//   dAppId: 'Gumball',
//   networkId: 0x0b,
//   onConnect: ({ setState, getWalletData }) => {
//     getWalletData(
//       requestBuilder(requestItem.oneTimeAccounts.withoutProofOfOwnership(1))
//     ).map(({ oneTimeAccounts }) => {
//       setState({ connected: true, loading: false })
//       document.getElementById('accountAddress').innerText = oneTimeAccounts[0].address
//       accountAddress = oneTimeAccounts[0].address
//     })
//   },
//   onDisconnect: ({ setState }) => {
//     setState({ connected: false })
//   },
// })
// console.log("connectBtn: ", connectBtn)

// There are four classes exported in the Gateway-SDK These serve as a thin wrapper around the gateway API
// API docs are available @ https://betanet-gateway.redoc.ly/
import { TransactionApi, StateApi, StatusApi, StreamApi } from "@radixdlt/babylon-gateway-api-sdk";

// Instantiate Gateway SDK
const transactionApi = new TransactionApi();
const stateApi = new StateApi();
const statusApi = new StatusApi();
const streamApi = new StreamApi();

// Global states
let accountAddress //: string // User account address
let componentAddress //: string  // GumballMachine component address
let resourceAddress //: string // GUM resource address
// You can use this packageAddress to skip the dashboard publishing step package_tdx_b_1qx5a2htahyjygp974tap7d0x7pn8lxl00muz7wjtdhxqe90wfd
// xrdAddress resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x


// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById('instantiateComponent').onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let flavor = document.getElementById("flavor").value;

  let manifest = new ManifestBuilder()
    .callFunction(packageAddress, "GumballMachine", "instantiate_gumball_machine", [Decimal("10"), `"${flavor}"`])
    .build()
    .toString();
  console.log("Instantiate Manifest: ", manifest)
  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Intantiate WalletSDK Result: ", result.value)

  // ************ Fetch the transaction status from the Gateway API ************
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Instantiate TransactionApi transaction/status:', status)

  // ************* fetch component address from gateway api and set componentAddress variable **************
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: 'intent_hash',
        value_hex: result.value.transactionIntentHash
      }
    }
  })
  console.log('Instantiate Committed Details Receipt', commitReceipt)

  // ****** set componentAddress and resourceAddress variables with gateway api commitReciept payload ******
  // componentAddress = commitReceipt.details.receipt.state_updates.new_global_entities[0].global_address <- long way -- shorter way below ->
  componentAddress = commitReceipt.details.referenced_global_entities[0]
  document.getElementById('componentAddress').innerText = componentAddress;

  resourceAddress = commitReceipt.details.referenced_global_entities[1]
  document.getElementById('gumAddress').innerText = resourceAddress;
}

document.getElementById('buyGumball').onclick = async function () {

  let manifest = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountAddress, 10, "resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x")
    .takeFromWorktopByAmount(10, "resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x", "xrd_bucket")
    .callMethod(componentAddress, "buy_gumball", [Bucket("xrd_bucket")])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString();

  console.log('buy_gumball manifest: ', manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Buy Gumball getMethods Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Buy Gumball TransactionAPI transaction/status: ', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: 'intent_hash',
        value_hex: result.value.transactionIntentHash
      }
    }
  })
  console.log('Buy Gumball Committed Details Receipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('receipt').innerText = JSON.stringify(commitReceipt.details.receipt, null, 2);
};
