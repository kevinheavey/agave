#[cfg(feature = "dev-context-only-utils")]
use {
    crate::{
        bank::Bank,
        genesis_utils::{self, GenesisConfigInfo, ValidatorVoteKeypairs},
    },
    solana_sdk::{pubkey::Pubkey, signature::Signer},
};
use {
    log::{debug, log_enabled, trace},
    solana_program_runtime::loaded_programs::Stats,
    solana_sdk::{clock::Slot, transaction::SanitizedTransaction},
    solana_svm::transaction_results::TransactionResults,
    solana_vote::{vote_parser, vote_sender_types::ReplayVoteSender},
    std::sync::atomic::Ordering,
};

#[cfg(feature = "dev-context-only-utils")]
pub fn setup_bank_and_vote_pubkeys_for_tests(
    num_vote_accounts: usize,
    stake: u64,
) -> (Bank, Vec<Pubkey>) {
    // Create some voters at genesis
    let validator_voting_keypairs: Vec<_> = (0..num_vote_accounts)
        .map(|_| ValidatorVoteKeypairs::new_rand())
        .collect();

    let vote_pubkeys: Vec<_> = validator_voting_keypairs
        .iter()
        .map(|k| k.vote_keypair.pubkey())
        .collect();
    let GenesisConfigInfo { genesis_config, .. } =
        genesis_utils::create_genesis_config_with_vote_accounts(
            10_000,
            &validator_voting_keypairs,
            vec![stake; validator_voting_keypairs.len()],
        );
    let bank = Bank::new_for_tests(&genesis_config);
    (bank, vote_pubkeys)
}

pub fn find_and_send_votes(
    sanitized_txs: &[SanitizedTransaction],
    tx_results: &TransactionResults,
    vote_sender: Option<&ReplayVoteSender>,
) {
    let TransactionResults {
        execution_results, ..
    } = tx_results;
    if let Some(vote_sender) = vote_sender {
        sanitized_txs
            .iter()
            .zip(execution_results.iter())
            .for_each(|(tx, result)| {
                if tx.is_simple_vote_transaction() && result.was_executed_successfully() {
                    if let Some(parsed_vote) = vote_parser::parse_sanitized_vote_transaction(tx) {
                        if parsed_vote.1.last_voted_slot().is_some() {
                            let _ = vote_sender.send(parsed_vote);
                        }
                    }
                }
            });
    }
}

/// Logs the measurement values
pub fn submit_loaded_programs_stats(stats: &Stats, slot: Slot) {
    let hits = stats.hits.load(Ordering::Relaxed);
    let misses = stats.misses.load(Ordering::Relaxed);
    let evictions: u64 = stats.evictions.values().sum();
    let reloads = stats.reloads.load(Ordering::Relaxed);
    let insertions = stats.insertions.load(Ordering::Relaxed);
    let lost_insertions = stats.lost_insertions.load(Ordering::Relaxed);
    let replacements = stats.replacements.load(Ordering::Relaxed);
    let one_hit_wonders = stats.one_hit_wonders.load(Ordering::Relaxed);
    let prunes_orphan = stats.prunes_orphan.load(Ordering::Relaxed);
    let prunes_environment = stats.prunes_environment.load(Ordering::Relaxed);
    let empty_entries = stats.empty_entries.load(Ordering::Relaxed);
    datapoint_info!(
        "loaded-programs-cache-stats",
        ("slot", slot, i64),
        ("hits", hits, i64),
        ("misses", misses, i64),
        ("evictions", evictions, i64),
        ("reloads", reloads, i64),
        ("insertions", insertions, i64),
        ("lost_insertions", lost_insertions, i64),
        ("replace_entry", replacements, i64),
        ("one_hit_wonders", one_hit_wonders, i64),
        ("prunes_orphan", prunes_orphan, i64),
        ("prunes_environment", prunes_environment, i64),
        ("empty_entries", empty_entries, i64),
    );
    debug!(
            "Loaded Programs Cache Stats -- Hits: {}, Misses: {}, Evictions: {}, Reloads: {}, Insertions: {} Lost-Insertions: {}, Replacements: {}, One-Hit-Wonders: {}, Prunes-Orphan: {}, Prunes-Environment: {}, Empty: {}",
            hits, misses, evictions, reloads, insertions, lost_insertions, replacements, one_hit_wonders, prunes_orphan, prunes_environment, empty_entries
        );
    if log_enabled!(log::Level::Trace) && !stats.evictions.is_empty() {
        let mut evictions = stats.evictions.iter().collect::<Vec<_>>();
        evictions.sort_by_key(|e| e.1);
        let evictions = evictions
            .into_iter()
            .rev()
            .map(|(program_id, evictions)| {
                format!("  {:<44}  {}", program_id.to_string(), evictions)
            })
            .collect::<Vec<_>>();
        let evictions = evictions.join("\n");
        trace!(
            "Eviction Details:\n  {:<44}  {}\n{}",
            "Program",
            "Count",
            evictions
        );
    }
}
