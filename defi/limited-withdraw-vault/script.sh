./build_rtm.sh
clear

resim reset

OP1=$(resim new-account)
export privkey1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export account1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export admin_badge1=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export admin_badge2=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

export employee_badge=$(resim new-token-fixed 20 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export manager_badge=$(resim new-token-fixed 20 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export executive_badge=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")
CP_OP=$(resim run transactions/component_creation.rtm)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")

resim run ./transactions/component_creation.rtm
resim call-method $component deposit 1000000,030000000000000000000000000000000000000000000000000004
resim run ./transactions/adding_withdraw_authorities.rtm
resim run ./transactions/withdraw_within_limit.rtm
resim run transactions/remove_withdraw_auth.rtm
resim run transactions/withdraw_after_authorization_removal.rtm