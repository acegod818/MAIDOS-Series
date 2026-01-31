//! LLM Provider Router
//!
//! <impl>
//! WHAT: Multi-provider routing with various strategies
//! WHY: Enable load balancing, cost optimization, and failover
//! HOW: Strategy pattern with pluggable routing algorithms
//! TEST: Unit tests for each strategy, integration with providers
//! </impl>

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::error::{LlmError, Result};

/// Routing strategy for provider selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RoutingStrategy {
    /// Round-robin rotation through providers
    RoundRobin,
    /// Select based on priority order
    Priority,
    /// Select cheapest available provider
    Cost,
    /// Select fastest responding provider
    Speed,
    /// Random selection with weights
    Weighted,
    /// Use primary, fallback on failure
    #[default]
    Fallback,
}


/// Provider health status
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Provider name
    pub name: String,
    /// Is provider currently healthy
    pub healthy: bool,
    /// Last check timestamp
    pub last_check: Instant,
    /// Consecutive failure count
    pub failure_count: u32,
    /// Average response time in milliseconds
    pub avg_response_ms: f64,
    /// Total requests handled
    pub total_requests: u64,
    /// Total failures
    pub total_failures: u64,
}

impl ProviderHealth {
    fn new(name: String) -> Self {
        Self {
            name,
            healthy: true,
            last_check: Instant::now(),
            failure_count: 0,
            avg_response_ms: 0.0,
            total_requests: 0,
            total_failures: 0,
        }
    }

    fn record_success(&mut self, response_ms: f64) {
        self.healthy = true;
        self.failure_count = 0;
        self.last_check = Instant::now();
        self.total_requests += 1;
        
        // Exponential moving average
        if self.avg_response_ms == 0.0 {
            self.avg_response_ms = response_ms;
        } else {
            self.avg_response_ms = self.avg_response_ms * 0.9 + response_ms * 0.1;
        }
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.total_failures += 1;
        self.total_requests += 1;
        self.last_check = Instant::now();
        
        // Mark unhealthy after 3 consecutive failures
        if self.failure_count >= 3 {
            self.healthy = false;
        }
    }
}

/// Provider configuration for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub name: String,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Cost per 1K tokens (input)
    pub cost_per_1k_input: f64,
    /// Cost per 1K tokens (output)
    pub cost_per_1k_output: f64,
    /// Weight for weighted routing (0-100)
    pub weight: u32,
    /// Is this provider enabled
    pub enabled: bool,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            priority: 100,
            cost_per_1k_input: 0.0,
            cost_per_1k_output: 0.0,
            weight: 50,
            enabled: true,
        }
    }
}

/// Router configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Routing strategy
    pub strategy: RoutingStrategy,
    /// Provider configurations
    pub providers: Vec<ProviderConfig>,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            strategy: RoutingStrategy::Fallback,
            providers: Vec::new(),
            health_check_interval_secs: 60,
            max_retries: 2,
            retry_delay_ms: 100,
        }
    }
}

/// LLM Provider Router
pub struct Router {
    config: RouterConfig,
    health: Arc<RwLock<HashMap<String, ProviderHealth>>>,
    round_robin_counter: AtomicUsize,
}

impl Router {
    /// Create a new router with configuration
    pub fn new(config: RouterConfig) -> Self {
        let mut health = HashMap::new();
        for provider in &config.providers {
            health.insert(provider.name.clone(), ProviderHealth::new(provider.name.clone()));
        }
        
        Self {
            config,
            health: Arc::new(RwLock::new(health)),
            round_robin_counter: AtomicUsize::new(0),
        }
    }

    /// Select the next provider based on routing strategy
    pub fn select_provider(&self) -> Result<String> {
        let available = self.get_available_providers();
        
        if available.is_empty() {
            return Err(LlmError::Provider("No available providers".into()));
        }

        match self.config.strategy {
            RoutingStrategy::RoundRobin => self.select_round_robin(&available),
            RoutingStrategy::Priority => self.select_priority(&available),
            RoutingStrategy::Cost => self.select_cost(&available),
            RoutingStrategy::Speed => self.select_speed(&available),
            RoutingStrategy::Weighted => self.select_weighted(&available),
            RoutingStrategy::Fallback => self.select_fallback(&available),
        }
    }

    /// Get list of available (enabled and healthy) providers
    pub fn get_available_providers(&self) -> Vec<&ProviderConfig> {
        let health = self.health.read();
        
        self.config.providers
            .iter()
            .filter(|p| {
                p.enabled && health.get(&p.name).map(|h| h.healthy).unwrap_or(false)
            })
            .collect()
    }

    /// Record successful request
    pub fn record_success(&self, provider: &str, response_ms: f64) {
        let mut health = self.health.write();
        if let Some(h) = health.get_mut(provider) {
            h.record_success(response_ms);
        }
    }

    /// Record failed request
    pub fn record_failure(&self, provider: &str) {
        let mut health = self.health.write();
        if let Some(h) = health.get_mut(provider) {
            h.record_failure();
        }
    }

    /// Get health status for all providers
    pub fn get_health_status(&self) -> Vec<ProviderHealth> {
        self.health.read().values().cloned().collect()
    }

    /// Reset health status for a provider
    pub fn reset_health(&self, provider: &str) {
        let mut health = self.health.write();
        if let Some(h) = health.get_mut(provider) {
            h.healthy = true;
            h.failure_count = 0;
        }
    }

    /// Get router configuration
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }

    // Strategy implementations

    fn select_round_robin(&self, available: &[&ProviderConfig]) -> Result<String> {
        let idx = self.round_robin_counter.fetch_add(1, Ordering::Relaxed) % available.len();
        Ok(available[idx].name.clone())
    }

    fn select_priority(&self, available: &[&ProviderConfig]) -> Result<String> {
        available
            .iter()
            .min_by_key(|p| p.priority)
            .map(|p| p.name.clone())
            .ok_or_else(|| LlmError::Provider("No providers available".into()))
    }

    fn select_cost(&self, available: &[&ProviderConfig]) -> Result<String> {
        available
            .iter()
            .min_by(|a, b| {
                let cost_a = a.cost_per_1k_input + a.cost_per_1k_output;
                let cost_b = b.cost_per_1k_input + b.cost_per_1k_output;
                cost_a.partial_cmp(&cost_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.name.clone())
            .ok_or_else(|| LlmError::Provider("No providers available".into()))
    }

    fn select_speed(&self, available: &[&ProviderConfig]) -> Result<String> {
        let health = self.health.read();
        
        available
            .iter()
            .min_by(|a, b| {
                let avg_a = health.get(&a.name).map(|h| h.avg_response_ms).unwrap_or(f64::MAX);
                let avg_b = health.get(&b.name).map(|h| h.avg_response_ms).unwrap_or(f64::MAX);
                avg_a.partial_cmp(&avg_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.name.clone())
            .ok_or_else(|| LlmError::Provider("No providers available".into()))
    }

    fn select_weighted(&self, available: &[&ProviderConfig]) -> Result<String> {
        let total_weight: u32 = available.iter().map(|p| p.weight).sum();
        if total_weight == 0 {
            return self.select_round_robin(available);
        }

        // Use counter as pseudo-random for deterministic selection
        let roll = (self.round_robin_counter.fetch_add(1, Ordering::Relaxed) as u32) % total_weight;
        
        let mut cumulative = 0u32;
        for provider in available {
            cumulative += provider.weight;
            if roll < cumulative {
                return Ok(provider.name.clone());
            }
        }

        // Fallback to first
        Ok(available[0].name.clone())
    }

    fn select_fallback(&self, available: &[&ProviderConfig]) -> Result<String> {
        // Same as priority - use highest priority available
        self.select_priority(available)
    }
}

/// Router builder for fluent configuration
pub struct RouterBuilder {
    config: RouterConfig,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self {
            config: RouterConfig::default(),
        }
    }

    pub fn strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    pub fn add_provider(mut self, config: ProviderConfig) -> Self {
        self.config.providers.push(config);
        self
    }

    pub fn health_check_interval(mut self, secs: u64) -> Self {
        self.config.health_check_interval_secs = secs;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn retry_delay(mut self, ms: u64) -> Self {
        self.config.retry_delay_ms = ms;
        self
    }

    pub fn build(self) -> Router {
        Router::new(self.config)
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_providers() -> Vec<ProviderConfig> {
        vec![
            ProviderConfig {
                name: "openai".into(),
                priority: 1,
                cost_per_1k_input: 0.01,
                cost_per_1k_output: 0.03,
                weight: 50,
                enabled: true,
            },
            ProviderConfig {
                name: "anthropic".into(),
                priority: 2,
                cost_per_1k_input: 0.008,
                cost_per_1k_output: 0.024,
                weight: 30,
                enabled: true,
            },
            ProviderConfig {
                name: "ollama".into(),
                priority: 3,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
                weight: 20,
                enabled: true,
            },
        ]
    }

    #[test]
    fn test_round_robin_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::RoundRobin)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .add_provider(create_test_providers()[2].clone())
            .build();

        let first = router.select_provider().unwrap();
        let second = router.select_provider().unwrap();
        let third = router.select_provider().unwrap();
        let fourth = router.select_provider().unwrap();

        // Should cycle through providers
        assert_eq!(first, "openai");
        assert_eq!(second, "anthropic");
        assert_eq!(third, "ollama");
        assert_eq!(fourth, "openai"); // Back to first
    }

    #[test]
    fn test_priority_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Priority)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .add_provider(create_test_providers()[2].clone())
            .build();

        // Should always select highest priority (lowest number)
        for _ in 0..5 {
            let selected = router.select_provider().unwrap();
            assert_eq!(selected, "openai");
        }
    }

    #[test]
    fn test_cost_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Cost)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .add_provider(create_test_providers()[2].clone())
            .build();

        // Should select cheapest (ollama with 0 cost)
        let selected = router.select_provider().unwrap();
        assert_eq!(selected, "ollama");
    }

    #[test]
    fn test_speed_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Speed)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .add_provider(create_test_providers()[2].clone())
            .build();

        // Record some response times
        router.record_success("openai", 500.0);
        router.record_success("anthropic", 200.0);
        router.record_success("ollama", 100.0);

        // Should select fastest
        let selected = router.select_provider().unwrap();
        assert_eq!(selected, "ollama");
    }

    #[test]
    fn test_health_tracking() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Priority)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .build();

        // Record failures for openai
        router.record_failure("openai");
        router.record_failure("openai");
        router.record_failure("openai"); // 3rd failure marks unhealthy

        // Should now select anthropic (next priority)
        let selected = router.select_provider().unwrap();
        assert_eq!(selected, "anthropic");

        // Reset health
        router.reset_health("openai");
        let selected = router.select_provider().unwrap();
        assert_eq!(selected, "openai");
    }

    #[test]
    fn test_no_available_providers() {
        let mut config = ProviderConfig::default();
        config.name = "disabled".into();
        config.enabled = false;

        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::RoundRobin)
            .add_provider(config)
            .build();

        let result = router.select_provider();
        assert!(result.is_err());
    }

    #[test]
    fn test_weighted_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Weighted)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .add_provider(create_test_providers()[2].clone())
            .build();

        // Should return valid provider
        let selected = router.select_provider().unwrap();
        assert!(["openai", "anthropic", "ollama"].contains(&selected.as_str()));
    }

    #[test]
    fn test_health_status() {
        let router = RouterBuilder::new()
            .add_provider(create_test_providers()[0].clone())
            .build();

        router.record_success("openai", 150.0);
        router.record_success("openai", 200.0);

        let status = router.get_health_status();
        assert_eq!(status.len(), 1);
        assert!(status[0].healthy);
        assert_eq!(status[0].total_requests, 2);
    }

    #[test]
    fn test_router_builder() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Cost)
            .health_check_interval(30)
            .max_retries(3)
            .retry_delay(200)
            .add_provider(ProviderConfig {
                name: "test".into(),
                priority: 1,
                ..Default::default()
            })
            .build();

        assert_eq!(router.config().strategy, RoutingStrategy::Cost);
        assert_eq!(router.config().health_check_interval_secs, 30);
        assert_eq!(router.config().max_retries, 3);
        assert_eq!(router.config().retry_delay_ms, 200);
    }

    #[test]
    fn test_fallback_strategy() {
        let router = RouterBuilder::new()
            .strategy(RoutingStrategy::Fallback)
            .add_provider(create_test_providers()[0].clone())
            .add_provider(create_test_providers()[1].clone())
            .build();

        // Fallback uses priority
        let selected = router.select_provider().unwrap();
        assert_eq!(selected, "openai");
    }

    #[test]
    fn test_response_time_averaging() {
        let router = RouterBuilder::new()
            .add_provider(create_test_providers()[0].clone())
            .build();

        router.record_success("openai", 100.0);
        
        let health = router.get_health_status();
        let openai = health.iter().find(|h| h.name == "openai").unwrap();
        assert_eq!(openai.avg_response_ms, 100.0);

        // Second measurement with EMA
        router.record_success("openai", 200.0);
        let health = router.get_health_status();
        let openai = health.iter().find(|h| h.name == "openai").unwrap();
        // EMA: 100 * 0.9 + 200 * 0.1 = 110
        assert!((openai.avg_response_ms - 110.0).abs() < 0.01);
    }
}
