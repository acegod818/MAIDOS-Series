//! Capability Policy Engine
//!
//! <impl>
//! WHAT: Rule-based policy engine for capability access control
//! WHY: Enable dynamic, configurable access control beyond simple capability checks
//! HOW: Policy rules with conditions, actions, and evaluation engine
//! TEST: Unit tests for rule matching, condition evaluation, policy decisions
//! </impl>

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::capability::{Capability, CapabilitySet};

/// Policy decision result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyDecision {
    /// Access is allowed
    Allow,
    /// Access is denied
    Deny,
    /// No matching rule found, use default
    NoMatch,
}

/// Condition operator for rule matching
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOp {
    /// Equals
    Eq,
    /// Not equals
    Ne,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Greater than or equal
    Gte,
    /// Less than or equal
    Lte,
    /// Contains (for strings/lists)
    Contains,
    /// Matches regex pattern
    Matches,
    /// Is in list
    In,
    /// Is not in list
    NotIn,
}

/// Condition value types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    /// String value
    String(String),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// List of strings
    StringList(Vec<String>),
}

impl ConditionValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ConditionValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ConditionValue::Int(i) => Some(*i),
            ConditionValue::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConditionValue::Float(f) => Some(*f),
            ConditionValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }
}

/// Policy condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Field to check
    pub field: String,
    /// Operator
    pub op: ConditionOp,
    /// Value to compare
    pub value: ConditionValue,
}

impl Condition {
    pub fn new(field: impl Into<String>, op: ConditionOp, value: ConditionValue) -> Self {
        Self {
            field: field.into(),
            op,
            value,
        }
    }

    /// Evaluate condition against context
    pub fn evaluate(&self, context: &PolicyContext) -> bool {
        let ctx_value = match context.get(&self.field) {
            Some(v) => v,
            None => return false,
        };

        match self.op {
            ConditionOp::Eq => ctx_value == &self.value,
            ConditionOp::Ne => ctx_value != &self.value,
            ConditionOp::Gt => self.compare_numeric(ctx_value, |a, b| a > b),
            ConditionOp::Lt => self.compare_numeric(ctx_value, |a, b| a < b),
            ConditionOp::Gte => self.compare_numeric(ctx_value, |a, b| a >= b),
            ConditionOp::Lte => self.compare_numeric(ctx_value, |a, b| a <= b),
            ConditionOp::Contains => self.check_contains(ctx_value),
            ConditionOp::Matches => self.check_matches(ctx_value),
            ConditionOp::In => self.check_in(ctx_value),
            ConditionOp::NotIn => !self.check_in(ctx_value),
        }
    }

    fn compare_numeric<F>(&self, ctx_value: &ConditionValue, cmp: F) -> bool
    where
        F: Fn(f64, f64) -> bool,
    {
        match (ctx_value.as_f64(), self.value.as_f64()) {
            (Some(a), Some(b)) => cmp(a, b),
            _ => false,
        }
    }

    fn check_contains(&self, ctx_value: &ConditionValue) -> bool {
        match (ctx_value, &self.value) {
            (ConditionValue::String(haystack), ConditionValue::String(needle)) => {
                haystack.contains(needle)
            }
            (ConditionValue::StringList(list), ConditionValue::String(item)) => {
                list.contains(item)
            }
            _ => false,
        }
    }

    fn check_matches(&self, ctx_value: &ConditionValue) -> bool {
        match (ctx_value.as_str(), self.value.as_str()) {
            (Some(s), Some(pattern)) => {
                // Simple glob matching (not regex for simplicity)
                if pattern == "*" {
                    true
                } else if let (Some(start), true) = (pattern.strip_prefix('*'), pattern.ends_with('*')) {
                    let inner = start.strip_suffix('*').unwrap_or(start);
                    s.contains(inner)
                } else if let Some(suffix) = pattern.strip_prefix('*') {
                    s.ends_with(suffix)
                } else if let Some(prefix) = pattern.strip_suffix('*') {
                    s.starts_with(prefix)
                } else {
                    s == pattern
                }
            }
            _ => false,
        }
    }

    fn check_in(&self, ctx_value: &ConditionValue) -> bool {
        match (&self.value, ctx_value.as_str()) {
            (ConditionValue::StringList(list), Some(s)) => list.iter().any(|item| item == s),
            _ => false,
        }
    }
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule name/ID
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Required capability (if any)
    pub capability: Option<Capability>,
    /// Conditions that must all be true
    pub conditions: Vec<Condition>,
    /// Decision if rule matches
    pub decision: PolicyDecision,
    /// Priority (higher = evaluated first)
    pub priority: i32,
    /// Is rule enabled
    pub enabled: bool,
}

impl PolicyRule {
    pub fn new(name: impl Into<String>, decision: PolicyDecision) -> Self {
        Self {
            name: name.into(),
            description: None,
            capability: None,
            conditions: Vec::new(),
            decision,
            priority: 0,
            enabled: true,
        }
    }

    pub fn with_capability(mut self, cap: Capability) -> Self {
        self.capability = Some(cap);
        self
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Check if rule matches the given context
    pub fn matches(&self, context: &PolicyContext) -> bool {
        if !self.enabled {
            return false;
        }

        // Check capability if specified
        if let Some(cap) = self.capability {
            if let Some(caps) = &context.capabilities {
                if !caps.has(cap) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check all conditions
        self.conditions.iter().all(|c| c.evaluate(context))
    }
}

/// Policy context for evaluation
#[derive(Debug, Clone, Default)]
pub struct PolicyContext {
    /// Context values
    values: HashMap<String, ConditionValue>,
    /// Capabilities to check
    capabilities: Option<CapabilitySet>,
}

impl PolicyContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capabilities(mut self, caps: CapabilitySet) -> Self {
        self.capabilities = Some(caps);
        self
    }

    pub fn set(&mut self, key: impl Into<String>, value: ConditionValue) {
        self.values.insert(key.into(), value);
    }

    pub fn set_string(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values
            .insert(key.into(), ConditionValue::String(value.into()));
    }

    pub fn set_int(&mut self, key: impl Into<String>, value: i64) {
        self.values.insert(key.into(), ConditionValue::Int(value));
    }

    pub fn set_bool(&mut self, key: impl Into<String>, value: bool) {
        self.values.insert(key.into(), ConditionValue::Bool(value));
    }

    pub fn get(&self, key: &str) -> Option<&ConditionValue> {
        self.values.get(key)
    }

    /// Create context with common fields
    pub fn with_defaults(mut self) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        self.set_int("timestamp", now);

        // Add hour of day (0-23)
        let hour = (now % 86400) / 3600;
        self.set_int("hour", hour);

        // Add day of week (0=Sun, 6=Sat) - simplified
        let day = ((now / 86400) + 4) % 7; // Jan 1, 1970 was Thursday (4)
        self.set_int("day_of_week", day);

        self
    }
}

/// Policy engine
pub struct PolicyEngine {
    rules: Vec<PolicyRule>,
    default_decision: PolicyDecision,
}

impl PolicyEngine {
    /// Create a new policy engine with default deny
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_decision: PolicyDecision::Deny,
        }
    }

    /// Create with default allow
    pub fn with_default_allow() -> Self {
        Self {
            rules: Vec::new(),
            default_decision: PolicyDecision::Allow,
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        // Sort by priority (descending)
        self.rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Remove a rule by name
    pub fn remove_rule(&mut self, name: &str) -> bool {
        let len_before = self.rules.len();
        self.rules.retain(|r| r.name != name);
        self.rules.len() < len_before
    }

    /// Evaluate policy for a context
    pub fn evaluate(&self, context: &PolicyContext) -> PolicyDecision {
        for rule in &self.rules {
            if rule.matches(context) {
                return rule.decision;
            }
        }
        self.default_decision
    }

    /// Check if access is allowed
    pub fn is_allowed(&self, context: &PolicyContext) -> bool {
        matches!(self.evaluate(context), PolicyDecision::Allow)
    }

    /// Get all rules
    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    /// Get enabled rules count
    pub fn enabled_rules_count(&self) -> usize {
        self.rules.iter().filter(|r| r.enabled).count()
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_eq() {
        let cond = Condition::new("role", ConditionOp::Eq, ConditionValue::String("admin".into()));

        let mut ctx = PolicyContext::new();
        ctx.set_string("role", "admin");
        assert!(cond.evaluate(&ctx));

        ctx.set_string("role", "user");
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_condition_numeric() {
        let cond = Condition::new("age", ConditionOp::Gte, ConditionValue::Int(18));

        let mut ctx = PolicyContext::new();
        ctx.set_int("age", 21);
        assert!(cond.evaluate(&ctx));

        ctx.set_int("age", 16);
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_condition_contains() {
        let cond = Condition::new(
            "email",
            ConditionOp::Contains,
            ConditionValue::String("@example.com".into()),
        );

        let mut ctx = PolicyContext::new();
        ctx.set_string("email", "user@example.com");
        assert!(cond.evaluate(&ctx));

        ctx.set_string("email", "user@other.com");
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_condition_in() {
        let cond = Condition::new(
            "role",
            ConditionOp::In,
            ConditionValue::StringList(vec!["admin".into(), "moderator".into()]),
        );

        let mut ctx = PolicyContext::new();
        ctx.set_string("role", "admin");
        assert!(cond.evaluate(&ctx));

        ctx.set_string("role", "user");
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_condition_matches_glob() {
        let cond = Condition::new(
            "path",
            ConditionOp::Matches,
            ConditionValue::String("/api/*".into()),
        );

        let mut ctx = PolicyContext::new();
        ctx.set_string("path", "/api/users");
        assert!(cond.evaluate(&ctx));

        ctx.set_string("path", "/web/users");
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_policy_rule_basic() {
        let rule = PolicyRule::new("allow_admin", PolicyDecision::Allow)
            .with_condition(Condition::new(
                "role",
                ConditionOp::Eq,
                ConditionValue::String("admin".into()),
            ));

        let mut ctx = PolicyContext::new();
        ctx.set_string("role", "admin");
        assert!(rule.matches(&ctx));

        ctx.set_string("role", "user");
        assert!(!rule.matches(&ctx));
    }

    #[test]
    fn test_policy_rule_with_capability() {
        let rule = PolicyRule::new("allow_file_read", PolicyDecision::Allow)
            .with_capability(Capability::FileRead);

        let caps = CapabilitySet::from_iter([Capability::FileRead]);
        let ctx = PolicyContext::new().with_capabilities(caps);
        assert!(rule.matches(&ctx));

        let caps_no_read = CapabilitySet::from_iter([Capability::FileWrite]);
        let ctx_no_read = PolicyContext::new().with_capabilities(caps_no_read);
        assert!(!rule.matches(&ctx_no_read));
    }

    #[test]
    fn test_policy_engine_priority() {
        let mut engine = PolicyEngine::new();

        // Lower priority deny rule
        engine.add_rule(
            PolicyRule::new("deny_all", PolicyDecision::Deny)
                .with_priority(0)
                .with_condition(Condition::new(
                    "active",
                    ConditionOp::Eq,
                    ConditionValue::Bool(true),
                )),
        );

        // Higher priority allow rule for admins
        engine.add_rule(
            PolicyRule::new("allow_admin", PolicyDecision::Allow)
                .with_priority(10)
                .with_condition(Condition::new(
                    "role",
                    ConditionOp::Eq,
                    ConditionValue::String("admin".into()),
                )),
        );

        let mut ctx = PolicyContext::new();
        ctx.set_string("role", "admin");
        ctx.set_bool("active", true);

        // Admin should be allowed (higher priority rule)
        assert_eq!(engine.evaluate(&ctx), PolicyDecision::Allow);

        ctx.set_string("role", "user");
        // Non-admin should be denied
        assert_eq!(engine.evaluate(&ctx), PolicyDecision::Deny);
    }

    #[test]
    fn test_policy_engine_default_decision() {
        let engine_deny = PolicyEngine::new();
        let engine_allow = PolicyEngine::with_default_allow();

        let ctx = PolicyContext::new();

        assert_eq!(engine_deny.evaluate(&ctx), PolicyDecision::Deny);
        assert_eq!(engine_allow.evaluate(&ctx), PolicyDecision::Allow);
    }

    #[test]
    fn test_policy_context_defaults() {
        let ctx = PolicyContext::new().with_defaults();

        assert!(ctx.get("timestamp").is_some());
        assert!(ctx.get("hour").is_some());
        assert!(ctx.get("day_of_week").is_some());
    }

    #[test]
    fn test_remove_rule() {
        let mut engine = PolicyEngine::new();
        engine.add_rule(PolicyRule::new("rule1", PolicyDecision::Allow));
        engine.add_rule(PolicyRule::new("rule2", PolicyDecision::Deny));

        assert_eq!(engine.rules().len(), 2);
        assert!(engine.remove_rule("rule1"));
        assert_eq!(engine.rules().len(), 1);
        assert!(!engine.remove_rule("rule1")); // Already removed
    }

    #[test]
    fn test_disabled_rule() {
        let mut engine = PolicyEngine::new();

        let mut rule = PolicyRule::new("disabled_rule", PolicyDecision::Allow);
        rule.enabled = false;
        engine.add_rule(rule);

        let ctx = PolicyContext::new();
        // Should fall through to default (deny) because rule is disabled
        assert_eq!(engine.evaluate(&ctx), PolicyDecision::Deny);
        assert_eq!(engine.enabled_rules_count(), 0);
    }

    #[test]
    fn test_multiple_conditions() {
        let rule = PolicyRule::new("complex_rule", PolicyDecision::Allow)
            .with_condition(Condition::new(
                "role",
                ConditionOp::Eq,
                ConditionValue::String("user".into()),
            ))
            .with_condition(Condition::new("age", ConditionOp::Gte, ConditionValue::Int(18)))
            .with_condition(Condition::new(
                "verified",
                ConditionOp::Eq,
                ConditionValue::Bool(true),
            ));

        let mut ctx = PolicyContext::new();
        ctx.set_string("role", "user");
        ctx.set_int("age", 21);
        ctx.set_bool("verified", true);
        assert!(rule.matches(&ctx));

        // Missing one condition
        ctx.set_bool("verified", false);
        assert!(!rule.matches(&ctx));
    }
}
