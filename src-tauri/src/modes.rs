/// Built-in modes: multi-step workflows combining multiple commands with constraint satisfaction
/// Proof that the semantic pipeline architecture works before full App.tsx refactor
use serde::{Deserialize, Serialize};

/// A mode is a named multi-step workflow with concurrent groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    pub id: String,
    pub name: String,
    pub description: String,
    pub groups: Vec<ConcurrentGroup>,
}

/// Steps in a group execute concurrently; groups execute sequentially
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrentGroup {
    pub label: String,
    pub steps: Vec<ModeStep>,
}

/// A single action within a mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeStep {
    pub action: String, // "enable_dnd", "close_app", "set_brightness", etc
    pub target: Option<String>,
    pub params: std::collections::HashMap<String, serde_json::Value>,
}

pub fn builtin_modes() -> Vec<Mode> {
    vec![study_mode(), focus_mode(), reading_mode()]
}

pub fn get_mode(id: &str) -> Option<Mode> {
    builtin_modes().into_iter().find(|m| m.id == id)
}

/// Study Mode: optimized for focused work on complex tasks
/// Disables interruptions, arranges workspace, dims brightness, sets timer
fn study_mode() -> Mode {
    Mode {
        id: "study".to_string(),
        name: "Study Mode".to_string(),
        description: "Optimized for focused deep work: disable notifications, dim display, close distractions"
            .to_string(),
        groups: vec![
            // Group 1: Disable interruptions (concurrent)
            ConcurrentGroup {
                label: "Disable interruptions".to_string(),
                steps: vec![
                    ModeStep {
                        action: "enable_dnd".to_string(),
                        target: None,
                        params: std::collections::HashMap::new(),
                    },
                    ModeStep {
                        action: "close_app".to_string(),
                        target: Some("Slack".to_string()),
                        params: std::collections::HashMap::new(),
                    },
                    ModeStep {
                        action: "close_app".to_string(),
                        target: Some("Discord".to_string()),
                        params: std::collections::HashMap::new(),
                    },
                    ModeStep {
                        action: "close_app".to_string(),
                        target: Some("Telegram".to_string()),
                        params: std::collections::HashMap::new(),
                    },
                ],
            },
            // Group 2: Optimize display (after interruptions disabled)
            ConcurrentGroup {
                label: "Optimize display".to_string(),
                steps: vec![
                    ModeStep {
                        action: "set_brightness".to_string(),
                        target: None,
                        params: {
                            let mut m = std::collections::HashMap::new();
                            m.insert("level".to_string(), serde_json::json!(75));
                            m
                        },
                    },
                    ModeStep {
                        action: "enable_dark_mode".to_string(),
                        target: None,
                        params: std::collections::HashMap::new(),
                    },
                ],
            },
        ],
    }
}

/// Focus Mode: lightweight interruption-only suppression
/// Just enable DND, everything else manual
fn focus_mode() -> Mode {
    Mode {
        id: "focus".to_string(),
        name: "Focus Mode".to_string(),
        description: "Lightweight mode: just disable notifications".to_string(),
        groups: vec![ConcurrentGroup {
            label: "Enable focus".to_string(),
            steps: vec![ModeStep {
                action: "enable_dnd".to_string(),
                target: None,
                params: std::collections::HashMap::new(),
            }],
        }],
    }
}

/// Reading Mode: optimize for reading with dark mode and high brightness
/// Disables notifications, enables dark mode, maximizes brightness
fn reading_mode() -> Mode {
    Mode {
        id: "reading".to_string(),
        name: "Reading Mode".to_string(),
        description: "Optimize for reading: dark mode, high brightness, disable notifications"
            .to_string(),
        groups: vec![
            ConcurrentGroup {
                label: "Disable interruptions".to_string(),
                steps: vec![ModeStep {
                    action: "enable_dnd".to_string(),
                    target: None,
                    params: std::collections::HashMap::new(),
                }],
            },
            ConcurrentGroup {
                label: "Optimize display for reading".to_string(),
                steps: vec![
                    ModeStep {
                        action: "enable_dark_mode".to_string(),
                        target: None,
                        params: std::collections::HashMap::new(),
                    },
                    ModeStep {
                        action: "set_brightness".to_string(),
                        target: None,
                        params: {
                            let mut m = std::collections::HashMap::new();
                            m.insert("level".to_string(), serde_json::json!(90));
                            m
                        },
                    },
                ],
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_study_mode_exists() {
        let mode = get_mode("study").expect("study mode should exist");
        assert_eq!(mode.name, "Study Mode");
        assert_eq!(mode.groups.len(), 2); // interruptions + display
    }

    #[test]
    fn test_focus_mode_exists() {
        let mode = get_mode("focus").expect("focus mode should exist");
        assert_eq!(mode.name, "Focus Mode");
        assert_eq!(mode.groups.len(), 1); // just DND
    }

    #[test]
    fn test_reading_mode_exists() {
        let mode = get_mode("reading").expect("reading mode should exist");
        assert_eq!(mode.name, "Reading Mode");
        assert_eq!(mode.groups.len(), 2); // interruptions + display
    }

    #[test]
    fn test_builtin_modes_has_all_three() {
        let modes = builtin_modes();
        assert_eq!(modes.len(), 3);
        assert!(modes.iter().any(|m| m.id == "study"));
        assert!(modes.iter().any(|m| m.id == "focus"));
        assert!(modes.iter().any(|m| m.id == "reading"));
    }

    #[test]
    fn test_study_mode_groups_sequential() {
        let mode = get_mode("study").expect("study mode");
        // Group 0: close apps (needs interruptions disabled first)
        // Group 1: set brightness (only after interruptions handled)
        assert!(mode.groups[0].label.contains("interrupt"));
        assert!(mode.groups[1].label.contains("display"));
    }

    #[test]
    fn test_mode_steps_have_actions() {
        let mode = get_mode("study").expect("study mode");
        for group in &mode.groups {
            for step in &group.steps {
                assert!(!step.action.is_empty());
            }
        }
    }
}
