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

//! # Court Pallet
//!
//! The Court pallet provides a set of functions that can allow users to submit lawsuit
//! and jury can vote the lawsuits.
//!
//! - [`Config`]
//! - [`Call`]
//!
//! ## Overview
//!
//! The jury could vote to the lawsuits and approve or reject a lawsuit
//!
//!
//! ### Terminology
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! ## GenesisConfig
//!
//! The Court depends on the [`GenesisConfig`].

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use sp_runtime::{traits::StaticLookup, RuntimeDebug};
use sp_std::prelude::*;

use frame_support::{
	inherent::Vec,
	traits::{Currency, Get, ReservableCurrency},
	weights::Weight,
};

pub use pallet::*;

pub type BalanceOf<T, I = ()> =
	<<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// An index of a proposal. Just a `u32`.
pub type ProposalIndex = u32;

/// A submitted lawsuit
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Lawsuit<AccountId, Balance> {
	/// The account proposing it.
	plaintiff: AccountId,
	/// The (total) amount that should be paid if the proposal is accepted.
	value: Balance,
	/// The account to whom the payment should be made if the proposal is accepted.
	defendent: AccountId,
	statement: Vec<u8>,
	pub voters: Vec<AccountId>,
	votes: Vec<bool>,
	pub approved: bool,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{ensure_root, pallet_prelude::*};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The staking balance.
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum number of approvals that can wait in the spending queue.
		///
		/// NOTE: This parameter is also used within the Bounties Pallet extension if enabled.
		#[pallet::constant]
		type MaxApprovals: Get<u32>;
	}

	/// Number of proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposal_count)]
	pub(crate) type ProposalCount<T, I = ()> = StorageValue<_, ProposalIndex, ValueQuery>;

	/// Proposals that have been made.
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Twox64Concat,
		ProposalIndex,
		Lawsuit<T::AccountId, BalanceOf<T, I>>,
		OptionQuery,
	>;

	/// Proposal indices that have been approved but not yet awarded.
	#[pallet::storage]
	#[pallet::getter(fn approvals)]
	pub type Approvals<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<ProposalIndex, T::MaxApprovals>, ValueQuery>;

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
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New proposal.
		Proposed {
			proposal_index: ProposalIndex,
		},
		/// We have ended a spend period and will now allocate funds.
		Spending {
			budget_remaining: BalanceOf<T, I>,
		},
		/// Some funds have been deposited.
		Deposit {
			value: BalanceOf<T, I>,
		},
		/// A new spend proposal has been approved.
		SpendApproved {
			proposal_index: ProposalIndex,
			amount: BalanceOf<T, I>,
			beneficiary: T::AccountId,
		},
		ProposalClosed {
			lawsuit_id: u32,
			approve: bool,
		},
		/// The inactive funds of the pallet have been updated.
		UpdatedInactive {
			reactivated: BalanceOf<T, I>,
			deactivated: BalanceOf<T, I>,
		},
		VoteSubmitted {
			lawsuit_id: u32,
			voter: T::AccountId,
			approve: bool,
		},
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Proposer's balance is too low.
		InsufficientProposersBalance,
		/// No proposal or bounty at that index.
		InvalidIndex,
		/// Too many approvals in the queue.
		TooManyApprovals,
		/// The spend origin is valid but the amount it is allowed to spend is lower than the
		/// amount to be spent.
		InsufficientPermission,
		/// Proposal has not been approved.
		ProposalNotApproved,
		ProposalAlreadyApproved,
		ProposalNotFound,
		LawsuitNotFound,
		DuplicateVote,
		StatementOverSize,
		VoterCountTooLow,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Submit a lawsuit
		///
		/// ## Complexity
		/// - O(1)
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn submit_sue(
			origin: OriginFor<T>,
			#[pallet::compact] value: BalanceOf<T, I>,
			defendent: AccountIdLookupOf<T>,
			statement: Vec<u8>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;
			let defendent = T::Lookup::lookup(defendent)?;
			ensure!(statement.len() < 512, Error::<T, I>::StatementOverSize);

			let c = Self::proposal_count();
			<ProposalCount<T, I>>::put(c + 1);
			let proposal = Lawsuit {
				plaintiff: proposer,
				value,
				defendent,
				statement: statement.clone(),
				voters: vec![],
				votes: vec![],
				approved: false,
			};
			<Proposals<T, I>>::insert(c, proposal);

			Self::deposit_event(Event::Proposed { proposal_index: c });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn vote_sue(origin: OriginFor<T>, lawsuit_id: u32, approve: bool) -> DispatchResult {
			let voter = ensure_signed(origin)?;

			let mut lawsuit =
				<Proposals<T, I>>::get(lawsuit_id).ok_or(Error::<T, I>::LawsuitNotFound)?;
			//.clone();
			<Proposals<T, I>>::remove(lawsuit_id);

			// Ensure the voter hasn't voted before
			ensure!(!lawsuit.voters.contains(&voter), Error::<T, I>::DuplicateVote);

			// Add the voter and their vote to the lawsuit
			lawsuit.voters.push(voter.clone());
			lawsuit.votes.push(approve);
			<Proposals<T, I>>::insert(lawsuit_id, lawsuit);

			Self::deposit_event(Event::VoteSubmitted { lawsuit_id, voter, approve });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight((Weight::zero(), DispatchClass::Operational))]
		pub fn process_sue(origin: OriginFor<T>, lawsuit_id: u32) -> DispatchResult {
			// Only root members can close the lawsuit
			ensure_root(origin)?;
			let mut proposal =
				<Proposals<T, I>>::get(lawsuit_id).ok_or(Error::<T, I>::ProposalNotFound)?;
			//.clone();
			<Proposals<T, I>>::remove(lawsuit_id);

			// Ensure that the proposal is not already approved
			ensure!(!proposal.approved, Error::<T, I>::ProposalAlreadyApproved);

			// Update the tally of votes
			let vote_count = proposal.votes.iter().filter(|v| **v).count() as u32;
			let voter_count = proposal.votes.len();

			ensure!(voter_count > 3, Error::<T, I>::VoterCountTooLow);

			let approval_threshold = (voter_count as u32) * 3 / 4; // Simple majority

			if vote_count >= approval_threshold {
				// Proposal is approved, execute the proposal
				T::Currency::transfer(
					&proposal.defendent,
					&proposal.plaintiff,
					proposal.value,
					frame_support::traits::ExistenceRequirement::KeepAlive,
				)?;
				proposal.approved = true;
				<Proposals<T, I>>::insert(lawsuit_id, &proposal);
				Approvals::<T, I>::try_append(lawsuit_id)
					.map_err(|_| Error::<T, I>::TooManyApprovals)?;
				Self::deposit_event(Event::ProposalClosed { lawsuit_id, approve: true });
			} else {
				proposal.approved = false;
				<Proposals<T, I>>::insert(lawsuit_id, &proposal);
				Self::deposit_event(Event::ProposalClosed { lawsuit_id, approve: false });
			}

			Ok(())
		}

		/// Force a previously approved lawsuit to be removed from the approval queue.
		///
		/// - `lawsuit_id`: The index of a lawsuit
		///
		/// ## Complexity
		/// - O(A) where `A` is the number of approvals
		///
		/// Errors:
		/// - `ProposalNotApproved`: The `lawsuit_id` supplied was not found in the approval queue,
		/// i.e., the lawsuit has not been approved. This could also mean the lawsuit does not
		/// exist altogether, thus there is no way it would have been approved in the first place.
		#[pallet::call_index(3)]
		#[pallet::weight((Weight::zero(), DispatchClass::Operational))]
		pub fn remove_sue(
			origin: OriginFor<T>,
			#[pallet::compact] lawsuit_id: ProposalIndex,
		) -> DispatchResult {
			ensure_root(origin)?;

			Approvals::<T, I>::try_mutate(|v| -> DispatchResult {
				if let Some(index) = v.iter().position(|x| x == &lawsuit_id) {
					v.remove(index);
					Ok(())
				} else {
					Err(Error::<T, I>::ProposalNotApproved.into())
				}
			})?;

			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn contribution(beneficiary: T::AccountId) -> u32 {
		Proposals::<T, I>::iter()
			.filter(|(_, p)| p.approved && p.voters.contains(&beneficiary))
			.count() as u32
	}
}
