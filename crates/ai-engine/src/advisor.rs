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

/// 把一批建议应用到 problem（内存快照，不回写 DB）。
/// - WidenWindow / WidenResourceWindow：只放宽不收窄（防 LLM 误缩）。
/// - DropDependency：移除 dependency 边。
/// - ChangeResourceCapacity / UpsertResourceSkill：覆盖对应字段（范围已在 LlmAdvisor::validate_item 校验）。
/// - AddResource：**跳过**——需从 DB 取 resource，app 层 rerun 单独处理（本函数只动内存）。
/// - SwapResource / ChangePercent：advisory，不强制生效（求解器无对应旋钮），本函数不改动 problem。
///
/// 冲突（同一目标多条）后写覆盖。返回被跳过的 AddResource 列表，供 rerun 从 DB 补。
pub fn apply_suggestions(problem: &mut AllocationProblem, suggestions: &[Suggestion]) -> Vec<Suggestion> {
    let mut add_resource_pending = Vec::new();
    for s in suggestions {
        match s {
            Suggestion::WidenWindow { task_id, new_start, new_end } => {
                if let Some(t) = problem.tasks.iter_mut().find(|t| t.id == *task_id) {
                    if *new_start <= t.start { t.start = *new_start; }
                    if *new_end >= t.end { t.end = *new_end; }
                }
            }
            Suggestion::WidenResourceWindow { resource_id, new_available_from, new_available_to } => {
                if let Some(r) = problem.resources.iter_mut().find(|r| r.id == *resource_id) {
                    // 只放宽：若原值 None 视为可放宽到新值；若 Some 只在新值更宽时覆盖。
                    let relax_from = match r.available_from {
                        Some(cur) => *new_available_from <= cur,
                        None => true,
                    };
                    if relax_from { r.available_from = Some(*new_available_from); }
                    let relax_to = match r.available_to {
                        Some(cur) => *new_available_to >= cur,
                        None => true,
                    };
                    if relax_to { r.available_to = Some(*new_available_to); }
                }
            }
            Suggestion::DropDependency { task_id, predecessor_id } => {
                problem.dependencies.retain(|d| !(d.task_id == *task_id && d.predecessor_id == *predecessor_id));
            }
            Suggestion::ChangeResourceCapacity { resource_id, new_daily_capacity_pd } => {
                if let Some(r) = problem.resources.iter_mut().find(|r| r.id == *resource_id) {
                    r.daily_capacity_pd = *new_daily_capacity_pd;
                }
            }
            Suggestion::UpsertResourceSkill { resource_id, skill_id, new_proficiency } => {
                if let Some(r) = problem.resources.iter_mut().find(|r| r.id == *resource_id) {
                    r.skills.insert(*skill_id, *new_proficiency);
                }
            }
            Suggestion::AddResource { .. } => add_resource_pending.push(s.clone()),
            // advisory：不改动 problem
            Suggestion::SwapResource { .. } | Suggestion::ChangePercent { .. } => {}
        }
    }
    add_resource_pending
}

/// `LlmAdvisor` 与 `select_advisor` 见 `llm` 子模块（仅 `llm` feature 编译）。
#[cfg(feature = "llm")]
pub mod llm {
    use super::*;
    use crate::explainer::llm as explainer_llm; // 复用 build_context / substitute / default_preamble
    use crate::llm_client::{completion_prompt, LlmClientConfig};
    use std::collections::HashSet;

    pub struct LlmAdvisor {
        pub provider: String,
        pub model: String,
        pub base_url: Option<String>,
        pub api_key: Option<String>,
        pub preamble: Option<String>,
    }

    #[async_trait]
    impl Advisor for LlmAdvisor {
        #[tracing::instrument(skip(self, problem, sol), fields(provider = %self.provider, model = %self.model))]
        async fn advise(&self, problem: &AllocationProblem, sol: &Solution) -> Vec<SuggestionItem> {
            tracing::debug!("generating LLM suggestions");
            let cfg = LlmClientConfig {
                provider: self.provider.clone(),
                base_url: self.base_url.clone(),
                api_key: self.api_key.clone(),
                model: self.model.clone(),
            };
            let prompt = build_advisor_prompt(problem, sol);
            let preamble = self.preamble.as_deref().unwrap_or_else(|| default_preamble());
            let text = match completion_prompt(&cfg, preamble, &prompt).await {
                Some(t) => t,
                None => return Vec::new(), // provider 错误 → 空建议（graceful degradation）
            };
            parse_suggestions(&text, problem)
        }
    }

    fn default_preamble() -> &'static str {
        "你是资源调度专家。只输出一个 JSON 数组，不要任何其它文字、不要 markdown 代码块。"
    }

    /// 复用 explainer 的 problem+solution 文本上下文，末尾追加 JSON 输出约束。
    fn build_advisor_prompt(problem: &AllocationProblem, sol: &Solution) -> String {
        let ctx = explainer_llm::render_default_context(problem, sol);
        format!(
            "{ctx}\n\n\
             ## 改进建议要求\n\
             基于以上方案，给出 0–6 条具体、可执行的改进建议。**只输出一个 JSON 数组**，\
             不要任何其它文字、不要 markdown 代码块。每个元素形如\
             {{\"kind\":\"...\", ...字段..., \"rationale\":\"<理由>\"}}。\n\
             kind 必须是这 8 个之一：\n\
             - swap_resource {{task_id, new_resource_id}}\n\
             - change_percent {{task_id, new_percent}}  (new_percent ∈ (0,1])\n\
             - widen_window {{task_id, new_start, new_end}}  (日期 YYYY-MM-DD，只放宽不收窄)\n\
             - drop_dependency {{task_id, predecessor_id}}\n\
             - add_resource {{resource_id}}\n\
             - widen_resource_window {{resource_id, new_available_from, new_available_to}}\n\
             - change_resource_capacity {{resource_id, new_daily_capacity_pd}}\n\
             - upsert_resource_skill {{resource_id, skill_id, new_proficiency}}  (1..=5)\n\
             不得引用上下文中不存在的 id。日期用 YYYY-MM-DD。"
        )
    }

    /// 解析 LLM 文本为建议列表。整体解析失败 → 空；逐条校验 id 合法性，非法丢弃。
    ///
    /// 解析策略：先 `from_str::<Vec<Value>>`（只要顶层是合法 JSON 数组即可，不校验元素
    /// 内部结构），再对每个元素 `from_value::<RawItem>` —— 未知 `kind` 会让该元素的
    /// `Suggestion` 反序列化失败，从而整条被丢弃，但不会拖垮其余合法条目。
    pub fn parse_suggestions(text: &str, problem: &AllocationProblem) -> Vec<SuggestionItem> {
        let trimmed = strip_code_fence(text);
        let raw_values: Vec<serde_json::Value> = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, prefix = %&text[..text.len().min(200)], "LLM suggestion JSON parse failed");
                return Vec::new();
            }
        };
        let task_ids: HashSet<i64> = problem.tasks.iter().map(|t| t.id).collect();
        let res_ids: HashSet<i64> = problem.resources.iter().map(|r| r.id).collect();
        let mut out = Vec::new();
        for val in raw_values {
            let r: RawItem = match serde_json::from_value(val) {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(error = %e, "dropping unparseable suggestion item");
                    continue;
                }
            };
            match validate_item(r, &task_ids, &res_ids) {
                Some(item) => out.push(item),
                None => { tracing::warn!("dropping invalid suggestion"); }
            }
        }
        out
    }

    #[derive(serde::Deserialize)]
    struct RawItem {
        #[serde(flatten)]
        suggestion: Suggestion,
        rationale: String,
    }

    fn strip_code_fence(text: &str) -> &str {
        let t = text.trim();
        if t.starts_with("```") {
            let after = t.trim_start_matches("```json").trim_start_matches("```").trim();
            after.trim_end_matches("```").trim()
        } else { t }
    }

    /// 校验单条建议：id 存在、数值范围合法、日期不反序。
    fn validate_item(
        r: RawItem,
        task_ids: &HashSet<i64>,
        res_ids: &HashSet<i64>,
    ) -> Option<SuggestionItem> {
        let s = &r.suggestion;
        let ok = match s {
            Suggestion::SwapResource { task_id, new_resource_id } => task_ids.contains(task_id) && res_ids.contains(new_resource_id),
            Suggestion::ChangePercent { task_id, new_percent } => task_ids.contains(task_id) && *new_percent > 0.0 && *new_percent <= 1.0,
            Suggestion::WidenWindow { task_id, new_start, new_end } => task_ids.contains(task_id) && new_start <= new_end,
            Suggestion::DropDependency { task_id, predecessor_id } => task_ids.contains(task_id) && task_ids.contains(predecessor_id),
            Suggestion::AddResource { resource_id } => res_ids.contains(resource_id),
            Suggestion::WidenResourceWindow { resource_id, new_available_from, new_available_to } => res_ids.contains(resource_id) && new_available_from <= new_available_to,
            Suggestion::ChangeResourceCapacity { resource_id, new_daily_capacity_pd } => res_ids.contains(resource_id) && *new_daily_capacity_pd > 0.0,
            Suggestion::UpsertResourceSkill { resource_id, skill_id, new_proficiency } => res_ids.contains(resource_id) && *new_proficiency >= 1 && *new_proficiency <= 5 && *skill_id > 0,
        };
        if !ok { return None; }
        Some(SuggestionItem { id: None, suggestion: r.suggestion, rationale_md: r.rationale, status: "proposed".into() })
    }
}
