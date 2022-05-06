use scrypto::crypto::{EcdsaPrivateKey, EcdsaPublicKey};
use scrypto::prelude::*;

use radix_engine::ledger::*;
use radix_engine::model::{Receipt, SignedTransaction};
use radix_engine::transaction::*;

/// A struct which defines the environment used for testing.
pub struct Environment<'a> {
    /// The executor which will be used to run all of the transactions
    pub executor: TransactionExecutor<'a, InMemorySubstateStore>,

    /// This is the address of the package that's currently being tested.
    pub package_address: PackageAddress,

    /// This is the admin account which will be used for the bootstrapping process.
    pub admin_account: Account,

    /// These are the other accounts which were created for testing.
    pub accounts: Vec<Account>,

    /// The resource addresses of the testing NFTs
    pub car_resource_address: ResourceAddress,
    pub phone_resource_address: ResourceAddress,
    pub laptop_resource_address: ResourceAddress,
}

impl<'a> Environment<'a> {
    pub fn new(ledger: &'a mut InMemorySubstateStore, number_of_accounts: u8) -> Self {
        // Setting up the executor from the substate store
        let mut executor: TransactionExecutor<InMemorySubstateStore> =
            TransactionExecutor::new(ledger, false);

        // Publishing the package and getting it's address.
        let package: PackageAddress = executor.publish_package(compile_package!()).unwrap();

        // Creating the admin account
        let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
            executor.new_account();
        let admin_account: Account = Account {
            public_key: pk,
            private_key: sk,
            component_address: account,
        };

        // Creating the required number of accounts
        let accounts: Vec<Account> = (0..number_of_accounts)
            .into_iter()
            .map(|_| {
                let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
                    executor.new_account();
                Account {
                    public_key: pk,
                    private_key: sk,
                    component_address: account,
                }
            })
            .collect::<Vec<Account>>();

        // Making the bootstrap transaction to setup the NFTs
        let bootstrap_tx: SignedTransaction = TransactionBuilder::new()
            .call_function(package, "Bootstrap", "bootstrap", args![])
            .call_method_with_all_resources(admin_account.component_address, "deposit_batch")
            .build(executor.get_nonce([admin_account.public_key]))
            .sign([&admin_account.private_key]);
        let bootstrap_receipt: Receipt = executor.validate_and_execute(&bootstrap_tx).unwrap();
        bootstrap_receipt
            .result
            .expect("Bootstrap transaction failed");

        // Getting the addresses of the newly created NFTs
        let (car, phone, laptop): (ResourceAddress, ResourceAddress, ResourceAddress) = (
            bootstrap_receipt.new_resource_addresses[0],
            bootstrap_receipt.new_resource_addresses[1],
            bootstrap_receipt.new_resource_addresses[2],
        );

        // Creating the test environment
        Self {
            executor: executor,
            package_address: package,
            admin_account: admin_account,
            accounts: accounts,
            car_resource_address: car,
            phone_resource_address: phone,
            laptop_resource_address: laptop,
        }
    }
}

/// A struct which defines the key-pair and component addresses associate with an account component
pub struct Account {
    pub public_key: EcdsaPublicKey,
    pub private_key: EcdsaPrivateKey,
    pub component_address: ComponentAddress,
}

impl Account {
    pub fn new<T: SubstateStore>(executor: &mut TransactionExecutor<T>) -> Self {
        let (pk, sk, account): (EcdsaPublicKey, EcdsaPrivateKey, ComponentAddress) =
            executor.new_account();
        Self {
            public_key: pk,
            private_key: sk,
            component_address: account,
        }
    }
}

// Implements `Clone` for account. Can't simply derive because `EcdsaPrivateKey` does not implement clone.
impl Clone for Account {
    fn clone(&self) -> Self {
        Self {
            public_key: self.public_key.clone(),
            private_key: EcdsaPrivateKey::from_bytes(&self.private_key.to_bytes()[0..32]).unwrap(),
            component_address: self.component_address.clone(),
        }
    }
}

#[test]
pub fn test_env() {
    let mut ledger: InMemorySubstateStore = InMemorySubstateStore::with_bootstrap();
    Environment::new(&mut ledger, 10);
}
