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


// Global states
let accountAddress // User account address
let componentAddress = "component_tdx_21_1cqulhyk42qcmdck5unzl4nvywje0x93a5xkxsmhek3pz62x9dsfhne" //GumballMachine component address
let gum_resourceAddress = "resource_tdx_21_1thcxgxs9t3je33dreplv2n2yyx6tmakm3gc2wepw0wm5p4fme9lnd5" // SCRYPTO GUM resource address
let xrdAddress = "resource_tdx_21_1tknxxxxxxxxxradxrdxxxxxxxxx009923554798xxxxxxxxxsmgder" //Enkinet XRD resource address
// You receive this badge(your resource address will be different) when you instantiate the component
let admin_badge = "resource_tdx_21_1tk7daptrcfh0xh7n3277qere5ek3fwpxfj0va7hj0pwy99pnq9ckkc"
let owner_badge = "resource_tdx_21_1t4yksfxzfjau9axdxv454a3av7n46qnsx5cr78tl9j2wxsw36qg4q3"
// You can use these addresses to skip package deployment steps
// enkinet package_address = package_tdx_21_1pk0vlq5nedaws4gzwqvnxwn4884uc6du7kgaw4cratqd9r36e8jthr


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
  let transactionStatus = await rdt.gatewayApi.transaction.getStatus(result.value.transactionIntentHash)
  console.log('Instantiate TransactionApi transaction/status:', transactionStatus)


  // ************ Fetch component address from gateway api and set componentAddress variable **************
  let getCommitReceipt = await rdt.gatewayApi.transaction.getCommittedDetails(result.value.transactionIntentHash)
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
  let transactionStatus = await rdt.gatewayApi.transaction.getStatus(result.value.transactionIntentHash)
  console.log('Buy Gumball TransactionAPI transaction/status: ', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await rdt.gatewayApi.transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('Buy Gumball Committed Details Receipt', getCommitReceipt)

  // Show the receipt in the DOM
  document.getElementById('receipt').innerText = JSON.stringify(getCommitReceipt);
}


// *********** Get Price ***********
document.getElementById('getPrice').onclick = async function () {
  // Use gateway state api to fetch component details including price field
  let getPrice = await rdt.gatewayApi.state.getEntityDetailsVaultAggregated(componentAddress)
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
  let transactionStatus = await rdt.gatewayApi.transaction.getStatus(result.value.transactionIntentHash)
  console.log('Set Price transaction status', transactionStatus)
  let getPrice = await rdt.gatewayApi.state.getEntityDetailsVaultAggregated(componentAddress)
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
  let transactionStatus = await rdt.gatewayApi.transaction.getStatus(result.value.transactionIntentHash)
  console.log('Withdraw Earnings status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await rdt.gatewayApi.transaction.getCommittedDetails(result.value.transactionIntentHash)
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
  let transactionStatus = await rdt.gatewayApi.transaction.getStatus(result.value.transactionIntentHash)
  console.log('mintStaffBadge status', transactionStatus)

  // fetch commit reciept from gateway api 
  let getCommitReceipt = await rdt.gatewayApi.transaction.getCommittedDetails(result.value.transactionIntentHash)
  console.log('mintStaffBadge commitReceipt', getCommitReceipt)

  // Show the receipt on the DOM
  document.getElementById('staffBadge').innerText = JSON.stringify(getCommitReceipt);
}