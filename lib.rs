#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod hotel_ink {
    use ink::prelude::vec::Vec;
    use ink::prelude::string::String;
    use ink::storage::Mapping;
    use ink::prelude::string::ToString;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use num_traits::cast::ToPrimitive;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct HotelInk {
        /// Mapping of guests
        guests: Mapping<u32, Guest>,
        rooms: Vec<u32>,
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
        pub payment: PaymentMethod,
        pub checkin: u64,
        pub room: u32
    }

    impl HotelInk {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn default() -> Self {
            let mut rooms = Vec::new();

            for room_number in 1..=20 {
                rooms.push(room_number);
            }

            Self {
                guests: Mapping::new(),
                rooms: rooms
            }
        }

        /// Adds a guest to the `guests` mapping
        #[ink(message)]
        pub fn add_guest(
            &mut self,
            id: u32,
            name: String,
            email: String,
            payment: PaymentMethod,
            room: u32
            ) -> Result<(), String> {
            if !self.rooms.contains(&room) {
                return Err("Room not available".to_string());
            }

            if id <= 99999999 || id > 999999999 {
                return Err("Invalid RG".to_string());
            }

            if name.trim().is_empty() {
                return Err("Empty name".to_string());
            }

            if email.trim().is_empty() {
                return Err("Empty email!".to_string());
            }
            
            let checkin = self.env().block_timestamp();

            let new = Guest {
                id,
                name,
                email,
                payment,
                checkin,
                room
            };
            self.guests.insert(id, &new);
            self.rooms.retain(|&r| r != room);
            return Ok(())
        }

        #[ink(message)]
        pub fn get_guest(&self, id: u32) -> Option<(u32, String, String, PaymentMethod, String, u32)> {
            self.guests.get(id).map(|guest| {
                let datetime = Self::convert_timestamp(guest.checkin);
                (guest.id, guest.name, guest.email, guest.payment, datetime, guest.room)
            })
        }

        #[ink(message)]
        pub fn update_guest(
            &mut self,
            id: u32,
            new_id: Option<u32>,
            name: Option<String>,
            email: Option<String>,
            payment: Option<PaymentMethod>,
            room: Option<u32>
            ) -> Result<bool, String> {
            if let Some(mut guest) = self.guests.get(id) {
                if let Some(new_id) = new_id {
                    if id <= 99999999 || id > 999999999 {
                        return Err("Invalid RG".to_string());
                    }

                    guest.id = new_id;
                    self.guests.remove(id);
                }

                if let Some(name) = name {
                    if name.trim().is_empty() {
                        return Err("Empty name".to_string());
                    }

                    guest.name = name;
                }

                if let Some(email) = email {
                    if email.trim().is_empty() {
                        return Err("Empty email!".to_string());
                    }

                    guest.email = email;
                }

                if let Some(payment) = payment {
                    guest.payment = payment;
                }

                if let Some(room) = room {
                    if !self.rooms.contains(&room) {
                        return Err("Room not available".to_string());
                    }
                    
                    self.rooms.retain(|&r| r != room);
                    self.rooms.push(guest.room);
                    guest.room = room;
                }

                self.guests.insert(guest.id, &guest);

                Ok(true)
            } else {
                Err("Guest not found".to_string())
            }
        }

        #[ink(message)]
        pub fn delete_guest(&mut self, id: u32) -> Result<bool, String> {
            if let Some(guest) = self.guests.get(id) {
                self.guests.remove(id);
                self.rooms.push(guest.room);
                Ok(true)
            } else {
                Err("Guest not found".to_string())
            }
        }

        /// Converte timestamp (u64) para string no formato ISO 8601
        fn convert_timestamp(timestamp: u64) -> String {
            // Converte u64 para i64 de forma segura, coloca pra s em vez de ms,
            // bota o fuso BR
            let timestamp_i64 = (timestamp/1000).to_i64().unwrap_or(0);
            let timestamp_i64 = timestamp_i64.checked_sub(3 * 3600).unwrap_or(0);
            let naive = NaiveDateTime::from_timestamp_opt(timestamp_i64, 0)
                .unwrap_or(NaiveDateTime::from_timestamp(0, 0));
            let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
            datetime.to_rfc3339() // Retorna "YYYY-MM-DDTHH:MM:SS+00:00"
        }

        #[ink(message)]
        pub fn list_available_rooms(&self) -> Vec<u32> {
            self.rooms.clone()
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
