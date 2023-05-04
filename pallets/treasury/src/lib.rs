// Copyright (C) SaaS3.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Treasury Pallet
//!
//! The Treasury pallet provides a "pot" of funds that can be managed by stakeholders in the system
//! and a structure for making spending claims from this pot.
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The Treasury Pallet itself provides the pot to store funds, and a means for jury to
//! claim their rewards. The chain will need to provide a method (e.g.
//! inflation, fees) for collecting funds.
//!
//! ### Terminology
//! - **Beneficiary:** An account who will receive the funds from a rewards claim iff the claim is
//!   approved.
//! - **Pot:** Unspent funds accumulated by the treasury pallet.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! General spending/proposal protocol:
//! - `receive` - Receiving funds from outside
//! - `claim_rewards` - Jury claim their rewards
//!
//! ## GenesisConfig
//!
//! The Treasury pallet depends on the [`GenesisConfig`].

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_runtime::{
	traits::{AccountIdConversion, Saturating},
	RuntimeDebug,
};
use sp_std::prelude::*;

use frame_support::{
	traits::{Currency, ExistenceRequirement::KeepAlive, Get, ReservableCurrency},
	weights::Weight,
	PalletId,
};

pub use pallet::*;

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// An index of a record. Just a `u32`.
pub type RecordIndex = u32;

#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Record<AccountId, Balance> {
	/// The account fund it
	funder: AccountId,
	/// The amount that funder send
	value: Balance,
	category_type: u32,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_court::Config<I> {
		/// The staking balance.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The treasury's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn record_count)]
	pub(crate) type RecordCount<T, I = ()> = StorageValue<_, RecordIndex, ValueQuery>;

	/// Proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn records)]
	pub type Records<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		RecordIndex,
		Record<T::AccountId, BalanceOf<T, I>>,
		OptionQuery,
	>;

	/// Claims that have been made
	#[pallet::storage]
	#[pallet::getter(fn claims)]
	pub type Claims<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T, I>, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig;

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self
		}
	}

	#[cfg(feature = "std")]
	impl GenesisConfig {
		/// Direct implementation of `GenesisBuild::assimilate_storage`.
		#[deprecated(
			note = "use `<GensisConfig<T, I> as GenesisBuild<T, I>>::assimilate_storage` instead"
		)]
		pub fn assimilate_storage<T: Config<I>, I: 'static>(
			&self,
			storage: &mut sp_runtime::Storage,
		) -> Result<(), String> {
			<Self as GenesisBuild<T, I>>::assimilate_storage(self, storage)
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> GenesisBuild<T, I> for GenesisConfig {
		fn build(&self) {
			// Create Treasury account
			let account_id = <Pallet<T, I>>::account_id();
			let min = <T as pallet::Config<I>>::Currency::minimum_balance();
			if <T as pallet::Config<I>>::Currency::free_balance(&account_id) < min {
				let _ = <T as pallet::Config<I>>::Currency::make_free_balance_be(&account_id, min);
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// Some funds have been deposited.
		Deposit {
			value: BalanceOf<T, I>,
		},
		/// A new rewards claim has been approved.
		ClaimApproved {
			amount: BalanceOf<T, I>,
			beneficiary: T::AccountId,
		},
		Spending {
			budget_remaining: BalanceOf<T, I>,
		},
		/// Some funds have been allocated.
		Awarded {
			award: BalanceOf<T, I>,
			account: T::AccountId,
		},
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The spend origin is valid but the amount it is allowed to spend is lower than the
		/// amount to be spent.
		InsufficientPermission,
		/// The remaining fund is less than the claimed amount
		InsufficientFund,
		/// Rewards claim has not been approved.
		ClaimNotApproved,
		ExceedClaim,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		/// ## Complexity
		/// - `O(A)` where `A` is the number of approvals
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Receive funds and save to treasury pot.
		///
		/// - `origin`:
		/// - `amount`: The amount to be transferred from origin to the treasury pot.
		/// - `category_type`: The source type of funds
		///
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn receive(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T, I>,
			category_type: u32,
		) -> DispatchResult {
			let sender = ensure_signed(origin.clone())?;
			ensure!(
				<T as pallet::Config<I>>::Currency::free_balance(&sender) >= amount,
				Error::<T, I>::InsufficientFund
			);
			let to = Self::account_id();
			<T as pallet::Config<I>>::Currency::transfer(&sender, &to, amount, KeepAlive)?;

			// record funds
			let c = Self::record_count();
			<RecordCount<T, I>>::put(c + 1);
			let record = Record { funder: sender, value: amount, category_type };
			<Records<T, I>>::insert(c, record);

			Self::deposit_event(Event::Deposit { value: (amount) });
			Ok(())
		}

		/// Jury claim their rewards. This call will transfer claimed value to beneficiary
		///
		/// - `amount`: The amount to be tranferred to Origin
		///
		/// ## Complexity
		///
		/// Errors:
		/// - `ClaimNotApproved`: The `origin` supplied was not found in the voting list.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn claim_rewards(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T, I>,
		) -> DispatchResult {
			let beneficiary = ensure_signed(origin)?;
			let budget_remaining = Self::pot();
			ensure!(amount <= budget_remaining, Error::<T, I>::InsufficientFund);
			Self::deposit_event(Event::Spending { budget_remaining });

			let account_id = Self::account_id();

			// fetch ended lawsuit
			let cnt = pallet_court::Pallet::<T, I>::contribution(beneficiary.clone());
			let bc = BalanceOf::<T, I>::from(cnt);

			// calculate rewards - claimed rewards
			let c = Claims::<T, I>::get(&beneficiary);
			if let Some(c) = c {
				// there are claims
				ensure!(bc - c > amount, Error::<T, I>::ExceedClaim);
				Claims::<T, I>::mutate(&beneficiary, |v| *v = Some(c + amount));
			} else {
				ensure!(bc > amount, Error::<T, I>::ExceedClaim);
				Claims::<T, I>::insert(&beneficiary, amount);
			}

			<T as pallet::Config<I>>::Currency::transfer(
				&account_id,
				&beneficiary,
				amount,
				KeepAlive,
			)?;
			Self::deposit_event(Event::Awarded { award: amount, account: beneficiary });

			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	// Add public immutables and private mutables.

	/// The account ID of the treasury pot.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Return the amount of money in the pot.
	// The existential deposit is not part of the pot so treasury account never gets deleted.
	pub fn pot() -> BalanceOf<T, I> {
		<T as pallet::Config<I>>::Currency::free_balance(&Self::account_id())
			// Must never be less than 0 but better be safe.
			.saturating_sub(<T as pallet::Config<I>>::Currency::minimum_balance())
	}
}
