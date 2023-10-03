import { get } from 'svelte/store';
import config from '../config.json';
import { rdt, poolInfo } from '../store';

export const getAddLiquidityManifest = ({
	amount1,
	amount2
}: {
	amount1: string;
	amount2: string;
}) => {
	const accountAddress = get(rdt)?.walletApi.getWalletData().accounts[0].address;
	return `
CALL_METHOD
  Address("${accountAddress}")
  "withdraw"
  Address("${config.poolResourceAddress1}")
  Decimal("${amount1}");
TAKE_ALL_FROM_WORKTOP
  Address("${config.poolResourceAddress1}")
  Bucket("resource_a");
CALL_METHOD
  Address("${accountAddress}")
  "withdraw"
  Address("${config.poolResourceAddress2}")
  Decimal("${amount2}");
TAKE_ALL_FROM_WORKTOP
  Address("${config.poolResourceAddress2}")
  Bucket("resource_b");
CALL_METHOD
  Address("${get(poolInfo).componentAddress}")
  "add_liquidity"
  Bucket("resource_a")
  Bucket("resource_b");
CALL_METHOD
  Address("${accountAddress}")
  "try_deposit_batch_or_abort"
  Expression("ENTIRE_WORKTOP")
  None;
`;
};
