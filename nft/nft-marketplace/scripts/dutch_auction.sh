set -x

resim reset

OP1=$(resim new-account)
export seller_private_key=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export seller_account=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

OP2=$(resim new-account)
export buyer_private_key=$(echo "$OP2" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export buyer_account=$(echo "$OP2" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

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
    \"DutchAuction\" 
    \"instantiate_dutch_auction\" 
    Vec<Bucket>(Bucket(\"bucket1\"), Bucket(\"bucket2\"))
    ResourceAddress(\"resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag\")
    Decimal(\"1000\")
    Decimal(\"500\")
    50u64;

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/dutch_auction_listing.rtm
CP_OP=$(resim run transactions/dutch_auction_listing.rtm)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")
export ownership_badge=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

# Switching to the buyer's account and purchasing the NFTs
resim set-current-epoch 25
resim set-default-account $buyer_account $buyer_private_key
resim call-method $component "buy" 750,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag
resim show $buyer_account

# Switching to the seller's account and withdrawing the payment
resim set-default-account $seller_account $seller_private_key
echo "
CALL_METHOD ComponentAddress(\"$seller_account\") \"create_proof\" ResourceAddress(\"$ownership_badge\");
CALL_METHOD ComponentAddress(\"$component\") \"withdraw_payment\";
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$seller_account\") \"deposit_batch\";
" > transactions/dutch_auction_withdraw_payment.rtm
resim run transactions/dutch_auction_withdraw_payment.rtm
resim show $seller_account