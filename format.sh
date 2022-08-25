#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd core; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {})
(cd defi; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {})
(cd nft; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {})

echo "All packages have been formatted."