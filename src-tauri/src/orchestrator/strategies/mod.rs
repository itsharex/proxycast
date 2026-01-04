//! 内置选择策略
//!
//! 提供多种模型选择策略实现。

mod cost_optimized;
mod load_balanced;
mod round_robin;
mod speed_optimized;
mod task_based;

pub use cost_optimized::CostOptimizedStrategy;
pub use load_balanced::LoadBalancedStrategy;
pub use round_robin::RoundRobinStrategy;
pub use speed_optimized::SpeedOptimizedStrategy;
pub use task_based::TaskBasedStrategy;

use super::strategy::StrategyRegistry;
use std::sync::Arc;

/// 注册所有内置策略
pub fn register_builtin_strategies(registry: &mut StrategyRegistry) {
    registry.register(Arc::new(RoundRobinStrategy::new()));
    registry.register(Arc::new(TaskBasedStrategy::new()));
    registry.register(Arc::new(CostOptimizedStrategy::new()));
    registry.register(Arc::new(SpeedOptimizedStrategy::new()));
    registry.register(Arc::new(LoadBalancedStrategy::new()));
}

/// 创建带有内置策略的注册表
pub fn create_default_registry() -> StrategyRegistry {
    let mut registry = StrategyRegistry::new();
    register_builtin_strategies(&mut registry);
    registry
}
