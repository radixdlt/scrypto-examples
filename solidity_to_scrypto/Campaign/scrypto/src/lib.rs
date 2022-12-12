use scrypto::prelude::*;

// Define a part of the ABI of the account component
// so that we can easily call its deposit method.
external_component! {
    AccountComponentTarget {
        fn deposit(&mut self, bucket: Bucket);
    }
}

// Define the data attached to the member NFTs.
// Right now its empty but you could add fields.
// ** Access control can be done at the platform-level which is very
// ** different from Solidity. Please read through this part of the documentation
// ** for more information: https://docs.radixdlt.com/main/scrypto/scrypto-lang/access-control/access-introduction.html 
#[derive(NonFungibleData)]
struct MemberData {

}

// Define the structure of the requests.
// The `scrypto(TypeId, Encode, Decode, Describe)` makes this structure
// compatible with the Radix network so that we can store instantiations 
// of it in the component's state
#[scrypto(TypeId, Encode, Decode, Describe)]
struct Request {
    description: String,
    amount: Decimal,
    recipient: ComponentAddress,
    complete: bool,
    approvals: BTreeSet<NonFungibleId>  
}

blueprint! {
    struct Campaign {
        // This vault will contain the badge allowed to mint more member NFTs
        // ** What is a badge: https://docs.radixdlt.com/main/scrypto/scrypto-lang/access-control/access-introduction.html#_what_is_a_badge
        member_minting_authority: Vault,
        // Store the address of the member NFT resource
        member_badge_address: ResourceAddress,
        requests: Vec<Request>,
        // This is the ID of the NFT badge owned by the manager
        manager_id: NonFungibleId,
        minimum_contribution: Decimal,
        approvers_count: u32,
        // We need a vault to hold the contributions unlike on Solidity
        // where you store the tokens on the component itself
        contributions: Vault
    }

    impl Campaign {
        pub fn create_campaign(minimum_contribution: Decimal) -> (ComponentAddress, Bucket) {
            // Create the badge that will need to be shown in order to mint member NFT badges.
            // This will be stored in the component's `member_minting_authority` vault.
            let member_badge_minter_authority = ResourceBuilder::new_fungible()
                .divisibility(DIVISIBILITY_NONE)
                .initial_supply(1);
            
            // Define the ID of the manager NFT
            let manager_member_id = NonFungibleId::random();

            // Create the resource used to authenticate members.
            // The resource is set as mintable and it is initialized with an initial supply
            // for the manager.
            // For more information please read the "user badge pattern" page: https://docs.radixdlt.com/main/scrypto/design-patterns/user-badge-pattern.html
            let manager_member_badge = ResourceBuilder::new_non_fungible(NonFungibleIdType::UUID)
                .metadata("name", "Campaign Member Badge")
                .metadata("symbol", "CMB")
                // Read this documentation page to learn more about the resource access control flags:
                // https://docs.radixdlt.com/main/scrypto/scrypto-lang/access-control/access-resources.html
                .mintable(rule!(require(member_badge_minter_authority.resource_address())), LOCKED)
                .initial_supply(vec![
                    (manager_member_id.clone(), MemberData{ })
                ]);
            
            // Instantiate our component
            let component = Self {
                member_minting_authority: Vault::with_bucket(member_badge_minter_authority),
                member_badge_address: manager_member_badge.resource_address(),
                
                requests: Vec::new(),
                manager_id: manager_member_id,
                minimum_contribution: minimum_contribution,
                approvers_count: 0,

                contributions: Vault::new(RADIX_TOKEN)
            }
            .instantiate()
            .globalize(); // Globalizing our component gives it a public component address so that anyone can call it.

            // Return the component address and the manager_member_badge back to the caller.
            (component, manager_member_badge)
        }

        // In Scrypto, we accept tokens through Buckets. Users take tokens from their account,
        // put them in a Bucket and send that Bucket to this method.
        // We are returning a Bucket which will contain a Member NFT badge that we will mint
        pub fn contribute(&mut self, contribution: Bucket) -> Bucket {
            assert!(contribution.amount() > self.minimum_contribution, "Need to provide bigger contribution!");
            
            // Insert the contribution in the component's contributions vault
            self.contributions.put(contribution);

            // Update the approvers_count
            self.approvers_count += 1;

            // Mint a member badge and send it back to the caller.
            // Remember that we need to show ownership of the "member_minting_authority" badge
            // That's why we do `.authorize()` here.
            self.member_minting_authority.authorize(|| {
                borrow_resource_manager!(self.member_badge_address)
                    .mint_non_fungible(&NonFungibleId::random(), MemberData{ })
            })
        }

        // The last parameter, which is a Proof, allows users to prove that
        // they own a particular member badge. This is to make sure unauthorized people
        // cannot call it.
        pub fn create_request(&mut self, 
                                description: String, 
                                amount: Decimal, 
                                recipient: ComponentAddress, 
                                member_proof: Proof) {
            // Verify that the caller is in fact a member of the dApp
            member_proof.validate_proof(self.member_badge_address).expect("Wrong member badge provided!");
        
            self.requests.push(Request{
                description,
                amount,
                recipient,
                complete: false,
                approvals: BTreeSet::new()
            });
        }

        pub fn approve_request(&mut self, index: usize, member_proof: Proof) {
            // Verify that the caller is in fact a member of the dApp
            let member_proof = member_proof.validate_proof(self.member_badge_address).expect("Wrong member badge provided!");
            // Find out the ID of the NFT that the caller sent a proof of
            let non_fungible: NonFungible<MemberData> = member_proof.non_fungible::<MemberData>();
            let caller_id = non_fungible.id();

            let request = self.requests.get_mut(index).expect("Request not found!");
            assert!(!request.approvals.contains(&caller_id), "You already voted for this proposal!");

            request.approvals.insert(caller_id.clone());
        }

        pub fn finalize_request(&mut self, index: usize, member_proof: Proof) {
            // Make sure the caller is the manager
            let member_proof = member_proof.validate_proof(self.member_badge_address).expect("Wrong member badge provided!");
            let non_fungible: NonFungible<MemberData> = member_proof.non_fungible::<MemberData>();
            let caller_id = non_fungible.id();
            assert_eq!(*caller_id, self.manager_id, "You are not the manager!");

            let mut request = self.requests.get_mut(index).expect("Request not found!");
            assert!(request.approvals.len() as u32 > (self.approvers_count / 2), "Not enough approvals");
            assert!(!request.complete, "The request is already completed");

            // Transfer the funds directly. Note that this is insecure and the `withdraw pattern`
            // should be used instead! https://docs.radixdlt.com/main/scrypto/design-patterns/withdraw-pattern.html
            let funds: Bucket = self.contributions.take(request.amount);
            AccountComponentTarget::at(request.recipient).deposit(funds);

            request.complete = true;
        }

        pub fn get_summary(&self) -> (Decimal, Decimal, usize, u32, NonFungibleId) {
            (self.minimum_contribution, 
                self.contributions.amount(), 
                self.requests.len(), 
                self.approvers_count,
                self.manager_id.clone())
        }
    }
}