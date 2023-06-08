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

//! Court pallet tests.

#![cfg(test)]

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use frame_support::{
	assert_noop, assert_ok,
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{ConstU32, ConstU64, OnInitialize},
};

use super::*;
use crate as court;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Court: court::{Pallet, Call, Storage, Config, Event<T>},
		Utility: pallet_utility,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Index = u64;
	type BlockNumber = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u128; // u64 is not enough to hold bytes used to generate bounty account
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_utility::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}

parameter_types! {}

impl Config for Test {
	type Currency = pallet_balances::Pallet<Test>;
	type RuntimeEvent = RuntimeEvent;
	type MaxApprovals = ConstU32<100>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	GenesisBuild::<Test>::assimilate_storage(&crate::GenesisConfig, &mut t).unwrap();
	t.into()
}

#[test]
fn submite_sue_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		<Court as OnInitialize<u64>>::on_initialize(2);
		assert_eq!(Court::approvals().len(), 0);
		assert_eq!(Court::proposal_count(), 1);
	});
}

#[test]
fn vote_sue_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(2), 0, true));
		assert_eq!(Court::approvals().len(), 0);
		assert_eq!(Court::proposal_count(), 1);
	});
}

#[test]
fn process_sue_works() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(2), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(3), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(4), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(5), 0, true));
		assert_ok!(Court::process_sue(RuntimeOrigin::root(), 0));
		assert_eq!(Court::approvals().len(), 1);
		assert_eq!(Court::proposal_count(), 1);
	});
}

#[test]
fn vote_against_works() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(2), 0, false));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(3), 0, false));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(4), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(5), 0, true));
		assert_ok!(Court::process_sue(RuntimeOrigin::root(), 0));
		assert_eq!(Court::approvals().len(), 0);
		assert_eq!(Court::proposal_count(), 1);
	});
}

#[test]
fn process_sue_before_vote() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_noop!(
			Court::process_sue(RuntimeOrigin::root(), 0),
				Error::<Test, _>::VoterCountTooLow
			);
		assert_eq!(Court::approvals().len(), 0);
		assert_eq!(Court::proposal_count(), 1);
	});
}


#[test]
fn remove_unapproved_sue() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_noop!(
			Court::remove_sue(RuntimeOrigin::root(), 0),
				Error::<Test, _>::ProposalNotApproved
			);
	});
}

#[test]
fn remove_approved_sue() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(2), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(3), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(4), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(5), 0, true));
		assert_ok!(Court::process_sue(RuntimeOrigin::root(), 0));
		assert_ok!(Court::remove_sue(RuntimeOrigin::root(), 0));
	});	
}

#[test]
fn contribution_works() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&0, 101);
		assert_ok!(Court::submit_sue(RuntimeOrigin::signed(1), 100, 0, vec![]));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(2), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(3), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(4), 0, true));
		assert_ok!(Court::vote_sue(RuntimeOrigin::signed(5), 0, true));
		assert_ok!(Court::process_sue(RuntimeOrigin::root(), 0));
		assert_eq!(Court::contribution(2), 1);
		assert_eq!(Court::contribution(3), 1);
		assert_eq!(Court::contribution(4), 1);
		assert_eq!(Court::contribution(5), 1);
		assert_eq!(Court::contribution(1), 0);
		assert_eq!(Court::contribution(11), 0);
	});
}