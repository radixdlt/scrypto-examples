import { RadixDappToolkit, ManifestBuilder, Decimal, Bucket, Expression, ResourceAddress } from '@radixdlt/radix-dapp-toolkit'

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
    networkId: 11,
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


// ####### RadLoan Methods #######
// instantiate_default(initial_liquidity: Bucket)
// available_liquidity(&self)
// add_liquidity(&mut self, tokens: Bucket)
// take_loan(&mut self, loan_amount: Decimal)
// repay_loan(&mut self, loan_repayment: Bucket, loan_terms: Bucket)