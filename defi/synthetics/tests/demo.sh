#!/usr/bin/env bash
set -x
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# ======================================================
# Setting up the envirnoment and deploying the packages
# ======================================================

resim reset

OP1=$(resim new-account)
export acc1_private_key=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export acc1_public_key=$(echo "$OP1" | sed -nr "s/Public key: ([[:alnum:]_]+)/\1/p")
export acc1_account=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export btc=$(echo "$(resim new-token-fixed --name Bitcoin --symbol BTC 100000000)" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
export usd=$(echo "$(resim new-token-fixed --name Tether --symbol USDT 100000000)" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
export snx=$(echo "$(resim new-token-fixed --name Synthetics --symbol SNX 100000000)" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")

export oracle_package=$(resim publish "$SCRIPT_DIR/../../price-oracle" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
sed -i '' 's/package_sim1qx2q6strr8hh203te4e6kkr9rae9pk48rvw0pusyrs6qrrc7t6/package_sim1q8pe9fczej7zhq4ty35q8uvf58h7wj45y3gufz539ysqm7l0ur/' $SCRIPT_DIR/../src/lib.rs

CP_OP=$(resim call-function $oracle_package PriceOracle instantiate_oracle 1)
export oracle_component=$(echo "$CP_OP" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
export oracle_admin=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p")
echo $oracle_component
echo $oracle_admin

echo "CALL_METHOD ComponentAddress(\"$acc1_account\") \"lock_fee\" Decimal(\"100\");" > tx.rtm 
echo "CALL_METHOD ComponentAddress(\"$acc1_account\") \"create_proof\" ResourceAddress(\"$oracle_admin\");" >> tx.rtm 
echo "CALL_METHOD ComponentAddress(\"$oracle_component\") \"update_price\" ResourceAddress(\"$snx\") ResourceAddress(\"$usd\") Decimal(\"12\");" >> tx.rtm 
echo "CALL_METHOD ComponentAddress(\"$oracle_component\") \"update_price\" ResourceAddress(\"$btc\") ResourceAddress(\"$usd\") Decimal(\"12\");" >> tx.rtm 
echo "CALL_METHOD ComponentAddress(\"$oracle_component\") \"update_price\" ResourceAddress(\"$snx\") ResourceAddress(\"$btc\") Decimal(\"12\");" >> tx.rtm 
resim run tx.rtm
rm tx.rtm

export synthetics_package=$(echo "$(resim publish "$SCRIPT_DIR/..")" | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
export synthetics_component=$(echo "$(resim call-function $synthetics_package SyntheticPool instantiate_pool $oracle_component $snx $usd 1.2)" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
echo $synthetics_component

# ================
# Test synthetics
# ================

# Create a Synthetics account
user1=`resim call-method $synthetics_component new_user | tee /dev/tty | awk '/Resource:/ {print $NF}'`

# Stake 1000 SNX
vault_badge=`resim call-method $synthetics_component stake 1,$user1 1000,$snx | tee /dev/tty | awk '/Resource:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1

# Unstake 200 SNX
resim call-method $synthetics_component unstake 1,$user1 200
resim call-method $synthetics_component get_user_summary $user1

# Add sBTC synth
sbtc=`resim call-method $synthetics_component add_synthetic_token "BTC" $btc | tee /dev/tty | awk '/Resource:/ {print $NF}'`
resim call-method $synthetics_component get_user_summary $user1

# Mint 0.01 sBTC
resim call-method $synthetics_component mint 1,$user1 0.01 "BTC"
resim call-method $synthetics_component get_user_summary $user1

# Burn 0.005 sBTC
resim call-method $synthetics_component burn 1,$user1 0.005,$sbtc
resim call-method $synthetics_component get_user_summary $user1