import { RadixDappToolkit, DataRequestBuilder } from '@radixdlt/radix-dapp-toolkit'
const dAppId = 'account_tdx_d_12805alyg3562gsphgeyc9re800qq0phlyz89cnu2tydmlp0gt947cw'
// Instantiate DappToolkit
const rdt = RadixDappToolkit({
  dAppDefinitionAddress: dAppId,
  networkId: 13,
})
console.log("dApp Toolkit: ", rdt)
// ************ Fetch the user's account address ************
rdt.walletApi.setRequestData(DataRequestBuilder.accounts().atLeast(1))

rdt.walletApi.walletData$.subscribe((walletData) => {
  console.log("subscription wallet data: ", walletData)
  document.getElementById('accountName').innerText = walletData.accounts[0].label
  document.getElementById('accountAddress').innerText = walletData.accounts[0].address
  accountAddress = walletData.accounts[0].address
})


// There are four classes exported in the Gateway-SDK These serve as a thin wrapper around the gateway API
// API docs are available @ https://betanet-gateway.redoc.ly/
// https://ansharnet-gateway.radixdlt.com/swagger/index.html
// Import the Gateway SDK
import { GatewayApiClient } from '@radixdlt/babylon-gateway-api-sdk'
// Instantiate Gateway SDK
const gatewayApi = GatewayApiClient.initialize({
  basePath: 'https://rcnet-v2.radixdlt.com'
})
// Destructuring the Gateway SDK classes
const { status, transaction, stream, state } = gatewayApi


// Global states
let accountAddress // User account address
let componentAddress = "component_tdx_d_1cr64u6v2uefcgtd53z4lxzaz0qfxrfujnpz0v9plx0t8eechym48an" //GumballMachine component address
let resourceAddress // GUM resource address
let xrdAddress = "resource_tdx_d_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxepwmma"
let admin_badge = "resource_tdx_d_1t52muhjfleh0txcme77xy95gc453jtfkdsvcggcvu67yzvwyuxz4am"
// You can use these addresses to skip package deployment steps
// package_tdx_d_1phr4w84xpy82kl244278ak5l33uaw0w62egrqam29aph52nywgdlr8


// ************ Instantiate component and fetch component and resource addresses *************
document.getElementById('instantiateComponent').onclick = async function () {
  let packageAddress = document.getElementById("packageAddress").value;
  let flavor = document.getElementById("flavor").value;
  let manifest = `
  CALL_FUNCTION
    Address("${packageAddress}")
    "GumballMachine"
    "instantiate_gumball_machine"
    Decimal("5")
    "${flavor}";
  CALL_METHOD
    Address("${accountAddress}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
    `
  console.log("Instantiate Manifest: ", manifest)

  // Send manifest to extension for signing
  const result = await rdt.walletApi
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })
  if (result.isErr()) throw result.error
  console.log("Intantiate WalletSDK Result: ", result.value)


  // ************ Fetch the transaction status from the Gateway API ************
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  console.log('Instantiate TransactionApi transaction/status:', transactionStatus)


  // ************ Fetch component address from gateway api and set componentAddress variable **************
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('Instantiate getCommittedDetails:', getCommitReceipt)
  // ****** Set componentAddress variable with gateway api commitReciept payload ******
  componentAddress = getCommitReceipt.transaction.affected_global_entities[5];
  // componentAddress = commitReceipt.transaction.receipt.output[1].programmatic_json.fields[0].value
  document.getElementById('componentAddress').innerText = componentAddress;
  // ****** Set resourceAddress variable with gateway api commitReciept payload ******
  admin_badge = getCommitReceipt.transaction.affected_global_entities[2];
  // admin_badge = commitReceipt.transaction.receipt.output[1].programmatic_json.fields[1].value
  document.getElementById('gumAddress').innerText = admin_badge;
}


// *********** Buy Gumball ***********
document.getElementById('buyGumball').onclick = async function () {
  let manifest = `
  CALL_METHOD
    Address("${accountAddress}")
    "withdraw"    
    Address("${xrdAddress}")
    Decimal("33");
  TAKE_ALL_FROM_WORKTOP
    Address("${xrdAddress}")
    Bucket("xrd");
  CALL_METHOD
    Address("${componentAddress}")
    "buy_gumball"
    Bucket("xrd");
  CALL_METHOD
    Address("${accountAddress}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
    `
  console.log('buy_gumball manifest: ', manifest)

  // Send manifest to extension for signing
  const result = await rdt.walletApi
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })
  if (result.isErr()) throw result.error
  console.log("Buy Gumball sendTransaction Result: ", result.value)


  // Fetch the transaction status from the Gateway SDK
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  console.log('Buy Gumball TransactionAPI transaction/status: ', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('Buy Gumball Committed Details Receipt', getCommitReceipt)

  // Show the receipt on the DOM
  document.getElementById('receipt').innerText = JSON.stringify(getCommitReceipt);
}


// *********** Get Price ***********
document.getElementById('getPrice').onclick = async function () {
  // TODO Refactor to use get state gateway api
  let getPrice = await state.getEntityDetailsVaultAggregated(componentAddress)
  console.log('getPrice', getPrice)

  // Show the receipt on the DOM
  document.getElementById('price').innerText = JSON.stringify(getPrice.details.state.programmatic_json.fields[2].value);

}


// *********** Set Price ***********
document.getElementById('setPrice').onclick = async function () {
  let newPrice = document.getElementById('newPrice').value
  // TODO Replace with String Manifest
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
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  // let status = await transactionApi.transactionStatus({
  //   transactionStatusRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // });
  console.log('Set Price status', transactionStatus)

  // fetch commit reciept from gateway api
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  // let commitReceipt = await transactionApi.transactionCommittedDetails({
  //   transactionCommittedDetailsRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // })
  console.log('Set price commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM .data_struct.struct_data.data_json.fields[2].value
  // TODO Refactor to use get state gateway api
  // document.getElementById('price').innerText = JSON.stringify(commitReceipt.details.receipt.state_updates.updated_substates[0].substate_data.data_struct.struct_data.data_json.fields[2].value);
  document.getElementById('price').innerText = JSON.stringify(getCommitReciept);

}
// *********** Withdraw Earnings ***********
document.getElementById('withdrawEarnings').onclick = async function () {
  // TODO Replace with String Manifest
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
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  // let status = await transactionApi.transactionStatus({
  //   transactionStatusRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // });
  console.log('Withdraw Earnings status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  // let commitReceipt = await transactionApi.transactionCommittedDetails({
  //   transactionCommittedDetailsRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // })
  console.log('Withdraw Earnings commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM
  // document.getElementById('withdraw').innerText = JSON.stringify(commitReceipt.details.receipt);
  document.getElementById('withdraw').innerText = JSON.stringify(getCommitReceipt);
}
// *********** Mint NFT Staff Badge ***********
document.getElementById('mintStaffBadge').onclick = async function () {
  // TODO Replace with String Manifest
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
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  // let status = await transactionApi.transactionStatus({
  //   transactionStatusRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // });
  console.log('mintStaffBadge status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  // let commitReceipt = await transactionApi.transactionCommittedDetails({
  //   transactionCommittedDetailsRequest: {
  //     intent_hash_hex: result.value.transactionIntentHash
  //   }
  // })
  console.log('mintStaffBadge commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM
  // document.getElementById('staffBadge').innerText = JSON.stringify(commitReceipt.details.receipt);
  document.getElementById('staffBadge').innerText = JSON.stringify(getCommitReceipt);
}