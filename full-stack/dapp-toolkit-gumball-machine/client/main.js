import {
  RadixDappToolkit,
  ManifestBuilder,
  Decimal,
  Bucket,
  Expression,
  Address
} from '@radixdlt/radix-dapp-toolkit'
const dAppId = 'account_tdx_c_1p9r7e09q9q3jcp98f0hjgmm2wjhc68gxxcc55egf9saqhzw9kl'

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
  {
    networkId: 12, // 12 is for RCnet 01 for Mainnet
    onDisconnect: () => {
      // clear your application state
    },
    onInit: ({ accounts }) => {
      // set your initial application state
      console.log("onInit accounts: ", accounts)
      if (accounts.length > 0) {
        document.getElementById('accountName').innerText = accounts[0].label
        document.getElementById('accountAddress').innerText = accounts[0].address
        accountAddress = accounts[0].address
      }
    },
  }
)
console.log("dApp Toolkit: ", rdt)


// There are four classes exported in the Gateway-SDK These serve as a thin wrapper around the gateway API
// API docs are available @ https://betanet-gateway.redoc.ly/
// https://kisharnet-gateway.radixdlt.com/swagger/index.html
import { TransactionApi, StateApi, StatusApi, StreamApi, Configuration } from "@radixdlt/babylon-gateway-api-sdk";

// Instantiate Gateway SDK
const transactionApi = new TransactionApi()
const stateApi = new StateApi();
const statusApi = new StatusApi();
const streamApi = new StreamApi();

// Global states
let accountAddress // User account address
let componentAddress = "component_tdx_c_1q03fknuu5g60rmu95xchgwzn7yaexexq5kclkqeesk3smdcnlk" //GumballMachine component address
let resourceAddress // GUM resource address
let xrdAddress = "resource_tdx_c_1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40v2wv"
let admin_badge = "resource_tdx_c_1q83fknuu5g60rmu95xchgwzn7yaexexq5kclkqeesk3s3v2a6d"
// You can use these addresses to skip steps
// package_tdx_c_1qpj4des34yyxv8fqgf8az8rec2np6fycw69e2tljep0qpuzt9m
// LAUNCH Gumball MAchine = component_tdx_c_1q03fknuu5g60rmu95xchgwzn7yaexexq5kclkqeesk3smdcnlk
// LAUNCH admin_badge = resource_tdx_c_1q83fknuu5g60rmu95xchgwzn7yaexexq5kclkqeesk3s3v2a6d

// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById('instantiateComponent').onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let flavor = document.getElementById("flavor").value;

  let manifest = new ManifestBuilder()
    .callFunction(packageAddress, "GumballMachine", "instantiate_gumball_machine", [Decimal("1"), `"${flavor}"`])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
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

  // ************ Fetch component address from gateway api and set componentAddress variable **************
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('Instantiate Committed Details Receipt', commitReceipt)

  // ****** Set componentAddress variable with gateway api commitReciept payload ******
  componentAddress = commitReceipt.details.referenced_global_entities[0]
  document.getElementById('componentAddress').innerText = componentAddress;
  // ****** Set resourceAddress variable with gateway api commitReciept payload ******
  admin_badge = commitReceipt.details.referenced_global_entities[1]
  document.getElementById('gumAddress').innerText = admin_badge;
}
// *********** Buy Gumball ***********
document.getElementById('buyGumball').onclick = async function () {
  let manifest = new ManifestBuilder()
    .callMethod(accountAddress, "withdraw", [Address(xrdAddress), Decimal("33")])
    .takeFromWorktop(xrdAddress, "xrd_bucket")
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

  console.log("Buy Gumball sendTransaction Result: ", result)

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
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('Buy Gumball Committed Details Receipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('receipt').innerText = JSON.stringify(commitReceipt.details.receipt, null, 2);
};
// *********** Get Price ***********
document.getElementById('getPrice').onclick = async function () {
  let manifest = new ManifestBuilder()
    .callMethod(componentAddress, "get_price", [])
    .build()
    .toString()
  console.log("get price manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("get_price sendTransaction Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Get Price status', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('get price commitReceipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('price').innerText = JSON.stringify(commitReceipt.details.receipt.output[1].data_json.value);

}
// *********** Set Price ***********
document.getElementById('setPrice').onclick = async function () {
  let newPrice = document.getElementById('newPrice').value
  let manifest = new ManifestBuilder()
    .callMethod(accountAddress, "create_proof", [Address(admin_badge)])
    .callMethod(componentAddress, "set_price", [Decimal(newPrice)])
    .build()
    .toString()
  console.log("set price manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Set Price sendTransaction Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Set Price status', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('Set price commitReceipt', commitReceipt)

  // Show the receipt on the DOM .data_struct.struct_data.data_json.fields[2].value
  document.getElementById('price').innerText = JSON.stringify(commitReceipt.details.receipt.state_updates.updated_substates[0].substate_data.data_struct.struct_data.data_json.fields[2].value);

}
// *********** Withdraw Earnings ***********
document.getElementById('withdrawEarnings').onclick = async function () {
  let manifest = new ManifestBuilder()
    .callMethod(accountAddress, "create_proof", [Address(admin_badge)])
    .callMethod(componentAddress, "withdraw_earnings", [])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString()
  console.log("Withdraw Earnings manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Withdraw Earnings sendTransaction Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Withdraw Earnings status', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('Withdraw Earnings commitReceipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('withdraw').innerText = JSON.stringify(commitReceipt.details.receipt);

}
// *********** Mint NFT Staff Badge ***********
document.getElementById('mintStaffBadge').onclick = async function () {
  let manifest = new ManifestBuilder()
    .callMethod(accountAddress, "create_proof", [Address(admin_badge)])
    .callMethod(componentAddress, "mint_staff_badge", [`"${"Number 2"}"`])
    .callMethod(accountAddress, "deposit_batch", [Expression("ENTIRE_WORKTOP")])
    .build()
    .toString()
  console.log("mintStaffBadge manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("mintStaffBadge sendTransaction Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('mintStaffBadge status', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('mintStaffBadge commitReceipt', commitReceipt)

  // Show the receipt on the DOM
  document.getElementById('staffBadge').innerText = JSON.stringify(commitReceipt.details.receipt);

}