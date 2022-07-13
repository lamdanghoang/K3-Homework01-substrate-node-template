#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::inherent::Vec;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[derive(TypeInfo, Default, Decode, Encode)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		dna: Vec<u8>,
		owner: T::AccountId,
		price: u32,
		gender: Gender,
	}

	pub type Id = u32;

	#[derive(TypeInfo, Decode, Encode)]
	pub enum Gender {
		Male,
		Female,
	}

	impl Default for Gender {
		fn default() -> Self {
			Gender::Male
		}
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn number_kitties)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type NumberKitties<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub(super) type Owner<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters.
		KittyStored(Vec<u8>, u32),

		KittyTransferedTo(T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		InvalidPrice,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		KittyNotOwned,

		InvalidAccount,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn create_kitty(origin: OriginFor<T>, dna: Vec<u8>, price: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			ensure!(price > 0, Error::<T>::InvalidPrice);

			let gender = Self::get_gender(dna.clone())?;

			let kitty = Kitty::<T> { dna: dna.clone(), owner: who.clone(), price, gender };

			let mut current_id = <NumberKitties<T>>::get();

			current_id += 1;

			<NumberKitties<T>>::put(current_id);

			<Kitties<T>>::insert(dna.clone(), kitty);

			// let mut vec_kitties = <Owner<T>>::get(who.clone());

			match <Owner<T>>::get(who.clone()) {
				Some(mut x) => {
					x.push(dna.clone());
					<Owner<T>>::insert(who, x);
				},
				None => {
					let mut vec_kitties = Vec::new();
					vec_kitties.push(dna.clone());
					<Owner<T>>::insert(who, vec_kitties);
				},
			}

			// Emit an event.
			Self::deposit_event(Event::KittyStored(dna, price));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_kitty(
			origin: OriginFor<T>,
			account: T::AccountId,
			dna: Vec<u8>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;
			// Get kitties of sender
			let mut sender_kitties = <Owner<T>>::get(who.clone()).unwrap();
			// Check ownership of kitty
			ensure!(sender_kitties.contains(&dna), Error::<T>::KittyNotOwned);
			// Get kitties of receiver
			let mut receiver_kitties = <Owner<T>>::get(account.clone()).unwrap_or(Vec::new());
			// Transfer kitty from sender to receiver
			sender_kitties.retain(|x| x != &dna);
			receiver_kitties.push(dna.clone());
			<Owner<T>>::insert(who, sender_kitties);
			<Owner<T>>::insert(account.clone(), receiver_kitties);
			// Update information of kitty
			let mut kitty_update = <Kitties<T>>::get(dna.clone()).unwrap();
			kitty_update.owner = account.clone();
			<Kitties<T>>::insert(dna.clone(), kitty_update);

			// Emit an event.
			Self::deposit_event(Event::KittyTransferedTo(account, dna));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}
}

//helper function
impl<T> Pallet<T> {
	fn get_gender(dna: Vec<u8>) -> Result<Gender, Error<T>> {
		let mut res = Gender::Male;
		if dna.len() % 2 != 0 {
			res = Gender::Female;
		}
		Ok(res)
	}
}
