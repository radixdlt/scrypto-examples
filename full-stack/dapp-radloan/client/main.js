import './style.css'
import scryptoLogo from './scryptoLogo.png'
import { RadixDappToolkit } from '@radixdlt/radix-dapp-toolkit'


document.querySelector('#app').innerHTML = `
  <div>
    <a href="https://vitejs.dev" target="_blank">
      <img src="/vite.svg" class="logo" alt="Vite logo" />
    </a>
    <a href="https://docs-babylon.radixdlt.com/main/scrypto/introduction.html" target="_blank">
      <img src="${scryptoLogo}" class="logo vanilla" alt="Scrypto logo" />
    </a>
    <h1>Hello Scrypto!</h1>  
    <p class="read-the-docs">
      Click on the Scrypto logo to learn more
    </p>
    <div class="card">
      <radix-connect-button />
    </ div>  
  </div>
`
const rdt = RadixDappToolkit(
  {
    dAppDefinitionAddress:
      "account_tdx_22_1pz7vywgwz4fq6e4v3aeeu8huamq0ctmsmzltay07vzpqm82mp5",
    dAppName: "Name of your dApp",
  },
  (requestData) => {
    requestData({
      accounts: { quantifier: "atLeast", quantity: 1 },
    }).map(({ data: { accounts } }) => {
      // set your application state
      console.log("requestData accounts: ",accounts)
    });
  },
  {
    networkId: 11, // for betanet 01 for mainnet
    onDisconnect: () => {
      // clear your application state
    },
    onInit: ({ accounts }) => {
      // set your initial application state
      console.log("onInit accounts: ",accounts)
    },
  }
);

