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
    #[tracing::instrument(skip(self, _problem, sol), fields(assignments = sol.assignments.len(), unscheduled = sol.unscheduled.len()))]
    async fn explain(&self, _problem: &AllocationProblem, sol: &Solution) -> String {
        tracing::debug!("generating template explanation");
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

/// Production LLM explainer via `rig` chat. Supports Ollama, OpenAI, Anthropic and
/// DeepSeek chat providers. The user prompt is rendered from a configurable template
/// (`prompt_template`) with `{placeholder}` substitution; the system preamble is also
/// configurable (`preamble`). Default templates include the full solver input (resources,
/// tasks, skills, dependencies, existing allocations) and output (proposed assignments,
/// unscheduled tasks, metrics) so the model can give concrete, actionable analysis.
/// Falls back to `TemplateExplainer` on any provider error (design §2.8 graceful degradation).
#[cfg(feature = "llm")]
pub mod llm {
    use crate::llm_client::{completion_prompt, LlmClientConfig};
    use crate::types::*;
    use async_trait::async_trait;
    use std::collections::{HashMap, HashSet};

    pub struct LlmExplainer {
        pub provider: String,
        pub model: String,
        pub base_url: Option<String>,
        pub api_key: Option<String>,
        /// User-defined prompt template. `None` uses the built-in default.
        pub prompt_template: Option<String>,
        /// User-defined system preamble. `None` uses the built-in default.
        pub preamble: Option<String>,
    }

    #[async_trait]
    impl super::Explainer for LlmExplainer {
        #[tracing::instrument(skip(self, problem, sol), fields(provider = %self.provider, model = %self.model, assignments = sol.assignments.len(), unscheduled = sol.unscheduled.len()))]
        async fn explain(&self, problem: &AllocationProblem, sol: &Solution) -> String {
            tracing::debug!("generating LLM explanation");
            let cfg = LlmClientConfig {
                provider: self.provider.clone(),
                base_url: self.base_url.clone(),
                api_key: self.api_key.clone(),
                model: self.model.clone(),
            };
            let template = self
                .prompt_template
                .as_deref()
                .unwrap_or_else(|| default_prompt_template());
            let prompt = render_template(template, problem, sol);
            let preamble = self
                .preamble
                .as_deref()
                .unwrap_or_else(|| default_preamble());
            match completion_prompt(&cfg, preamble, &prompt).await {
                Some(text) => format!("## 优化方案说明（AI）\n\n{}", text),
                None => super::TemplateExplainer.explain(problem, sol).await,
            }
        }
    }

    fn default_preamble() -> &'static str {
        "你是资深项目经理，擅长资源分配与风险识别。请基于完整数据给出具体、可执行的分析，避免泛泛而谈。"
    }

    fn default_prompt_template() -> &'static str {
        "# 项目排期优化结果分析\n\
         \n\
         ## 求解器与目标\n\
         - 求解器后端: {solver_backend}\n\
         - 求解状态: {solver_status}\n\
         - 优化权重: 技能匹配 {weights_skill_fit}, 资源均衡 {weights_balance}, 预算 {weights_budget}\n\
         - 项目预算: {budget_pd} PD\n\
         \n\
         ## 资源清单 ({resource_count} 人)\n\
         {resources}\n\
         \n\
         ## 任务清单 ({task_count} 个)\n\
         {tasks}\n\
         \n\
         {existing_allocs}\n\
         ## 优化结果指标\n\
         - 综合评分: {metrics_overall}/100\n\
         - 技能匹配: {metrics_skill_fit}/100\n\
         - 排期覆盖率: {metrics_scheduled_ratio}%\n\
         - 资源公平性: {metrics_fairness}/100\n\
         - 已分配任务: {assignment_count} 个\n\
         - 未排期任务: {unscheduled_count} 个\n\
         \n\
         ## 优化器建议分配 ({assignment_count} 条)\n\
         {assignments}\n\
         \n\
         {unscheduled}\n\
         ## 分析要求\n\
         请基于以上完整信息回答：\n\
         1. 为什么优化器给出当前分配方案？\n\
         2. 未排期任务最可能的原因（技能缺口、时间窗冲突、资源容量不足、预算限制、依赖阻塞等）。\n\
         3. 当前方案的主要风险。\n\
         4. 给出 3-5 条具体、可执行的改进建议（例如：调整哪些任务的时间窗、补充哪些技能/资源、解除哪些依赖）。\n\
         请用中文输出。"
    }

    /// Render a `{placeholder}` template using the problem/solution context.
    /// Unknown placeholders are left as-is so users can see what keys are available.
    fn render_template(template: &str, problem: &AllocationProblem, sol: &Solution) -> String {
        let mut ctx = build_context(problem, sol);
        // Convenience: the default template never references `{full_context}`, so rendering it
        // here is safe and avoids infinite recursion.
        if template != default_prompt_template() {
            ctx.insert(
                "full_context",
                render_template(default_prompt_template(), problem, sol),
            );
        }
        substitute(template, &ctx)
    }

    fn substitute(template: &str, ctx: &HashMap<&'static str, String>) -> String {
        let mut out = String::with_capacity(template.len() * 2);
        let mut chars = template.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c == '{' {
                // Check for escaped `{{` -> literal `{`
                if chars.peek().map(|(_, ch)| *ch) == Some('{') {
                    chars.next();
                    out.push('{');
                    continue;
                }
                // Find matching `}`
                let start = i + 1;
                let mut end = None;
                while let Some((j, ch)) = chars.peek() {
                    if *ch == '}' {
                        end = Some(*j);
                        chars.next();
                        break;
                    }
                    chars.next();
                }
                if let Some(end) = end {
                    let key = &template[start..end];
                    if let Some(value) = ctx.get(key) {
                        out.push_str(value);
                    } else {
                        // Keep unknown placeholder for debugging.
                        out.push('{');
                        out.push_str(key);
                        out.push('}');
                    }
                } else {
                    // Unclosed `{`, copy literally.
                    out.push('{');
                }
            } else if c == '}' {
                // stray `}`
                out.push('}');
            } else {
                out.push(c);
            }
        }
        out
    }

    fn build_context<'a>(
        problem: &'a AllocationProblem,
        sol: &'a Solution,
    ) -> HashMap<&'static str, String> {
        let mut ctx = HashMap::new();

        // Solver / objective.
        ctx.insert("solver_backend", problem.config.backend.clone());
        ctx.insert("solver_status", sol.status.as_str().to_string());
        ctx.insert("weights_skill_fit", format!("{:.2}", problem.weights.skill_fit));
        ctx.insert("weights_balance", format!("{:.2}", problem.weights.balance));
        ctx.insert("weights_budget", format!("{:.2}", problem.weights.budget));
        ctx.insert(
            "budget_pd",
            problem
                .budget_pd
                .map(|b| format!("{:.2}", b))
                .unwrap_or_else(|| "未设置".into()),
        );

        // Counts.
        ctx.insert("resource_count", problem.resources.len().to_string());
        ctx.insert("task_count", problem.tasks.len().to_string());
        ctx.insert("assignment_count", sol.assignments.len().to_string());
        ctx.insert("unscheduled_count", sol.unscheduled.len().to_string());

        // Formatted lists / sections.
        ctx.insert("resources", format_resources(&problem.resources));
        ctx.insert("tasks", format_tasks(&problem.tasks, &problem.dependencies));
        ctx.insert(
            "existing_allocs",
            if problem.existing.is_empty() {
                String::new()
            } else {
                format!(
                    "## 已占用分配 ({} 条)\n{}\n",
                    problem.existing.len(),
                    format_existing(&problem.existing, &problem.resources)
                )
            },
        );
        ctx.insert(
            "assignments",
            format_assignments(&sol.assignments, &problem.resources, &problem.tasks),
        );
        ctx.insert(
            "unscheduled",
            if sol.unscheduled.is_empty() {
                String::new()
            } else {
                format!(
                    "## 未排期任务 ({} 个)\n{}\n",
                    sol.unscheduled.len(),
                    format_unscheduled(
                        &problem.tasks,
                        &sol.unscheduled,
                        &problem.resources,
                        &problem.dependencies
                    )
                )
            },
        );

        // Metrics (pre-formatted).
        ctx.insert("metrics_overall", format!("{:.0}", sol.metrics.overall));
        ctx.insert("metrics_skill_fit", format!("{:.0}", sol.metrics.skill_fit));
        ctx.insert(
            "metrics_scheduled_ratio",
            format!("{:.0}", sol.metrics.scheduled_ratio),
        );
        ctx.insert("metrics_fairness", format!("{:.0}", sol.metrics.fairness));

        ctx
    }

    /// 给 advisor 复用：渲染默认模板（完整 resources/tasks/metrics/assignments/unscheduled）。
    pub fn render_default_context(problem: &AllocationProblem, sol: &Solution) -> String {
        render_template(default_prompt_template(), problem, sol)
    }

    fn format_resources(resources: &[CandidateResource]) -> String {
        if resources.is_empty() {
            return "（无可用资源）".into();
        }
        resources
            .iter()
            .map(|r| {
                let skills = if r.skills.is_empty() {
                    "无".into()
                } else {
                    r.skills
                        .iter()
                        .map(|(sid, prof)| format!("skill_id:{}=prof:{}", sid, prof))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                let tags = if r.tags.is_empty() {
                    "无".into()
                } else {
                    r.tags.join(", ")
                };
                let window = match (r.available_from, r.available_to) {
                    (Some(s), Some(e)) => format!("{} ~ {}", s, e),
                    (Some(s), None) => format!("{} ~ 无限制", s),
                    (None, Some(e)) => format!("无限制 ~ {}", e),
                    (None, None) => "无限制".into(),
                };
                format!(
                    "- R{} [{}]: 日容量 {} PD, 可用期 {}, 技能 [{}], 标签 [{}]",
                    r.id, r.name, r.daily_capacity_pd, window, skills, tags
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_tasks(tasks: &[CandidateTask], deps: &[TaskDependency]) -> String {
        if tasks.is_empty() {
            return "（无任务）".into();
        }
        let dep_map: HashMap<i64, Vec<i64>> =
            deps.iter().fold(HashMap::new(), |mut m, d| {
                m.entry(d.task_id).or_default().push(d.predecessor_id);
                m
            });
        tasks
            .iter()
            .map(|t| {
                let reqs = if t.skill_reqs.is_empty() {
                    "无".into()
                } else {
                    t.skill_reqs
                        .iter()
                        .map(|req| {
                            format!(
                                "skill_id:{} 最低熟练度:{} {}{}",
                                req.skill_id,
                                req.min_proficiency,
                                if req.is_mandatory { "必需" } else { "建议" },
                                if req.weight != 1.0 {
                                    format!(" 权重:{:.2}", req.weight)
                                } else {
                                    String::new()
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                let preds = dep_map
                    .get(&t.id)
                    .map(|ids| ids.iter().map(|id| format!("T{}", id)).collect::<Vec<_>>().join(", "))
                    .unwrap_or_else(|| "无".into());
                format!(
                    "- T{} [{}]: 预估 {:.2} PD, 窗口 {} ~ {}, 优先级 {}, 技能要求 [{}], 依赖 [{}]",
                    t.id, t.title, t.estimate_pd, t.start, t.end, t.priority, reqs, preds
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_existing(existing: &[ExistingAlloc], resources: &[CandidateResource]) -> String {
        let name_map: HashMap<i64, &str> = resources
            .iter()
            .map(|r| (r.id, r.name.as_str()))
            .collect();
        existing
            .iter()
            .map(|a| {
                let name = name_map.get(&a.resource_id).unwrap_or(&"?");
                format!(
                    "- R{} [{}] 已占用: {} ~ {}, 占比 {:.0}%",
                    a.resource_id,
                    name,
                    a.start,
                    a.end,
                    a.percent * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_assignments(
        assignments: &[ScoredAssignment],
        resources: &[CandidateResource],
        tasks: &[CandidateTask],
    ) -> String {
        if assignments.is_empty() {
            return "（无分配）".into();
        }
        let res_map: HashMap<i64, &str> = resources
            .iter()
            .map(|r| (r.id, r.name.as_str()))
            .collect();
        let task_map: HashMap<i64, &str> = tasks
            .iter()
            .map(|t| (t.id, t.title.as_str()))
            .collect();
        assignments
            .iter()
            .map(|a| {
                let rname = res_map.get(&a.resource_id).unwrap_or(&"?");
                let tname = task_map.get(&a.task_id).unwrap_or(&"?");
                format!(
                    "- R{} [{}] -> T{} [{}]: {} ~ {}, 占比 {:.0}%, 匹配度 {:.0}/100",
                    a.resource_id,
                    rname,
                    a.task_id,
                    tname,
                    a.start,
                    a.end,
                    a.percent * 100.0,
                    a.score * 100.0
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_unscheduled(
        tasks: &[CandidateTask],
        unscheduled: &[i64],
        resources: &[CandidateResource],
        deps: &[TaskDependency],
    ) -> String {
        let task_map: HashMap<i64, &CandidateTask> =
            tasks.iter().map(|t| (t.id, t)).collect();
        let unsched_set: HashSet<i64> = unscheduled.iter().copied().collect();
        unscheduled
            .iter()
            .map(|tid| {
                let t = match task_map.get(tid) {
                    Some(t) => t,
                    None => return format!("- T{}: （任务信息缺失）", tid),
                };
                let reasons = heuristic_reasons(t, resources, deps, &unsched_set);
                let reasons_txt = if reasons.is_empty() {
                    "原因待分析".into()
                } else {
                    reasons.join("；")
                };
                format!(
                    "- T{} [{}]: 预估 {:.2} PD, 窗口 {} ~ {}, 可能原因: {}",
                    t.id, t.title, t.estimate_pd, t.start, t.end, reasons_txt
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn heuristic_reasons(
        task: &CandidateTask,
        resources: &[CandidateResource],
        deps: &[TaskDependency],
        unsched_set: &HashSet<i64>,
    ) -> Vec<String> {
        let mut reasons = Vec::new();

        // Dependency blocker.
        for dep in deps {
            if dep.task_id == task.id && unsched_set.contains(&dep.predecessor_id) {
                reasons.push(format!("前置任务 T{} 未排期", dep.predecessor_id));
            }
        }

        // Mandatory skill coverage.
        let mandatory: Vec<&SkillReq> = task
            .skill_reqs
            .iter()
            .filter(|r| r.is_mandatory)
            .collect();
        if !mandatory.is_empty() {
            let mut missing = Vec::new();
            for req in &mandatory {
                let has_resource = resources.iter().any(|r| {
                    r.skills
                        .get(&req.skill_id)
                        .map(|p| *p >= req.min_proficiency)
                        .unwrap_or(false)
                });
                if !has_resource {
                    missing.push(format!(
                        "skill_id:{} 熟练度≥{}",
                        req.skill_id, req.min_proficiency
                    ));
                }
            }
            if !missing.is_empty() {
                reasons.push(format!("缺少必需技能: {}", missing.join(", ")));
            }
        }

        // Time window vs estimate sanity check.
        let window_days = task.end.signed_duration_since(task.start).num_days() + 1;
        if window_days > 0 && task.estimate_pd > window_days as f64 {
            reasons.push(format!(
                "时间窗 {} 天小于预估 {} PD",
                window_days, task.estimate_pd
            ));
        }

        // No resources at all.
        if resources.is_empty() {
            reasons.push("没有可用资源".into());
        }

        reasons
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::explainer::Explainer;

        #[tokio::test]
        #[ignore = "needs LLM provider running with the chat model"]
        async fn smoke_llm() {
            let _ = LlmExplainer {
                provider: "ollama".into(),
                model: "qwen2.5:7b".into(),
                base_url: None,
                api_key: None,
                prompt_template: None,
                preamble: None,
            }
            .explain(&AllocationProblem::default(), &Solution::default())
            .await;
        }

        #[test]
        fn default_prompt_includes_resource_and_task_details() {
            let problem = sample_problem();
            let sol = Solution::default();
            let prompt = render_template(default_prompt_template(), &problem, &sol);
            assert!(prompt.contains("Alice"));
            assert!(prompt.contains("backend"));
            assert!(prompt.contains("skill_id:10"));
            assert!(prompt.contains("优化结果指标"));
        }

        #[test]
        fn custom_prompt_substitution_works() {
            let problem = sample_problem();
            let sol = Solution::default();
            let template = "资源数:{resource_count}, 任务数:{task_count}, 已分配:{assignment_count}";
            let prompt = render_template(template, &problem, &sol);
            assert_eq!(prompt, "资源数:1, 任务数:1, 已分配:0");
        }

        #[test]
        fn custom_prompt_can_use_full_context() {
            let problem = sample_problem();
            let sol = Solution::default();
            let template = "{full_context}";
            let prompt = render_template(template, &problem, &sol);
            assert!(prompt.contains("Alice"));
            assert!(prompt.contains("backend"));
        }

        fn sample_problem() -> AllocationProblem {
            AllocationProblem {
                resources: vec![CandidateResource {
                    id: 1,
                    name: "Alice".into(),
                    skills: [(10, 4)].into_iter().collect(),
                    tags: vec!["rust".into()],
                    daily_capacity_pd: 1.0,
                    available_from: None,
                    available_to: None,
                }],
                tasks: vec![CandidateTask {
                    id: 1,
                    project_id: 1,
                    title: "backend".into(),
                    estimate_pd: 5.0,
                    start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                    end: chrono::NaiveDate::from_ymd_opt(2026, 7, 10).unwrap(),
                    priority: 3,
                    skill_reqs: vec![SkillReq {
                        skill_id: 10,
                        min_proficiency: 3,
                        is_mandatory: true,
                        weight: 1.0,
                    }],
                }],
                ..Default::default()
            }
        }
    }
}
