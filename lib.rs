#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod hotel_ink {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::prelude::string::ToString;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct HotelInk {
        /// Mapping of guests
        guests: Mapping<u32, Guest>,
    }

    #[derive(scale::Encode, scale::Decode, Default, Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    #[allow(clippy::cast_possible_truncation)]
    pub enum PaymentMethod {
        #[default]
        Credit,
        Cash,
        Pix
    }
    
    #[derive(scale::Encode, scale::Decode, Default, Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Guest {
        pub id: u32,
        pub name: String,
        pub email: String,
        pub payment: PaymentMethod
    }

    impl HotelInk {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                guests: Mapping::new()
            }
        }

        /// Adds a guest to the `guests` mapping
        #[ink(message)]
        pub fn add_guest(
            &mut self,
            id: u32,
            name: String,
            email: String,
            payment: PaymentMethod
            ) {
            let new = Guest {
                id,
                name,
                email,
                payment
            };
            self.guests.insert(id, &new);   
        }

        #[ink(message)]
        pub fn get_guest(&self, id: u32) -> Option<Guest> {
            self.guests.get(id)
        }

        #[ink(message)]
        pub fn update_guest(
            &mut self,
            id: u32,
            new_id: Option<u32>,
            name: Option<String>,
            email: Option<String>,
            payment: Option<PaymentMethod>
            ) -> Result<bool, String> {
            if let Some(mut guest) = self.guests.get(id) {
                if let Some(new_id) = new_id {
                    guest.id = new_id;
                    self.guests.remove(id);
                }

                if let Some(name) = name {
                    guest.name = name;
                }

                if let Some(email) = email {
                    guest.email = email;
                }

                if let Some(payment) = payment {
                    guest.payment = payment;
                }

                self.guests.insert(guest.id, &guest);

                Ok(true)
            } else {
                Err("Guest not found".to_string())
            }
        }

        #[ink(message)]
        pub fn delete_guest(&mut self, id: u32) -> Result<bool, String> {
            if let Some(_guest) = self.guests.get(id) {
                self.guests.remove(id);
                Ok(true)
            } else {
                Err("Guest not found".to_string())
            }
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = HotelInkRef::default();

            // When
            let contract = client
                .instantiate("hotel_ink", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<HotelInk>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = HotelInkRef::new(false);
            let contract = client
                .instantiate("hotel_ink", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<HotelInk>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
