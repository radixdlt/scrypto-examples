set -x

resim reset

OP1=$(resim new-account)
export seller_private_key=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export seller_account=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export bidder1_private_key=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export bidder1_account=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP3=$(resim new-account)
export bidder2_private_key=$(echo "$OP3" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export bidder2_account=$(echo "$OP3" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP4=$(resim new-account)
export bidder3_private_key=$(echo "$OP4" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export bidder3_account=$(echo "$OP4" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

# Creating a number of test NFTs through the Bootstrap blueprint
CP_OP=$(resim call-function $package Bootstrap bootstrap)
export cars_nft=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export phones_nft=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '2!d')
export laptops_nft=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')

# Selling three car NFTs and one laptop NFTs
echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"3\") ResourceAddress(\"$cars_nft\");
CALL_METHOD ComponentAddress(\"$seller_account\") \"withdraw_by_amount\" Decimal(\"1\") ResourceAddress(\"$phones_nft\");

TAKE_FROM_WORKTOP ResourceAddress(\"$cars_nft\") Bucket(\"bucket1\");
TAKE_FROM_WORKTOP ResourceAddress(\"$phones_nft\") Bucket(\"bucket2\");

CALL_FUNCTION 
    PackageAddress(\"$package\") 
    \"EnglishAuction\" 
    \"instantiate_english_auction\" 
    Vec<Bucket>(Bucket(\"bucket1\"), Bucket(\"bucket2\"))
    ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\")
    50u64;

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/english_auction_listing.rtm
CP_OP=$(resim run transactions/english_auction_listing.rtm)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
export ownership_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export bidders_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '3!d')

# Switching to the bidders accounts and making bids
resim set-default-account $bidder1_account $bidder1_private_key
resim call-method $component "bid" 1000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

resim set-default-account $bidder2_account $bidder2_private_key
resim call-method $component "bid" 2000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

resim set-default-account $bidder3_account $bidder3_private_key
resim call-method $component "bid" 3000,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag

# Passing the epochs and then withdrawing the NFTs using the winner's account
resim set-current-epoch 50
resim call-method $component "claim_nfts" 1,$bidders_badge
resim show $bidder3_account

# Switching to the other bidder's accounts to get their bids back
resim set-default-account $bidder1_account $bidder1_private_key
resim call-method $component "cancel_bid" 1,$bidders_badge

resim set-default-account $bidder2_account $bidder2_private_key
resim call-method $component "cancel_bid" 1,$bidders_badge

# Switching to the seller's account and withdrawing the funds
resim set-default-account $seller_account $seller_private_key

echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"create_proof\" ResourceAddress(\"$ownership_badge\");
CALL_METHOD ComponentAddress(\"$component\") \"withdraw_payment\";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/english_auction_withdraw_payment.rtm
resim run transactions/english_auction_withdraw_payment.rtm
resim show $seller_account