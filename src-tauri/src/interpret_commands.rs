use crate::{arbiter, interpret_local};

#[tauri::command]
pub async fn debug_interpret_local(input: String) -> Result<String, String> {
    let candidates = interpret_local::interpret(&input);
    let arbitration = arbiter::decide(&candidates);

    let mut lines = vec![
        format!("decision: {:?}", arbitration.decision),
        format!("explanation: {}", arbitration.explanation),
    ];

    for (index, candidate) in candidates.iter().enumerate() {
        let missing = if candidate.missing_slots.is_empty() {
            "none".to_string()
        } else {
            candidate.missing_slots.join(", ")
        };

        let slots = if candidate.slots.is_empty() {
            "none".to_string()
        } else {
            candidate
                .slots
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect::<Vec<_>>()
                .join(", ")
        };

        lines.push(format!(
            "candidate[{index}]: family={:?} action={:?} executor={:?} source={:?} confidence={:.2} slots={} missing_slots={} clarify={}",
            candidate.family,
            candidate.canonical_action,
            candidate.executor_family,
            candidate.source,
            candidate.confidence,
            slots,
            missing,
            candidate.clarification_needed,
        ));
    }

    Ok(lines.join("\n"))
}
