set -x 

resim reset

OP=$(resim new-account)
export privkey=$(echo "$OP" | sed -nr "s/Private key: ([[:alnum:]_]+)/\1/p")
export account=$(echo "$OP" | sed -nr "s/Account component address: ([[:alnum:]_]+)/\1/p")

export package=$(resim publish . | sed -nr "s/Success! New Package: ([[:alnum:]_]+)/\1/p")

CP_OP=$(resim call-function $package GumballMachine instantiate_gumball_machine 10)
export component=$(echo "$CP_OP" | sed -nr "s/.*Component: ([[:alnum:]_]+)/\1/p")
export gumball=$(echo "$CP_OP" | sed -nr "s/.*Resource: ([[:alnum:]_]+)/\1/p" | sed '1!d')

resim call-method $component buy_gumball 100,resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag