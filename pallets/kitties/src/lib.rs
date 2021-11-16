#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::*};
	use frame_system::{ensure_signed, pallet_prelude::*};
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::{ Randomness, Currency, tokens::ExistenceRequirement },
		transactional
	};
	use scale_info::{TypeInfo, build::NoFields};
	use codec::{Decode, Encode, MaxEncodedLen};

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	use sp_io::hashing::blake2_128;

    type AccountOf<T> = <T as frame_system::Config>::AccountId;
    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	// TODO Part II: Struct for holding Kitty information.

    #[derive(Clone,Encode,Decode,PartialEq,RuntimeDebug,TypeInfo,MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
   	pub struct Kitty<Balance, Account>{
        dna:[u8;16],
        price:Option<Balance>,
        gender:Gender,
        owner:Account,
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
		type MaxKittyOwned:Get<u32>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Handles arithmetic overflow when incrementing the Kitty counter.
		KittyCntOverflow,
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		/// Handles checking that the Kitty is owned by the account transferring, buying or setting a price for it.
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Kitty was successfully created. \[sender, kitty_id\]
		Created(T::AccountId,T::Hash),
		/// Kitty price was successfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId,T::Hash,Option<BalanceOf<T>>),
		/// A Kitty was successfully transferred. \[from, to, kitty_id\]
		Transfer(T::AccountId,T::AccountId,T::Hash),
		/// A Kitty was successfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId,T::AccountId,T::Hash,BalanceOf<T>),
	}

	// ACTION: Storage item to keep a count of all existing Kitties.
	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	pub(super) type KittyCnt<T: Config> = StorageValue<_, u64, ValueQuery>;

	// TODO Part II: Remaining storage items.
	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T:Config> = StorageMap<_,Twox64Concat,T::Hash,Kitty<BalanceOf<T>,AccountOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type OwnerKitties<T:Config> = StorageMap<_,Twox64Concat,T::AccountId,BoundedVec<T::Hash,T::MaxKittyOwned>,ValueQuery>;
	/// Keeps track of what accounts own what Kitty.

	// TODO Part III: Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T:Config>{
		pub kitties:Vec<(T::AccountId,[u8;16],Gender)>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T:Config> GenesisBuild<T> for GenesisConfig<T>{
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be supplied.
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), None);
				// let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO Part III: create_kitty
		#[pallet::weight(1000)]
		pub fn create_kitty(origin:OriginFor<T>)->DispatchResult{
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::mint(&sender,None,None)?;
			log::info!("A kitty is born with ID: {:?}.",kitty_id);
			Ok(())
		}

		// TODO Part III: set_price
		#[pallet::weight(1000)]
		pub fn set_price(origin:OriginFor<T>,kitty_id:T::Hash,new_price:BalanceOf<T>)->DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id,&sender)?,Error::<T>::NotKittyOwner);
			let mut kitty = Self::kitties(&kitty_id).unwrap();
			kitty.price = Some(new_price.clone());
			<Kitties<T>>::insert(kitty_id,kitty);
			Self::deposit_event(<Event<T>>::PriceSet(sender,kitty_id,Some(new_price)));
			Ok(())
		}

		// TODO Part III: transfer
		#[pallet::weight(1000)]
		pub fn transfer(origin:OriginFor<T>,to:T::AccountId,kitty_id:T::Hash)->DispatchResult{
			let from = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			ensure!(Self::is_kitty_owner(&kitty_id, &from)?, <Error<T>>::NotKittyOwner);
			ensure!(from != to,Error::<T>::BuyerIsKittyOwner);
			// Verify the recipient has the capacity to receive one more kitty
			let to_owen = Self::kitties_owned(&to);
			ensure!((to_owen.len() as u32) < T::MaxKittyOwned::get(),Error::<T>::KittyCntOverflow);
			Self::transfer_kitty_to(&kitty_id, &to)?;
			Ok(())
		}



		// TODO Part III: buy_kitty
		#[transactional]
		#[pallet::weight(1000)]
		pub fn buy_kitty(origin:OriginFor<T>,kitty_id:T::Hash,bid_price:BalanceOf<T>)->DispatchResult{
			let buyer = ensure_signed(origin)?;

			let kitty = Self::kitties(kitty_id).ok_or(<Error<T>>::KittyNotExist)?;
			if let Some(ask_price) = kitty.price{
				ensure!(ask_price <= bid_price,Error::<T>::KittyBidPriceTooLow);
			}else{
				Err(Error::<T>::KittyNotForSale)?;
			}
			ensure!(bid_price<=T::Currency::free_balance(&buyer),Error::<T>::NotEnoughBalance);
			// Verify the buyer has the capacity to receive one more kitty
			let to_owned = <OwnerKitties<T>>::get(&buyer);
			ensure!((to_owned.len() as u32)<T::MaxKittyOwned::get(),Error::<T>::KittyCntOverflow);
			let seller = kitty.owner.clone();
			ensure!(buyer!=seller,<Error<T>>::BuyerIsKittyOwner);
			// Transfer the amount from buyer to seller
			T::Currency::transfer(&buyer,&seller,bid_price,ExistenceRequirement::KeepAlive)?;
			Self::transfer_kitty_to(&kitty_id,&buyer)?;
			Self::deposit_event(Event::<T>::Bought(buyer,seller,kitty_id,bid_price));
			Ok(())
		}

		#[pallet::weight(1000)]
		pub fn breed_kitty(origin:OriginFor<T>,parent1:T::Hash,parent2:T::Hash)->DispatchResult{
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&parent1,&sender)?,Error::<T>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&parent2,&sender)?,Error::<T>::NotKittyOwner);
			let new_dna = Self::breed_dna(&parent1,&parent2)?;
			Self::mint(&sender,Some(new_dna),None)?;
			Ok(())
		}
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

        // ACTION #6: function to randomly generate DNA
		fn gen_dna()->[u8;16]{
			let payload = (
				T::KittyRandomness::random(b"dna").0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		pub fn breed_dna(parent1:&T::Hash,parent2:&T::Hash)->Result<[u8;16],Error<T>>{
			let dna1 = Self::kitties(parent1).ok_or(Error::<T>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(parent2).ok_or(Error::<T>::KittyNotExist)?.dna;
			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len(){
				new_dna[i] = (new_dna[i]&dna1[i])|(!new_dna[i]&dna2[i]);
			}
			Ok(new_dna)
		}

		pub fn is_kitty_owner(kitty_id:&T::Hash,acct:&T::AccountId)->Result<bool,Error<T>>{
			match Self::kitties(kitty_id){
				Some(kitty)=>Ok(kitty.owner == *acct),
				None=>Err(<Error<T>>::KittyNotExist),
			}
		}

		#[transactional]
		pub fn transfer_kitty_to(kitty_id:&T::Hash,to:&T::AccountId)->Result<(),Error<T>>{
			let mut kitty = Self::kitties(kitty_id).ok_or(Error::<T>::KittyNotExist)?;
			let pre_owner = kitty.owner;
			// Remove `kitty_id` from the KittyOwned vector of `prev_kitty_owner`
			<OwnerKitties<T>>::try_mutate(&pre_owner,|owened|{
				if let Some(ind) = owened.iter().position(|&id|id==*kitty_id){
					owened.remove(ind);
					return Ok(());
				};
				Err(Error::<T>::KittyNotExist)
			})?;

			kitty.owner = to.clone();
			kitty.price = None;
			Kitties::<T>::insert(kitty_id,kitty);
			<OwnerKitties<T>>::try_mutate(to,|vec|{
				vec.try_push(*kitty_id)
			}).map_err(|_|Error::<T>::ExceedMaxKittyOwned)?;
			Self::deposit_event(Event::<T>::Transfer(pre_owner,to.clone(),*kitty_id));
			Ok(())
		}

		#[transactional]
		pub fn mint(owner:&T::AccountId,dna:Option<[u8;16]>,gender:Option<Gender>)->Result<T::Hash,Error<T>>{
			let kitty = Kitty::<BalanceOf<T>,AccountOf<T>>{
				dna:dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender:gender.unwrap_or_else(Self::gen_gender),
				owner:owner.clone()
			};
			let kitty_id = T::Hashing::hash_of(&kitty);
			let new_cnt = Self::kitty_cnt().checked_add(1).ok_or(<Error<T>>::KittyCntOverflow)?;
			<OwnerKitties<T>>::try_mutate(&owner,|kitty_vec|
				kitty_vec.try_push(kitty_id)
			).map_err(|_|Error::<T>::ExceedMaxKittyOwned)?;
			<Kitties<T>>::insert(kitty_id,kitty);
			<KittyCnt<T>>::put(new_cnt);
			Self::deposit_event(Event::<T>::Created(owner.clone(),kitty_id));
			Ok(kitty_id)
		}
    }
}
