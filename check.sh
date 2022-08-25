#!/bin/bash

set -x
set -e

cd "$(dirname "$0")"

(cd core; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {} --check --quiet)
(cd defi; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {} --check --quiet)
(cd nft; find . -maxdepth 1 -type d \( ! -name . \) -print0 | xargs -0 -n1 -I '{}' scrypto fmt --path {} --check --quiet)

echo "Code format check passed!"