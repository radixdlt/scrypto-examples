set -x

resim reset

# This is the account which we will be using for everything throughout this example run. We could create more accounts 
# but this would just add more uncecessary complexity.
OP1=$(resim new-account)
export privkey1=$(echo "$OP1" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export account1=$(echo "$OP1" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

# Creating the tokens which will be used throughout this examople
export admin_badge1=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export admin_badge2=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

export user_badge1=$(resim new-token-fixed 20 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export user_badge2=$(resim new-token-fixed 20 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')
export user_badge3=$(resim new-token-fixed 1 | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

# The instantiation transaction
echo "CALL_FUNCTION PackageAddress(\"$package\") \"LimitedWithdrawVault\" \"instantiate_custom_limited_withdraw_vault\" Enum(\"Protected\", Enum(\"AllOf\", Vec<Enum>(Enum(\"ProofRule\", Enum(\"Require\", Enum(\"StaticResource\", ResourceAddress(\"$admin_badge1\")))), Enum(\"ProofRule\", Enum(\"Require\", Enum(\"StaticResource\", ResourceAddress(\"$admin_badge2\"))))))) ResourceAddress(\"030000000000000000000000000000000000000000000000000004\");
CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$account1\") \"deposit_batch\";
" > transactions/component_creation.rtm
CP_OP=$(resim run transactions/component_creation.rtm)
export component=$(echo "$CP_OP" | sed -nr "s/└─ Component: ([[:alnum:]_]+)/\1/p")

# Depositing funds into the component
resim call-method $component deposit 1000000,030000000000000000000000000000000000000000000000000004

# Building tx manifest for the adding of entities
echo "CALL_METHOD ComponentAddress(\"$account1\") \"create_proof\" ResourceAddress(\"$admin_badge1\");
CALL_METHOD ComponentAddress(\"$account1\") \"create_proof\" ResourceAddress(\"$admin_badge2\");

CALL_METHOD ComponentAddress(\"$component\") \"add_withdraw_authority\" Enum(\"Protected\", Enum(\"AllOf\", Vec<Enum>(Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"20\")), Enum(\"Static\", ResourceAddress(\"$user_badge1\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"15\")), Enum(\"Static\", ResourceAddress(\"$user_badge2\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"1\")), Enum(\"Static\", ResourceAddress(\"$user_badge3\"))))))) Enum(\"Finite\", Decimal(\"1000\"));
CALL_METHOD ComponentAddress(\"$component\") \"add_withdraw_authority\" Enum(\"Protected\", Enum(\"AllOf\", Vec<Enum>(Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"20\")), Enum(\"Static\", ResourceAddress(\"$user_badge1\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"12\")), Enum(\"Static\", ResourceAddress(\"$user_badge2\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"1\")), Enum(\"Static\", ResourceAddress(\"$user_badge3\"))))))) Enum(\"Finite\", Decimal(\"500\"));
CALL_METHOD ComponentAddress(\"$component\") \"add_withdraw_authority\" Enum(\"Protected\", Enum(\"AllOf\", Vec<Enum>(Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"20\")), Enum(\"Static\", ResourceAddress(\"$user_badge1\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"10\")), Enum(\"Static\", ResourceAddress(\"$user_badge2\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"1\")), Enum(\"Static\", ResourceAddress(\"$user_badge3\"))))))) Enum(\"Finite\", Decimal(\"100\"));
CALL_METHOD ComponentAddress(\"$component\") \"add_withdraw_authority\" Enum(\"Protected\", Enum(\"AllOf\", Vec<Enum>(Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"20\")), Enum(\"Static\", ResourceAddress(\"$user_badge1\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"20\")), Enum(\"Static\", ResourceAddress(\"$user_badge2\")))), Enum(\"ProofRule\", Enum(\"AmountOf\", Enum(\"Static\", Decimal(\"1\")), Enum(\"Static\", ResourceAddress(\"$user_badge3\"))))))) Enum(\"Infinite\");

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$account1\") \"deposit_batch\";
" > transactions/adding_withdraw_authorities.rtm
resim run transactions/adding_withdraw_authorities.rtm

# Building tx manifest for the withdrawal of tokens
echo "CALL_METHOD ComponentAddress(\"$account1\") \"create_proof_by_amount\" Decimal(\"20\") ResourceAddress(\"$user_badge1\");
CALL_METHOD ComponentAddress(\"$account1\") \"create_proof_by_amount\" Decimal(\"10\") ResourceAddress(\"$user_badge2\"); 
CALL_METHOD ComponentAddress(\"$account1\") \"create_proof_by_amount\" Decimal(\"1\") ResourceAddress(\"$user_badge3\"); 

CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"$user_badge1\") Proof(\"Proof1\");
CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"$user_badge2\") Proof(\"Proof2\");
CREATE_PROOF_FROM_AUTH_ZONE ResourceAddress(\"$user_badge3\") Proof(\"Proof3\");

CALL_METHOD ComponentAddress(\"$component\") \"withdraw\" Decimal(\"100\") Vec<Proof>(Proof(\"Proof1\"), Proof(\"Proof2\"), Proof(\"Proof3\"));

CALL_METHOD_WITH_ALL_RESOURCES ComponentAddress(\"$account1\") \"deposit_batch\";
" > transactions/withdraw_withtin_limit.rtm 
resim run transactions/withdraw_withtin_limit.rtm 