use std::{fs::read_to_string, num::NonZeroU32, path::Path};

use sanctum_spl_stake_pool_lib::{
    FindTransientStakeAccount, FindTransientStakeAccountArgs, FindValidatorStakeAccount,
    FindValidatorStakeAccountArgs,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use spl_stake_pool_interface::{Fee, FutureEpochFee, StakeStatus, ValidatorStakeInfo};

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFileTomlFile {
    pub pool: ConfigFileRaw,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFileRaw {
    pub mint: Option<String>,
    pub pool: Option<String>,
    pub validator_list: Option<String>,
    pub reserve: Option<String>,
    pub manager: Option<String>,
    pub manager_fee_account: Option<String>,
    pub staker: Option<String>,
    pub stake_deposit_auth: Option<String>,
    pub sol_deposit_auth: Option<String>,
    pub sol_withdraw_auth: Option<String>,
    pub preferred_deposit_validator: Option<String>,
    pub preferred_withdraw_validator: Option<String>,
    pub max_validators: Option<u32>,
    pub stake_deposit_referral_fee: Option<u8>,
    pub sol_deposit_referral_fee: Option<u8>,
    pub epoch_fee: Option<Fee>,
    pub stake_withdrawal_fee: Option<Fee>,
    pub sol_withdrawal_fee: Option<Fee>,
    pub stake_deposit_fee: Option<Fee>,
    pub sol_deposit_fee: Option<Fee>,
    pub total_lamports: Option<u64>,
    pub pool_token_supply: Option<u64>,
    pub last_update_epoch: Option<u64>,
    pub next_epoch_fee: Option<FutureEpochFee>,
    pub next_stake_withdrawal_fee: Option<FutureEpochFee>,
    pub next_sol_withdrawal_fee: Option<FutureEpochFee>,
    pub last_epoch_pool_token_supply: Option<u64>,
    pub last_epoch_total_lamports: Option<u64>,
    pub stake_withdraw_auth: Option<String>, // fixed PDA, only displayed for info purposes
    pub old_manager: Option<String>,         // only present for sync-pool
    pub validators: Option<Vec<ValidatorConfigRaw>>, // put this last so it gets outputted last in toml Serialize
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct ValidatorConfigRaw {
    pub vote: String,
    pub active_stake_lamports: Option<u64>,
    pub transient_stake_lamports: Option<u64>,
    pub last_update_epoch: Option<u64>,
    pub validator_seed_suffix: Option<NonZeroU32>,
    pub transient_seed_suffix: Option<u64>,
    pub status: Option<StakeStatus>,
    pub validator_stake_account: Option<String>,
    pub transient_stake_account: Option<String>,
}

impl ValidatorConfigRaw {
    pub fn from_vsi_program_pool(
        ValidatorStakeInfo {
            active_stake_lamports,
            transient_stake_lamports,
            last_update_epoch,
            transient_seed_suffix,
            validator_seed_suffix,
            status,
            vote_account_address,
            ..
        }: &ValidatorStakeInfo,
        program_id: &Pubkey,
        pool: &Pubkey,
    ) -> Self {
        let validator_seed_suffix = NonZeroU32::new(*validator_seed_suffix);
        Self {
            vote: vote_account_address.to_string(),
            active_stake_lamports: Some(*active_stake_lamports),
            transient_stake_lamports: Some(*transient_stake_lamports),
            last_update_epoch: Some(*last_update_epoch),
            validator_seed_suffix,
            transient_seed_suffix: Some(*transient_seed_suffix),
            status: Some(status.clone()),
            validator_stake_account: Some(
                FindValidatorStakeAccount::new(FindValidatorStakeAccountArgs {
                    pool: *pool,
                    vote: *vote_account_address,
                    seed: validator_seed_suffix,
                })
                .run_for_prog(program_id)
                .0
                .to_string(),
            ),
            transient_stake_account: Some(
                FindTransientStakeAccount::new(FindTransientStakeAccountArgs {
                    pool: *pool,
                    vote: *vote_account_address,
                    seed: *transient_seed_suffix,
                })
                .run_for_prog(program_id)
                .0
                .to_string(),
            ),
        }
    }
}

impl ConfigFileRaw {
    pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<ConfigFileRaw, std::io::Error> {
        // toml crate only handles strings, not io::Read lol
        let s = read_to_string(path)?;
        let ConfigFileTomlFile { pool } =
            toml::from_str(&s).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(pool)
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ConfigFileTomlOutput<'a> {
    pub pool: &'a ConfigFileRaw,
}

impl<'a> std::fmt::Display for ConfigFileTomlOutput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&toml::to_string_pretty(self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use sanctum_solana_test_utils::test_fixtures_dir;
    use spl_stake_pool_interface::Fee;

    use super::*;

    #[test]
    fn deser_example_config() {
        let example_path = test_fixtures_dir().join("example-init-pool-config.toml");
        let res = ConfigFileRaw::read_from_path(example_path).unwrap();
        // sample some fields
        assert_eq!(res.max_validators, Some(2));
        assert_eq!(
            res.epoch_fee,
            Some(Fee {
                denominator: 100,
                numerator: 6
            })
        );
        assert_eq!(res.validators.as_ref().unwrap().len(), 2);
        assert_eq!(
            res.validators.as_ref().unwrap()[0].vote,
            "BLADE1qNA1uNjRgER6DtUFf7FU3c1TWLLdpPeEcKatZ2"
        );
        assert_eq!(
            res.validators.as_ref().unwrap()[1].vote,
            "FnAPJkzf19s87sm24Qhv6bHZMZvZ43gjNUBRgjwXpD4v"
        );

        eprintln!("{}", ConfigFileTomlOutput { pool: &res });
    }
}
