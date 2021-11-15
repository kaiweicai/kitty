#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::{ Randomness, Currency, tokens::ExistenceRequirement },
		transactional
	};
	use scale_info::TypeInfo;
	use codec::{Encode, Decode, MaxEncodedLen};

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
use sp_io::hashing::blake2_128;

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// TODO Part II: Struct for holding Kitty information.
    
    #[derive(Clone,Encode,Decode,PartialEq,RuntimeDebug,TypeInfo,MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Kitty<T:Config>{
        dna:[u8;128],
        price:Option<BalanceOf<T>>,
        gender:Gender,
        owner:AccountOf<T>,
    }

    #[derive(Clone,Encode,Decode,PartialEq,RuntimeDebug,TypeInfo,MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    #[cfg_attr(feature="std", derive(Serialize,Deserialize))]
    pub enum Gender {
        Male,
        Female,
    }

	impl Default for Gender{
		fn default() -> Self {
			Gender::Male
		}
	}

	// TODO Part II: Enum and implementation to handle Gender type in Kitty struct.

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::generate_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		// type Event: Parameter
		// 	+ Member
		// 	+ From<Event<Self>>
		// 	+ Debug
		// 	+ IsType<<Self as frame_system::Config>::Event>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

        type KittyRandomness:Randomness<Self::Hash,Self::BlockNumber>;

		#[pallet::constant]
		type MaxKittyOwened:Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		// TODO Part III
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part III
	}

	// ACTION: Storage item to keep a count of all existing Kitties.
	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// TODO Part II: Remaining storage items.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T:Config> = StorageMap<_,Twox64Concat,T::Hash,Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type OwnerKitties<T:Config> = StorageMap<_,Twox64Concat,T::AccountId,BoundedVec<T::Hash,T::MaxKittyOwened>,ValueQuery>;
	/// Keeps track of what accounts own what Kitty.

	// TODO Part III: Our pallet's genesis configuration.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO Part III: create_kitty

		// TODO Part III: set_price

		// TODO Part III: transfer

		// TODO Part III: buy_kitty

		// TODO Part III: breed_kitty
	}

	impl<T: Config> Pallet<T> {

        // ACTION #4: helper function for Kitty struct
        fn gen_gender()->Gender{
            let random = T::KittyRandomness::random(&b"gender"[..]).0;
            // let ref = random.as_ref();
            match random.as_ref()[0]%2{
                0=>Gender::Male,
                _=>Gender::Female,
            }
        }

        // TODO Part III: helper functions for dispatchable functions

        // ACTION #6: funtion to randomly generate DNA
		fn gen_dna()->[u8;16]{
			let payload = (
				T::KittyRandomness::random(b"dna").0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

        // TODO Part III: mint

        // TODO Part IV: transfer_kitty_to
    }
}
