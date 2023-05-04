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

//! Treasury pallet tests.

#![cfg(test)]

use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BadOrigin, BlakeTwo256, Dispatchable, IdentityLookup},
};

use frame_support::{
	assert_err_ignore_postinfo, assert_noop, assert_ok,
	pallet_prelude::GenesisBuild,
	parameter_types,
	traits::{ConstU32, ConstU64, OnInitialize},
	PalletId,
};

use super::*;
use crate as treasury;
use pallet_court as court;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type UtilityCall = pallet_utility::Call<Test>;
type TreasuryCall = crate::Call<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Treasury: treasury::{Pallet, Call, Storage, Config, Event<T>},
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

parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
}

impl pallet_court::Config for Test {
	type Currency = pallet_balances::Pallet<Test>;
	type RuntimeEvent = RuntimeEvent;
	type MaxApprovals = ConstU32<100>;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type PalletId = TreasuryPalletId;
	type Currency = pallet_balances::Pallet<Test>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		// Total issuance will be 200 with treasury account initialized at ED.
		balances: vec![(0, 100), (1, 98), (2, 1)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	GenesisBuild::<Test>::assimilate_storage(&crate::GenesisConfig, &mut t).unwrap();
	t.into()
}

#[test]
fn genesis_config_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Treasury::pot(), 0);
	});
}

#[test]
fn receive_should_works() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 101);
		assert_ok!(Treasury::receive(RuntimeOrigin::signed(1), 10, 1));
	});
}

#[test]
fn claim_rewards_exceed_claim() {
	new_test_ext().execute_with(|| {
		Balances::make_free_balance_be(&1, 101);
		assert_ok!(Treasury::receive(RuntimeOrigin::signed(1), 10, 1));
		assert_noop!(
			Treasury::claim_rewards(RuntimeOrigin::signed(1), 10),
			Error::<Test, _>::ExceedClaim
		);
	});
}
