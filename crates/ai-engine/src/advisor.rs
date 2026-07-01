//! LLM 建议器：审视 solver 方案，产出结构化改进建议（Vec<SuggestionItem>）。
//! 与 `Explainer` 平级——Explainer 出人话解释，Advisor 出可执行建议。
//! LLM 从不直接产出最终分配；建议是"对 problem 的修改意图"，落地经 rerun 重跑求解器。

use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Advisor: Send + Sync {
    /// 审视 solver 方案，返回零或多条结构化建议。空 Vec 等价"无建议"
    /// （LLM 不可用、解析失败、或模型确实没建议时都走这里）。
    async fn advise(&self, problem: &AllocationProblem, sol: &Solution) -> Vec<SuggestionItem>;
}

/// 默认实现：不产建议。对应 `use_llm_advisor` 关闭时的行为（功能等同关闭）。
pub struct NoAdvisor;

#[async_trait]
impl Advisor for NoAdvisor {
    async fn advise(&self, _problem: &AllocationProblem, _sol: &Solution) -> Vec<SuggestionItem> {
        Vec::new()
    }
}

/// `LlmAdvisor` 与 `select_advisor` 见 `llm` 子模块（仅 `llm` feature 编译）。
#[cfg(feature = "llm")]
pub mod llm {
    use super::*;
    // Task 4 填充。
}
