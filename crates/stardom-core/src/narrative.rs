use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptTrigger {
    MonthReached(u32),
    WeekReached(u32),
    Manual,
}

impl ScriptTrigger {
    pub fn matches_month(&self, month: u32) -> bool {
        matches!(self, ScriptTrigger::MonthReached(m) if *m == month)
    }
    pub fn matches_week(&self, week: u32) -> bool {
        matches!(self, ScriptTrigger::WeekReached(w) if *w == week)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptEffect {
    ChangeReputation(i32),
    ChangePopularity(i32),
    ChangeStress(i32),
    ChangeRecognition(i64),
    AddMoney(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub label: String,
    pub next_node: Option<u32>,
    pub effects: Vec<ScriptEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: u32,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<DialogueChoice>,
    pub next: Option<u32>,         // auto-advance if no choices
    pub condition: Option<String>, // placeholder for future conditions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptDef {
    pub id: ScriptId,
    pub name: String,
    pub trigger: ScriptTrigger,
    pub nodes: Vec<DialogueNode>,
}

impl ScriptDef {
    pub fn get_node(&self, id: u32) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

/// Walks through a script's dialogue tree.
pub struct ScriptRunner<'a> {
    script: &'a ScriptDef,
    current: Option<u32>,
}

impl<'a> ScriptRunner<'a> {
    pub fn new(script: &'a ScriptDef) -> Self {
        Self {
            script,
            current: script.nodes.first().map(|n| n.id),
        }
    }

    pub fn current_node_id(&self) -> u32 {
        self.current.unwrap_or(u32::MAX)
    }

    pub fn current_node(&self) -> Option<&DialogueNode> {
        self.current.and_then(|id| self.script.get_node(id))
    }

    pub fn is_finished(&self) -> bool {
        self.current.is_none()
    }

    /// Advance to next node. choice_index selects a choice if available.
    /// Returns effects from the chosen path.
    pub fn advance(&mut self, choice_index: Option<usize>) -> Vec<ScriptEffect> {
        let Some(node) = self.current_node() else {
            self.current = None;
            return vec![];
        };
        let (next_id, effects) = if let Some(idx) = choice_index {
            if let Some(choice) = node.choices.get(idx) {
                (choice.next_node, choice.effects.clone())
            } else {
                (node.next, vec![])
            }
        } else {
            (node.next, vec![])
        };
        self.current = next_id;
        effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a 3-node test script:
    /// node 0: auto-advance to node 1
    /// node 1: two choices — choice 0 goes nowhere (terminal), choice 1 goes to node 2
    /// node 2: terminal (next = None)
    fn sample_script() -> ScriptDef {
        ScriptDef {
            id: ScriptId(1),
            name: "Test Script".to_string(),
            trigger: ScriptTrigger::MonthReached(12),
            nodes: vec![
                DialogueNode {
                    id: 0,
                    speaker: "Manager".to_string(),
                    text: "Welcome.".to_string(),
                    choices: vec![],
                    next: Some(1),
                    condition: None,
                },
                DialogueNode {
                    id: 1,
                    speaker: "Artist".to_string(),
                    text: "What now?".to_string(),
                    choices: vec![
                        DialogueChoice {
                            label: "Leave".to_string(),
                            next_node: None,
                            effects: vec![],
                        },
                        DialogueChoice {
                            label: "Continue".to_string(),
                            next_node: Some(2),
                            effects: vec![ScriptEffect::ChangeReputation(5)],
                        },
                    ],
                    next: None,
                    condition: None,
                },
                DialogueNode {
                    id: 2,
                    speaker: "Manager".to_string(),
                    text: "Great work.".to_string(),
                    choices: vec![],
                    next: None,
                    condition: None,
                },
            ],
        }
    }

    #[test]
    fn script_starts_at_first_node() {
        let script = sample_script();
        let runner = ScriptRunner::new(&script);
        assert_eq!(runner.current_node_id(), 0);
    }

    #[test]
    fn advance_follows_next() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        // node 0 auto-advances to node 1
        let effects = runner.advance(None);
        assert!(effects.is_empty());
        assert_eq!(runner.current_node_id(), 1);
    }

    #[test]
    fn advance_with_choice() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        // advance from node 0 to node 1
        runner.advance(None);
        // at node 1, choose index 1 → node 2, effects = ChangeReputation(5)
        let effects = runner.advance(Some(1));
        assert_eq!(runner.current_node_id(), 2);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], ScriptEffect::ChangeReputation(5)));
    }

    #[test]
    fn runner_is_finished_at_terminal_node() {
        let script = sample_script();
        let mut runner = ScriptRunner::new(&script);
        runner.advance(None); // 0 → 1
        runner.advance(Some(1)); // 1 → 2
        runner.advance(None); // 2 → None (terminal)
        assert!(runner.is_finished());
    }

    #[test]
    fn trigger_matches_month() {
        let trigger = ScriptTrigger::MonthReached(12);
        assert!(trigger.matches_month(12));
        assert!(!trigger.matches_month(6));
    }

    #[test]
    fn serialization_roundtrip() {
        let script = sample_script();
        let serialized = ron::ser::to_string_pretty(&script, ron::ser::PrettyConfig::default())
            .expect("serialize");
        let deserialized: ScriptDef = ron::from_str(&serialized).expect("deserialize");
        assert_eq!(deserialized.id, script.id);
        assert_eq!(deserialized.name, script.name);
        assert_eq!(deserialized.nodes.len(), script.nodes.len());
        assert_eq!(deserialized.nodes[1].choices.len(), 2);
    }
}
