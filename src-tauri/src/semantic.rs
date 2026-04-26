/// Semantic decomposition of raw input
/// Universal shape for all command types before intent interpretation
/// Does NOT decide what to do; just decomposes the shape of the user's request
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticFrame {
    /// Primary action verb (open, close, set, find, move, etc)
    pub verb: String,

    /// Object of the action (Safari, volume, file.txt, etc)
    /// Optional: some commands are verb-only ("maximize window")
    pub target: Option<String>,

    /// Scope/location qualifier (in Finder, on desktop, to trash, etc)
    /// Optional: narrows where action applies
    pub scope: Option<String>,

    /// Style/manner qualifier (gently, forcefully, quietly, permanently, etc)
    /// Optional: describes how to perform action
    pub qualifier: Option<String>,

    /// Temporal/duration reference (for 1 hour, at 3pm, until done, etc)
    /// Optional: when/how long
    pub temporal: Option<String>,

    /// Relative magnitude 0.0-1.0 (0=minimal, 1=maximum)
    /// Used for: "set volume to 50" → 0.5, "volume max" → 1.0, "dim brightness" → 0.3
    pub intensity: f32,
}

impl SemanticFrame {
    pub fn new(verb: String) -> Self {
        Self {
            verb,
            target: None,
            scope: None,
            qualifier: None,
            temporal: None,
            intensity: 0.5, // default: neutral
        }
    }

    pub fn with_target(mut self, target: String) -> Self {
        self.target = Some(target);
        self
    }

    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn with_qualifier(mut self, qualifier: String) -> Self {
        self.qualifier = Some(qualifier);
        self
    }

    pub fn with_temporal(mut self, temporal: String) -> Self {
        self.temporal = Some(temporal);
        self
    }

    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.intensity = intensity.clamp(0.0, 1.0);
        self
    }

    /// Check if this frame is "clear" (verb + target both present and recognized)
    pub fn is_clear(&self) -> bool {
        !self.verb.is_empty() && self.target.is_some()
    }

    /// Check if this frame is "complete" (has enough detail to execute)
    pub fn is_complete(&self) -> bool {
        self.is_clear() && self.intensity > 0.0
    }

    /// Get disambiguated verb (lowercase, trimmed)
    pub fn canonical_verb(&self) -> String {
        self.verb.trim().to_lowercase()
    }

    /// Get disambiguated target (lowercase, trimmed)
    pub fn canonical_target(&self) -> Option<String> {
        self.target.as_ref().map(|t| t.trim().to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_frame_builder() {
        let frame = SemanticFrame::new("open".to_string())
            .with_target("Safari".to_string())
            .with_intensity(1.0);

        assert_eq!(frame.verb, "open");
        assert_eq!(frame.target, Some("Safari".to_string()));
        assert_eq!(frame.intensity, 1.0);
        assert!(frame.is_clear());
    }

    #[test]
    fn test_canonical_forms() {
        let frame = SemanticFrame::new("OPEN".to_string()).with_target("  Safari  ".to_string());

        assert_eq!(frame.canonical_verb(), "open");
        assert_eq!(frame.canonical_target(), Some("safari".to_string()));
    }

    #[test]
    fn test_intensity_clamping() {
        let frame = SemanticFrame::new("set".to_string()).with_intensity(2.5); // should clamp to 1.0

        assert_eq!(frame.intensity, 1.0);
    }

    #[test]
    fn test_incomplete_frame() {
        let frame = SemanticFrame::new("open".to_string());
        assert!(!frame.is_clear()); // no target
        assert!(!frame.is_complete()); // no target or intensity
    }
}
