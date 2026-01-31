//! LLM Budget Control
//!
//! <impl>
//! WHAT: Usage tracking and budget enforcement for LLM API calls
//! WHY: Prevent cost overruns and enable usage monitoring
//! HOW: Track tokens/costs per provider, enforce limits, auto-downgrade
//! TEST: Unit tests for tracking, limit enforcement, reset cycles
//! </impl>

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{LlmError, Result};

/// Time period for budget tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum BudgetPeriod {
    /// Daily budget (resets at midnight UTC)
    Daily,
    /// Weekly budget (resets Sunday midnight UTC)
    Weekly,
    /// Monthly budget (resets 1st of month UTC)
    #[default]
    Monthly,
    /// No automatic reset
    Lifetime,
}


/// Budget limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimit {
    /// Maximum cost in USD
    pub max_cost: f64,
    /// Maximum input tokens
    pub max_input_tokens: Option<u64>,
    /// Maximum output tokens
    pub max_output_tokens: Option<u64>,
    /// Maximum total tokens
    pub max_total_tokens: Option<u64>,
    /// Maximum requests
    pub max_requests: Option<u64>,
    /// Budget period
    pub period: BudgetPeriod,
    /// Warning threshold (0.0-1.0)
    pub warning_threshold: f64,
}

impl Default for BudgetLimit {
    fn default() -> Self {
        Self {
            max_cost: 100.0,
            max_input_tokens: None,
            max_output_tokens: None,
            max_total_tokens: None,
            max_requests: None,
            period: BudgetPeriod::Monthly,
            warning_threshold: 0.8,
        }
    }
}

/// Usage statistics for a provider
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    /// Total cost in USD
    pub total_cost: f64,
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Total requests
    pub requests: u64,
    /// Last reset timestamp (Unix epoch seconds)
    pub last_reset: u64,
    /// Period start timestamp
    pub period_start: u64,
}

impl UsageStats {
    fn new() -> Self {
        let now = current_timestamp();
        Self {
            total_cost: 0.0,
            input_tokens: 0,
            output_tokens: 0,
            requests: 0,
            last_reset: now,
            period_start: now,
        }
    }

    fn reset(&mut self) {
        let now = current_timestamp();
        self.total_cost = 0.0;
        self.input_tokens = 0;
        self.output_tokens = 0;
        self.requests = 0;
        self.last_reset = now;
        self.period_start = now;
    }
}

/// Budget status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetStatus {
    /// Provider name
    pub provider: String,
    /// Current usage stats
    pub usage: UsageStats,
    /// Budget limit
    pub limit: BudgetLimit,
    /// Remaining cost budget
    pub remaining_cost: f64,
    /// Usage percentage (0.0-1.0)
    pub usage_percentage: f64,
    /// Is at warning level
    pub at_warning: bool,
    /// Is exceeded
    pub exceeded: bool,
    /// Time until reset (seconds)
    pub time_until_reset: Option<u64>,
}

/// Budget exceeded action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ExceededAction {
    /// Block all requests
    Block,
    /// Downgrade to cheaper provider
    Downgrade,
    /// Allow with warning
    #[default]
    Warn,
    /// No action (just track)
    Track,
}


/// Budget controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Global budget limit
    pub global_limit: Option<BudgetLimit>,
    /// Per-provider limits
    pub provider_limits: HashMap<String, BudgetLimit>,
    /// Action when budget exceeded
    pub exceeded_action: ExceededAction,
    /// Downgrade provider order
    pub downgrade_order: Vec<String>,
    /// Enable auto-reset
    pub auto_reset: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            global_limit: Some(BudgetLimit::default()),
            provider_limits: HashMap::new(),
            exceeded_action: ExceededAction::Warn,
            downgrade_order: Vec::new(),
            auto_reset: true,
        }
    }
}

/// Budget controller for LLM usage
pub struct BudgetController {
    config: BudgetConfig,
    global_usage: Arc<RwLock<UsageStats>>,
    provider_usage: Arc<RwLock<HashMap<String, UsageStats>>>,
}

impl BudgetController {
    /// Create a new budget controller
    pub fn new(config: BudgetConfig) -> Self {
        Self {
            config,
            global_usage: Arc::new(RwLock::new(UsageStats::new())),
            provider_usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record usage for a request
    pub fn record_usage(
        &self,
        provider: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
    ) {
        // Update global usage
        {
            let mut global = self.global_usage.write();
            self.maybe_reset_period(&mut global, self.config.global_limit.as_ref());
            global.total_cost += cost;
            global.input_tokens += input_tokens;
            global.output_tokens += output_tokens;
            global.requests += 1;
        }

        // Update provider usage
        {
            let mut providers = self.provider_usage.write();
            let usage = providers.entry(provider.to_string()).or_default();
            
            let limit = self.config.provider_limits.get(provider);
            self.maybe_reset_period(usage, limit);
            
            usage.total_cost += cost;
            usage.input_tokens += input_tokens;
            usage.output_tokens += output_tokens;
            usage.requests += 1;
        }
    }

    /// Check if a request is allowed
    pub fn check_budget(&self, provider: &str) -> Result<()> {
        // Check global limit
        if let Some(ref limit) = self.config.global_limit {
            let global = self.global_usage.read();
            if self.is_exceeded(&global, limit) {
                return self.handle_exceeded("global");
            }
        }

        // Check provider limit
        if let Some(limit) = self.config.provider_limits.get(provider) {
            let providers = self.provider_usage.read();
            if let Some(usage) = providers.get(provider) {
                if self.is_exceeded(usage, limit) {
                    return self.handle_exceeded(provider);
                }
            }
        }

        Ok(())
    }

    /// Get budget status for a provider
    pub fn get_status(&self, provider: &str) -> Option<BudgetStatus> {
        let providers = self.provider_usage.read();
        let usage = providers.get(provider)?;
        let limit = self.config.provider_limits.get(provider)?;

        Some(self.build_status(provider, usage, limit))
    }

    /// Get global budget status
    pub fn get_global_status(&self) -> Option<BudgetStatus> {
        let limit = self.config.global_limit.as_ref()?;
        let usage = self.global_usage.read();

        Some(self.build_status("global", &usage, limit))
    }

    /// Get all provider statuses
    pub fn get_all_statuses(&self) -> Vec<BudgetStatus> {
        let mut statuses = Vec::new();

        // Add global status
        if let Some(status) = self.get_global_status() {
            statuses.push(status);
        }

        // Add provider statuses
        let providers = self.provider_usage.read();
        for (name, usage) in providers.iter() {
            if let Some(limit) = self.config.provider_limits.get(name) {
                statuses.push(self.build_status(name, usage, limit));
            }
        }

        statuses
    }

    /// Get suggested provider based on budget
    pub fn suggest_provider(&self) -> Option<String> {
        // If exceeded, return downgrade option
        if let Some(ref limit) = self.config.global_limit {
            let global = self.global_usage.read();
            if self.is_exceeded(&global, limit) && !self.config.downgrade_order.is_empty() {
                // Find first provider not exceeded
                for provider in &self.config.downgrade_order {
                    if self.check_budget(provider).is_ok() {
                        return Some(provider.clone());
                    }
                }
            }
        }

        None
    }

    /// Reset usage for a provider
    pub fn reset_provider(&self, provider: &str) {
        let mut providers = self.provider_usage.write();
        if let Some(usage) = providers.get_mut(provider) {
            usage.reset();
        }
    }

    /// Reset global usage
    pub fn reset_global(&self) {
        let mut global = self.global_usage.write();
        global.reset();
    }

    /// Reset all usage
    pub fn reset_all(&self) {
        self.reset_global();
        let mut providers = self.provider_usage.write();
        for usage in providers.values_mut() {
            usage.reset();
        }
    }

    /// Get configuration
    pub fn config(&self) -> &BudgetConfig {
        &self.config
    }

    // Helper methods

    fn is_exceeded(&self, usage: &UsageStats, limit: &BudgetLimit) -> bool {
        if usage.total_cost >= limit.max_cost {
            return true;
        }
        if let Some(max) = limit.max_input_tokens {
            if usage.input_tokens >= max {
                return true;
            }
        }
        if let Some(max) = limit.max_output_tokens {
            if usage.output_tokens >= max {
                return true;
            }
        }
        if let Some(max) = limit.max_total_tokens {
            if usage.input_tokens + usage.output_tokens >= max {
                return true;
            }
        }
        if let Some(max) = limit.max_requests {
            if usage.requests >= max {
                return true;
            }
        }
        false
    }

    fn handle_exceeded(&self, provider: &str) -> Result<()> {
        match self.config.exceeded_action {
            ExceededAction::Block => {
                Err(LlmError::BudgetExceeded(format!("Budget exceeded for {}", provider)))
            }
            ExceededAction::Downgrade => {
                if let Some(alt) = self.suggest_provider() {
                    Err(LlmError::BudgetExceeded(format!(
                        "Budget exceeded for {}, suggest using {}",
                        provider, alt
                    )))
                } else {
                    Err(LlmError::BudgetExceeded(format!(
                        "Budget exceeded for {}, no alternatives available",
                        provider
                    )))
                }
            }
            ExceededAction::Warn => Ok(()),
            ExceededAction::Track => Ok(()),
        }
    }

    fn build_status(&self, provider: &str, usage: &UsageStats, limit: &BudgetLimit) -> BudgetStatus {
        let remaining = (limit.max_cost - usage.total_cost).max(0.0);
        let percentage = if limit.max_cost > 0.0 {
            usage.total_cost / limit.max_cost
        } else {
            0.0
        };

        BudgetStatus {
            provider: provider.to_string(),
            usage: usage.clone(),
            limit: limit.clone(),
            remaining_cost: remaining,
            usage_percentage: percentage,
            at_warning: percentage >= limit.warning_threshold,
            exceeded: self.is_exceeded(usage, limit),
            time_until_reset: self.time_until_reset(limit.period),
        }
    }

    fn maybe_reset_period(&self, usage: &mut UsageStats, limit: Option<&BudgetLimit>) {
        if !self.config.auto_reset {
            return;
        }

        let period = limit.map(|l| l.period).unwrap_or(BudgetPeriod::Monthly);
        if self.should_reset(usage.period_start, period) {
            usage.reset();
        }
    }

    fn should_reset(&self, period_start: u64, period: BudgetPeriod) -> bool {
        let now = current_timestamp();
        match period {
            BudgetPeriod::Daily => {
                let day_start = now - (now % 86400);
                period_start < day_start
            }
            BudgetPeriod::Weekly => {
                let week_start = now - (now % (7 * 86400));
                period_start < week_start
            }
            BudgetPeriod::Monthly => {
                // Simplified: 30 days
                now - period_start > 30 * 86400
            }
            BudgetPeriod::Lifetime => false,
        }
    }

    fn time_until_reset(&self, period: BudgetPeriod) -> Option<u64> {
        let now = current_timestamp();
        match period {
            BudgetPeriod::Daily => {
                let next_day = ((now / 86400) + 1) * 86400;
                Some(next_day - now)
            }
            BudgetPeriod::Weekly => {
                let next_week = ((now / (7 * 86400)) + 1) * (7 * 86400);
                Some(next_week - now)
            }
            BudgetPeriod::Monthly => {
                // Simplified: 30 days from period start
                Some(30 * 86400)
            }
            BudgetPeriod::Lifetime => None,
        }
    }
}

/// Budget controller builder
pub struct BudgetBuilder {
    config: BudgetConfig,
}

impl BudgetBuilder {
    pub fn new() -> Self {
        Self {
            config: BudgetConfig::default(),
        }
    }

    pub fn global_limit(mut self, limit: BudgetLimit) -> Self {
        self.config.global_limit = Some(limit);
        self
    }

    pub fn no_global_limit(mut self) -> Self {
        self.config.global_limit = None;
        self
    }

    pub fn provider_limit(mut self, provider: &str, limit: BudgetLimit) -> Self {
        self.config.provider_limits.insert(provider.to_string(), limit);
        self
    }

    pub fn exceeded_action(mut self, action: ExceededAction) -> Self {
        self.config.exceeded_action = action;
        self
    }

    pub fn downgrade_order(mut self, order: Vec<String>) -> Self {
        self.config.downgrade_order = order;
        self
    }

    pub fn auto_reset(mut self, enabled: bool) -> Self {
        self.config.auto_reset = enabled;
        self
    }

    pub fn build(self) -> BudgetController {
        BudgetController::new(self.config)
    }
}

impl Default for BudgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 10.0,
                ..Default::default()
            })
            .build();

        controller.record_usage("openai", 100, 50, 0.01);

        let status = controller.get_global_status().unwrap();
        assert_eq!(status.usage.input_tokens, 100);
        assert_eq!(status.usage.output_tokens, 50);
        assert_eq!(status.usage.requests, 1);
        assert!((status.usage.total_cost - 0.01).abs() < 0.001);
    }

    #[test]
    fn test_budget_exceeded() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 0.05,
                ..Default::default()
            })
            .exceeded_action(ExceededAction::Block)
            .build();

        controller.record_usage("openai", 100, 50, 0.03);
        assert!(controller.check_budget("openai").is_ok());

        controller.record_usage("openai", 100, 50, 0.03);
        assert!(controller.check_budget("openai").is_err());
    }

    #[test]
    fn test_provider_limit() {
        let controller = BudgetBuilder::new()
            .no_global_limit()
            .provider_limit("openai", BudgetLimit {
                max_cost: 1.0,
                ..Default::default()
            })
            .exceeded_action(ExceededAction::Block)
            .build();

        controller.record_usage("openai", 100, 50, 0.5);
        assert!(controller.check_budget("openai").is_ok());

        controller.record_usage("openai", 100, 50, 0.6);
        assert!(controller.check_budget("openai").is_err());

        // Different provider should be OK
        assert!(controller.check_budget("anthropic").is_ok());
    }

    #[test]
    fn test_token_limit() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 100.0,
                max_input_tokens: Some(500),
                ..Default::default()
            })
            .exceeded_action(ExceededAction::Block)
            .build();

        controller.record_usage("openai", 400, 50, 0.01);
        assert!(controller.check_budget("openai").is_ok());

        controller.record_usage("openai", 200, 50, 0.01);
        assert!(controller.check_budget("openai").is_err());
    }

    #[test]
    fn test_warning_threshold() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 1.0,
                warning_threshold: 0.5,
                ..Default::default()
            })
            .build();

        controller.record_usage("openai", 100, 50, 0.4);
        let status = controller.get_global_status().unwrap();
        assert!(!status.at_warning);

        controller.record_usage("openai", 100, 50, 0.2);
        let status = controller.get_global_status().unwrap();
        assert!(status.at_warning);
    }

    #[test]
    fn test_downgrade_suggestion() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 0.1,
                ..Default::default()
            })
            .downgrade_order(vec!["ollama".into(), "anthropic".into()])
            .build();

        controller.record_usage("openai", 100, 50, 0.15);
        
        let suggestion = controller.suggest_provider();
        assert_eq!(suggestion, Some("ollama".into()));
    }

    #[test]
    fn test_reset_usage() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit::default())
            .build();

        controller.record_usage("openai", 100, 50, 0.01);
        
        let status = controller.get_global_status().unwrap();
        assert_eq!(status.usage.requests, 1);

        controller.reset_global();
        
        let status = controller.get_global_status().unwrap();
        assert_eq!(status.usage.requests, 0);
    }

    #[test]
    fn test_multiple_providers() {
        let controller = BudgetBuilder::new()
            .no_global_limit()
            .provider_limit("openai", BudgetLimit {
                max_cost: 1.0,
                ..Default::default()
            })
            .provider_limit("anthropic", BudgetLimit {
                max_cost: 2.0,
                ..Default::default()
            })
            .build();

        controller.record_usage("openai", 100, 50, 0.5);
        controller.record_usage("anthropic", 200, 100, 1.0);

        let openai_status = controller.get_status("openai").unwrap();
        let anthropic_status = controller.get_status("anthropic").unwrap();

        assert!((openai_status.usage.total_cost - 0.5).abs() < 0.001);
        assert!((anthropic_status.usage.total_cost - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_usage_percentage() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 10.0,
                ..Default::default()
            })
            .build();

        controller.record_usage("openai", 100, 50, 2.5);
        
        let status = controller.get_global_status().unwrap();
        assert!((status.usage_percentage - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_request_limit() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 100.0,
                max_requests: Some(3),
                ..Default::default()
            })
            .exceeded_action(ExceededAction::Block)
            .build();

        controller.record_usage("openai", 10, 5, 0.001);
        controller.record_usage("openai", 10, 5, 0.001);
        assert!(controller.check_budget("openai").is_ok());

        controller.record_usage("openai", 10, 5, 0.001);
        assert!(controller.check_budget("openai").is_err());
    }

    #[test]
    fn test_exceeded_action_warn() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit {
                max_cost: 0.01,
                ..Default::default()
            })
            .exceeded_action(ExceededAction::Warn)
            .build();

        controller.record_usage("openai", 100, 50, 0.05);
        // Should not error with Warn action
        assert!(controller.check_budget("openai").is_ok());
    }

    #[test]
    fn test_all_statuses() {
        let controller = BudgetBuilder::new()
            .global_limit(BudgetLimit::default())
            .provider_limit("openai", BudgetLimit::default())
            .provider_limit("anthropic", BudgetLimit::default())
            .build();

        controller.record_usage("openai", 100, 50, 0.01);
        controller.record_usage("anthropic", 200, 100, 0.02);

        let statuses = controller.get_all_statuses();
        assert!(statuses.len() >= 1); // At least global
    }

    #[test]
    fn test_builder_defaults() {
        let controller = BudgetBuilder::new().build();
        
        // Should have default global limit
        assert!(controller.config().global_limit.is_some());
        assert_eq!(controller.config().exceeded_action, ExceededAction::Warn);
        assert!(controller.config().auto_reset);
    }
}
