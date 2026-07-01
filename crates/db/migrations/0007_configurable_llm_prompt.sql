-- 0007: make the LLM explainer prompt template configurable.
--
-- Adds two columns to settings so users can customize the user prompt template and
-- the system preamble used when generating optimization result explanations.
-- Both have sensible defaults that match the previous built-in prompt.

ALTER TABLE settings ADD COLUMN ai_explanation_prompt TEXT NOT NULL DEFAULT '# 项目排期优化结果分析

## 求解器与目标
- 求解器后端: {solver_backend}
- 求解状态: {solver_status}
- 优化权重: 技能匹配 {weights_skill_fit}, 资源均衡 {weights_balance}, 预算 {weights_budget}
- 项目预算: {budget_pd} PD

## 资源清单 ({resource_count} 人)
{resources}

## 任务清单 ({task_count} 个)
{tasks}

{existing_allocs}
## 优化结果指标
- 综合评分: {metrics_overall}/100
- 技能匹配: {metrics_skill_fit}/100
- 排期覆盖率: {metrics_scheduled_ratio}%
- 资源公平性: {metrics_fairness}/100
- 已分配任务: {assignment_count} 个
- 未排期任务: {unscheduled_count} 个

## 优化器建议分配 ({assignment_count} 条)
{assignments}

{unscheduled}
## 分析要求
请基于以上完整信息回答：
1. 为什么优化器给出当前分配方案？
2. 未排期任务最可能的原因（技能缺口、时间窗冲突、资源容量不足、预算限制、依赖阻塞等）。
3. 当前方案的主要风险。
4. 给出 3-5 条具体、可执行的改进建议（例如：调整哪些任务的时间窗、补充哪些技能/资源、解除哪些依赖）。
请用中文输出。';

ALTER TABLE settings ADD COLUMN ai_explanation_preamble TEXT NOT NULL DEFAULT '你是资深项目经理，擅长资源分配与风险识别。请基于完整数据给出具体、可执行的分析，避免泛泛而谈。';
