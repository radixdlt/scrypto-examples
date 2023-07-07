import './style.css'
// import scryptoLogo from './scryptoLogo.png'
import { 
  RadixDappToolkit, 
 } from "@radixdlt/radix-dapp-toolkit";
import { 
  NetworkId,
  ManifestBuilder, 
  ManifestAstValue, 
  InstructionList, 
  // Transactions
  NotarizedTransaction,
  PrivateKey,
  TransactionBuilder,
  TransactionHeader,
  TransactionManifest,
  ValidationConfig,
  generateRandomNonce,
  Convert,
  TransactionIntent,
  SignedTransactionIntent,
  RadixEngineToolkit,
  PublicKey,
} from '@radixdlt/radix-engine-toolkit'
import fs from 'fs';



// document.querySelector('#app').innerHTML = `
//   <div>
//     <a href="https://vitejs.dev" target="_blank">
//       <img src="/vite.svg" class="logo" alt="Vite logo" />
//     </a>
//     <a href="https://developer.mozilla.org/en-US/docs/Web/JavaScript" target="_blank">
//       <img src="${scryptoLogo}" class="logo vanilla" alt="JavaScript logo" />
//     </a>
//     <h1>Hello Scrypto!</h1>
//     <div class="card">
//       <radix-connect-button />
//     </div>
//     <p class="read-the-docs">
//       Click on the Scrypto logo to learn more
//     </p>
//   </div>
// `

const dAppId = 'account_tdx_c_1p8l69nnvnens5awhkmxfkkxjvfpv9zvd65a0ra9sfh5sds7tfe'

const rdt = RadixDappToolkit(
  { dAppDefinitionAddress: dAppId, dAppName: "Radiswap" },
  (requestData) => {
    requestData({
      accounts: { quantifier: 'atLeast', quantity: 1 },
    }).map(({ data: { accounts } }) => {
      // add accounts to dApp application state
      console.log("account data: ", accounts)
      const accountNameElement = document.getElementById('accountName');
      if (accountNameElement !== null && accounts[0]?.label !== undefined) {
        accountNameElement.innerText = accounts[0]?.label;
      }
      const accountAddressElement = document.getElementById('accountName');
      if (accountAddressElement !== null && accounts[0]?.label !== undefined) {
        accountAddressElement.innerText = accounts[0]?.label;
      }
      accountAddress = accounts[0].address
    })
  },
  {
    networkId: 12, // 12 is for RCnet 01 for Mainnet
    onDisconnect: () => {
      // clear your application state
    },
    onInit: ({ accounts = [] }) => {
      // set your initial application state
      console.log("onInit accounts: ", accounts)
      if (accounts.length > 0) {
        const accountNameElement = document.getElementById('accountName');
        if (accountNameElement !== null && accounts[0]?.label !== undefined) {
          accountNameElement.innerText = accounts[0]?.label;
        }
        const accountAddressElement = document.getElementById('accountName');
        if (accountAddressElement !== null && accounts[0]?.label !== undefined) {
          accountAddressElement.innerText = truncateMiddle(accounts[0]?.label);
        }
        accountAddress = accounts[0].address
      }
    },
  }
)
console.log("dApp Toolkit: ", rdt)

import { TransactionApi, StateApi, StatusApi, StreamApi, } from "@radixdlt/babylon-gateway-api-sdk";

const transactionApi = new TransactionApi();
const stateApi = new StateApi();
const statusApi = new StatusApi();
const streamApi = new StreamApi();

let accountAddress // User account address
let componentAddress
let packageAddress = "package_tdx_c_1qppzt8sxhgwu62y6ywmewe2j3s37uyc63nye4yx9etjs3tv8x9"
let tokenAAddress 
let tokenBAddress 
let swapFee
let xrdAddress = "resource_tdx_c_1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq40v2wv"
let poolUnitsAddress
let txLink = "https://rcnet-dashboard.radixdlt.com/transaction/"
interface FungibleMetadata {
  metadata: string;
  resource_address: string;
}

let fungibles_metadata: FungibleMetadata[] = []

let token_pair = []
let componentAddressList = []

document.getElementById('createToken').onclick = async function () {
  const code = fs.readFileSync('/scrypto/target/wasm32-unknown-unknown/release/rcnet_radiswap.wasm');
  const schema = fs.readFileSync('/scrypto/target/wasm32-unknown-unknown/release/rcnet_radiswap.schema');

  let notaryPrivateKey = new PrivateKey.EcdsaSecp256k1(
    "40c1b9deccc56c0da69821dd652782887b5d31fe6bf6ead519a23f9e9472b49b"
  );

  let signer1PublicKey = new PublicKey.EcdsaSecp256k1(
    "020166fe820d94d7e207d2076e85f740b7593189e9ea1f166fa0db6831f73247bc"
    )
  
  let signer1PrivateKey = new PrivateKey.EcdsaSecp256k1(
    "df1fdb3f36d7a079c4b7bd288ae2122e2abdbb5836bb9ac3c9c9eef53807b2eb"
  );

  let virtualAccountAddress =
  await RadixEngineToolkit.deriveVirtualAccountAddress(
    signer1PublicKey,
    NetworkId.RCnetV1
  );

  console.log(virtualAccountAddress);

  let transactionHeader = new TransactionHeader(
    1 /* The transaction version. Currently always 1 */,
    NetworkId.RCnetV1 /* The network that this transaction is destined to */,
    6182 /* The start epoch (inclusive) of when this transaction becomes valid */,
    6250 /* The end epoch (exclusive) of when this transaction is no longer valid */,
    generateRandomNonce() /* A random nonce */,
    notaryPrivateKey.publicKey() /* The public key of the notary */,
    true /* Whether the notary signature is also considered as an intent signature */,
    100_000_000 /* A limit on the amount of cost units that the transaction can consume */,
    0 /* The percentage of fees that goes to validators */
  );


  let manifest = new ManifestBuilder()
    .publishPackage(
      code,
      schema,
      new ManifestAstValue.Map(
        ManifestAstValue.Kind.String,
        ManifestAstValue.Kind.String,
        []
      ),
      new ManifestAstValue.Map(
        ManifestAstValue.Kind.String,
        ManifestAstValue.Kind.String,
        []
      ),
      new ManifestAstValue.Tuple([
        new ManifestAstValue.Map(
          ManifestAstValue.Kind.Tuple,
          ManifestAstValue.Kind.Enum,
          []
        ),
        new ManifestAstValue.Map(
          ManifestAstValue.Kind.String,
          ManifestAstValue.Kind.Enum,
          []
        ),
        new ManifestAstValue.Enum(new ManifestAstValue.EnumU8Discriminator(0)),
        new ManifestAstValue.Map(
          ManifestAstValue.Kind.Tuple,
          ManifestAstValue.Kind.Enum,
          []
        ),
        new ManifestAstValue.Map(
          ManifestAstValue.Kind.String,
          ManifestAstValue.Kind.Enum,
          []
        ),
        new ManifestAstValue.Enum(new ManifestAstValue.EnumU8Discriminator(0)),
      ])
    )

  // let tokenName = document.getElementById("tokenName").value;
  // let tokenSymbol = document.getElementById("tokenSymbol").value;

  // let manifest = new ManifestBuilder()
  // .createFungibleResourceWithInitialSupply(
  //   new ManifestAstValue.U8(18),
  //   new ManifestAstValue.Map(
  //     ManifestAstValue.Kind.String,
  //     ManifestAstValue.Kind.String,
  //     [
  //       [new ManifestAstValue.String("name"), new ManifestAstValue.String(tokenName)],
  //       [new ManifestAstValue.String("symbol"), new ManifestAstValue.String(tokenSymbol)],
  //     ], 
  //   ),
  //   new ManifestAstValue.Map(
  //     ManifestAstValue.Kind.Enum,
  //     ManifestAstValue.Kind.Tuple,
  //     []
  //   ),
  //   new ManifestAstValue.Decimal(10000)
  // )
  // .callMethod(accountAddress, "deposit_batch", [
  //   ManifestAstValue.Expression.entireWorktop()
  // ])

  // We then build the transaction manifest
  // let manifest = new ManifestBuilder()
  //   .callMethod(
  //     virtualAccountAddress,
  //     "lock_fee",
  //     [new ManifestAstValue.Decimal(10)]
  //   )
  //   .callMethod(
  //     virtualAccountAddress,
  //     "withdraw",
  //     [
  //       new ManifestAstValue.Address(
  //         xrdAddress
  //       ),
  //       new ManifestAstValue.Decimal(10),
  //     ]
  //   )
  //   .takeFromWorktop(
  //     xrdAddress,
  //     (builder, bucket) =>
  //       builder.callMethod(
  //         virtualAccountAddress,
  //         "deposit",
  //         [bucket]
  //       )
  //     )
  //     .build();

  // console.log(manifest)

  // We may now build the complete transaction through the transaction builder.
  let transaction = await TransactionBuilder.new().then(
    (builder) =>
      builder
        .header(transactionHeader)
        .manifest(manifest)
        .sign(signer1PrivateKey)
        .notarize(notaryPrivateKey)
  );

  let notarizedTransactionUint8Array = await transaction.compile();
  let notarizedTransactionHex = Convert.Uint8Array.toHexString(notarizedTransactionUint8Array);
  console.log(notarizedTransactionHex)

  await transactionApi.transactionSubmit({
    transactionSubmitRequest: {
      notarized_transaction_hex: notarizedTransactionHex,
    }
  })

  let retrieveTransactionId = await transaction.transactionId();
  let transactionIdHash = Convert.Uint8Array.toHexString(retrieveTransactionId);
  console.log(transactionIdHash)

    // // ************ Fetch component address from gateway api and set componentAddress variable **************
    // let commitReceipt = await transactionApi.transactionCommittedDetails({
    //   transactionCommittedDetailsRequest: {
    //     intent_hash_hex: transactionIdHash
    //   }
    // })
    // console.log('Instantiate Committed Details Receipt', commitReceipt)
    const createTokenTxLink = document.querySelector(".createTokenTx");
    let tx = txLink + transactionIdHash;
    createTokenTxLink.href= tx;
    createTokenTxLink.style.display = "inline";

  // let sendTransaction = await transactionApi.transactionSubmit({
  //     transactionSubmitRequest: {
  //       notarized_transaction_hex: noterized
  //     }
  // })

  // let transactionIntentHash = await transaction.signedIntentHash();
  // let transactionIntentHashHex = Convert.Uint8Array.toHexString(transactionIntentHash);

  // let commitReceipt = await transactionApi.transactionCommittedDetails({
  //   transactionCommittedDetailsRequest: {
  //     intent_hash_hex: transactionIntentHashHex
  //   }
  // })
  // console.log('Swap Committed Details Receipt', commitReceipt)


  // console.log(sendTransaction)
  
  

  // Check that the transaction that we've just built is statically valid.
  // (
  //   await transaction.staticallyValidate(
  //     ValidationConfig.default(NetworkId.RCnetV1)
  //   )
  // ).throwIfInvalid();


  }



// document.getElementById('createToken').onclick = async function () {
//   let tokenName = document.getElementById("tokenName").value;
//   let tokenSymbol = document.getElementById("tokenSymbol").value;

//   let manifest = new ManifestBuilder()
//   .createFungibleResourceWithInitialSupply(
//     new ManifestAstValue.U8(18),
//     new ManifestAstValue.Map(
//       ManifestAstValue.Kind.String,
//       ManifestAstValue.Kind.String,
//       [
//         [new ManifestAstValue.String("name"), new ManifestAstValue.String(tokenName)],
//         [new ManifestAstValue.String("symbol"), new ManifestAstValue.String(tokenSymbol)],
//       ], 
//     ),
//     new ManifestAstValue.Map(
//       ManifestAstValue.Kind.Enum,
//       ManifestAstValue.Kind.Tuple,
//       []
//     ),
//     new ManifestAstValue.Decimal(10000)
//   )
//   .callMethod(accountAddress, "deposit_batch", [
//     ManifestAstValue.Expression.entireWorktop()
//   ])
//   .build();

//   console.log(manifest)

// let converted_manifest = await manifest.convert(
//   InstructionList.Kind.String,
//   NetworkId.RCnetV1
// );

// console.log("Conversion: ", converted_manifest)

// let string_converted_manifest = converted_manifest.instructions.value;

// console.log("Create Token Manifest: ", string_converted_manifest)

// // Send manifest to extension for signing
// const result = await rdt
//   .sendTransaction({
//     transactionManifest: string_converted_manifest,
//     version: 1,
//   })

// if (result.isErr()) throw result.error

// console.log("Intantiate WalletSDK Result: ", result.value)

// // ************ Fetch the transaction status from the Gateway API ************
// let status = await transactionApi.transactionStatus({
//   transactionStatusRequest: {
//     intent_hash_hex: result.value.transactionIntentHash
//   }
// });
// console.log('Instantiate TransactionApi transaction/status:', status)

// // ************ Fetch entity addresses from gateway api and set entity variable **************
// let commitReceipt = await transactionApi.transactionCommittedDetails({
//   transactionCommittedDetailsRequest: {
//     intent_hash_hex: result.value.transactionIntentHash
//   }
// })
// console.log('Instantiate Committed Details Receipt', commitReceipt)

// // Retrieve entity address
// document.getElementById('newTokenAddress').innerText = commitReceipt.details.referenced_global_entities[0];

// const createTokenTxLink = document.querySelector(".createTokenTx");
// let tx = txLink + commitReceipt.transaction.intent_hash_hex;
// createTokenTxLink.href= tx;
// createTokenTxLink.style.display = "inline";

// }


const instantiateComponentElement = document.getElementById('instantiateComponent');
instantiateComponentElement?.addEventListener('click', async function () {

  tokenAAddress = (document.getElementById("selectTokenA") as HTMLSelectElement)?.value;
  let tokenAAmount = (document.getElementById("amountA") as HTMLSelectElement)?.value;
  tokenBAddress = (document.getElementById("selectTokenB") as HTMLSelectElement)?.value;
  let tokenBAmount = (document.getElementById("amountB") as HTMLSelectElement)?.value;
  swapFee = (document.getElementById("swapFee") as HTMLSelectElement)?.value;

  // We create a Transaction Manifest using the ManifestBuilder class conveniently provided  by the RadixEngineToolkit.
  // This Transaction Manifest has the following instructions:
  // 1. Withdraw the first selected token and amount out of the user's account based on the user's input.
  // 2. Withdraw the second selected token and amount out of the user's account based on the user's input.
  // 3. Take the first token resource and amount and place it in a `Bucket`.
  // 4. Take the second token resource and amount and place it in a `Bucket.
  // 5. Pass the buckets as argument inputs to instantiate the Radiswap component along witht he determined
  // swap fee based on the user's input.
  // 6. Deposit any (Pool Units resourece) resource returned from the instantiation function to the user's account.
  let manifest = new ManifestBuilder()
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(tokenAAddress),
        new ManifestAstValue.Decimal(tokenAAmount),
      ]
    )    
    .callMethod(
      accountAddress,
      "withdraw", 
      [
      new ManifestAstValue.Address(tokenBAddress),
      new ManifestAstValue.Decimal(tokenBAmount)
      ]
    )
    .takeFromWorktop(
      tokenAAddress,
      (builder, tokenABucket) =>
      builder.takeFromWorktop(
        tokenBAddress,
        (builder, tokenBBucket) =>
        builder.callFunction(
          packageAddress,
          "Radiswap",
          "instantiate_radiswap",
          [
            tokenABucket,
            tokenBBucket,
            new ManifestAstValue.Decimal(swapFee),
          ]
        )
      )
    )
    .callMethod(
      accountAddress,
      "deposit_batch",[
      ManifestAstValue.Expression.entireWorktop()
      ]
    )
    .build();

  let converted_manifest = await manifest.convert(
    InstructionList.Kind.String,
    NetworkId.RCnetV1
  );
  
  let string_converted_manifest = converted_manifest.instructions.value;
          
  console.log("Instantiate Manifest: ", string_converted_manifest)
  
  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: string_converted_manifest,
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
  let componentAddress: string = commitReceipt.details.referenced_global_entities[0];
  document.getElementById('componentAddress')!.innerText = truncateMiddle(componentAddress);
  
  let poolUnitsAddress: string = truncateMiddle(commitReceipt.details.referenced_global_entities[2]);
  document.getElementById('poolUnitsAddress')!.innerText = poolUnitsAddress;
  
  const createTokenTxLink: HTMLAnchorElement = document.querySelector(".instantiateComponentTx")!;
  let tx: string = txLink + commitReceipt.transaction.intent_hash_hex;
  createTokenTxLink.href = tx;
  createTokenTxLink.style.display = "inline";
  
  loadPools();
  loadTokenPair();
  loadPoolInformation();
  
});
// ************ Instantiate component and fetch component and resource addresses *************
// This function is used to instantiate a Radiswap component which creates a two token liquidity pool
document.getElementById('instantiateComponent').onclick = async function () {
  // We first retrieve our global variables (tokenAAddress and tokenBAddress) to set its value so that we
  // can conveniently use the variable in other parts of our code. The value that will be saved to this 
  // variable will be the selected token resources that the user wishes to create this liquidity pool.
  // Additionally, since we expect the user to have an indeterminate amount that they would like to 
  // deposit into this pool for every instance, we will only create local variables for the amount inputs
  // (tokenAAmount and tokenBAmount) to be used only once for this function. Likewise with our packageAddress
  // variable.
  // let packageAddress = document.getElementById("packageAddress").value;
 

document.getElementById('swapToken').onclick = async function () {
  let inputToken = document.getElementById("swapDropDown")?.value;
  let inputAmount = document.getElementById("inputAmount")?.value;

  let manifest = new ManifestBuilder()
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(inputToken),
        new ManifestAstValue.Decimal(inputAmount),
      ]
    )
    .takeFromWorktop(
      inputToken,
      (builder, inputBucket) => 
      builder.callMethod(
        componentAddress,
        "swap",
        [
          inputBucket
        ]
      )
    )
    .callMethod(
      accountAddress,
      "deposit_batch",
      [
        ManifestAstValue.Expression.entireWorktop()
      ]
    )
    .build();

  console.log(manifest)

  let converted_manifest = await manifest.convert(
    InstructionList.Kind.String,
    NetworkId.RCnetV1
  );

  let string_converted_manifest = converted_manifest.instructions.value;

  console.log("Create Token Manifest: ", string_converted_manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: string_converted_manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Swap WalletSDK Result: ", result.value)

    // ************ Fetch the transaction status from the Gateway API ************
    let status = await transactionApi.transactionStatus({
      transactionStatusRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    });
    console.log('Swap TransactionApi transaction/status:', status)
  
    // ************ Fetch component address from gateway api and set componentAddress variable **************
    let commitReceipt = await transactionApi.transactionCommittedDetails({
      transactionCommittedDetailsRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    })
    console.log('Swap Committed Details Receipt', commitReceipt)
  
    const createTokenTxLink = document.querySelector(".swapTx");
    let tx = txLink + commitReceipt.transaction.intent_hash_hex;
    createTokenTxLink.href= tx;
    createTokenTxLink.style.display = "inline";

  loadPoolInformation();
}

document.getElementById('getAmount').onclick = async function () {
  let requestedToken = document.getElementById("exactSwapDropDown").value;
  let requestedAmount = document.getElementById("requestedAmount").value;

  console.log(componentAddress)

    // Sorting logic
    let inputTokenAddress, outputTokenAddress;
    if (requestedToken === tokenAAddress ) {
      inputTokenAddress = tokenBAddress; 
      outputTokenAddress = tokenAAddress;
    } else {
      inputTokenAddress = tokenAAddress;
      outputTokenAddress = tokenBAddress; 
    };

    console.log(inputTokenAddress)

  // Making request to gateway
  let inputTokenRequest = await stateApi.entityFungibleResourceVaultPage(
    {
      stateEntityFungibleResourceVaultsPageRequest: {
        address: componentAddress,
        resource_address: inputTokenAddress,
      }
    });
  
  console.log("Token A Request: ", inputTokenRequest);

  let outputTokenRequest = await stateApi.entityFungibleResourceVaultPage(
    {
      stateEntityFungibleResourceVaultsPageRequest: {
        address: componentAddress,
        resource_address: outputTokenAddress,
      }
    });

  console.log("Token B Request: ", outputTokenRequest);


  let x = inputTokenRequest.items[0].amount;
  let y = outputTokenRequest.items[0].amount;
  let dy = requestedAmount;
  let r = (1 - swapFee) / 1;
  let dx = (dy * x) / (r * (y - dy));

  document.getElementById('requiredResource').innerText = inputTokenAddress
  document.getElementById('requiredAmount').innerText = dx 
}

document.getElementById('exactSwapToken').onclick = async function () {
  let requiredResource = document.getElementById('requiredResource').innerText;
  let requiredAmount = document.getElementById("requiredAmount").innerHTML;  

  let manifest = new ManifestBuilder()
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(requiredResource),
        new ManifestAstValue.Decimal(requiredAmount),
      ]
    )
    .takeFromWorktop(
      requiredResource,
      (builder, requiredBucket) => 
      builder.callMethod(
        componentAddress,
        "swap",
        [requiredBucket]      
      )
    )
    .callMethod(
      accountAddress,
      "deposit_batch", 
      [
      ManifestAstValue.Expression.entireWorktop()
      ]
    )
    .build();

  console.log(manifest)

  let converted_manifest = await manifest.convert(
    InstructionList.Kind.String,
    NetworkId.RCnetV1
  )

  let string_converted_manifest = converted_manifest.instructions.value;
  
  console.log("Create Token Manifest: ", string_converted_manifest)

  // Send manifest to extension for signing
  const result = await rdt
    .sendTransaction({
      transactionManifest: string_converted_manifest,
      version: 1,
    })

  if (result.isErr()) throw result.error

  console.log("Exact Swap sendTransaction Result: ", result)

  // Fetch the transaction status from the Gateway SDK
  let status = await transactionApi.transactionStatus({
    transactionStatusRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  });
  console.log('Exact Swap TransactionAPI transaction/status: ', status)

  // fetch commit reciept from gateway api 
  let commitReceipt = await transactionApi.transactionCommittedDetails({
    transactionCommittedDetailsRequest: {
      intent_hash_hex: result.value.transactionIntentHash
    }
  })
  console.log('Exact Swap Committed Details Receipt', commitReceipt)

  const createTokenTxLink = document.querySelector(".exactSwapTx");
  let tx = txLink + commitReceipt.transaction.intent_hash_hex;
  createTokenTxLink.href= tx;
  createTokenTxLink.style.display = "inline";

  loadPoolInformation();
}

document.getElementById('addLiquidity').onclick = async function () {
  let tokenAAmount = document.getElementById("tokenAAmount").value;
  let tokenBAmount = document.getElementById("tokenBAmount").value;

  let manifest = new ManifestBuilder()
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(tokenAAddress),
        new ManifestAstValue.Decimal(tokenAAmount)
      ]
    )
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(tokenBAddress),
        new ManifestAstValue.Decimal(tokenBAmount)
      ]
    )
    .takeFromWorktop(
      tokenAAddress,
      (builder, tokenABucket) =>
      builder.takeFromWorktop(
        tokenBAddress,
        (builder, tokenBBucket) =>
        builder.callMethod(
          componentAddress,
          "add_liquidity",
          [
            tokenABucket,
            tokenBBucket
          ]
        )
      )
    )
    .callMethod(
      accountAddress,
      "deposit_batch",
      [
        ManifestAstValue.Expression.entireWorktop()
      ]
    )
    .build();

    let converted_manifest = await manifest.convert(
      InstructionList.Kind.String,
      NetworkId.RCnetV1
    )

    let string_converted_manifest = converted_manifest.instructions.value;
  
    console.log("Create Token Manifest: ", string_converted_manifest)
  
    // Send manifest to extension for signing
    const result = await rdt
      .sendTransaction({
        transactionManifest: string_converted_manifest,
        version: 1,
      })
  
    if (result.isErr()) throw result.error
  
    console.log("Add Liquidity sendTransaction Result: ", result)

    // Fetch the transaction status from the Gateway SDK
    let status = await transactionApi.transactionStatus({
      transactionStatusRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    });
    console.log('Add Liquidity TransactionAPI transaction/status: ', status)
  
    // fetch commit reciept from gateway api 
    let commitReceipt = await transactionApi.transactionCommittedDetails({
      transactionCommittedDetailsRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    })
    console.log('Add Liquidity Committed Details Receipt', commitReceipt)
  
    const createTokenTxLink = document.querySelector(".addLiquidityTx");
    let tx = txLink + commitReceipt.transaction.intent_hash_hex;
    createTokenTxLink.href= tx;
    createTokenTxLink.style.display = "inline";

    loadPoolInformation();
}


document.getElementById('removeLiquidity').onclick = async function () {
  let poolUnitsAmount = document.getElementById("poolUnitsAmount").value;

  let manifest = new ManifestBuilder()
    .callMethod(
      accountAddress,
      "withdraw",
      [
        new ManifestAstValue.Address(poolUnitsAddress),
        new ManifestAstValue.Decimal(poolUnitsAmount)
      ]
    )
    .takeFromWorktop(
      poolUnitsAddress,
      (builder, poolUnitBucket) =>
      builder.callMethod(
        componentAddress,
        "remove_liquidity",
        [
          poolUnitBucket
        ]
      )
    )
    .callMethod(
      accountAddress,
      "deposit_batch",
      [
        ManifestAstValue.Expression.entireWorktop()
      ]
    )
    .build();

    let converted_manifest = await manifest.convert(
      InstructionList.Kind.String,
      NetworkId.RCnetV1
    )

    let string_converted_manifest = converted_manifest.instructions.value;
  
    console.log("Create Token Manifest: ", string_converted_manifest)
  
    // Send manifest to extension for signing
    const result = await rdt
      .sendTransaction({
        transactionManifest: string_converted_manifest,
        version: 1,
      })
  
    if (result.isErr()) throw result.error

    console.log("Remove Liquidity sendTransaction Result: ", result)

    // Fetch the transaction status from the Gateway SDK
    let status = await transactionApi.transactionStatus({
      transactionStatusRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    });
    console.log('Remove Liquidity TransactionAPI transaction/status: ', status)
  
    // fetch commit reciept from gateway api 
    let commitReceipt = await transactionApi.transactionCommittedDetails({
      transactionCommittedDetailsRequest: {
        intent_hash_hex: result.value.transactionIntentHash
      }
    })
    console.log('Remove Liquidity Committed Details Receipt', commitReceipt)
  
    const createTokenTxLink = document.querySelector(".removeLiquidityTx");
    let tx = txLink + commitReceipt.transaction.intent_hash_hex;
    createTokenTxLink.href= tx;
    createTokenTxLink.style.display = "inline";

    loadPoolInformation();
}


// ****** EXTRA ******
window.onload = async function fetchData() {
  var fungibles = [];

  // Getway Request //
  let accountState = await stateApi.stateEntityDetails({
    stateEntityDetailsRequest: {
      addresses: [accountAddress]
    }
  })
  
  accountState.items[0].fungible_resources?.items.forEach(item => fungibles.push(item))
  //
  
  let i = 0;

  while (i < fungibles.length) {

    let fungible_object = {};

    let fungible_string = fungibles[i].resource_address;

    try {
      let metadata = await stateApi.entityMetadataPage({
        stateEntityMetadataPageRequest: {
          address: fungible_string
        }
      })

      if (metadata.items[1]) {

        let metadataValue = metadata.items[1].value.as_string;

        fungible_object.metadata = metadataValue;

      } else {
        fungible_object.metadata = "N/A";
      }
      fungible_object.resource_address = fungible_string;
    } catch (error) {
      console.log(`Error retrieving metadata for ${fungible_string}: ${error}`);

      fungible_object.metadata = "N/A";
      fungible_object.resource_address = fungibles[i].resource_address;
    }
    fungibles_metadata.push(fungible_object);
    i++;
  }
  
  var select = document.createElement("select");

  var selectTokenA = document.getElementById("selectTokenA");
  var selectTokenB = document.getElementById("selectTokenB");

  for (const val of fungibles_metadata)
  {
      var option = document.createElement("option");
      option.value = val.resource_address;
      option.text =  val.metadata + " - " + truncateMiddle(val.resource_address);
      select.appendChild(option);
      selectTokenA.appendChild(option.cloneNode(true));
      selectTokenB.appendChild(option.cloneNode(true));
  }
}

async function loadPoolInformation() {
  document.getElementById("tokenPair").innerText = 
    fungibles_metadata[0].metadata + " - " + truncateMiddle(fungibles_metadata[0].resource_address) 
    + "/" + 
    fungibles_metadata[1].metadata + " - " + truncateMiddle(fungibles_metadata[0].resource_address);

  let tokenARequest = await stateApi.entityFungibleResourceVaultPage({
    stateEntityFungibleResourceVaultsPageRequest: {
      address: componentAddress,
      resource_address: tokenAAddress,
    }
  })

  let tokenBRequest = await stateApi.entityFungibleResourceVaultPage({
    stateEntityFungibleResourceVaultsPageRequest: {
      address: componentAddress,
      resource_address: tokenBAddress,
    }
  })

  document.getElementById("liquidity").innerText = 
    tokenARequest.items[0].amount + 
    "/" + 
    tokenBRequest.items[0].amount;
}

// Retrieves TokenPair
async function loadTokenPair() {
  const select = document.createElement("select");
  const swapDropDown = document.getElementById("swapDropDown");
  const exactSwapDropDown = document.getElementById("exactSwapDropDown");

  fungibles_metadata
    .filter((val) => val.resource_address === tokenAAddress || val.resource_address === tokenBAddress)
    .forEach((val) => {
      const option = document.createElement("option");
      option.value = val.resource_address;
      option.text = `${val.metadata} - ${truncateMiddle(val.resource_address)}`;
      select.appendChild(option);
      swapDropDown.appendChild(option.cloneNode(true));
      exactSwapDropDown.appendChild(option.cloneNode(true));

      if (val.resource_address === tokenAAddress) {
        document.getElementById("tokenAAddress")!.innerText = `${val.metadata} - ${truncateMiddle(val.resource_address)}`;
      } else if (val.resource_address === tokenBAddress) {
        document.getElementById("tokenBAddress")!.innerText = `${val.metadata} - ${truncateMiddle(val.resource_address)}`;
      }
    });
}



function truncateMiddle(str: string) {
  if (str.length <= 10) {
    return str;
  }

  const ellipsis = "...";
  const charsToShow = 18 - ellipsis.length;
  const frontChars = Math.ceil(charsToShow / 2);
  const backChars = Math.floor(charsToShow / 2);

  const truncatedStr = str.substr(0, frontChars) + ellipsis + str.substr(str.length - backChars);
  return truncatedStr;
}

