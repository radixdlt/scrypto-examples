import { get } from 'svelte/store';
import { rdt } from '../store';

// ******** Fetch a transaction preview form the gateway api ********
export const transactionPreview = async (manifest: string) => {
	const gatewayApi = get(rdt)?.gatewayApi;
	if (!gatewayApi) throw new Error('Gateway API not found');

	// Get current epoch
	const status = await gatewayApi.status.getCurrent();
	const currentEpoch = status.ledger_state.epoch;

	// Create transaction preview request (using the input manifest)
	const transactionPreviewRequest = {
		manifest,
		start_epoch_inclusive: currentEpoch,
		end_epoch_exclusive: currentEpoch + 1,
		tip_percentage: 0,
		nonce: Math.round(Math.random() * 10e8),
		signer_public_keys: [],
		flags: {
			use_free_credit: true,
			assume_all_signature_proofs: true,
			skip_epoch_check: true
		}
	};

	console.log('transactionPreviewRequest', transactionPreviewRequest);

	// Fetch transaction preview from gateway api
	const response = await gatewayApi.transaction.innerClient.transactionPreview({
		transactionPreviewRequest
	});

	// Return transaction preview
	return response;
};
