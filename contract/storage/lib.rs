#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
pub mod contract {
    use common::mtc::{
        ep::{EP_UNFINISH_PENALTY, INITIAL_EP},
        ghost::choose_ghosts,
        setup::{build_initial_ghost_states, build_pool, PLAYER_INITIAL_HEALTH},
        shop::coin::get_upgrade_coin,
    };
    use common::{codec_types::*, mtc::*};
    use ink_env::hash::Blake2x128;
    use ink_prelude::vec::Vec;
    use ink_storage::{traits::SpreadAllocate, Mapping};
    use scale::Decode;

    #[ink(storage)]
    #[derive(SpreadAllocate)]
    pub struct Storage {
        emo_bases: Option<emo::Bases>,
        deck_fixed_emo_base_ids: Option<Vec<u16>>,
        deck_built_emo_base_ids: Option<Vec<u16>>,

        matchmaking_ghosts: Mapping<u16, Vec<(AccountId, u16, mtc::Ghost)>>,

        player_ep: Mapping<AccountId, u16>,
        player_seed: Mapping<AccountId, u64>,

        // remove for each mtc
        player_pool: Mapping<AccountId, Vec<mtc::Emo>>,
        player_health: Mapping<AccountId, u8>,
        player_grade_and_board_history: Mapping<AccountId, Vec<mtc::GradeAndBoard>>,
        player_upgrade_coin: Mapping<AccountId, Option<u8>>,
        player_ghosts: Mapping<AccountId, Vec<(AccountId, u16, mtc::Ghost)>>,
        player_ghost_states: Mapping<AccountId, Vec<mtc::GhostState>>,
        player_battle_ghost_index: Mapping<AccountId, u8>,

        // allowed accounts
        allowed_accounts: Vec<AccountId>,
    }

    impl Storage {
        #[ink(constructor)]
        pub fn new() -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.allowed_accounts.push(Self::env().caller());
            })
        }

        #[ink(message)]
        pub fn update_emo_bases(
            &mut self,
            new_bases: emo::Bases,
            fixed_base_ids: Vec<u16>,
            built_base_ids: Vec<u16>,
            force_bases_update: bool,
        ) {
            self.only_allowed_caller();

            let bases = emo_bases::check_and_build_emo_bases(
                self.get_emo_bases(),
                new_bases,
                &fixed_base_ids,
                &built_base_ids,
                force_bases_update,
            )
            .expect("update_emo_bases: invalig arg");

            self.set_emo_bases(Some(bases));
            self.set_deck_fixed_emo_base_ids(Some(fixed_base_ids));
            self.set_deck_built_emo_base_ids(Some(built_base_ids));
        }

        #[ink(message)]
        pub fn start_mtc(&mut self, caller: AccountId, deck_emo_base_ids: [u16; 6]) {
            self.only_allowed_caller();

            let (
                emo_bases,
                deck_fixed_emo_base_ids,
                deck_built_emo_base_ids,
                player_ep,
                _,
                _,
                player_health,
                ..,
            ) = self.get_player_batch(
                caller, true, true, true, true, false, false, true, false, false, false, false,
                false,
            );

            let ep = if player_health.is_some() {
                // the previous mtc didn't normally finish
                player_ep
                    .expect("player ep none")
                    .saturating_sub(EP_UNFINISH_PENALTY)
            } else {
                player_ep.unwrap_or(INITIAL_EP)
            };

            let seed = self.get_random_seed(caller, b"start_mtc");

            let selected_ghosts =
                choose_ghosts(ep, seed, &|ep_band| self.get_matchmaking_ghosts(ep_band));

            self.set_player_batch(
                caller,
                if let Some(e) = player_ep {
                    if e == ep {
                        None
                    } else {
                        Some(ep)
                    }
                } else {
                    Some(ep)
                },
                Some(seed),
                Some(
                    build_pool(
                        &deck_emo_base_ids,
                        &emo_bases.expect("emo_bases none"),
                        &deck_fixed_emo_base_ids.expect("deck_fixed_emo_base_ids none"),
                        &deck_built_emo_base_ids.expect("deck_built_emo_base_ids none"),
                    )
                    .expect("failed to build player pool"),
                ),
                Some(PLAYER_INITIAL_HEALTH),
                Some(Vec::new()),
                Some(get_upgrade_coin(2)),
                Some(selected_ghosts),
                Some(build_initial_ghost_states()),
                Some(0),
            );
        }

        fn get_random_seed(&self, caller: AccountId, subject: &[u8]) -> u64 {
            let (seed, _) = self
                .env()
                .random(&self.env().hash_encoded::<Blake2x128, _>(&(subject, caller)));
            <u64>::decode(&mut seed.as_ref()).expect("failed to get seed")
        }

        #[ink(message)]
        pub fn get_emo_bases(&self) -> Option<emo::Bases> {
            self.emo_bases.clone()
        }

        #[ink(message)]
        pub fn set_emo_bases(&mut self, value: Option<emo::Bases>) {
            self.only_allowed_caller();
            self.emo_bases = value;
        }

        #[ink(message)]
        pub fn get_deck_fixed_emo_base_ids(&self) -> Option<Vec<u16>> {
            self.deck_fixed_emo_base_ids.clone()
        }

        #[ink(message)]
        pub fn set_deck_fixed_emo_base_ids(&mut self, value: Option<Vec<u16>>) {
            self.only_allowed_caller();
            self.deck_fixed_emo_base_ids = value;
        }

        #[ink(message)]
        pub fn get_deck_built_emo_base_ids(&self) -> Option<Vec<u16>> {
            self.deck_built_emo_base_ids.clone()
        }

        #[ink(message)]
        pub fn set_deck_built_emo_base_ids(&mut self, value: Option<Vec<u16>>) {
            self.only_allowed_caller();
            self.deck_built_emo_base_ids = value;
        }

        #[ink(message)]
        pub fn get_matchmaking_ghosts(
            &self,
            ep_band: u16,
        ) -> Option<Vec<(AccountId, u16, mtc::Ghost)>> {
            self.matchmaking_ghosts.get(ep_band)
        }

        #[ink(message)]
        pub fn set_matchmaking_ghosts(
            &mut self,
            ep_band: u16,
            value: Vec<(AccountId, u16, mtc::Ghost)>,
        ) {
            self.only_allowed_caller();
            self.matchmaking_ghosts.insert(ep_band, &value);
        }

        #[ink(message)]
        pub fn remove_matchmaking_ghosts(&mut self, ep_band: u16) {
            self.only_allowed_caller();
            self.matchmaking_ghosts.remove(ep_band);
        }

        #[ink(message)]
        pub fn get_player_ep(&self, account: AccountId) -> Option<u16> {
            self.player_ep.get(account)
        }

        #[ink(message)]
        pub fn set_player_ep(&mut self, account: AccountId, value: u16) {
            self.only_allowed_caller();
            self.player_ep.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_ep(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_ep.remove(account)
        }

        #[ink(message)]
        pub fn get_player_seed(&self, account: AccountId) -> Option<u64> {
            self.player_seed.get(account)
        }

        #[ink(message)]
        pub fn set_player_seed(&mut self, account: AccountId, value: u64) {
            self.only_allowed_caller();
            self.player_seed.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_seed(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_seed.remove(account)
        }

        #[ink(message)]
        pub fn get_player_pool(&self, account: AccountId) -> Option<Vec<mtc::Emo>> {
            self.player_pool.get(&account)
        }

        #[ink(message)]
        pub fn set_player_pool(&mut self, account: AccountId, value: Vec<mtc::Emo>) {
            self.only_allowed_caller();
            self.player_pool.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_pool(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_pool.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_health(&self, account: AccountId) -> Option<u8> {
            self.player_health.get(&account)
        }

        #[ink(message)]
        pub fn set_player_health(&mut self, account: AccountId, value: u8) {
            self.only_allowed_caller();
            self.player_health.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_health(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_health.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_grade_and_board_history(
            &self,
            account: AccountId,
        ) -> Option<Vec<mtc::GradeAndBoard>> {
            self.player_grade_and_board_history.get(&account)
        }

        #[ink(message)]
        pub fn set_player_grade_and_board_history(
            &mut self,
            account: AccountId,
            value: Vec<mtc::GradeAndBoard>,
        ) {
            self.only_allowed_caller();
            self.player_grade_and_board_history.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_grade_and_board_history(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_grade_and_board_history.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_upgrade_coin(&self, account: AccountId) -> Option<Option<u8>> {
            self.player_upgrade_coin.get(&account)
        }

        #[ink(message)]
        pub fn set_player_upgrade_coin(&mut self, account: AccountId, value: Option<u8>) {
            self.only_allowed_caller();
            self.player_upgrade_coin.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_upgrade_coin(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_upgrade_coin.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_ghosts(
            &self,
            account: AccountId,
        ) -> Option<Vec<(AccountId, u16, mtc::Ghost)>> {
            self.player_ghosts.get(&account)
        }

        #[ink(message)]
        pub fn set_player_ghosts(
            &mut self,
            account: AccountId,
            value: Vec<(AccountId, u16, mtc::Ghost)>,
        ) {
            self.only_allowed_caller();
            self.player_ghosts.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_ghosts(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_ghosts.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_ghost_states(&self, account: AccountId) -> Option<Vec<mtc::GhostState>> {
            self.player_ghost_states.get(&account)
        }

        #[ink(message)]
        pub fn set_player_ghost_states(&mut self, account: AccountId, value: Vec<mtc::GhostState>) {
            self.only_allowed_caller();
            self.player_ghost_states.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_ghost_states(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_ghost_states.remove(&account)
        }

        #[ink(message)]
        pub fn get_player_battle_ghost_index(&self, account: AccountId) -> Option<u8> {
            self.player_battle_ghost_index.get(&account)
        }

        #[ink(message)]
        pub fn set_player_battle_ghost_index(&mut self, account: AccountId, value: u8) {
            self.only_allowed_caller();
            self.player_battle_ghost_index.insert(account, &value);
        }

        #[ink(message)]
        pub fn remove_player_battle_ghost_index(&mut self, account: AccountId) {
            self.only_allowed_caller();
            self.player_battle_ghost_index.remove(&account)
        }

        // batch ops

        #[ink(message)]
        pub fn get_player_batch(
            &self,
            player_id: AccountId,

            emo_bases: bool,
            deck_fixed_emo_base_ids: bool,
            deck_built_emo_base_ids: bool,

            player_ep: bool,
            player_seed: bool,

            player_pool: bool,
            player_health: bool,
            player_grade_and_board_history: bool,
            player_upgrade_coin: bool,
            player_ghosts: bool,
            player_ghost_states: bool,
            player_battle_ghost_index: bool,
        ) -> (
            Option<emo::Bases>,
            Option<Vec<u16>>,
            Option<Vec<u16>>,
            Option<u16>,
            Option<u64>,
            Option<Vec<mtc::Emo>>,
            Option<u8>,
            Option<Vec<mtc::GradeAndBoard>>,
            Option<Option<u8>>,
            Option<Vec<(AccountId, u16, mtc::Ghost)>>,
            Option<Vec<mtc::GhostState>>,
            Option<u8>,
        ) {
            (
                if emo_bases {
                    self.emo_bases.clone()
                } else {
                    None
                },
                if deck_fixed_emo_base_ids {
                    self.deck_fixed_emo_base_ids.clone()
                } else {
                    None
                },
                if deck_built_emo_base_ids {
                    self.deck_built_emo_base_ids.clone()
                } else {
                    None
                },
                if player_ep {
                    self.player_ep.get(player_id)
                } else {
                    None
                },
                if player_seed {
                    self.player_seed.get(player_id)
                } else {
                    None
                },
                if player_pool {
                    self.player_pool.get(player_id)
                } else {
                    None
                },
                if player_health {
                    self.player_health.get(player_id)
                } else {
                    None
                },
                if player_grade_and_board_history {
                    self.player_grade_and_board_history.get(player_id)
                } else {
                    None
                },
                if player_upgrade_coin {
                    self.player_upgrade_coin.get(player_id)
                } else {
                    None
                },
                if player_ghosts {
                    self.player_ghosts.get(player_id)
                } else {
                    None
                },
                if player_ghost_states {
                    self.player_ghost_states.get(player_id)
                } else {
                    None
                },
                if player_battle_ghost_index {
                    self.player_battle_ghost_index.get(player_id)
                } else {
                    None
                },
            )
        }

        #[ink(message)]
        pub fn set_player_batch(
            &mut self,
            player_id: AccountId,

            player_ep: Option<u16>,
            player_seed: Option<u64>,

            player_pool: Option<Vec<mtc::Emo>>,
            player_health: Option<u8>,
            player_grade_and_board_history: Option<Vec<mtc::GradeAndBoard>>,
            player_upgrade_coin: Option<Option<u8>>,
            player_ghosts: Option<Vec<(AccountId, u16, mtc::Ghost)>>,
            player_ghost_states: Option<Vec<mtc::GhostState>>,
            player_battle_ghost_index: Option<u8>,
        ) {
            self.only_allowed_caller();

            if let Some(p) = player_ep {
                self.player_ep.insert(player_id, &p)
            }
            if let Some(p) = player_seed {
                self.player_seed.insert(player_id, &p)
            }
            if let Some(p) = player_pool {
                self.player_pool.insert(player_id, &p)
            }
            if let Some(p) = player_health {
                self.player_health.insert(player_id, &p)
            }
            if let Some(p) = player_grade_and_board_history {
                self.player_grade_and_board_history.insert(player_id, &p)
            }
            if let Some(p) = player_upgrade_coin {
                self.player_upgrade_coin.insert(player_id, &p)
            }
            if let Some(p) = player_ghosts {
                self.player_ghosts.insert(player_id, &p)
            }
            if let Some(p) = player_ghost_states {
                self.player_ghost_states.insert(player_id, &p)
            }
            if let Some(p) = player_battle_ghost_index {
                self.player_battle_ghost_index.insert(player_id, &p)
            }
        }

        #[ink(message)]
        pub fn remove_player_mtc(&mut self, account: AccountId) {
            self.only_allowed_caller();

            self.player_pool.remove(&account);
            self.player_health.remove(&account);
            self.player_grade_and_board_history.remove(&account);
            self.player_upgrade_coin.remove(&account);
            self.player_ghosts.remove(&account);
            self.player_ghost_states.remove(&account);
            self.player_battle_ghost_index.remove(&account);
        }

        #[ink(message)]
        pub fn update_for_logic_finish_mtc_shop_finish_mtc(
            &mut self,
            account: AccountId,
            player_ep: u16,
            player_seed: u64,
            matchmaking_ghosts: Option<(u16, Vec<(AccountId, u16, mtc::Ghost)>)>,
        ) {
            self.only_allowed_caller();

            self.player_ep.insert(account, &player_ep);
            self.player_seed.insert(account, &player_seed);

            if let Some((ep_band, g)) = matchmaking_ghosts {
                self.matchmaking_ghosts.insert(ep_band, &g);
            }

            self.remove_player_mtc(account);
        }

        // allowed accounts

        #[ink(message)]
        pub fn get_allowed_accounts(&self) -> Vec<AccountId> {
            self.allowed_accounts.clone()
        }

        #[ink(message)]
        pub fn allow_account(&mut self, account_id: AccountId) {
            self.only_allowed_caller();
            self.allowed_accounts.push(account_id);
        }

        #[ink(message)]
        pub fn disallow_account(&mut self, account_id: AccountId) {
            self.only_allowed_caller();
            self.allowed_accounts.retain(|a| a != &account_id);
        }

        fn only_allowed_caller(&self) {
            assert!(
                self.allowed_accounts.contains(&self.env().caller()),
                "only_allowed_caller: this caller is not allowed",
            );
        }
    }
}
