use std::collections::HashMap;

use crate::ledger::evidence::{self, Evidence};
use crate::ledger::obligations::Obligation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
}

#[derive(Debug)]
pub struct GateEval<'a> {
    pub verdict: Verdict,
    pub obligations_total: usize,
    pub open_critical: Vec<&'a Obligation>,
}

pub fn evaluate<'a>(
    obligations: &'a [Obligation],
    evidence_by_id: &HashMap<String, Vec<Evidence>>,
    current_workspace_hash: &str,
) -> GateEval<'a> {
    let mut open_critical = Vec::new();

    for ob in obligations {
        if !ob.critical {
            continue;
        }
        let expected_proof_hash = evidence::proof_hash(&ob.proof_cmd);
        let discharged = evidence_by_id
            .get(&ob.id)
            .map(|recs| evidence::is_discharged(recs, &expected_proof_hash, current_workspace_hash))
            .unwrap_or(false);
        if !discharged {
            open_critical.push(ob);
        }
    }

    let verdict = if open_critical.is_empty() {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    GateEval {
        verdict,
        obligations_total: obligations.len(),
        open_critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ObligationKind;

    fn ob(id: &str, proof_cmd: &str, critical: bool) -> Obligation {
        Obligation {
            id: id.into(),
            claim: format!("claim for {id}"),
            kind: ObligationKind::Other,
            critical,
            proof_cmd: proof_cmd.into(),
            created_at: chrono::Utc::now(),
        }
    }

    fn ev(obligation_id: &str, exit_code: i32, proof_cmd: &str, ws_hash: &str) -> Evidence {
        Evidence {
            obligation_id: obligation_id.into(),
            command: proof_cmd.into(),
            exit_code,
            stdout_tail: String::new(),
            stderr_tail: String::new(),
            proof_hash: evidence::proof_hash(proof_cmd),
            workspace_hash: ws_hash.into(),
            recorded_at: chrono::Utc::now(),
        }
    }

    const WS: &str = "current_workspace_hash_abc123";

    #[test]
    fn no_obligations_is_pass() {
        let result = evaluate(&[], &HashMap::new(), WS);
        assert_eq!(result.verdict, Verdict::Pass);
        assert!(result.open_critical.is_empty());
    }

    #[test]
    fn all_advisory_is_pass() {
        let obs = vec![ob("O-001", "true", false), ob("O-002", "echo", false)];
        let result = evaluate(&obs, &HashMap::new(), WS);
        assert_eq!(result.verdict, Verdict::Pass);
        assert_eq!(result.obligations_total, 2);
    }

    #[test]
    fn critical_without_evidence_is_fail() {
        let obs = vec![ob("O-001", "true", true)];
        let result = evaluate(&obs, &HashMap::new(), WS);
        assert_eq!(result.verdict, Verdict::Fail);
        assert_eq!(result.open_critical.len(), 1);
        assert_eq!(result.open_critical[0].id, "O-001");
    }

    #[test]
    fn critical_with_passing_fresh_evidence_is_pass() {
        let obs = vec![ob("O-001", "true", true)];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "true", WS)]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Pass);
    }

    #[test]
    fn stale_evidence_does_not_discharge() {
        let obs = vec![ob("O-001", "true", true)];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "true", "old_hash")]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Fail);
    }

    #[test]
    fn wrong_proof_hash_does_not_discharge() {
        let obs = vec![ob("O-001", "true", true)];
        let mut evidence_map = HashMap::new();
        // Evidence was recorded with a different proof command
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "echo bypass", WS)]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Fail);
    }

    #[test]
    fn failing_evidence_does_not_discharge() {
        let obs = vec![ob("O-001", "true", true)];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 1, "true", WS)]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Fail);
    }

    #[test]
    fn mixed_critical_and_advisory() {
        let obs = vec![
            ob("O-001", "true", true),
            ob("O-002", "false", false), // advisory — doesn't block
        ];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "true", WS)]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Pass);
        assert_eq!(result.obligations_total, 2);
    }

    #[test]
    fn multiple_critical_one_open() {
        let obs = vec![ob("O-001", "true", true), ob("O-002", "test", true)];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "true", WS)]);
        // O-002 has no evidence
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Fail);
        assert_eq!(result.open_critical.len(), 1);
        assert_eq!(result.open_critical[0].id, "O-002");
    }

    #[test]
    fn unknown_evidence_ids_are_ignored() {
        let obs = vec![ob("O-001", "true", true)];
        let mut evidence_map = HashMap::new();
        evidence_map.insert("O-001".into(), vec![ev("O-001", 0, "true", WS)]);
        evidence_map.insert("O-999".into(), vec![ev("O-999", 0, "true", WS)]);
        let result = evaluate(&obs, &evidence_map, WS);
        assert_eq!(result.verdict, Verdict::Pass);
    }

    #[test]
    fn discharge_is_monotone() {
        let obs = vec![ob("O-001", "true", true)];
        let passing = ev("O-001", 0, "true", WS);
        let extra_fail = ev("O-001", 1, "true", WS);

        let mut map1 = HashMap::new();
        map1.insert("O-001".into(), vec![passing.clone()]);
        let r1 = evaluate(&obs, &map1, WS);

        let mut map2 = HashMap::new();
        map2.insert("O-001".into(), vec![passing, extra_fail]);
        let r2 = evaluate(&obs, &map2, WS);

        // Adding a failing record doesn't un-discharge
        assert_eq!(r1.verdict, Verdict::Pass);
        assert_eq!(r2.verdict, Verdict::Pass);
    }
}
