import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  ResourceAddress
} from '@radixdlt/radix-dapp-toolkit'


const XRD = 'resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp'

const examples = (accountA, accountB, accountC, componentAddress1, componentAddress2, gumballResourceAddress, adminBadge) => {
  const example1 = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountA, 10, XRD)
    .takeFromWorktopByAmount(10, XRD, 'xrd')
    .callMethod(componentAddress1, 'buy_gumball', [Bucket('xrd')])
    .takeFromWorktopByAmount(1, gumballResourceAddress, 'gumball')
    .callMethod(accountA, 'deposit', [Bucket('gumball')])
    .callMethod(accountB, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()

  const example2 = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountA, 0.5, XRD)
    .withdrawFromAccountByAmount(accountB, 0.5, XRD)
    .takeFromWorktopByAmount(1, XRD, 'xrd')
    .callMethod(componentAddress1, 'buy_gumball', [Bucket('xrd')])
    .callMethod(accountA, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()

  const example3 = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountA, 5, XRD)
    .withdrawFromAccountByAmount(accountA, 3, XRD)
    .takeFromWorktopByAmount(2, XRD, 'Delta')
    .takeFromWorktopByAmount(2.5, XRD, 'Echo')
    .takeFromWorktopByAmount(3.5, XRD, 'Foxtrot')
    .callMethod(componentAddress1, 'buy_gumball', [Bucket('Delta')])
    .takeFromWorktopByAmount(1, XRD, 'Golf')
    .callMethod(accountA, 'deposit_batch', [Bucket('Echo'), Bucket('Foxtrot')])
    .callMethod(accountC, 'deposit', [Bucket('Golf')])
    .callMethod(accountA, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()


  const example4 = new ManifestBuilder()
    .createProofFromAccount(accountA, adminBadge)
    .callMethod(componentAddress1, 'withdraw_funds', [])
    .callMethod(accountA, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()

  const example5 = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountB, 10, XRD)
    .createProofFromAccount(accountA, adminBadge)
    .takeFromWorktopByAmount(5, XRD, 'xrd')
    .callMethod(componentAddress1, 'buy_gumball', [Bucket('xrd')])
    .callMethod(componentAddress1, 'withdraw_funds', [])
    .callMethod(accountA, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()

  const example6 = new ManifestBuilder()
    .withdrawFromAccountByAmount(accountA, 2, XRD)
    .takeFromWorktopByAmount(2, XRD, 'xrd')
    .callMethod(componentAddress1, 'buy_gumball', [Bucket('xrd')])
    .takeFromWorktop(XRD, 'restxrd')
    .callMethod(componentAddress2, 'buy_gumball', [Bucket('restxrd')])
    .callMethod(accountB, 'deposit_batch', [Expression('ENTIRE_WORKTOP')]).build().toString()

  return {
    example1,
    example2,
    example3,
    example4,
    example5,
    example6
  }
}

const dAppId = 'account_tdx_22_1prd6gfrqj0avlyxwldgyza09fp7gn4vjmga7clhe9p2qv0qt58'

const rdt = RadixDappToolkit(
  { dAppDefinitionAddress: dAppId, dAppName: 'GumballMachine' },
  (requestData) => {
    requestData({
      accounts: { quantifier: 'atLeast', quantity: 1 },
    }).map(({ data: { accounts } }) => {
      // add accounts to dApp application state
      console.log("account data: ", accounts)
      document.getElementById('accountName').innerText = accounts[0].label
      document.getElementById('accountAddress').innerText = accounts[0].address
      accountAddress = accounts[0].address
    })
  },
  { networkId: 11 }
)
console.log("dApp Toolkit: ", rdt)


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
// You can use this packageAddress to skip the dashboard publishing step package_tdx_b_1qxtzcuyh8jmcp9khn72k0gs4fp8gjqaz9a8jsmcwmh9qhax345
let xrdAddress = "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp"

// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById('instantiateComponent').onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let flavor = document.getElementById("flavor").value;

  let manifest = new ManifestBuilder()
    .callMethod(accountAddress, "create_proof", [ResourceAddress("resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp")])
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
    .withdrawFromAccountByAmount(accountAddress, 10, "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp")
    .takeFromWorktopByAmount(10, "resource_tdx_b_1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8z96qp", "xrd_bucket")
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

const getExampleValues = () => [
  document.getElementById('account-input-a').value,
  document.getElementById('account-input-b').value,
  document.getElementById('account-input-c').value,
  document.getElementById('gumball-component-1').value,
  document.getElementById('gumball-component-2').value,
  document.getElementById('gumball-resource').value,
  document.getElementById('admin-badge').value,
]

const sendExampleTx = (number) => rdt.sendTransaction({
  transactionManifest: examples(...getExampleValues())[`example${number}`],
  version: 1,
})

document.getElementById(`example-1`).onclick = () => sendExampleTx(1)
document.getElementById(`example-2`).onclick = () => sendExampleTx(2)
document.getElementById(`example-3`).onclick = () => sendExampleTx(3)
document.getElementById(`example-4`).onclick = () => sendExampleTx(4)
document.getElementById(`example-5`).onclick = () => sendExampleTx(5)
document.getElementById(`example-6`).onclick = () => sendExampleTx(6)