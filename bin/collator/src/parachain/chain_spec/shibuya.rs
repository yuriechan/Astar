// This file is part of Astar.

// Copyright (C) Stake Technologies Pte.Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later

// Astar is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Astar is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Astar. If not, see <http://www.gnu.org/licenses/>.

//! Shibuya chain specifications.

use cumulus_primitives_core::ParaId;
use sc_service::ChainType;
use shibuya_runtime::{
    wasm_binary_unwrap, AccountId, AuraConfig, AuraId, Balance, BalancesConfig,
    CollatorSelectionConfig, DappStakingConfig, EVMChainIdConfig, EVMConfig, InflationConfig,
    InflationParameters, OracleMembershipConfig, ParachainInfoConfig, Precompiles,
    PriceAggregatorConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys, Signature, SudoConfig,
    SystemConfig, TierThreshold, VestingConfig, SBY,
};
use sp_core::{sr25519, Pair, Public};

use astar_primitives::oracle::CurrencyAmount;
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Permill,
};

use super::{get_from_seed, Extensions};

const PARA_ID: u32 = 1000;

/// Specialized `ChainSpec` for Shibuya testnet.
pub type ShibuyaChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

/// Gen Shibuya chain specification for given parachain id.
pub fn get_chain_spec() -> ShibuyaChainSpec {
    // Alice as default
    let sudo_key = get_account_id_from_seed::<sr25519::Public>("Alice");
    let endowned = vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            1_000_000_000 * SBY,
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            1_000_000_000 * SBY,
        ),
    ];

    let mut properties = serde_json::map::Map::new();
    properties.insert("tokenSymbol".into(), "SBY".into());
    properties.insert("tokenDecimals".into(), 18.into());

    ShibuyaChainSpec::from_genesis(
        "Shibuya Testnet",
        "shibuya",
        ChainType::Development,
        move || make_genesis(endowned.clone(), sudo_key.clone(), PARA_ID.into()),
        vec![],
        None,
        None,
        None,
        Some(properties),
        Extensions {
            bad_blocks: Default::default(),
            relay_chain: "tokyo".into(),
            para_id: PARA_ID,
        },
    )
}

fn session_keys(aura: AuraId) -> SessionKeys {
    SessionKeys { aura }
}

/// Helper function to create Shibuya RuntimeGenesisConfig.
fn make_genesis(
    balances: Vec<(AccountId, Balance)>,
    root_key: AccountId,
    parachain_id: ParaId,
) -> RuntimeGenesisConfig {
    let authorities = vec![
        (
            get_account_id_from_seed::<sr25519::Public>("Alice"),
            get_from_seed::<AuraId>("Alice"),
        ),
        (
            get_account_id_from_seed::<sr25519::Public>("Bob"),
            get_from_seed::<AuraId>("Bob"),
        ),
    ];

    // This is supposed the be the simplest bytecode to revert without returning any data.
    // We will pre-deploy it under all of our precompiles to ensure they can be called from
    // within contracts.
    // (PUSH1 0x00 PUSH1 0x00 REVERT)
    let revert_bytecode = vec![0x60, 0x00, 0x60, 0x00, 0xFD];

    RuntimeGenesisConfig {
        system: SystemConfig {
            code: wasm_binary_unwrap().to_vec(),
            ..Default::default()
        },
        sudo: SudoConfig {
            key: Some(root_key),
        },
        parachain_info: ParachainInfoConfig {
            parachain_id,
            ..Default::default()
        },
        balances: BalancesConfig { balances },
        vesting: VestingConfig { vesting: vec![] },
        session: SessionConfig {
            keys: authorities
                .iter()
                .map(|x| (x.0.clone(), x.0.clone(), session_keys(x.1.clone())))
                .collect::<Vec<_>>(),
        },
        aura: AuraConfig {
            authorities: vec![],
        },
        aura_ext: Default::default(),
        collator_selection: CollatorSelectionConfig {
            desired_candidates: 32,
            candidacy_bond: 32_000 * SBY,
            invulnerables: authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
        },
        evm: EVMConfig {
            // We need _some_ code inserted at the precompile address so that
            // the evm will actually call the address.
            accounts: Precompiles::used_addresses()
                .map(|addr| {
                    (
                        addr,
                        fp_evm::GenesisAccount {
                            nonce: Default::default(),
                            balance: Default::default(),
                            storage: Default::default(),
                            code: revert_bytecode.clone(),
                        },
                    )
                })
                .collect(),
            ..Default::default()
        },
        evm_chain_id: EVMChainIdConfig {
            chain_id: 0x51,
            ..Default::default()
        },
        ethereum: Default::default(),
        polkadot_xcm: Default::default(),
        assets: Default::default(),
        parachain_system: Default::default(),
        transaction_payment: Default::default(),
        dapp_staking: DappStakingConfig {
            reward_portion: vec![
                Permill::from_percent(40),
                Permill::from_percent(30),
                Permill::from_percent(20),
                Permill::from_percent(10),
            ],
            slot_distribution: vec![
                Permill::from_percent(10),
                Permill::from_percent(20),
                Permill::from_percent(30),
                Permill::from_percent(40),
            ],
            tier_thresholds: vec![
                TierThreshold::DynamicTvlAmount {
                    amount: 100 * SBY,
                    minimum_amount: 80 * SBY,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 50 * SBY,
                    minimum_amount: 40 * SBY,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 20 * SBY,
                    minimum_amount: 20 * SBY,
                },
                TierThreshold::FixedTvlAmount { amount: 10 * SBY },
            ],
            slots_per_tier: vec![10, 20, 30, 40],
            ..Default::default()
        },
        inflation: InflationConfig {
            params: InflationParameters::default(),
            ..Default::default()
        },
        oracle_membership: OracleMembershipConfig {
            members: vec![
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                get_account_id_from_seed::<sr25519::Public>("Bob"),
            ]
            .try_into()
            .expect("Assumption is that at least two members will be allowed."),
            ..Default::default()
        },
        price_aggregator: PriceAggregatorConfig {
            circular_buffer: vec![CurrencyAmount::from_rational(5, 10)]
                .try_into()
                .expect("Must work since buffer should have at least a single value."),
        },
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
