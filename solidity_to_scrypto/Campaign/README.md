# Campaign smart contract

## How to test
1. Go into the scrypto directory: `cd scrypto`
1. Reset the simulator: `resim reset`
1. Create the manager account: `resim new-account` -> Store the account address and private key somewhere
1. Create the member account: `resim new-account` -> Store the account address and private key somewhere
1. Build and deploy the blueprint: `resim publish .`
1. Instantiate a Campaign component: `resim call-function [package_address] Campaign create_campaign 10` -> Store the component address and the second resource address (member badge address)
1. Switch to the second account: `resim set-default-account [account2_address] [account2_priv_key]`
1. Contribute 11 XRD to become a member: `resim call-method [component_address] contribute 11,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag`
1. Create a request to send 10 XRD to the manager: `resim call-method [component_address] create_request "hello" 10 [account1_address] 1,[member_badge_address]`
1. Switch back to the first account: `resim set-default-account [account1_address] [account1_priv_key]`
1. Approve the request: `resim call-method [component_address] approve_request 0 1,[member_badge_address]`
1. Finalize the request: `resim call-method [component_address] finalize_request 0 1,[member_badge_address]`
1. Look at the resource in account1: `resim show [account1_address]`. It should show 1010 XRD!