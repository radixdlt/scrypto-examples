import { get } from 'svelte/store';
import config from '../config.json';
import { rdt } from '../store';

export const getInstantiateManifest = () => `CALL_FUNCTION
Address("${config.packageAddress}")
  "Radiswap"
  "new"
  Enum<0u8>()
  Address("${config.poolResourceAddress1}")
  Address("${config.poolResourceAddress2}");
CALL_METHOD
  Address("${get(rdt)?.walletApi.getWalletData().accounts[0].address}")
  "try_deposit_batch_or_abort"
  Expression("ENTIRE_WORKTOP")
  None;
`;
