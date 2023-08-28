import { RadixDappToolkit, DataRequestBuilder } from '@radixdlt/radix-dapp-toolkit'
const dAppId = 'account_tdx_d_12805alyg3562gsphgeyc9re800qq0phlyz89cnu2tydmlp0gt947cw'
// Instantiate DappToolkit
const rdt = RadixDappToolkit({
  dAppDefinitionAddress: dAppId,
  networkId: 33,
})
console.log("dApp Toolkit: ", rdt)

// ************ Fetch the user's account address ************
rdt.walletApi.setRequestData(DataRequestBuilder.accounts().atLeast(1))
// Subscribe to updates to the user's shared wallet data
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
let componentAddress = "component_tdx_d_1cq6y74zwdxcjrsn3uzs543au5upmtk3drgkw6hw8q06cccd6vsnpcd" //GumballMachine component address
let gum_resourceAddress = "resource_tdx_d_1tk83nnrjj75mfgnltkqql0al052g295g8krxqnyuz7q232xss4cws8" // SCRYPTO GUM resource address
let xrdAddress = "resource_tdx_d_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxepwmma"
// You receive this badge(your resource address will be different) when you instantiate the component
let admin_badge = "resource_tdx_d_1t4n8uen586jyv3texjtr0ezxzrsepg43rk5qc923zp8y22h9nu8f9h"
let owner_badge = "resource_tdx_d_1t5pqpfa8wuauufgs020eks73x27zevr4q75qlwrjhefx8dvnepaau6"
// You can use these addresses to skip package deployment steps
// package_tdx_d_1ph7a0m0qwtea3p58t8ney6kt2xwmsp7js4welyswlljsctmhwmckra


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

  // ****** Set componentAddress variable with gateway api getCommitReciept payload ******
  componentAddress = getCommitReceipt.transaction.affected_global_entities[2];
  document.getElementById('componentAddress').innerText = componentAddress;

  // ****** Set admin_badge variable with gateway api getCommitReciept payload ******
  admin_badge = getCommitReceipt.transaction.affected_global_entities[4];
  document.getElementById('admin_badge').innerText = admin_badge;

  // ****** Set owner_badge variable with gateway api getCommitReciept payload ******
  owner_badge = getCommitReceipt.transaction.affected_global_entities[3];
  document.getElementById('owner_badge').innerText = owner_badge;

  // ****** Set gum_resourceAddress variable with gateway api getCommitReciept payload ******
  gum_resourceAddress = getCommitReceipt.transaction.affected_global_entities[6];
  document.getElementById('gum_resourceAddress').innerText = gum_resourceAddress;
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

  // Show the receipt in the DOM
  document.getElementById('receipt').innerText = JSON.stringify(getCommitReceipt);
}


// *********** Get Price ***********
document.getElementById('getPrice').onclick = async function () {
  // Use gateway state api to fetch component details including price field
  let getPrice = await state.getEntityDetailsVaultAggregated(componentAddress)
  console.log('getPrice', getPrice)

  // Show the price in the DOM
  document.getElementById('price').innerText = JSON.stringify(getPrice.details.state.programmatic_json.fields[2].value);
}


// *********** Set Price ***********
document.getElementById('setPrice').onclick = async function () {
  let newPrice = document.getElementById('newPrice').value
  let manifest = `
  CALL_METHOD
    Address("${accountAddress}")
    "create_proof_of_amount"    
    Address("${admin_badge}")
    Decimal("1");
CALL_METHOD
    Address("${componentAddress}")
    "set_price"
    Decimal("${newPrice}");
  `
  console.log("Set Price manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt.walletApi
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })
  if (result.isErr()) throw result.error
  console.log("Set Price sendTransaction result: ", result.value)

  // Fetch the transaction status from the Gateway SDK
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  console.log('Set Price transaction status', transactionStatus)
  let getPrice = await state.getEntityDetailsVaultAggregated(componentAddress)
  console.log('Set Price new value', getPrice)

  // Show the New Price in the DOM
  document.getElementById('price').innerText = JSON.stringify(getPrice.details.state.programmatic_json.fields[2].value);
}


// *********** Withdraw Earnings ***********
document.getElementById('withdrawEarnings').onclick = async function () {
  // TODO Replace with String Manifest
  let manifest = `
  CALL_METHOD
    Address("${accountAddress}")
    "create_proof_of_amount"    
    Address("${owner_badge}")
    Decimal("1");
  CALL_METHOD
    Address("${componentAddress}")
    "withdraw_earnings";
  CALL_METHOD
    Address("${accountAddress}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
    `
  console.log("Withdraw Earnings manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt.walletApi
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })
  if (result.isErr()) throw result.error
  console.log("Withdraw Earnings sendTransaction Result: ", result.value)

  // Fetch the transaction status from the Gateway SDK
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  console.log('Withdraw Earnings status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('Withdraw Earnings commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM
  document.getElementById('withdraw').innerText = JSON.stringify(getCommitReceipt);
}
// *********** Mint NFT Staff Badge ***********
document.getElementById('mintStaffBadge').onclick = async function () {
  // TODO Replace with String Manifest
  let manifest = `
  CALL_METHOD
    Address("${accountAddress}")
    "create_proof_of_amount"    
    Address("${admin_badge}")
    Decimal("1");
CALL_METHOD
    Address("${componentAddress}")
    "mint_staff_badge"
    "Number2";
CALL_METHOD
    Address("${accountAddress}")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP");
    `
  console.log("mintStaffBadge manifest", manifest)

  // Send manifest to extension for signing
  const result = await rdt.walletApi
    .sendTransaction({
      transactionManifest: manifest,
      version: 1,
    })
  if (result.isErr()) throw result.error

  console.log("mintStaffBadge sendTransaction Result: ", result.value)

  // Fetch the transaction status from the Gateway SDK
  let transactionStatus = await transaction.getStatus(result.value.transactionIntentHash)
  console.log('mintStaffBadge status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('mintStaffBadge commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM
  document.getElementById('staffBadge').innerText = JSON.stringify(getCommitReceipt);
}