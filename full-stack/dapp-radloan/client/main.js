import { RadixDappToolkit } from '@radixdlt/radix-dapp-toolkit'
import { RadixEngineToolkit, ManifestBuilder, ManifestAstValue } from '@radixdlt/radix-engine-toolkit'

const rdt = RadixDappToolkit(
  {
    dAppDefinitionAddress:
      'account_tdx_22_1pz7vywgwz4fq6e4v3aeeu8huamq0ctmsmzltay07vzpqm82mp5',
    dAppName: 'RadLoan',
  },
  (requestData) => {
    requestData({
      accounts: { quantifier: 'atLeast', quantity: 1 },
    }).map(({ data: { accounts } }) => {
      // set your application state
      console.log("requestData accounts: ", accounts)
    })
  },
  {
    networkId: 12,
    onDisconnect: () => {
      // clear your application state
    },
    onInit: ({ accounts }) => {
      // set your initial application state
      console.log("onInit accounts: ", accounts)
      if (accounts.length > 0) {
        // document.getElementById('accountName').innerText = accounts[0].label
        // document.getElementById('accountAddress').innerText = accounts[0].address
        // accountAddress = accounts[0].address
      }
    },
  }
)


// Handle Form Data
if (location.pathname === '/index.html') {
  const form = document.getElementById('transactionForm')
  form.addEventListener('submit', (e) => {
    e.preventDefault();
    const data = new FormData(e.target);
    console.log(Object.fromEntries(data));

  })
}

let accountAddress = "account_tdx_22_1pz7vywgwz4fq6e4v3aeeu8huamq0ctmsmzltay07vzpqm82mp5"
let resourceAddr = "resource_tdx_22_1pz7vywgwz4fq6e4v3aeeu8huamq0ctmsmzltay07vzpqm82mp5"
let packageAddress = "package_tdx_22_1pz7vywgwz4fq6e4v3aeeu8huamq0ctmsmzltay07vzpqm82mp5"

let manifest = new ManifestBuilder()
  .callMethod(accountAddress, "withdraw", [
    new ManifestAstValue.Address(resourceAddr),
    new ManifestAstValue.Decimal(100),
  ])
  .takeFromWorktop(
    resourceAddr,
    (builder, bucket) => builder.callFunction(
      packageAddress,
      'RadLoan',
      'instantiate',
      [bucket]
    )
  )
  .build()


let ret = new RadixEngineToolkit()
console.log("manifest", manifest)
console.log("ret", ret)
console.log("rdt", rdt)

// ####### RadLoan Methods #######
// instantiate_default(initial_liquidity: Bucket)
// available_liquidity(&self)
// add_liquidity(&mut self, tokens: Bucket)
// take_loan(&mut self, loan_amount: Decimal)
// repay_loan(&mut self, loan_repayment: Bucket, loan_terms: Bucket)