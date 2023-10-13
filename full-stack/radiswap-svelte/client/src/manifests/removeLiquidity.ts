import { get } from 'svelte/store';
import { poolInfo, rdt } from '../store';

export const getRemoveLiquidityManifest = ({ amount }: { amount: string }) => {
	const accountAddress = get(rdt)?.walletApi.getWalletData().accounts[0].address;
	const info = get(poolInfo);

	return `
CALL_METHOD
  Address("${accountAddress}")
  "withdraw"
  Address("${info.poolUnit?.address}")
  Decimal("${amount}");
TAKE_ALL_FROM_WORKTOP
  Address("${info.poolUnit?.address}")
  Bucket("pool_unit");
CALL_METHOD
  Address("${info.componentAddress}")
  "remove_liquidity"
  Bucket("pool_unit");
CALL_METHOD
  Address("${accountAddress}")
  "try_deposit_batch_or_abort"
Expression("ENTIRE_WORKTOP")
  None;
`;
};
