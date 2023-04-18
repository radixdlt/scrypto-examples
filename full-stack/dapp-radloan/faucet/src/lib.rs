use scrypto::prelude::*;

#[blueprint]
mod faucet {
    struct Faucet {
        token_vault: Vault,
    }

    impl Faucet {
        pub fn instantiate(name: String, symbol: String) -> ComponentAddress {
            let token = ResourceBuilder::new_fungible()
                .metadata("name", name)
                .metadata("symbol", symbol)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .mintable(AccessRule::AllowAll, LOCKED)
                .mint_initial_supply(10000);
            Self {
                token_vault: Vault::with_bucket(token),
            }
            .instantiate()
            .globalize()
        }

        pub fn free_tokens(&mut self) -> Bucket {
            let free_tokens = self.token_vault.take_all();
            let token_minter = borrow_resource_manager!(self.token_vault.resource_address());
            self.token_vault.put(token_minter.mint(10000));
            free_tokens
        }
    }
}
