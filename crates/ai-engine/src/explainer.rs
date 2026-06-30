use crate::types::*;
use async_trait::async_trait;
#[async_trait]
pub trait Explainer: Send + Sync {
    async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String;
}

/// Deterministic rule-based explainer (no LLM). Emits a markdown summary of
/// assignment counts, unscheduled tasks, and aggregate scores.
pub struct TemplateExplainer;

#[async_trait]
impl Explainer for TemplateExplainer {
    async fn explain(&self, _problem: &AllocationProblem, sol: &Solution) -> String {
        let n = sol.assignments.len();
        let unsched = sol.unscheduled.len();
        let avg = if n > 0 {
            sol.assignments.iter().map(|a| a.score).sum::<f64>() / n as f64
        } else {
            0.0
        };
        let mut md = format!(
            "## 优化方案说明\n\n- 已分配 **{}** 个任务，未排期 **{}** 个。\n- 平均技能匹配 {:.0}/100。\n- 综合评分 {:.0}/100。\n",
            n, unsched, avg * 100.0, sol.metrics.overall);
        if unsched > 0 {
            md.push_str(&format!(
                "\n⚠ 未排期任务 {} 个：建议补充人力或调整时间窗。\n",
                unsched
            ));
        }
        md.push_str("\n（规则模板解释；启用 LLM 可获得更细粒度的风险与改进建议。）");
        md
    }
}

/// Production LLM explainer via `rig` chat (local Ollama default, design §5.6).
/// Builds a prompt from the solution metrics and asks the model for a risk/Improvement
/// summary. Falls back to `TemplateExplainer` on any provider error (confirmed #8).
#[cfg(feature = "llm")]
pub mod llm {
    use crate::types::*;
    use async_trait::async_trait;
    use rig::agent::AgentBuilder;
    use rig::client::{CompletionClient, ProviderClient};
    use rig::completion::Prompt;
    use rig::providers::ollama;

    pub struct LlmExplainer {
        pub model: String,
        pub base_url: Option<String>,
    }

    impl LlmExplainer {
        fn client(&self) -> Option<ollama::Client> {
            let _ = &self.base_url; // reserved for a future builder() override
            ollama::Client::from_env().ok()
        }
    }

    #[async_trait]
    impl super::Explainer for LlmExplainer {
        async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String {
            let Some(client) = self.client() else {
                return super::TemplateExplainer.explain(problem, sol).await;
            };
            let prompt = format!(
                "你是一个项目排期助手。请用中文总结以下分配方案，指出风险与改进建议：\n\
                 - 已分配 {} 个任务，未排期 {} 个\n\
                 - 综合评分 {:.0}/100，技能匹配 {:.0}/100，排期率 {:.0}%\n\
                 请给出 3-5 条要点。",
                sol.assignments.len(),
                sol.unscheduled.len(),
                sol.metrics.overall,
                sol.metrics.skill_fit,
                sol.metrics.utilization
            );
            let agent = AgentBuilder::new(client.completion_model(&self.model))
                .preamble("你是资深项目经理，擅长资源分配与风险识别。")
                .build();
            match agent.prompt(prompt).await {
                Ok(text) => format!("## 优化方案说明（AI）\n\n{}", text.trim()),
                Err(_) => super::TemplateExplainer.explain(problem, sol).await,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::explainer::Explainer;
        #[tokio::test]
        #[ignore = "needs Ollama running with the chat model"]
        async fn smoke_llm() {
            let _ = LlmExplainer {
                model: "qwen2.5:7b".into(),
                base_url: None,
            }
            .explain(&AllocationProblem::default(), &Solution::default())
            .await;
        }
    }
}
