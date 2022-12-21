import WalletSdk, {
  requestBuilder,
  requestItem,
  ManifestBuilder,
  ResourceAddress,
  Bucket,
  Expression,
  Decimal,
} from '@radixdlt/wallet-sdk';
// There are four classes exported in the Gateway-SDK These serve as a thin wrapper around the gateway API
// API docs are available @ https://betanet-gateway.redoc.ly/
import { TransactionApi, StateApi, StatusApi, StreamApi } from "@radixdlt/babylon-gateway-api-sdk";

const transactionApi = new TransactionApi();
const stateApi = new StateApi();
const statusApi = new StatusApi();
const streamApi = new StreamApi();

const walletSdk = WalletSdk({ dAppId: 'Gumball', networkId: 0x0b })
console.log("walletSdk: ", walletSdk)

// Global states
let accountAddress: string // User account address
let componentAddress: string  // GumballMachine component address
let resourceAddress: string // GUM resource address
// packageAddress package_tdx_b_1qywqgg8an0xz2dk87dr038sy7zu472mt349kpxe8ljqscaur8z
// xrdAddress resource_tdx_b_1qzkcyv5dwq3r6kawy6pxpvcythx8rh8ntum6ws62p95s9hhz9x

// Fetch list of account Addresses on button click
document.getElementById('fetchAccountAddress').onclick = async function () {
  // Retrieve extension user account addresses
  console.log('getting account info')
  const result = await walletSdk.request(
    // the number passed as arg is the max number of addresses you wish to fetch
    requestBuilder(requestItem.oneTimeAccounts.withoutProofOfOwnership(1))
  )

  if (result.isErr()) {
    throw result.error
  }

  const { oneTimeAccounts } = result.value
  console.log("requestItem.oneTimeAccounts.withoutProofOfOwnership(1) ", result)
  if (!oneTimeAccounts) return

  document.getElementById('accountAddress').innerText = oneTimeAccounts[0].address
  accountAddress = oneTimeAccounts[0].address
}

// Instantiate component
document.getElementById('instantiateComponent').onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let flavor = document.getElementById("flavor").value;

  let manifest = new ManifestBuilder()
    .callFunction(packageAddress, "GumballMachine", "instantiate_gumball_machine", [Decimal("10"), `"${flavor}"`])
    .build()
    .toString();
  console.log("Instantiate Manifest: ", manifest)
  // Send manifest to extension for signing
  const result = await walletSdk
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Intantiate WalletSDK Result: ", result.value)

  // Fetch the transaction status from the Gateway API
  let response = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Instantiate TransactionApi Response', response)


  // fetch component address from gateway api and set componentAddress variable 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: 'payload_hash',
        value_hex: response.known_payloads[0].payload_hash_hex
      }
    }
  })
  console.log('Instantiate Committed Details Receipt', commitReceipt)

  // fetch component address from gateway api and set componentAddress variable 
  // componentAddress = commitReceipt.details.receipt.state_updates.new_global_entities[0].global_address
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
  const result = await walletSdk
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Buy Gumball WalletSDK Result: ", result)

  // Fetch the receipt from the Gateway SDK
  let response = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Buy Gumball TransactionAPI Response', response)

  // fetch component address from gateway api and set componentAddress variable 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      transaction_identifier: {
        type: 'payload_hash',
        value_hex: response.known_payloads[0].payload_hash_hex
      }
    }
  })
  console.log('Buy Gumball Committed Details Receipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('receipt').innerText = JSON.stringify(commitReceipt.details.receipt, null, 2);
};
