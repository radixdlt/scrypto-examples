import { get } from 'svelte/store';
import { poolInfo, rdt } from '../store';

export const getSwapManifest = ({
	resourceAddress,
	amount
}: {
	resourceAddress: string;
	amount: string;
}) => {
	const accountAddress = get(rdt)?.walletApi.getWalletData().accounts[0].address;

	return `
CALL_METHOD
  Address("${accountAddress}")
  "withdraw"
  Address("${resourceAddress}")
  Decimal("${amount}");
TAKE_ALL_FROM_WORKTOP
  Address("${resourceAddress}")
  Bucket("resource_in");
CALL_METHOD
  Address("${get(poolInfo).componentAddress}")
  "swap"
  Bucket("resource_in");
CALL_METHOD
  Address("${accountAddress}")
  "try_deposit_batch_or_abort"
  Expression("ENTIRE_WORKTOP")
  None;
`;
};
