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
        } else { 0.0 };
        let mut md = format!(
            "## 优化方案说明\n\n- 已分配 **{}** 个任务，未排期 **{}** 个。\n- 平均技能匹配 {:.0}/100。\n- 综合评分 {:.0}/100。\n",
            n, unsched, avg * 100.0, sol.metrics.overall);
        if unsched > 0 {
            md.push_str(&format!("\n⚠ 未排期任务 {} 个：建议补充人力或调整时间窗。\n", unsched));
        }
        md.push_str("\n（规则模板解释；启用 LLM 可获得更细粒度的风险与改进建议。）");
        md
    }
}
