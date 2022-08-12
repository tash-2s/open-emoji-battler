use common::{codec_types::*, mtc::*};
use ink_prelude::{vec, vec::Vec};

pub fn finish_battle(
    upgrade_coin: Option<u8>,
    ghost_states: &[mtc::GhostState],
    battle_ghost_index: u8,
    new_seed: u64,
    grade_and_board_history: &[mtc::GradeAndBoard],
) -> (Option<u8>, u8) {
    if finish::exceeds_grade_and_board_history_limit(grade_and_board_history) {
        panic!("max turn exceeded");
    }

    let upgrade_coin = shop::coin::decrease_upgrade_coin(upgrade_coin);

    let new_battle_ghost_index =
        battle::organizer::select_battle_ghost_index(ghost_states, battle_ghost_index, new_seed)
            .expect("battle ghost selection failed");

    (upgrade_coin, new_battle_ghost_index)
}

pub fn calc_new_ep(place: u8, old_ep: u16) -> u16 {
    if let Some(plus) = match place {
        1 => Some(70),
        2 => Some(50),
        _ => None,
    } {
        return if old_ep > ep::INITIAL_EP {
            let x = (old_ep - ep::INITIAL_EP) / 40;
            if x < plus {
                plus - x
            } else {
                1
            }
        } else {
            plus
        };
    }

    let minus = match place {
        3 => 30,
        4 => 50,
        _ => panic!("unsupported place: {}", place),
    };

    let e = old_ep.saturating_sub(minus);
    if e > ep::MIN_EP {
        e
    } else {
        ep::MIN_EP
    }
}

const LEADERBOARD_SIZE: u8 = 100;
const LEADERBOARD_SURPLUS_SIZE: u8 = 30;
const LEADERBOARD_REAL_SIZE: u8 = LEADERBOARD_SIZE + LEADERBOARD_SURPLUS_SIZE;

pub fn update_leaderboard<A: Eq + Copy>(leaderboard: &mut Vec<(u16, A)>, ep: u16, account: &A) {
    let mut same_account_index_opt = None;
    let mut new_place_index_opt = None;

    for (index, (iter_ep, iter_account)) in leaderboard.iter().enumerate() {
        if iter_account == account {
            same_account_index_opt = Some(index);
        }
        if iter_ep <= &ep {
            new_place_index_opt = Some(index);
        }
    }

    if let Some(same_account_index) = same_account_index_opt {
        if let Some(new_place_index) = new_place_index_opt {
            leaderboard.swap(same_account_index, new_place_index);
        } else {
            let len = leaderboard.len();
            if len < LEADERBOARD_REAL_SIZE.into() {
                leaderboard.swap(same_account_index, len - 1);
            } else {
                leaderboard.remove(same_account_index);
            }
        }
    } else if let Some(new_place_index) = new_place_index_opt {
        leaderboard.insert(new_place_index, (ep, *account));
        leaderboard.truncate(LEADERBOARD_REAL_SIZE.into());
    } else if leaderboard.len() < LEADERBOARD_REAL_SIZE.into() {
        leaderboard.push((ep, *account));
    }
}

pub fn build_initial_ghost_states(ep: u16) -> Vec<mtc::GhostState> {
    let health = match ep::get_ep_band(ep) {
        0 => 14,
        1 => 16,
        2 => 18,
        3 => 20,
        4 => 22,
        5 => 24,
        6 => 26,
        7 => 28,
        8.. => 30,
    };

    vec![mtc::GhostState::Active { health }; 3]
}
