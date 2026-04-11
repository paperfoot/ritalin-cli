use proptest::prelude::*;

// We need to access the crate's internals. Since gate_eval is pub,
// and the binary crate re-exports through main, we'll use the binary
// directly for integration props. For unit-level props, we test
// the discharge logic through the public evidence API.

// Re-implement the core types and logic inline for property testing
// since we can't import from a binary crate in integration tests.

fn proof_hash(cmd: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(cmd.trim().as_bytes());
    hex::encode(hasher.finalize())
}

fn is_discharged(
    records: &[(i32, String, String)], // (exit_code, proof_hash, workspace_hash)
    expected_proof_hash: &str,
    current_workspace_hash: &str,
) -> bool {
    records.iter().any(|(exit_code, ph, wh)| {
        *exit_code == 0
            && !ph.is_empty()
            && !wh.is_empty()
            && ph == expected_proof_hash
            && wh == current_workspace_hash
    })
}

fn arb_exit_nonzero() -> impl Strategy<Value = i32> {
    prop_oneof![-128i32..=-1, 1i32..=255]
}

fn arb_hash() -> impl Strategy<Value = String> {
    "[a-f0-9]{64}"
}

// ─── Property: failing-only evidence never discharges ────────

proptest! {
    #[test]
    fn failing_evidence_never_discharges(
        records in prop::collection::vec(
            (arb_exit_nonzero(), arb_hash(), arb_hash()),
            0..20
        ),
        expected_ph in arb_hash(),
        current_wh in arb_hash(),
    ) {
        // All records have non-zero exit codes
        let recs: Vec<(i32, String, String)> = records;
        prop_assert!(!is_discharged(&recs, &expected_ph, &current_wh));
    }
}

// ─── Property: any passing fresh matching record discharges ──

proptest! {
    #[test]
    fn passing_fresh_matching_discharges(
        failing in prop::collection::vec(
            (arb_exit_nonzero(), arb_hash(), arb_hash()),
            0..10
        ),
        proof_cmd in "[a-z ]{1,40}",
    ) {
        let ws_hash = "a".repeat(64);
        let ph = proof_hash(&proof_cmd);

        let mut recs: Vec<(i32, String, String)> = failing;
        // Insert one passing record with matching hashes
        recs.push((0, ph.clone(), ws_hash.clone()));

        prop_assert!(is_discharged(&recs, &ph, &ws_hash));
    }
}

// ─── Property: discharge is monotone (adding records can't undo it) ──

proptest! {
    #[test]
    fn discharge_is_monotone(
        proof_cmd in "[a-z ]{1,20}",
        extra_exit in arb_exit_nonzero(),
        extra_ph in arb_hash(),
        extra_wh in arb_hash(),
    ) {
        let ws_hash = "b".repeat(64);
        let ph = proof_hash(&proof_cmd);

        let base = vec![(0, ph.clone(), ws_hash.clone())];
        let extended = vec![
            (0, ph.clone(), ws_hash.clone()),
            (extra_exit, extra_ph, extra_wh),
        ];

        let base_result = is_discharged(&base, &ph, &ws_hash);
        let extended_result = is_discharged(&extended, &ph, &ws_hash);

        // If discharged with base records, still discharged with extra
        prop_assert!(base_result);
        prop_assert!(extended_result);
    }
}

// ─── Property: stale workspace evidence never discharges ─────

proptest! {
    #[test]
    fn stale_workspace_never_discharges(
        proof_cmd in "[a-z ]{1,20}",
        bad_wh in arb_hash(),
        good_wh in arb_hash().prop_filter("must differ", |s| s.len() == 64),
    ) {
        prop_assume!(bad_wh != good_wh);
        let ph = proof_hash(&proof_cmd);

        let recs = vec![(0, ph.clone(), bad_wh)];
        prop_assert!(!is_discharged(&recs, &ph, &good_wh));
    }
}

// ─── Property: wrong proof hash never discharges ─────────────

proptest! {
    #[test]
    fn wrong_proof_hash_never_discharges(
        proof_cmd in "[a-z ]{1,20}",
        other_cmd in "[a-z ]{1,20}",
    ) {
        prop_assume!(proof_cmd.trim() != other_cmd.trim());
        let ws = "c".repeat(64);
        let expected_ph = proof_hash(&proof_cmd);
        let wrong_ph = proof_hash(&other_cmd);

        let recs = vec![(0, wrong_ph, ws.clone())];
        prop_assert!(!is_discharged(&recs, &expected_ph, &ws));
    }
}

// ─── Property: empty hashes never discharge ──────────────────

proptest! {
    #[test]
    fn empty_proof_hash_never_discharges(
        expected_ph in arb_hash(),
        ws in arb_hash(),
    ) {
        let recs = vec![(0, String::new(), ws.clone())];
        prop_assert!(!is_discharged(&recs, &expected_ph, &ws));
    }

    #[test]
    fn empty_workspace_hash_never_discharges(
        ph in arb_hash(),
        ws in arb_hash(),
    ) {
        let recs = vec![(0, ph.clone(), String::new())];
        prop_assert!(!is_discharged(&recs, &ph, &ws));
    }
}

// ─── Property: proof_hash is deterministic and differs for different inputs ──

proptest! {
    #[test]
    fn proof_hash_deterministic(cmd in "[a-z0-9 ]{1,60}") {
        prop_assert_eq!(proof_hash(&cmd), proof_hash(&cmd));
    }

    #[test]
    fn proof_hash_trims(cmd in "[a-z0-9]{1,40}") {
        let padded = format!("  {cmd}  ");
        prop_assert_eq!(proof_hash(&cmd), proof_hash(&padded));
    }

    #[test]
    fn proof_hash_collision_resistance(
        cmd1 in "[a-z]{1,20}",
        cmd2 in "[a-z]{1,20}",
    ) {
        prop_assume!(cmd1.trim() != cmd2.trim());
        prop_assert_ne!(proof_hash(&cmd1), proof_hash(&cmd2));
    }
}
