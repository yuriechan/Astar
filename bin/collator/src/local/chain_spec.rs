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

//! Chain specifications.

use local_runtime::{
    wasm_binary_unwrap, AccountId, AuraConfig, AuraId, BalancesConfig, DappStakingConfig,
    EVMConfig, GrandpaConfig, GrandpaId, InflationConfig, InflationParameters, Precompiles,
    RuntimeGenesisConfig, Signature, SudoConfig, SystemConfig, TierThreshold, VestingConfig, AST,
};
use sc_service::ChainType;
use sp_core::{crypto::Ss58Codec, sr25519, Pair, Public};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    Permill,
};

type AccountPublic = <Signature as Verify>::Signer;

/// Specialized `ChainSpec` for Shiden Network.
pub type ChainSpec = sc_service::GenericChainSpec<local_runtime::RuntimeGenesisConfig>;

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
    let mut properties = serde_json::map::Map::new();
    properties.insert("tokenSymbol".into(), "LOC".into());
    properties.insert("tokenDecimals".into(), 18.into());
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                vec![authority_keys_from_seed("Alice")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    // Arrakis.TEST account in MetaMask
                    // Import known test account with private key
                    // 0x01ab6e801c06e59ca97a14fc0a1978b27fa366fc87450e0b65459dd3515b7391
                    // H160 address: 0xaaafB3972B05630fCceE866eC69CdADd9baC2771
                    AccountId::from_ss58check("5FQedkNQcF2fJPwkB6Z1ZcMgGti4vcJQNs6x85YPv3VhjBBT")
                        .unwrap(),
                ],
            )
        },
        vec![],
        None,
        None,
        None,
        Some(properties),
        None,
    )
}

fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
) -> RuntimeGenesisConfig {
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
        balances: BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000_000 * AST))
                .collect(),
        },
        vesting: VestingConfig { vesting: vec![] },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
            ..Default::default()
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
        ethereum: Default::default(),
        sudo: SudoConfig {
            key: Some(root_key),
        },
        assets: Default::default(),
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
                    amount: 100 * AST,
                    minimum_amount: 80 * AST,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 50 * AST,
                    minimum_amount: 40 * AST,
                },
                TierThreshold::DynamicTvlAmount {
                    amount: 20 * AST,
                    minimum_amount: 20 * AST,
                },
                TierThreshold::FixedTvlAmount { amount: 10 * AST },
            ],
            slots_per_tier: vec![10, 20, 30, 40],
            ..Default::default()
        },
        inflation: InflationConfig {
            params: InflationParameters::default(),
            ..Default::default()
        },
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sp_runtime::BuildStorage;

    #[test]
    fn test_create_development_chain_spec() {
        development_config().build_storage().unwrap();
    }
}
