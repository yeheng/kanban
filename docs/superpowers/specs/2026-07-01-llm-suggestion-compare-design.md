# LLM 建议对比特性设计

- 日期：2026-07-01
- 范围：ai-engine / app / server / web
- 关联：`resource_optimization_task.md`（MILP+LLM 优化主线，已完成 G,A,B,C,D,E,F）

## 1. 目标

LLM 审视 MILP 求解器产出的方案，给出**结构化、可执行**的改进建议（绑定到具体 task/resource，类型有限）。用户勾选若干建议后，系统**修改 problem → 重跑求解器 → 生成新 run**，前端并排展示原方案 vs 重跑方案，用户二选一 apply。

核心不变量：**LLM 从不直接产出最终分配**。最终分配永远由求解器产出，硬约束（容量、时间窗、依赖、预算）始终由求解器保证。LLM 只产"对 problem 的修改意图"。

## 2. 架构与组件边界

新增与 `Explainer` 平级的 `Advisor` 组件，不改动现有 explainer / solver / apply 路径。

```
ai-engine crate
├── explainer.rs   (不变)            → Trait Explainer：出 markdown 解释
├── advisor.rs     (新增, llm feature)→ Trait Advisor：出结构化建议 Vec<SuggestionItem>
├── llm_client.rs  (不变)            → completion_prompt 返回 Option<String>（自由文本）
└── types.rs       (+Suggestion / SuggestionItem 类型)

app crate service/optimization.rs
├── run_for_project (不变；末尾可选调 advisor.advise 并写建议表)
├── rerun           (新增)           → 父 run 重建 problem → 应用选中建议 → 重跑求解器 → 插新 run(parent_run_id)
├── list_suggestions / set_suggestion_status (新增)
└── apply / reject  (不变；apply 对任意 run 都按各自 output_plan_json 工作)
```

关键决策：
- **rerun 从父 run 的 `input_snapshot_json` 反序列化重建 problem**，而非从 DB 现状重建。保证重跑与原跑基于同一输入基线，差异只来自建议。
- **新 run 与原 run 是两张行**，通过 `parent_run_id` 链接。`apply` 对两者一视同仁（都读各自 `output_plan_json`）。用户 apply 一个、reject 另一个（前端编排，后端 apply 不联动 reject）。
- **LLM 调用**走现有 `completion_prompt`（自由文本），prompt 要求模型只输出 JSON 数组，服务端 `serde_json::from_str` 解析；解析失败 → 空 Vec（与 explainer 同样的 graceful degradation）。不引入 rig 的 structured-output（当前未用，多 provider 支持不一）。
- **降级**：`select_advisor`（镜像 `select_explainer`）按设置 `ai.use_llm_advisor` 选 `LlmAdvisor` 或 `NoAdvisor`（后者返回空建议，等价关闭）。

## 3. 数据模型

### 3.1 迁移 `0008_optimization_suggestions.sql`

```sql
-- LLM 给出的、对 solver 方案的结构化改进建议。每条建议绑定到 assignment 或 task/resource，
-- 是"对 problem 的修改意图"而非最终分配；采纳后经 rerun 重跑求解器才落地。
CREATE TABLE ai_optimization_suggestions (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id             INTEGER NOT NULL REFERENCES ai_optimization_runs(id) ON DELETE CASCADE,
    -- 建议类型（有限枚举，见 Suggestion）
    kind               TEXT    NOT NULL CHECK (kind IN (
                            'swap_resource','change_percent','widen_window','drop_dependency',
                            'add_resource','widen_resource_window','change_resource_capacity',
                            'upsert_resource_skill')),
    target_task_id     INTEGER,
    target_resource_id INTEGER,
    -- 完整 Suggestion 序列化（kind + 各字段）。冗余 target_* 便于按 task/resource 查询。
    payload_json       TEXT    NOT NULL,
    rationale_md       TEXT    NOT NULL,
    -- proposed | accepted | skipped | applied
    --   accepted = 用户选中参与重跑；applied = 所在重跑 run 已 apply（追溯用）
    status             TEXT    NOT NULL DEFAULT 'proposed'
                        CHECK (status IN ('proposed','accepted','skipped','applied')),
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_optimization_suggestions_run ON ai_optimization_suggestions(run_id);

-- 重跑产生的 run 指回父 run；NULL = 首次跑（原 run）。复用现有行加可空列。
ALTER TABLE ai_optimization_runs ADD COLUMN parent_run_id INTEGER
    REFERENCES ai_optimization_runs(id) ON DELETE SET NULL;
CREATE INDEX idx_optimization_runs_parent ON ai_optimization_runs(parent_run_id);
```

设置开关：`0006_add_llm_feature_flags.sql` 已在 `settings` 表加 `use_llm_explainer`（`INTEGER NOT NULL DEFAULT 1 CHECK (in (0,1))`）。本特性在同一 `settings` 表新增 `use_llm_advisor`（同样 `DEFAULT 0 CHECK (in (0,1))`——**默认关**，因为建议功能比解释更重，需用户显式开）。迁移**并入 0008**（同一文件里 `ALTER TABLE settings ADD COLUMN use_llm_advisor INTEGER NOT NULL DEFAULT 0 CHECK (use_llm_advisor IN (0,1));`），不另起 0009。`select_advisor` 读 `ai.use_llm_advisor`，镜像 `select_explainer` 读 `ai.use_llm_explainer`（`AiSettings` 结构体同步加字段）。

### 3.2 Rust 类型（`ai-engine/src/types.rs`）

```rust
/// LLM 对 solver 方案的一条结构化改进建议。是"对 problem 的修改意图"，采纳后经 rerun
/// 重跑求解器落地——LLM 从不直接产出最终分配，硬约束始终由求解器保证。
#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionItem {
    pub id: Option<i64>,        // None until persisted
    pub suggestion: Suggestion,
    pub rationale_md: String,
    pub status: String,         // proposed | accepted | skipped | applied
}
```

`#[serde(tag = "kind")]` 内部标签：LLM 输出 `{"kind":"swap_resource","task_id":5,"new_resource_id":3,"rationale":"..."}` 直接反序列化；未知 kind 被拒。

## 4. Advisor trait 与 LLM JSON 契约

```rust
// ai-engine/src/advisor.rs
#[async_trait]
pub trait Advisor: Send + Sync {
    async fn advise(&self, problem: &AllocationProblem, sol: &Solution) -> Vec<SuggestionItem>;
}
pub struct NoAdvisor;                      // 默认：空建议
#[cfg(feature = "llm")]
pub struct LlmAdvisor { /* provider/model/base_url/api_key/preamble/prompt_template，同 LlmExplainer */ }
```

`select_advisor`（镜像 `select_explainer`）按 `ai.use_llm_advisor`。

**prompt 契约**：复用 `explainer.rs` 的 `build_context`（同一份 problem+solution 文本上下文），末尾追加约束——只输出一个 JSON 数组、无其它文字、无 markdown 代码块；kind 必须是 8 个之一；字段名严格匹配；日期 `YYYY-MM-DD`；不得引用上下文中不存在的 id。每个元素含 `rationale`。

**解析与校验**：
1. `serde_json::from_str::<Vec<RawSuggestionItem>>(&text)` —— 整体解析失败 → 返回空 Vec，warn。
2. 逐条校验：task_id/resource_id 在 problem 里存在；`new_percent ∈ (0,1]`；日期不反序、`new_proficiency ∈ 1..=5`、`new_daily_capacity_pd > 0`。非法条目丢弃、warn，合法的保留。
3. `rationale` 字段映射到 `rationale_md`。

## 5. rerun 应用逻辑

```
rerun(parent_run_id, accepted_suggestion_ids) -> Result<RunResult>:
  1. 读父 run 行：input_snapshot_json, weights_json, config, project_id (从 scope_project_ids)
  2. let mut problem: AllocationProblem = serde_json::from_str(input_snapshot_json)
  3. 从 ai_optimization_suggestions 读 run_id=parent 且 id IN accepted 的条目
  4. 对每条 Suggestion 在内存 problem 上应用（见下表）；互相冲突时后写覆盖、warn
  5. problem.run_id = 0（待重跑填）
  6. 复用 run_for_project 下半段：scorer.matrix → solve_with_fallback → explainer.explain
  7. INSERT 新 run 行（与原 run 同列），parent_run_id = parent_run_id
  8. 被采纳建议 status 改 'applied'；未采纳保持 proposed/skipped
  9. 返回 RunResult{run_id: 新id, plan}
```

### 5.1 逐类型应用规则

| Suggestion | 在 AllocationProblem 上的应用 |
|---|---|
| `WidenWindow` | `tasks[*].id==task_id`，若 `new_start≤原start && new_end≥原end` 则覆盖 start/end（**只放宽不收窄**，防 LLM 误缩）。 |
| `DropDependency` | 从 `dependencies` 移除 `(task_id, predecessor_id)` 边。 |
| `AddResource` | 按 resource_id 从 DB 查 resources+skills+tags+capacity，构造 `CandidateResource` push 进 `problem.resources`（已存在则跳过）。**唯一需回 DB 的类型**。 |
| `WidenResourceWindow` | `resources[*].id`，放宽 available_from/to（只放宽不收窄）。 |
| `ChangeResourceCapacity` | `resources[*].id`，覆盖 `daily_capacity_pd`（>0 校验）。 |
| `UpsertResourceSkill` | `resources[*].id`，`skills.insert(skill_id, new_proficiency)`（1..=5 校验）。 |
| `SwapResource` | **advisory，不强制生效**。当前求解器是"选谁做"的广义指派，无"必须让 R3 做 T5"硬指令旋钮。参与重跑（problem 不变），在结果对比里标注"建议换人，实际由求解器决定"。 |
| `ChangePercent` | **advisory，不强制生效**。problem 无 per-task percent 字段（percent 由 needed_percent 推导）。rerun 暂不应用，UI 标"未影响重跑"。 |

**诚实标注**：`SwapResource`/`ChangePercent` 在当前求解器模型下没有干净的 problem 旋钮可拧。若要真正强制生效，需给求解器加"指派固定/占比约束"变量——列为后续增强，不在本特性范围。当前仅 advisory。

## 6. 服务方法与 HTTP 路由

### 6.1 服务方法（`app/src/service/optimization.rs`）

```rust
list_suggestions(pool, run_id) -> Vec<SuggestionItem>
rerun(pool, parent_run_id, accepted_suggestion_ids: Vec<i64>) -> RunResult
set_suggestion_status(pool, suggestion_id, status) -> ()
```

`run_for_project` 末尾追加：若 `select_advisor` 非 NoAdvisor，调 `advise` 批量 INSERT 建议（status='proposed'）。原 run 的 `explanation_md` 仍由 explainer 独立产出。

### 6.2 路由（`server/src/routes/optimization.rs`，新增 3 条）

```rust
.route("/api/optimization/runs/{id}/suggestions", get(list_suggestions))
.route("/api/optimization/runs/{id}/rerun", post(rerun_run))    // body: {suggestion_ids: number[]}
.route("/api/optimization/suggestions/{id}", patch(set_suggestion_status)) // body: {status}
```

返回：`list_suggestions` → `Json<Vec<SuggestionItem>>`；`rerun` → `Json<RunResult>`（新 run）；`set_suggestion_status` → 204。

## 7. 错误处理与降级

| 情况 | 行为 |
|---|---|
| LLM provider 错误 / 返回 None | `advise` 返回空 Vec；run 正常落库，前端显示"AI 无建议"。 |
| LLM 返回非 JSON / 解析失败 | 整批丢弃，空 Vec；`tracing::warn!` 含原始文本前 200 字。不抛错。 |
| 单条建议字段非法 | 丢弃该条，保留其余；逐条 warn。 |
| `rerun` 父 run 不存在 | `AppError::not_found`（404）。 |
| `rerun` accepted_ids 含不属于该 run 的 id | 忽略越界 id（只取 run_id 匹配的），warn。不报错。 |
| `rerun` 求解器 infeasible/timeout | `solve_with_fallback` 已有 greedy 兜底，永远返回 Feasible；新 run 落库，`solver_status` 反映真实。 |
| `AddResource` 的 resource_id 在 DB 不存在 | 跳过该建议，warn；其余照常应用。 |
| `rerun` 方案与父方案完全相同 | 不特殊处理——仍生成新 run，前端并排展示"无差异"，用户自行决定。 |

**不破坏现有 apply**：`apply()` 完全不变，对原 run 和重跑 run 都按各自 `output_plan_json` 工作。

## 8. 前端

### 8.1 `src/types.ts`

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

### 8.2 `src/api/index.ts`

```ts
listSuggestions: (runId: number): Promise<SuggestionItem[]> =>
  request("GET", `/api/optimization/runs/${runId}/suggestions`),
rerun: (runId: number, suggestionIds: number[]): Promise<RunResult> =>
  request("POST", `/api/optimization/runs/${runId}/rerun`, { suggestion_ids: suggestionIds }),
setSuggestionStatus: (id: number, status: string): Promise<void> =>
  request("PATCH", `/api/optimization/suggestions/${id}`, { status }),
```

### 8.3 `src/stores/optimization.ts` 扩展

```ts
const suggestions = ref<SuggestionItem[]>([]);
const compareTarget = ref<RunResult | null>(null); // 父 run，与 current(重跑 run) 并排
async function loadSuggestions(runId) { suggestions.value = await api.listSuggestions(runId); }
async function rerun(runId, ids) {
  const parent = current.value;
  current.value = await api.rerun(runId, ids);
  compareTarget.value = parent;
  await loadSuggestions(current.value.run_id);
}
async function toggleSuggestion(id, on) {
  await api.setSuggestionStatus(id, on ? "accepted" : "skipped");
  // 本地更新 status，不重跑（重跑是显式按钮）
}
```

### 8.4 对比 UI（`src/components/PlanCompare.vue` 新增，复用 shadcn Table）

- **建议区**：复选框列表，每条显示 kind 图标 + 目标 task/resource + rationale_md。底部"用选中建议重跑"按钮（无 accepted 或 busy 时禁用）。
- **对比区**（`compareTarget` 与 `current` 都在时）：两列 `PlanReview` 子组件；metrics 行高亮差异（升绿降红）；assignment 表按 task_id 对齐、标出新增/移除/换人。底部"采纳左/右"按钮 → `accept` 一个 + `reject` 另一个。
- `src/pages/ai/index.vue`：`<PlanCompare v-if="opt.suggestions.length || opt.compareTarget" />` 替换原单一 `<PlanReview>`。

## 9. 测试

### 9.1 Rust

`crates/app/tests/optimization.rs` + `crates/ai-engine/tests/advisor.rs`（新增）：

- `advisor_parse_invalid_json_returns_empty`
- `advisor_drops_unknown_kind_keeps_valid`
- `advisor_validates_ids_against_problem`
- `rerun_applies_widen_window_and_rewrites_solution`
- `rerun_drop_dependency_unschedules_cascade_reversed`
- `rerun_upsert_skill_enables_previously_infeasible_task`
- `rerun_add_resource_from_db`
- `rerun_swap_percent_advisory_no_force`
- `rerun_inserts_new_run_with_parent_run_id`
- `rerun_ignores_suggestion_ids_not_in_run`
- `rerun_solver_infeasible_falls_back_to_greedy`
- `no_advisor_returns_empty_suggestions`

### 9.2 前端

- `refresh.test.ts` api mock 补 `listSuggestions/rerun/setSuggestionStatus`。
- 新增 `optimization.test.ts`：`toggleSuggestion` 本地状态更新；`rerun` 后 `compareTarget` = 原 `current`。

## 10. 范围外（后续增强）

- 求解器加"指派固定/占比约束"变量，使 `SwapResource`/`ChangePercent` 真正强制生效（当前仅 advisory）。
- 资源/技能变更永久回写主数据（当前只影响 rerun 内存快照）。
- LLM 解释与建议合并为一次调用（当前两次独立调用）。
