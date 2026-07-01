# LLM 建议对比特性 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** LLM 审视 MILP 方案，产出 8 种结构化建议；用户勾选后改 problem 重跑求解器生成新 run，前端并排对比二选一 apply。

**Architecture:** 新增 `Advisor` trait（与 `Explainer` 平级，`llm` feature gated），产 `Vec<SuggestionItem>`；服务层新增 `rerun` 从父 run 的 `input_snapshot_json` 重建 problem → 应用建议 → 重跑 → 插新 run(`parent_run_id`)。LLM 只产"对 problem 的修改意图"，最终分配永远由求解器产出，硬约束始终由求解器保证。

**Tech Stack:** Rust (ai-engine/app/server crates, sqlx, async-trait, rig), SQLite 迁移, Vue 3 + Pinia + shadcn-vue, Vitest。

参考 spec: `docs/superpowers/specs/2026-07-01-llm-suggestion-compare-design.md`。

任务依赖链：Task 1（迁移）→ Task 2（类型）→ Task 3（Advisor trait + NoAdvisor）→ Task 4（LlmAdvisor + select_advisor）→ Task 5（应用建议纯函数 + 单测）→ Task 6（服务 rerun/list_suggestions/set_suggestion_status）→ Task 7（run_for_project 末尾接线）→ Task 8（路由）→ Task 9（前端 store/api/types）→ Task 10（PlanCompare 组件 + 页面接线）→ Task 11（前端测试）→ Task 12（全量验证）。

---

### Task 1: 数据库迁移

**Files:**
- Create: `crates/db/migrations/0008_optimization_suggestions.sql`

- [ ] **Step 1: 写迁移文件**

```sql
-- 0008_optimization_suggestions.sql
-- LLM 给出的、对 solver 方案的结构化改进建议。每条建议绑定到 task/resource，
-- 是"对 problem 的修改意图"而非最终分配；采纳后经 rerun 重跑求解器才落地。
CREATE TABLE ai_optimization_suggestions (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id             INTEGER NOT NULL REFERENCES ai_optimization_runs(id) ON DELETE CASCADE,
    kind               TEXT    NOT NULL CHECK (kind IN (
                            'swap_resource','change_percent','widen_window','drop_dependency',
                            'add_resource','widen_resource_window','change_resource_capacity',
                            'upsert_resource_skill')),
    target_task_id     INTEGER,
    target_resource_id INTEGER,
    payload_json       TEXT    NOT NULL,
    rationale_md       TEXT    NOT NULL,
    status             TEXT    NOT NULL DEFAULT 'proposed'
                        CHECK (status IN ('proposed','accepted','skipped','applied')),
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_optimization_suggestions_run ON ai_optimization_suggestions(run_id);

ALTER TABLE ai_optimization_runs ADD COLUMN parent_run_id INTEGER
    REFERENCES ai_optimization_runs(id) ON DELETE SET NULL;
CREATE INDEX idx_optimization_runs_parent ON ai_optimization_runs(parent_run_id);

ALTER TABLE settings ADD COLUMN use_llm_advisor INTEGER NOT NULL DEFAULT 0 CHECK (use_llm_advisor IN (0,1));
```

- [ ] **Step 2: 验证迁移可跑**

Run: `cargo test -p db --test migrate 2>&1 | tail -20` （若无 migrate 测试则用 app 测试套件冒烟）
Fallback Run: `cargo test -p app --test optimization run_then_apply_creates_ai_allocations 2>&1 | tail -20`
Expected: PASS（迁移在内存 SQLite 上跑通，现有测试不受影响——`parent_run_id`/新表/新列都是可空或默认值，不破坏现有行）

- [ ] **Step 3: Commit**

```bash
git add crates/db/migrations/0008_optimization_suggestions.sql
git commit -m "feat(db): 0008 optimization suggestions + parent_run_id + use_llm_advisor"
```

---

### Task 2: Rust 类型 Suggestion / SuggestionItem

**Files:**
- Modify: `crates/ai-engine/src/types.rs`（末尾追加）
- Test: `crates/ai-engine/tests/advisor.rs`（本 task 先建文件壳，Task 3+ 填测试）

- [ ] **Step 1: 写类型定义**

在 `crates/ai-engine/src/types.rs` 末尾追加：

```rust
/// LLM 对 solver 方案的一条结构化改进建议。是"对 problem 的修改意图"，采纳后经 rerun
/// 重跑求解器落地——LLM 从不直接产出最终分配，硬约束始终由求解器保证。
/// `#[serde(tag = "kind")]` 内部标签：LLM 输出 `{"kind":"swap_resource",...}` 直接反序列化；
/// 未知 kind 被拒（解析时整条丢弃）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Suggestion {
    // —— Task 轴 ——
    SwapResource { task_id: i64, new_resource_id: i64 },
    ChangePercent { task_id: i64, new_percent: f64 },
    WidenWindow { task_id: i64, new_start: NaiveDate, new_end: NaiveDate },
    DropDependency { task_id: i64, predecessor_id: i64 },
    // —— Resource 轴 ——
    AddResource { resource_id: i64 },
    WidenResourceWindow { resource_id: i64, new_available_from: NaiveDate, new_available_to: NaiveDate },
    ChangeResourceCapacity { resource_id: i64, new_daily_capacity_pd: f64 },
    UpsertResourceSkill { resource_id: i64, skill_id: i64, new_proficiency: i64 },
}

impl Suggestion {
    /// 该建议所针对的 task（resource 轴建议返回 None）。
    pub fn target_task_id(&self) -> Option<i64> {
        match self {
            Suggestion::SwapResource { task_id, .. }
            | Suggestion::ChangePercent { task_id, .. }
            | Suggestion::WidenWindow { task_id, .. }
            | Suggestion::DropDependency { task_id, .. } => Some(*task_id),
            _ => None,
        }
    }
    /// 该建议所针对的 resource（task 轴的 SwapResource 也涉及 new_resource_id）。
    pub fn target_resource_id(&self) -> Option<i64> {
        match self {
            Suggestion::SwapResource { new_resource_id, .. } => Some(*new_resource_id),
            Suggestion::AddResource { resource_id }
            | Suggestion::WidenResourceWindow { resource_id, .. }
            | Suggestion::ChangeResourceCapacity { resource_id, .. }
            | Suggestion::UpsertResourceSkill { resource_id, .. } => Some(*resource_id),
            _ => None,
        }
    }
}

/// 带理由的、可持久化的建议条目。`id` 为 None 直到落库。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SuggestionItem {
    pub id: Option<i64>,
    pub suggestion: Suggestion,
    pub rationale_md: String,
    pub status: String, // proposed | accepted | skipped | applied
}
```

- [ ] **Step 2: 写反序列化测试（验证 serde tag 行为）**

创建 `crates/ai-engine/tests/advisor.rs`：

```rust
use ai_engine::types::{Suggestion, SuggestionItem};
use chrono::NaiveDate;

#[test]
fn suggestion_deserializes_with_kind_tag() {
    let json = r#"{"kind":"widen_window","task_id":5,"new_start":"2026-07-01","new_end":"2026-07-20"}"#;
    let s: Suggestion = serde_json::from_str(json).unwrap();
    assert_eq!(s.target_task_id(), Some(5));
    match s {
        Suggestion::WidenWindow { new_end, .. } => {
            assert_eq!(new_end, NaiveDate::from_ymd_opt(2026, 7, 20).unwrap());
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn suggestion_rejects_unknown_kind() {
    let json = r#"{"kind":"bogus","task_id":5}"#;
    let r: Result<Suggestion, _> = serde_json::from_str(json);
    assert!(r.is_err(), "unknown kind must be rejected");
}

#[test]
fn suggestion_item_roundtrips() {
    let item = SuggestionItem {
        id: None,
        suggestion: Suggestion::DropDependency { task_id: 3, predecessor_id: 1 },
        rationale_md: "解依赖以放行 T3".into(),
        status: "proposed".into(),
    };
    let json = serde_json::to_string(&item).unwrap();
    let back: SuggestionItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back, item);
}
```

- [ ] **Step 3: 跑测试**

Run: `cargo test -p ai-engine --test advisor 2>&1 | tail -20`
Expected: PASS（3 个测试通过）

- [ ] **Step 4: Commit**

```bash
git add crates/ai-engine/src/types.rs crates/ai-engine/tests/advisor.rs
git commit -m "feat(ai-engine): Suggestion/SuggestionItem types with serde tag"
```

---

### Task 3: Advisor trait + NoAdvisor

**Files:**
- Create: `crates/ai-engine/src/advisor.rs`
- Modify: `crates/ai-engine/src/lib.rs`

- [ ] **Step 1: 写 trait 与 NoAdvisor**

创建 `crates/ai-engine/src/advisor.rs`：

```rust
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
```

- [ ] **Step 2: 导出模块**

修改 `crates/ai-engine/src/lib.rs`，在 `pub mod explainer;` 后加一行：

```rust
pub mod advisor;
```

并在 `pub use explainer::Explainer;` 后加：

```rust
pub use advisor::Advisor;
```

- [ ] **Step 3: 写 NoAdvisor 测试**

在 `crates/ai-engine/tests/advisor.rs` 末尾追加：

```rust
use ai_engine::advisor::{Advisor, NoAdvisor};
use ai_engine::types::{AllocationProblem, Solution};

#[tokio::test]
async fn no_advisor_returns_empty() {
    let advisor = NoAdvisor;
    let items = advisor.advise(&AllocationProblem::default(), &Solution::default()).await;
    assert!(items.is_empty(), "NoAdvisor must return zero suggestions");
}
```

- [ ] **Step 4: 跑测试**

Run: `cargo test -p ai-engine --test advisor 2>&1 | tail -20`
Expected: PASS（4 个测试，no-feature 下编译通过——NoAdvisor 不依赖 llm feature）

- [ ] **Step 5: Commit**

```bash
git add crates/ai-engine/src/advisor.rs crates/ai-engine/src/lib.rs crates/ai-engine/tests/advisor.rs
git commit -m "feat(ai-engine): Advisor trait + NoAdvisor default"
```

---

### Task 4: LlmAdvisor + select_advisor

**Files:**
- Modify: `crates/ai-engine/src/advisor.rs`（填 `llm` 子模块）
- Modify: `crates/app/src/service/optimization.rs`（加 `select_advisor`）
- Modify: `crates/db/src/repo/settings.rs`（`AiSettings` 加 `use_llm_advisor`）
- Modify: `crates/db/src/repo/settings.rs`（`SettingsRepo::ai_settings` 查询加列）

- [ ] **Step 1: AiSettings 加字段 + AiSettingsRow 加列**

在 `crates/db/src/repo/settings.rs` 的 `AiSettings` 结构体（line 254-268）里，`use_llm_explainer` 字段后加：

```rust
    /// Whether to use the LLM-based advisor for structured optimization suggestions.
    pub use_llm_advisor: bool,
```

在 `AiSettingsRow` 结构体（line 234-251）的 `use_llm_explainer: Option<i64>,` 后加：

```rust
    use_llm_advisor: Option<i64>,
```

- [ ] **Step 2: ai_settings 查询加列 + 映射**

在 `ai_settings`（line 201-231）的 SELECT 语句（line 205-206）把 `use_llm_explainer,` 后追加 `use_llm_advisor,`：

```rust
            "SELECT ai_provider, ai_base_url, ai_api_key_enc, ai_chat_model, embed_provider, \
             embed_base_url, embed_api_key_enc, embed_model, embed_dim, solver_backend, \
             solver_timeout_ms, use_semantic_scorer, use_llm_explainer, use_llm_advisor, \
             ai_explanation_prompt, ai_explanation_preamble FROM settings WHERE id = 1",
```

在返回构造（line 227 的 `use_llm_explainer` 行后）加：

```rust
            use_llm_advisor: row.use_llm_advisor.unwrap_or(0) != 0,  // 默认关（0），与 use_llm_explainer 默认 1 相反
```

- [ ] **Step 3: 写 LlmAdvisor（llm 子模块）**

替换 `crates/ai-engine/src/advisor.rs` 里 Task 3 占位的 `llm` 子模块为：

```rust
#[cfg(feature = "llm")]
pub mod llm {
    use super::*;
    use crate::explainer::llm as explainer_llm; // 复用 build_context / substitute / default_preamble
    use crate::llm_client::{completion_prompt, LlmClientConfig};

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
        // explainer::llm 的 build_context 是私有的；这里用其 render_template 间接复用
        // 默认模板（它已包含完整 resources/tasks/metrics/assignments/unscheduled）。
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
    fn parse_suggestions(text: &str, problem: &AllocationProblem) -> Vec<SuggestionItem> {
        // 容错：模型可能仍包了 ```json ... ``` 代码块，剥离首尾。
        let trimmed = strip_code_fence(text);
        let raw: Vec<RawItem> = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, prefix = %&text[..text.len().min(200)], "LLM suggestion JSON parse failed");
                return Vec::new();
            }
        };
        let task_ids: std::collections::HashSet<i64> = problem.tasks.iter().map(|t| t.id).collect();
        let res_ids: std::collections::HashSet<i64> = problem.resources.iter().map(|r| r.id).collect();
        let mut out = Vec::new();
        for r in raw {
            match validate_item(r, &task_ids, &res_ids) {
                Some(item) => out.push(item),
                None => { tracing::warn!(item = ?r.suggestion, "dropping invalid suggestion"); }
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

    /// 校验单条建议：id 存在、数值范围合法、日期不反序、widen 只放宽不收窄。
    fn validate_item(
        r: RawItem,
        task_ids: &std::collections::HashSet<i64>,
        res_ids: &std::collections::HashSet<i64>,
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
```

> **注意 build_context 复用**：`explainer::llm::build_context` 当前是私有不导出。本 task 需在 `crates/ai-engine/src/explainer.rs` 的 `llm` 子模块里加一个 `pub fn render_default_context(problem, sol) -> String`（内部调 `build_context` 并 `substitute(default_prompt_template(), ctx)`），供 advisor 复用，避免重复格式化逻辑。先读 `explainer.rs:133-144`（render_template/build_context）确认签名后加该 pub 包装。

- [ ] **Step 4: 加 pub render_default_context 包装**

在 `crates/ai-engine/src/explainer.rs` 的 `pub mod llm` 内（`build_context` 定义之后）加：

```rust
    /// 给 advisor 复用：渲染默认模板（完整 resources/tasks/metrics/assignments/unscheduled）。
    pub fn render_default_context(problem: &AllocationProblem, sol: &Solution) -> String {
        render_template(default_prompt_template(), problem, sol)
    }
```

- [ ] **Step 5: 写 select_advisor**

在 `crates/app/src/service/optimization.rs` 的 `select_explainer`（line 538-552）之后加：

```rust
/// Pick the advisor based on AI settings. 当 `llm` feature 编译且 `use_llm_advisor` 为真时
/// 用 `LlmAdvisor`；否则 `NoAdvisor`（无建议，功能等同关闭）。`LlmAdvisor` 在 provider
/// 错误或 JSON 解析失败时返回空 Vec（graceful degradation）。
fn select_advisor(ai: &db::AiSettings) -> Arc<dyn ai_engine::advisor::Advisor> {
    #[cfg(feature = "llm")]
    if ai.use_llm_advisor {
        return Arc::new(ai_engine::advisor::llm::LlmAdvisor {
            provider: ai.chat.provider.clone(),
            model: ai.chat.model.clone(),
            base_url: ai.chat.base_url.clone(),
            api_key: ai.chat.api_key_enc.clone(),
            preamble: None,
        });
    }
    let _ = ai;
    Arc::new(ai_engine::advisor::NoAdvisor)
}
```

- [ ] **Step 6: 写解析测试（不经 LLM）**

在 `crates/ai-engine/tests/advisor.rs` 末尾追加。但 `parse_suggestions` 在 `llm` 子模块内私有——改测公开行为：用 `#[cfg(feature="llm")]` 跳过，或在 advisor.rs 暴露 `pub(crate)` 测试辅助。**简化**：把 `parse_suggestions` 改为 `pub fn`（llm 子模块内 `pub fn`），测试 gate 在 `#[cfg(feature="llm")]`：

```rust
#[cfg(feature = "llm")]
mod llm_parse_tests {
    use ai_engine::advisor::llm::parse_suggestions; // 需 parse_suggestions 为 pub
    use ai_engine::types::*;
    use chrono::NaiveDate;

    fn prob_with_task_resource() -> AllocationProblem {
        AllocationProblem {
            tasks: vec![CandidateTask {
                id: 5, project_id: 1, title: "T5".into(), estimate_pd: 3.0,
                start: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 7, 7).unwrap(),
                priority: 3, skill_reqs: vec![],
            }],
            resources: vec![CandidateResource {
                id: 2, name: "Bob".into(), skills: Default::default(), tags: vec![],
                daily_capacity_pd: 1.0, available_from: None, available_to: None,
            }],
            ..Default::default()
        };
        AllocationProblem::default() // 占位；实际用上面的（见下）
    }

    #[test]
    fn parse_invalid_json_returns_empty() {
        let p = AllocationProblem::default();
        assert!(parse_suggestions("not json at all", &p).is_empty());
    }

    #[test]
    fn parse_drops_unknown_kind_keeps_valid() {
        let p = prob_with_task_resource();
        let txt = r#"[
            {"kind":"bogus","task_id":5},
            {"kind":"widen_window","task_id":5,"new_start":"2026-07-01","new_end":"2026-07-20","rationale":"x"}
        ]"#;
        let v = parse_suggestions(txt, &p);
        assert_eq!(v.len(), 1);
        assert!(matches!(v[0].suggestion, Suggestion::WidenWindow { .. }));
    }

    #[test]
    fn parse_strips_code_fence() {
        let p = prob_with_task_resource();
        let txt = "```json\n[{\"kind\":\"widen_window\",\"task_id\":5,\"new_start\":\"2026-07-01\",\"new_end\":\"2026-07-20\",\"rationale\":\"x\"}]\n```";
        assert_eq!(parse_suggestions(txt, &p).len(), 1);
    }
}
```

> 修正 `prob_with_task_resource`：去掉末尾多余的 `AllocationProblem::default()` 占位行，直接 `AllocationProblem { tasks: ..., resources: ..., ..Default::default() }`。

> 同时需在 `advisor.rs` 的 `llm` 子模块把 `fn parse_suggestions` 改为 `pub fn parse_suggestions`。

- [ ] **Step 7: 跑测试（llm feature）**

Run: `cargo test -p ai-engine --features llm --test advisor 2>&1 | tail -25`
Expected: PASS（解析测试通过；注意 llm feature 需 `rig` 依赖，CI 上若无 provider 仅测纯解析逻辑，不触网）

- [ ] **Step 8: 跑测试（无 feature，确认 NoAdvisor 路径仍绿）**

Run: `cargo test -p ai-engine --test advisor 2>&1 | tail -25`
Expected: PASS（4 个非 llm 测试通过；llm_parse_tests 在无 feature 下不编译、不运行）

- [ ] **Step 9: Commit**

```bash
git add crates/ai-engine/src/advisor.rs crates/ai-engine/src/explainer.rs crates/ai-engine/tests/advisor.rs crates/app/src/service/optimization.rs crates/db/src/repo/settings.rs
git commit -m "feat(ai-engine,app,db): LlmAdvisor + select_advisor + use_llm_advisor setting"
```

---

### Task 5: 应用建议纯函数 apply_suggestions + 单测

**Files:**
- Modify: `crates/ai-engine/src/advisor.rs`（加 `pub fn apply_suggestions`）
- Test: `crates/ai-engine/tests/advisor.rs`

这个纯函数把 `Vec<Suggestion>` 应用到 `&mut AllocationProblem`，是 rerun 的核心逻辑，独立单测（不触 DB、不经 LLM）。

- [ ] **Step 1: 写纯函数**

在 `crates/ai-engine/src/advisor.rs` 的 `NoAdvisor` 之后、`#[cfg(feature="llm")] pub mod llm` 之前加（**非 feature-gated**，因为 rerun 在 app 层无论是否 llm 都要用）：

```rust
use std::collections::HashSet;

/// 把一批建议应用到 problem（内存快照，不回写 DB）。
/// - WidenWindow / WidenResourceWindow：只放宽不收窄（防 LLM 误缩）。
/// - DropDependency：移除 dependency 边。
/// - ChangeResourceCapacity / UpsertResourceSkill：覆盖对应字段（范围已在 Task 4 校验）。
/// - AddResource：**跳过**——需从 DB 取 resource，app 层 rerun 单独处理（本函数只动内存）。
/// - SwapResource / ChangePercent：advisory，不强制生效（求解器无对应旋钮），本函数不改动 problem。
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
                    if new_available_from <= &r.available_from.unwrap_or(*new_available_from) {
                        r.available_from = Some(*new_available_from);
                    }
                    if new_available_to >= &r.available_to.unwrap_or(*new_available_to) {
                        r.available_to = Some(*new_available_to);
                    }
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
```

- [ ] **Step 2: 写单测**

在 `crates/ai-engine/tests/advisor.rs` 末尾追加：

```rust
use ai_engine::advisor::apply_suggestions;
use ai_engine::types::*;
use chrono::NaiveDate;

fn task(id: i64, start: &str, end: &str) -> CandidateTask {
    CandidateTask { id, project_id: 1, title: format!("T{id}"), estimate_pd: 3.0,
        start: NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap(),
        end: NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap(),
        priority: 3, skill_reqs: vec![] }
}
fn resource(id: i64) -> CandidateResource {
    CandidateResource { id, name: format!("R{id}"), skills: Default::default(), tags: vec![],
        daily_capacity_pd: 1.0, available_from: None, available_to: None }
}
fn date(s: &str) -> NaiveDate { NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap() }

#[test]
fn widen_window_only_relaxes_not_narrows() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-03", "2026-07-07")], ..Default::default() };
    // LLM 试图收窄到 7/4–7/6（应被忽略，保留 7/3–7/7）
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenWindow { task_id: 5, new_start: date("2026-07-04"), new_end: date("2026-07-06") },
    ]);
    assert_eq!(p.tasks[0].start, date("2026-07-03"));
    assert_eq!(p.tasks[0].end, date("2026-07-07"));
}

#[test]
fn widen_window_relaxes_outward() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-03", "2026-07-07")], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenWindow { task_id: 5, new_start: date("2026-07-01"), new_end: date("2026-07-20") },
    ]);
    assert_eq!(p.tasks[0].start, date("2026-07-01"));
    assert_eq!(p.tasks[0].end, date("2026-07-20"));
}

#[test]
fn drop_dependency_removes_edge() {
    let mut p = AllocationProblem {
        dependencies: vec![TaskDependency { task_id: 3, predecessor_id: 1 }],
        ..Default::default()
    };
    let _ = apply_suggestions(&mut p, &[Suggestion::DropDependency { task_id: 3, predecessor_id: 1 }]);
    assert!(p.dependencies.is_empty());
}

#[test]
fn upsert_skill_overrides_proficiency() {
    let mut p = AllocationProblem { resources: vec![resource(2)], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[Suggestion::UpsertResourceSkill { resource_id: 2, skill_id: 10, new_proficiency: 4 }]);
    assert_eq!(p.resources[0].skills.get(&10), Some(&4));
}

#[test]
fn change_resource_capacity_overrides() {
    let mut p = AllocationProblem { resources: vec![resource(2)], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[Suggestion::ChangeResourceCapacity { resource_id: 2, new_daily_capacity_pd: 1.2 }]);
    assert!((p.resources[0].daily_capacity_pd - 1.2).abs() < 1e-9);
}

#[test]
fn add_resource_and_advisory_kinds_returned_or_skipped() {
    let mut p = AllocationProblem { tasks: vec![task(5, "2026-07-01", "2026-07-07")], ..Default::default() };
    let pending = apply_suggestions(&mut p, &[
        Suggestion::AddResource { resource_id: 9 },
        Suggestion::SwapResource { task_id: 5, new_resource_id: 2 },
        Suggestion::ChangePercent { task_id: 5, new_percent: 0.5 },
    ]);
    // AddResource pending for DB fetch; advisory kinds did nothing to problem
    assert_eq!(pending.len(), 1);
    assert!(matches!(pending[0], Suggestion::AddResource { resource_id: 9 }));
}

#[test]
fn widen_resource_window_only_relaxes() {
    let mut p = AllocationProblem { resources: vec![CandidateResource {
        id: 2, name: "R2".into(), skills: Default::default(), tags: vec![],
        daily_capacity_pd: 1.0,
        available_from: Some(date("2026-07-05")), available_to: Some(date("2026-07-10")),
    }], ..Default::default() };
    let _ = apply_suggestions(&mut p, &[
        Suggestion::WidenResourceWindow { resource_id: 2, new_available_from: date("2026-07-01"), new_available_to: date("2026-07-20") },
    ]);
    assert_eq!(p.resources[0].available_from, Some(date("2026-07-01")));
    assert_eq!(p.resources[0].available_to, Some(date("2026-07-20")));
}
```

- [ ] **Step 3: 跑测试**

Run: `cargo test -p ai-engine --test advisor 2>&1 | tail -25`
Expected: PASS（纯函数测试无需 llm feature）

- [ ] **Step 4: Commit**

```bash
git add crates/ai-engine/src/advisor.rs crates/ai-engine/tests/advisor.rs
git commit -m "feat(ai-engine): apply_suggestions pure fn + unit tests"
```

---

### Task 6: 服务层 rerun / list_suggestions / set_suggestion_status

**Files:**
- Modify: `crates/app/src/service/optimization.rs`
- Test: `crates/app/tests/optimization.rs`

- [ ] **Step 1: 写 list_suggestions**

在 `OptimizationService` impl 块内（`get_run` 之后）加：

```rust
    /// 列出某 run 的全部 LLM 建议。
    pub async fn list_suggestions(pool: &SqlitePool, run_id: i64) -> Result<Vec<SuggestionItem>, AppError> {
        #[derive(sqlx::FromRow)]
        struct Row { id: i64, payload_json: String, rationale_md: String, status: String }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT id, payload_json, rationale_md, status FROM ai_optimization_suggestions WHERE run_id=? ORDER BY id")
            .bind(run_id).fetch_all(pool).await?;
        let mut out = Vec::new();
        for r in rows {
            let suggestion: ai_engine::types::Suggestion =
                serde_json::from_str(&r.payload_json).map_err(|e| AppError::internal(format!("invalid suggestion json: {e}")))?;
            out.push(SuggestionItem { id: Some(r.id), suggestion, rationale_md: r.rationale_md, status: r.status });
        }
        Ok(out)
    }

    /// 标记某条建议状态（proposed/accepted/skipped/applied）。
    pub async fn set_suggestion_status(pool: &SqlitePool, suggestion_id: i64, status: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE ai_optimization_suggestions SET status=? WHERE id=?")
            .bind(status).bind(suggestion_id).execute(pool).await?;
        Ok(())
    }
```

- [ ] **Step 2: 写 rerun**

在 `apply`/`reject` 附近加。需 `ai_engine::advisor::apply_suggestions` 已 export（Task 5 是 `pub fn` 在 `advisor.rs` 顶层，`use ai_engine::advisor::apply_suggestions` 可直接调）。

```rust
    /// 用采纳的建议重跑求解器：从父 run 的 input_snapshot 重建 problem → 应用建议 →
    /// 重跑 → 插新 run（parent_run_id 指回父 run）→ 标建议 applied。返回新 RunResult。
    #[tracing::instrument(skip(pool), fields(parent_run_id = parent_run_id))]
    pub async fn rerun(
        pool: &SqlitePool, parent_run_id: i64, accepted_ids: Vec<i64>,
    ) -> Result<RunResult, AppError> {
        // 1. 读父 run 快照 + 配置。
        #[derive(sqlx::FromRow)]
        struct ParentRow {
            input_snapshot_json: String, weights_json: String, config_json: String,
            scope_project_ids: Option<String>,
        }
        let parent: ParentRow = sqlx::query_as(
            "SELECT input_snapshot_json, weights_json, config_json, scope_project_ids \
             FROM ai_optimization_runs WHERE id=?")
            .bind(parent_run_id).fetch_optional(pool).await?
            .ok_or_else(|| AppError::not_found(format!("optimization run {parent_run_id}")))?;
        let mut problem: ai_engine::types::AllocationProblem =
            serde_json::from_str(&parent.input_snapshot_json)
                .map_err(|e| AppError::internal(format!("invalid input snapshot json: {e}")))?;

        // 2. 读采纳的建议（只取 run_id 匹配的，越界 id 忽略）。
        #[derive(sqlx::FromRow)]
        struct SugRow { id: i64, payload_json: String }
        let rows: Vec<SugRow> = sqlx::query_as(
            "SELECT id, payload_json FROM ai_optimization_suggestions WHERE run_id=? AND status='accepted'")
            .bind(parent_run_id).fetch_all(pool).await?;
        let accepted_set: std::collections::HashSet<i64> = accepted_ids.into_iter().collect();
        let mut applied_suggestion_ids: Vec<i64> = Vec::new();
        let mut suggestions: Vec<ai_engine::types::Suggestion> = Vec::new();
        for r in rows {
            if !accepted_set.contains(&r.id) { continue; }
            match serde_json::from_str::<ai_engine::types::Suggestion>(&r.payload_json) {
                Ok(s) => { suggestions.push(s); applied_suggestion_ids.push(r.id); }
                Err(e) => tracing::warn!(suggestion_id = r.id, error = %e, "skipping unparseable suggestion"),
            }
        }

        // 3. 应用内存可改建议；AddResource 单独从 DB 补。
        let add_pending = ai_engine::advisor::apply_suggestions(&mut problem, &suggestions);
        for s in &add_pending {
            if let ai_engine::types::Suggestion::AddResource { resource_id } = s {
                if let Some(cr) = load_candidate_resource(pool, *resource_id).await? {
                    if !problem.resources.iter().any(|r| r.id == cr.id) {
                        problem.resources.push(cr);
                    }
                } else {
                    tracing::warn!(resource_id, "AddResource: resource not found in DB, skipping");
                }
            }
        }

        // 4. 复用 run_for_project 的下半段（scorer/solve/explain/persist）。
        problem.run_id = 0;
        let ai = db::SettingsRepo::ai_settings(pool).await?;
        let total = (problem.weights.skill_fit + problem.weights.balance).max(0.001);
        let scorer = select_scorer(&ai, problem.weights.skill_fit / total, problem.weights.balance / total);
        let explainer = select_explainer(&ai);
        let (solver, effective_backend) = select_solver(&ai);
        problem.config.backend = effective_backend.to_string();
        problem.config.timeout_ms = ai.solver_timeout_ms;
        let milp_active = cfg!(feature = "milp") && ai.solver_backend == "good_lp";
        let started = chrono::Utc::now();
        let scores = scorer.matrix(&problem).await;
        let solution = solve_with_fallback(&ai, solver, milp_active, &problem, &scores).await;
        let explanation_md = explainer.explain(&problem, &solution).await;
        let mut plan = ai_engine::OptimizedPlan { solution, explanation_md };
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds();

        let cfg = serde_json::to_string(&problem.config).unwrap_or_default();
        let wts = serde_json::to_string(&problem.weights).unwrap_or_default();
        // 5. 插新 run（parent_run_id 指回父 run）。scope 沿用父 run 的 project_ids。
        let (run_id,): (i64,) = sqlx::query_as(
            "INSERT INTO ai_optimization_runs (seed, scope, scope_project_ids, config_json, constraints_json, \
                weights_json, input_snapshot_json, output_plan_json, score_overall, score_skill_fit, \
                score_scheduled_ratio, score_fairness, explanation_md, provider, chat_model, embed_model, \
                solver_backend, solver_status, status, started_at, finished_at, duration_ms, parent_run_id) \
             VALUES (?,?,?,?,'',?,'','',?,?,?,?,?,?,?,?,?,?,?,?,?,'proposed',?,?,?,?) RETURNING id")
            .bind(0i64).bind("full").bind(parent.scope_project_ids)
            .bind(cfg).bind(wts)
            .bind(plan.solution.metrics.overall).bind(plan.solution.metrics.skill_fit)
            .bind(plan.solution.metrics.scheduled_ratio).bind(plan.solution.metrics.fairness)
            .bind(&plan.explanation_md)
            .bind(&ai.chat.provider).bind(&ai.chat.model).bind(&ai.embed.model).bind(effective_backend)
            .bind(plan.solution.status.as_str())
            .bind(started.to_rfc3339()).bind(finished.to_rfc3339()).bind(duration_ms)
            .bind(parent_run_id)
            .fetch_one(pool).await?;
        problem.run_id = run_id;
        plan.solution.run_id = run_id;
        let snap = serde_json::to_string(&problem).unwrap_or_default();
        let out = serde_json::to_string(&plan.solution.assignments).unwrap_or_default();
        let cons = serde_json::to_string(&problem.dependencies).unwrap_or_else(|_| "[]".into());
        sqlx::query("UPDATE ai_optimization_runs SET input_snapshot_json=?, output_plan_json=?, constraints_json=? WHERE id=?")
            .bind(snap).bind(out).bind(cons).bind(run_id).execute(pool).await?;

        // 6. 标采纳建议 applied。
        for id in &applied_suggestion_ids {
            sqlx::query("UPDATE ai_optimization_suggestions SET status='applied' WHERE id=?")
                .bind(id).execute(pool).await?;
        }
        tracing::info!(run_id = run_id, parent_run_id = parent_run_id, applied_suggestions = applied_suggestion_ids.len(), "rerun completed");
        Ok(RunResult { run_id, plan })
    }
```

再加 `load_candidate_resource` 辅助（与 `build_problem` 同款查 resources+skills+tags，但按单个 id）：

```rust
/// 按 id 从 DB 加载单个 CandidateResource（含 skills/tags/capacity/window），供 AddResource 建议用。
/// 不存在或已删除返回 None。
async fn load_candidate_resource(pool: &SqlitePool, resource_id: i64) -> Result<Option<ai_engine::types::CandidateResource>, AppError> {
    use std::collections::HashMap;
    type ResRow = (i64, String, f64, Option<NaiveDate>, Option<NaiveDate>);
    let row: Option<ResRow> = sqlx::query_as(
        "SELECT id, name, daily_capacity_pd, available_from, available_to \
         FROM resources WHERE id=? AND deleted_at IS NULL AND status='active'")
        .bind(resource_id).fetch_optional(pool).await?;
    let Some((id, name, cap, af, at)) = row else { return Ok(None); };
    let skill_rows: Vec<(i64, i64, i64)> = sqlx::query_as(
        "SELECT resource_id, skill_id, proficiency FROM resource_skills WHERE resource_id=?")
        .bind(id).fetch_all(pool).await?;
    let mut skills: HashMap<i64, i64> = HashMap::new();
    for (_, sid, prof) in skill_rows { skills.insert(sid, prof); }
    let tag_rows: Vec<(i64, String)> = sqlx::query_as(
        "SELECT rt.resource_id, t.name FROM resource_tags rt JOIN tags t ON t.id=rt.tag_id WHERE rt.resource_id=?")
        .bind(id).fetch_all(pool).await?;
    let tags: Vec<String> = tag_rows.into_iter().map(|(_, n)| n).collect();
    Ok(Some(ai_engine::types::CandidateResource {
        id, name, skills, tags, daily_capacity_pd: cap, available_from: af, available_to: at,
    }))
}
```

- [ ] **Step 3: 写 rerun 集成测试**

在 `crates/app/tests/optimization.rs` 末尾加。需先有一个父 run（用 `run_for_project`），手工往 `ai_optimization_suggestions` 插一条 `accepted` 建议再调 `rerun`。

```rust
use ai_engine::types::Suggestion;
use chrono::NaiveDate;

#[tokio::test]
async fn rerun_inserts_new_run_with_parent_and_applies_widen_window() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    TasksService::create(&pool, pid, "T1", None, 5.0, Some("2026-07-03"), Some("2026-07-05"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();

    // 父 run：T1 窗口 7/3–7/5 太窄装不下 5PD → 应 unscheduled。
    let parent = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();

    // 插一条 widen_window 建议（放宽到 7/1–7/10），标 accepted。
    let sug = Suggestion::WidenWindow { task_id: parent.plan.solution.assignments.first().map(|a| a.task_id).unwrap_or(1), new_start: NaiveDate::from_ymd_opt(2026,7,1).unwrap(), new_end: NaiveDate::from_ymd_opt(2026,7,10).unwrap() };
    let _ = parent.plan.solution.assignments.first(); // 若父方案为空（unscheduled），用 task id 1
    let task_id: i64 = parent.plan.solution.assignments.first().map(|a| a.task_id).unwrap_or(1);
    let sug = Suggestion::WidenWindow { task_id, new_start: NaiveDate::from_ymd_opt(2026,7,1).unwrap(), new_end: NaiveDate::from_ymd_opt(2026,7,10).unwrap() };
    let payload = serde_json::to_string(&sug).unwrap();
    let (sug_id,): (i64,) = sqlx::query_as("INSERT INTO ai_optimization_suggestions (run_id, kind, target_task_id, payload_json, rationale_md, status) VALUES (?,?,?,?,?,?,'accepted') RETURNING id")
        .bind(parent.run_id).bind("widen_window").bind(task_id).bind(payload).bind("放宽窗口")
        .fetch_one(&pool).await.unwrap();

    let rerun = OptimizationService::rerun(&pool, parent.run_id, vec![sug_id]).await.unwrap();
    assert_ne!(rerun.run_id, parent.run_id);
    let (parent_id,): (Option<i64>,) = sqlx::query_as("SELECT parent_run_id FROM ai_optimization_runs WHERE id=?").bind(rerun.run_id).fetch_one(&pool).await.unwrap();
    assert_eq!(parent_id, Some(parent.run_id));
    // 建议被标 applied
    let (st,): (String,) = sqlx::query_as("SELECT status FROM ai_optimization_suggestions WHERE id=?").bind(sug_id).fetch_one(&pool).await.unwrap();
    assert_eq!(st, "applied");
}

#[tokio::test]
async fn rerun_ignores_suggestion_ids_not_in_run() {
    let pool = connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("../db/migrations").run(&pool).await.unwrap();
    let pid = ProjectsService::create(&pool, "P", None, None, None, 5, 0.0).await.unwrap();
    let rust = CatalogService::ensure_skill(&pool, "Rust").await.unwrap();
    sqlx::query("INSERT INTO resources (id,name) VALUES (1,'Alice')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO resource_skills (resource_id,skill_id,proficiency) VALUES (1,?,4)").bind(rust).execute(&pool).await.unwrap();
    TasksService::create(&pool, pid, "T1", None, 3.0, Some("2026-07-01"), Some("2026-07-07"), false, None, None, 0, &[(rust, 3, true, 1.0)], &[]).await.unwrap();
    let parent = OptimizationService::run_for_project(&pool, pid, None).await.unwrap();
    // accepted_ids 含一个不属于该 run 的 id（99999）→ 不报错
    let rerun = OptimizationService::rerun(&pool, parent.run_id, vec![99999]).await.unwrap();
    assert_ne!(rerun.run_id, parent.run_id);
}
```

> **修正上面测试里的重复变量**：删除第一个 `let sug = ...` 与多余 `let _ = ...` 行，只保留第二个 `let task_id = ...; let sug = ...`。

- [ ] **Step 4: 跑测试**

Run: `cargo test -p app --test optimization rerun 2>&1 | tail -25`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/service/optimization.rs crates/app/tests/optimization.rs
git commit -m "feat(app): rerun/list_suggestions/set_suggestion_status service methods"
```

---

### Task 7: run_for_project 末尾接线 advisor

**Files:**
- Modify: `crates/app/src/service/optimization.rs`

- [ ] **Step 1: 在 run_for_project 末尾插建议**

在 `run_for_project` 的 `UPDATE ... backfill` 之后、`tracing::info!` 之前（约 line 108 后）加：

```rust
        // LLM 建议：若启用 advisor，审视方案产出结构化建议并写表（status='proposed'）。
        // 无建议（LLM 不可用/解析空）时表里无行，前端显示"AI 无建议"。
        let advisor = select_advisor(&ai);
        let suggestions = advisor.advise(&problem, &plan.solution).await;
        for item in &suggestions {
            let kind = match &item.suggestion {
                ai_engine::types::Suggestion::SwapResource { .. } => "swap_resource",
                ai_engine::types::Suggestion::ChangePercent { .. } => "change_percent",
                ai_engine::types::Suggestion::WidenWindow { .. } => "widen_window",
                ai_engine::types::Suggestion::DropDependency { .. } => "drop_dependency",
                ai_engine::types::Suggestion::AddResource { .. } => "add_resource",
                ai_engine::types::Suggestion::WidenResourceWindow { .. } => "widen_resource_window",
                ai_engine::types::Suggestion::ChangeResourceCapacity { .. } => "change_resource_capacity",
                ai_engine::types::Suggestion::UpsertResourceSkill { .. } => "upsert_resource_skill",
            };
            let payload = serde_json::to_string(&item.suggestion).unwrap_or_default();
            sqlx::query(
                "INSERT INTO ai_optimization_suggestions (run_id, kind, target_task_id, target_resource_id, payload_json, rationale_md, status) \
                 VALUES (?,?,?,?,?,?,?)")
                .bind(run_id).bind(kind)
                .bind(item.suggestion.target_task_id())
                .bind(item.suggestion.target_resource_id())
                .bind(payload).bind(&item.rationale_md).bind("proposed")
                .execute(pool).await?;
        }
        tracing::debug!(run_id = run_id, suggestions = suggestions.len(), "advisor suggestions persisted");
```

- [ ] **Step 2: 验证现有测试不回归**

Run: `cargo test -p app --test optimization run_then_apply_creates_ai_allocations 2>&1 | tail -20`
Expected: PASS（默认 `use_llm_advisor=0` → NoAdvisor → 无建议行，行为不变）

- [ ] **Step 3: Commit**

```bash
git add crates/app/src/service/optimization.rs
git commit -m "feat(app): wire advisor into run_for_project tail"
```

---

### Task 8: HTTP 路由

**Files:**
- Modify: `crates/server/src/routes/optimization.rs`

- [ ] **Step 1: 加路由与 handler**

在 `router()` 的 `.route("/api/optimization/runs/{id}/reject", ...)` 后加 3 条：

```rust
        .route("/api/optimization/runs/{id}/suggestions", get(list_suggestions))
        .route("/api/optimization/runs/{id}/rerun", post(rerun_run))
        .route("/api/optimization/suggestions/{id}", patch(set_suggestion_status))
```

在文件末尾加 handler：

```rust
#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn list_suggestions(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
) -> Result<Json<Vec<ai_engine::types::SuggestionItem>>, HttpError> {
    Ok(Json(OptimizationService::list_suggestions(&state.pool, run_id).await?))
}

#[derive(Debug, Deserialize)]
struct RerunBody { suggestion_ids: Vec<i64> }

#[tracing::instrument(skip(state), fields(run_id = run_id))]
async fn rerun_run(
    State(state): State<AppState>,
    Path(run_id): Path<i64>,
    body: Json<RerunBody>,
) -> Result<Json<RunResult>, HttpError> {
    Ok(Json(OptimizationService::rerun(&state.pool, run_id, body.suggestion_ids).await?))
}

#[derive(Debug, Deserialize)]
struct SuggestionStatusBody { status: String }

#[tracing::instrument(skip(state), fields(suggestion_id = suggestion_id))]
async fn set_suggestion_status(
    State(state): State<AppState>,
    Path(suggestion_id): Path<i64>,
    body: Json<SuggestionStatusBody>,
) -> Result<axum::http::StatusCode, HttpError> {
    OptimizationService::set_suggestion_status(&state.pool, suggestion_id, &body.status).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
```

> 确认 `ai_engine::types::SuggestionItem` 实现 `Serialize`（Task 2 已派生）。`axum::http::StatusCode` 路径按 server 现有导入习惯，若项目惯用 `http::StatusCode` 则改之——先 `grep -n "StatusCode" crates/server/src/routes/optimization.rs crates/server/src/routes/*.rs | head` 对齐。

- [ ] **Step 2: 编译验证**

Run: `cargo build -p server 2>&1 | tail -20`
Expected: 编译通过（patch 路由需 `axum::routing::patch` 已在 use 列表，Task 8 Step1 的 use 已含 patch）

- [ ] **Step 3: 确认 use 行含 patch**

修改 `crates/server/src/routes/optimization.rs` 顶部 `use axum::routing::{get, post};` 为 `use axum::routing::{get, post, patch};`。

- [ ] **Step 4: Commit**

```bash
git add crates/server/src/routes/optimization.rs
git commit -m "feat(server): suggestions/rerun/suggestion-status routes"
```

---

### Task 9: 前端 types / api / store

**Files:**
- Modify: `src/types.ts`
- Modify: `src/api/index.ts`
- Modify: `src/stores/optimization.ts`

- [ ] **Step 1: types.ts 加类型**

在 `src/types.ts` 的 `RunList` 接口后加：

```ts
export type Suggestion =
  | { kind: "swap_resource"; task_id: number; new_resource_id: number }
  | { kind: "change_percent"; task_id: number; new_percent: number }
  | { kind: "widen_window"; task_id: number; new_start: string; new_end: string }
  | { kind: "drop_dependency"; task_id: number; predecessor_id: number }
  | { kind: "add_resource"; resource_id: number }
  | { kind: "widen_resource_window"; resource_id: number; new_available_from: string; new_available_to: string }
  | { kind: "change_resource_capacity"; resource_id: number; new_daily_capacity_pd: number }
  | { kind: "upsert_resource_skill"; resource_id: number; skill_id: number; new_proficiency: number };

export interface SuggestionItem {
  id: number;
  suggestion: Suggestion;
  rationale_md: string;
  status: "proposed" | "accepted" | "skipped" | "applied";
}
```

- [ ] **Step 2: api/index.ts 加方法**

在 `src/api/index.ts` 的 `rejectSolution` 之后（optimization 区块内）加：

```ts
  listSuggestions: (runId: number): Promise<SuggestionItem[]> =>
    request("GET", `/api/optimization/runs/${runId}/suggestions`),
  rerun: (runId: number, suggestionIds: number[]): Promise<RunResult> =>
    request("POST", `/api/optimization/runs/${runId}/rerun`, { suggestion_ids: suggestionIds }),
  setSuggestionStatus: (id: number, status: string): Promise<void> =>
    request("PATCH", `/api/optimization/suggestions/${id}`, { status }),
```

并在该文件顶部 `import type { ... }` 列表里加 `SuggestionItem`（与现有 `RunResult, RunList` 同行）。

- [ ] **Step 3: store 扩展**

修改 `src/stores/optimization.ts`：在 `import type { RunResult, RunList, ObjectiveWeights } from "../types";` 加 `SuggestionItem`。

在 store 工厂内，`const pageSize = ref(10);` 之后加：

```ts
  const suggestions = ref<SuggestionItem[]>([]);
  const compareTarget = ref<RunResult | null>(null);
```

在 `loadRun` 之后加：

```ts
  async function loadSuggestions(runId: number) {
    suggestions.value = await api.listSuggestions(runId);
  }
  async function rerun(runId: number, ids: number[]) {
    busy.value = true;
    try {
      const parent = current.value;
      current.value = await api.rerun(runId, ids);
      compareTarget.value = parent;
      await loadSuggestions(current.value.run_id);
    } finally { busy.value = false; }
  }
  async function toggleSuggestion(id: number, on: boolean) {
    await api.setSuggestionStatus(id, on ? "accepted" : "skipped");
    const s = suggestions.value.find((s) => s.id === id);
    if (s) s.status = on ? "accepted" : "skipped";
  }
```

并在 `loadRun` 内追加加载建议：

```ts
  async function loadRun(runId: number) {
    current.value = await api.getOptimizationRun(runId);
    await loadSuggestions(runId);
  }
```

末尾 return 加 `suggestions, compareTarget, loadSuggestions, rerun, toggleSuggestion`。

- [ ] **Step 4: 类型检查**

Run: `npx vue-tsc --noEmit 2>&1 | tail -20`
Expected: 无错误

- [ ] **Step 5: Commit**

```bash
git add src/types.ts src/api/index.ts src/stores/optimization.ts
git commit -m "feat(web): Suggestion types + api + store (rerun/compare)"
```

---

### Task 10: PlanCompare 组件 + 页面接线

**Files:**
- Create: `src/components/PlanCompare.vue`
- Modify: `src/pages/ai/index.vue`

- [ ] **Step 1: 写 PlanCompare.vue**

```vue
<script setup lang="ts">
import { computed } from "vue";
import { Button } from "@/components/ui/button";
import { Table, TableHead, TableHeader, TableRow, TableBody, TableCell, TableHead as TH } from "@/components/ui/table";
import { useOptimizationStore } from "@/stores/optimization";
import type { SuggestionItem } from "@/types";

const opt = useOptimizationStore();

const acceptedIds = computed(() =>
  opt.suggestions.filter((s) => s.status === "accepted").map((s) => s.id)
);

function kindLabel(s: SuggestionItem): string {
  const map: Record<string, string> = {
    swap_resource: "换人", change_percent: "改占比", widen_window: "放宽任务窗",
    drop_dependency: "解依赖", add_resource: "加资源", widen_resource_window: "放宽资源窗",
    change_resource_capacity: "改容量", upsert_resource_skill: "补技能",
  };
  return map[s.suggestion.kind] ?? s.suggestion.kind;
}

async function doRerun() {
  if (!opt.current) return;
  await opt.rerun(opt.current.run_id, acceptedIds.value);
}
async function pick(which: "current" | "parent") {
  const pickRun = which === "current" ? opt.current : opt.compareTarget;
  const other = which === "current" ? opt.compareTarget : opt.current;
  if (pickRun) await opt.accept(pickRun.run_id);
  if (other) await opt.reject(other.run_id);
  opt.compareTarget = null;
}
</script>

<template>
  <div v-if="opt.suggestions.length || opt.compareTarget" class="mt-6 space-y-6">
    <!-- 建议区 -->
    <div v-if="opt.suggestions.length">
      <h3 class="text-lg font-semibold mb-2">AI 改进建议</h3>
      <Table>
        <TableHeader>
          <TableRow>
            <TH class="w-10">采纳</TH>
            <TH>类型</TH>
            <TH>理由</TH>
            <TH>状态</TH>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="s in opt.suggestions" :key="s.id">
            <TableCell>
              <input
                type="checkbox"
                :checked="s.status === 'accepted'"
                :disabled="s.status === 'applied'"
                @change="opt.toggleSuggestion(s.id, ($event.target as HTMLInputElement).checked)"
              />
            </TableCell>
            <TableCell>{{ kindLabel(s) }}</TableCell>
            <TableCell class="text-sm text-muted-foreground">{{ s.rationale_md }}</TableCell>
            <TableCell>{{ s.status }}</TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <Button class="mt-2" :disabled="!acceptedIds.length || opt.busy" @click="doRerun">
        用选中建议重跑求解器
      </Button>
    </div>

    <!-- 对比区 -->
    <div v-if="opt.compareTarget && opt.current" class="grid grid-cols-2 gap-4">
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">原方案 #{{ opt.compareTarget.run_id }}</h4>
        <div class="text-sm">综合 {{ opt.compareTarget.plan.solution.metrics.overall.toFixed(0) }}</div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('parent')">采纳此方案</Button>
      </div>
      <div class="rounded-md border p-3">
        <h4 class="font-medium mb-1">重跑方案 #{{ opt.current.run_id }}</h4>
        <div class="text-sm">综合 {{ opt.current.plan.solution.metrics.overall.toFixed(0) }}</div>
        <Button size="sm" variant="outline" class="mt-2" @click="pick('current')">采纳此方案</Button>
      </div>
    </div>
  </div>
</template>
```

> `Table*` 导入按 `src/pages/ai/index.vue` 现有导入对齐（先 `grep -n "Table" src/pages/ai/index.vue | head`），确保组件名一致。

- [ ] **Step 2: 页面接线**

在 `src/pages/ai/index.vue` 的 `<PlanReview v-if="opt.current" />` 之后加：

```vue
    <PlanCompare />
```

并在 script setup 顶部 import：

```ts
import PlanCompare from "@/components/PlanCompare.vue";
```

- [ ] **Step 3: 构建验证**

Run: `npm run build 2>&1 | tail -20`
Expected: 构建通过

- [ ] **Step 4: Commit**

```bash
git add src/components/PlanCompare.vue src/pages/ai/index.vue
git commit -m "feat(web): PlanCompare component (suggestions + side-by-side)"
```

---

### Task 11: 前端测试

**Files:**
- Modify: `src/stores/refresh.test.ts`
- Create: `src/stores/optimization.test.ts`

- [ ] **Step 1: refresh.test.ts mock 补全**

在 `src/stores/refresh.test.ts` 的 `vi.mock("../api", ...)` 的 `api:` 对象里，`rejectSolution` 之后加：

```ts
    listSuggestions: vi.fn().mockResolvedValue([]),
    rerun: vi.fn(),
    setSuggestionStatus: vi.fn().mockResolvedValue(undefined),
    getOptimizationRun: vi.fn(),
```

- [ ] **Step 2: 写 optimization store 测试**

创建 `src/stores/optimization.test.ts`：

```ts
import { setActivePinia, createPinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useOptimizationStore } from "./optimization";
import { api } from "../api";

vi.mock("../api", () => ({
  api: {
    runOptimization: vi.fn(),
    listOptimizationRuns: vi.fn().mockResolvedValue({ rows: [], total: 0 }),
    getOptimizationRun: vi.fn(),
    listSuggestions: vi.fn().mockResolvedValue([]),
    rerun: vi.fn(),
    setSuggestionStatus: vi.fn().mockResolvedValue(undefined),
    applySolution: vi.fn().mockResolvedValue(undefined),
    rejectSolution: vi.fn().mockResolvedValue(undefined),
  },
}));

beforeEach(() => setActivePinia(createPinia()));

describe("optimization store", () => {
  it("toggleSuggestion updates local status", async () => {
    const opt = useOptimizationStore();
    opt.suggestions = [
      { id: 1, suggestion: { kind: "widen_window", task_id: 5, new_start: "2026-07-01", new_end: "2026-07-20" }, rationale_md: "x", status: "proposed" },
    ];
    await opt.toggleSuggestion(1, true);
    expect(opt.suggestions[0].status).toBe("accepted");
    await opt.toggleSuggestion(1, false);
    expect(opt.suggestions[0].status).toBe("skipped");
  });

  it("rerun sets compareTarget to the prior current", async () => {
    const opt = useOptimizationStore();
    const parent = { run_id: 7, plan: { solution: { run_id: 7, assignments: [], unscheduled: [], metrics: { overall: 50, skill_fit: 0, scheduled_ratio: 0, fairness: 0 }, status: "feasible" as const }, explanation_md: "" } };
    const child = { run_id: 8, plan: { solution: { run_id: 8, assignments: [], unscheduled: [], metrics: { overall: 70, skill_fit: 0, scheduled_ratio: 0, fairness: 0 }, status: "feasible" as const }, explanation_md: "" } };
    opt.current = parent as any;
    (api.rerun as any).mockResolvedValueOnce(child);
    await opt.rerun(7, [1]);
    expect(opt.compareTarget).toEqual(parent);
    expect(opt.current).toEqual(child);
  });
});
```

- [ ] **Step 3: 跑测试**

Run: `npx vitest run src/stores/optimization.test.ts src/stores/refresh.test.ts 2>&1 | tail -25`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/stores/refresh.test.ts src/stores/optimization.test.ts
git commit -m "test(web): optimization store rerun/toggle + refresh mock"
```

---

### Task 12: 全量验证

**Files:** 无（仅运行验证）

- [ ] **Step 1: ai-engine 全 feature 矩阵**

Run: `cargo test -p ai-engine 2>&1 | tail -15 && cargo test -p ai-engine --features llm 2>&1 | tail -15`
Expected: 全 PASS

- [ ] **Step 2: app 全 feature 矩阵**

Run: `cargo test -p app 2>&1 | tail -15 && cargo test -p app --features milp 2>&1 | tail -15`
Expected: 全 PASS（含 Task 6 rerun 测试）

- [ ] **Step 3: server/tauri 编译**

Run: `cargo build -p server 2>&1 | tail -10 && cargo build -p src-tauri 2>&1 | tail -10`
Expected: 编译通过

- [ ] **Step 4: clippy 新代码**

Run: `cargo clippy -p ai-engine -p app -p server --features milp -- -D warnings 2>&1 | tail -20`
Expected: 无 warning

- [ ] **Step 5: 前端构建 + 测试**

Run: `npm run build 2>&1 | tail -10 && npx vitest run 2>&1 | tail -15`
Expected: 构建通过、全测试 PASS

- [ ] **Step 6: 更新 memory**

更新 `/Users/yeheng/.claude/projects/-Users-yeheng-workspaces-Github-kanban/memory/resource-optimization-milp-impl.md`，追加一段记录 LLM 建议对比特性已实现（migration 0008、Advisor trait、rerun、PlanCompare），并把 MEMORY.md 索引行对应更新。

- [ ] **Step 7: 最终 commit（若有残留改动）**

```bash
git add -A
git commit -m "chore: llm-suggestion-compare feature verification complete"
```

---

## 实现顺序与依赖

Task 1→2→3 必须顺序（迁移、类型、trait）。Task 4 依赖 1（settings 列）、2、3。Task 5 依赖 2。Task 6 依赖 1、2、5（apply_suggestions）。Task 7 依赖 4、6。Task 8 依赖 6、7。Task 9 依赖 8（API 契约对齐）。Task 10 依赖 9。Task 11 依赖 9、10。Task 12 全部完成后。
