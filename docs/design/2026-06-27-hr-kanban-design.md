# Development Resource Kanban（开发人力资源管理看板）设计方案

> **一句话定位**：用 AI 优化「开发人力 → 任务/项目」配置的跨平台桌面看板——以人/团队为单位实时掌握 workload，并以 Kanban / Gantt / 日历 可视化、输出报表。简称 DevResource Kanban。

> **技术栈**：Tauri v2 + Rust（tokio / sqlx / SQLite+SQLCipher / rig）+ 前端 Vite + Vue 3。

> **关键决策**：AI 优化 = 经典优化器（硬约束）+ LLM（技能语义匹配 & 方案解释）；模型本地优先（Ollama）+ 可选云（rig）；单用户本地桌面 + 本地 SQLite；DB 加密默认开启。


---

## 目录 (TOC)

- [1. 概述与目标 / Overview & Goals](#1-概述与目标-overview-goals)
- [2. 系统架构 / System Architecture](#2-系统架构-system-architecture)
- [3. 数据模型 / Data Model (SQLite)](#3-数据模型-data-model-sqlite)
- [4. 工作负载与容量模型 / Workload & Capacity](#4-工作负载与容量模型-workload-capacity)
- [5. AI 优化引擎 / AI Optimization Engine](#5-ai-优化引擎-ai-optimization-engine)
- [6. 后端命令与服务层 / Backend Service & Tauri Commands](#6-后端命令与服务层-backend-service-tauri-commands)
- [7. 前端与 UI / Frontend & UI](#7-前端与-ui-frontend-ui)
- [8. 报表与导出 / Reporting & Export](#8-报表与导出-reporting-export)
- [9. 路线图 / 非功能 / 风险 / 开放问题](#9-路线图-非功能-风险-开放问题)
  - [1.1 产品定位](#11-产品定位)
  - [1.2 核心目标清单](#12-核心目标清单)
  - [1.3 目标用户](#13-目标用户)
  - [1.4 关键功能要点](#14-关键功能要点)
  - [1.5 非目标（YAGNI / 明确不做）](#15-非目标（yagni-明确不做）)
  - [1.6 成功标准 / 验收信号](#16-成功标准-验收信号)
  - [1.7 高层用例列表](#17-高层用例列表)
  - [1.8 假设（本节）](#18-假设（本节）)
  - [1.9 开放问题（本节）](#19-开放问题（本节）)
  - [2.1 分层概览](#21-分层概览)
  - [2.2 进程内分层与数据流（文字图）](#22-进程内分层与数据流（文字图）)
  - [2.3 模块边界与职责](#23-模块边界与职责)
  - [2.4 为什么单用户本地仍用 tokio（优化在进程内运行）](#24-为什么单用户本地仍用-tokio（优化在进程内运行）)
  - [2.5 IPC 边界设计原则](#25-ipc-边界设计原则)
  - [2.6 目录结构与 crates 划分](#26-目录结构与-crates-划分)
  - [2.7 构建 / 打包考量](#27-构建-打包考量)
  - [2.8 AI 功能降级链（架构层面）](#28-ai-功能降级链（架构层面）)
  - [2.9 单位换算 N 值（固定，不做地区动态计算）](#29-单位换算-n-值（固定，不做地区动态计算）)
  - [2.10 本章假设](#210-本章假设)
  - [2.11 本章开放问题](#211-本章开放问题)
  - [3.1 设计原则与通用约定](#31-设计原则与通用约定)
  - [3.2 实体关系总览（ER 概览）](#32-实体关系总览（er-概览）)
  - [3.3 完整 DDL](#33-完整-ddl)
  - [3.4 长期任务的建模](#34-长期任务的建模)
  - [3.5 跨项目复用的建模](#35-跨项目复用的建模)
  - [3.6 关键查询示例](#36-关键查询示例)
  - [3.7 并发写入与事务策略](#37-并发写入与事务策略)
  - [3.8 容量/分配一致性约束（硬约束清单）](#38-容量分配一致性约束（硬约束清单）)
  - [3.9 迁移策略（sqlx migrate）](#39-迁移策略（sqlx-migrate）)
  - [3.10 数值示例（PD/PM 换算与利用率）](#310-数值示例（pdpm-换算与利用率）)
  - [3.11 与 AI 引擎的接口契约（数据层）](#311-与-ai-引擎的接口契约（数据层）)
  - [3.12 假设（本章）](#312-假设（本章）)
  - [3.13 开放问题（本章，需 impl 期决策）](#313-开放问题（本章，需-impl-期决策）)
  - [4.1 计量单位：PD 与 PM 的可配置换算](#41-计量单位：pd-与-pm-的可配置换算)
  - [4.2 日历模型 / Calendar Model](#42-日历模型-calendar-model)
  - [4.3 容量计算 / Capacity](#43-容量计算-capacity)
  - [4.4 工作负载计算 / Workload](#44-工作负载计算-workload)
  - [4.5 利用率与过载检测 / Utilization & Overload](#45-利用率与过载检测-utilization-overload)
  - [4.6 两种聚合口径 / Aggregation Scopes](#46-两种聚合口径-aggregation-scopes)
  - [4.7 实时性策略 / Real-time Strategy](#47-实时性策略-real-time-strategy)
  - [4.8 单位切换的一致换算 / Unit Switching](#48-单位切换的一致换算-unit-switching)
  - [4.9 Rust 计算核心：签名与伪代码](#49-rust-计算核心：签名与伪代码)
  - [4.10 数值示例：某工程师跨两项目的一周](#410-数值示例：某工程师跨两项目的一周)
  - [4.11 本章假设](#411-本章假设)
  - [4.12 本章开放问题](#412-本章开放问题)
  - [5.1 设计目标与原则](#51-设计目标与原则)
  - [5.2 混合管线总览](#52-混合管线总览)
  - [5.3 阶段 (a)：输入构建](#53-阶段-a：输入构建)
  - [5.4 阶段 (b)：语义匹配打分](#54-阶段-b：语义匹配打分)
  - [5.5 阶段 (c)：经典优化求解](#55-阶段-c：经典优化求解)
  - [5.6 阶段 (d)：LLM 解释](#56-阶段-d：llm-解释)
  - [5.7 阶段 (e)：人机闭环与落库](#57-阶段-e：人机闭环与落库)
  - [5.8 横切关注点](#58-横切关注点)
  - [5.9 Tauri IPC 集成](#59-tauri-ipc-集成)
  - [5.10 引擎编排总览（Engine）](#510-引擎编排总览（engine）)
  - [6.1 分层结构与目录约定](#61-分层结构与目录约定)
  - [6.2 命令分组清单](#62-命令分组清单)
  - [6.3 DTO 与领域模型映射](#63-dto-与领域模型映射)
  - [6.4 统一错误模型](#64-统一错误模型)
  - [6.5 事务边界（sqlx 封装）](#65-事务边界（sqlx-封装）)
  - [6.6 异步与进度回传（长任务）](#66-异步与进度回传（长任务）)
  - [6.7 校验层（预检）](#67-校验层（预检）)
  - [6.8 配置与密钥管理](#68-配置与密钥管理)
  - [6.9 典型 Command 签名与前端调用示例](#69-典型-command-签名与前端调用示例)
  - [6.10 Tauri 注册与 AppState](#610-tauri-注册与-appstate)
  - [假设（本节）](#假设（本节）)
  - [开放问题（本节）](#开放问题（本节）)
  - [7.1 技术选型确认](#71-技术选型确认)
  - [7.2 目录与模块划分](#72-目录与模块划分)
  - [7.3 状态管理设计（Pinia 按领域拆分）](#73-状态管理设计（pinia-按领域拆分）)
  - [7.4 与后端交互层](#74-与后端交互层)
  - [7.5 核心视图设计](#75-核心视图设计)
  - [7.6 跨视图联动](#76-跨视图联动)
  - [7.7 关键组件清单与契约示例](#77-关键组件清单与契约示例)
  - [7.8 前端与 Rust 后端的 IPC 命令清单（摘要）](#78-前端与-rust-后端的-ipc-命令清单（摘要）)
  - [7.9 前端性能与体积约束](#79-前端性能与体积约束)
  - [7.10 MVP 分阶段建议（仅前端范围）](#710-mvp-分阶段建议（仅前端范围）)
  - [7.11 假设（本节）](#711-假设（本节）)
  - [7.12 开放问题（本节）](#712-开放问题（本节）)
  - [8.0 数据模型依赖（与第 3 节 schema 的对齐说明）](#80-数据模型依赖（与第-3-节-schema-的对齐说明）)
  - [8.1 设计原则](#81-设计原则)
  - [8.2 报表类型清单](#82-报表类型清单)
  - [8.3 数据口径](#83-数据口径)
  - [8.4 报表参数](#84-报表参数)
  - [8.4a R4「AI 决策记录」报表的内容边界（决策 4）](#84a-r4「ai-决策记录」报表的内容边界（决策-4）)
  - [8.5 报表表头结构示例](#85-报表表头结构示例)
  - [8.6 导出格式](#86-导出格式)
  - [8.7 生成方式分工（前端渲染 vs Rust 端生成）](#87-生成方式分工（前端渲染-vs-rust-端生成）)
  - [8.8 报表模板与可配置](#88-报表模板与可配置)
  - [8.9 人力快照存档（R8）](#89-人力快照存档（r8）)
  - [8.10 审计与可复现元数据](#810-审计与可复现元数据)
  - [8.11 实现优先级与路线对齐（决策 9）](#811-实现优先级与路线对齐（决策-9）)
  - [8.12 与 §7 TrendExplainer 的引用对齐](#812-与-§7-trendexplainer-的引用对齐)
  - [8.13 假设 / Assumptions](#813-假设-assumptions)
  - [8.14 开放问题 / Open Questions](#814-开放问题-open-questions)
  - [9.1 分阶段 MVP 路线](#91-分阶段-mvp-路线)
  - [9.2 非功能需求](#92-非功能需求)
  - [9.3 主要风险与缓解](#93-主要风险与缓解)
  - [9.4 开放问题清单](#94-开放问题清单)


---

## 术语表 (Glossary)

| 术语 | 含义 |
|---|---|
| **Resource / 资源** | 一名开发者（人）。 |
| **Team / 团队** | 资源的集合，可跨项目。 |
| **Tag** | 资源或任务上的自由标签。 |
| **Skill / 技能** | 可被度量/匹配的能力（1-5 熟练度），区分 must-have / nice-to-have。 |
| **Project / 项目** | 顶层容器，含周期/优先级/预算(budget_pd)。 |
| **Task / 任务** | 项目下的工作项，含所需技能/工作量/时间窗/依赖/状态。 |
| **Long-term task / 长期任务** | 跨度超阈值(long_task_span_limit)强制分段排期的任务。 |
| **Allocation / 分配** | 资源（含投入比例 percent）在某时间区间绑定到任务（统一用词）。 |
| **Workload / 工作负载** | 时间窗内已被分配的人力总量（PD/PM）。 |
| **Capacity / 容量** | 时间窗内可用人力（扣节假日/请假，受 day_factor 影响）。 |
| **Utilization / 利用率** | Workload / Raw Capacity；阈值默认 110%，可配、可按团队/角色分级。 |
| **Person-Day (PD) / 人日** | 基本工时单位（默认 1 PD = 8h，可团队级覆盖）。 |
| **Person-Month (PM) / 人月** | 默认 1 PM = N PD（N=20，可自定义）。 |
| **day_factor / 日历因子** | 某资源某日有效容量比例（work_week_template×holiday×time_off 叠加，0~1）。 |
| **work_week_template / holiday / time_off** | 工作周模板（含 mon_frac..sun_frac 非均匀日）/ 节假日 / 资源请假（三表单一真相源）。 |
| **team_overrides** | 团队级覆盖（pd_hours/pm_workdays/利用率阈值），effective 值回落 settings 全局。 |
| **resource_project_rates** | resource×project[,period] 维度费率，支持按项目/周期浮动 daily_rate_pd。 |
| **effective_daily_rate(r,p,d)** | 成本单价解析：resource_project_rates → resources.daily_rate_pd → N/A。 |
| **ai_optimization_runs** | AI 优化运行记录表（INTEGER PK，含输入快照/seed/score/trigger，可复现）。 |
| **ObjectiveWeights** | 多目标权重（均衡负载/技能最优/预算…），UI 可调，写入目标函数。 |
| **db::with_write_tx** | 统一写事务封装（&SqlitePool, BEGIN IMMEDIATE + busy_timeout + 退避重试）。 |
| **AppError / DomainError** | IPC 错误模型 / 领域错误；DomainError 不派生 serde，映射为 AppError::Validation/Domain(DomainErrorDetail)。 |
| **TemplateExplainer / FallbackScorer** | 降级链：LLM 不可用→规则模板解释；embedding 不可用→关键词+熟练度打分。 |
| **good_lp + HiGHS** | 默认 ILP 后端（静态链接，体积无上限）。 |
| **Tauri v2** | 跨平台桌面外壳，WebView+Rust，tauri::command IPC。 |
| **sqlx / SQLite(WAL)** | Rust 异步 SQL + 本地 SQLite（WAL，SQLCipher 整库加密默认开启）。 |
| **rig** | 0xPlaygrounds/rig，LLM/embedding/provider 抽象（本地 Ollama 默认 + 可选云）。 |
| **Ollama** | 本地大模型运行时（默认 chat=qwen2.5:7b, embed=nomic-embed-text）。 |
| **IPC / DTO** | 前端 invoke()↔Rust command；DTO 与领域模型分离。 |
| **MILP** | 混合整数线性规划。 |


---

## 正文

## 1. 概述与目标 / Overview & Goals

### 1.1 产品定位

**Development Resource Kanban**（全称；日常简称 **DevResource Kanban**）是一款面向「开发人力资源」的本地优先（local-first）跨平台桌面管理看板。它把传统的「人员排班表 + 项目甘特图 + 资源利用率表」整合进单一应用，并以 **AI 驱动的人→任务/项目分配优化** 为核心差异化能力。

> **命名规范**：对外文档、版权头、安装包名等正式场合统一使用全称 *Development Resource Kanban*；UI 标题栏、日志、命令行、README、代码模块名等可使用简称 *DevResource Kanban*。两者指向同一产品，不得并存出现第三种写法。

> **一句话价值主张**
> *用 AI 把正确的开发者，在正确的时间，按正确的投入比例，分配到正确的任务上——并以 Kanban / Gantt / 日历 多视角可视化整个研发组织的真实人力占用，最终一键产出可解释的人力配置报表。*

产品形态：Tauri v2 桌面应用（单用户、本地 SQLite、完全可离线），定位为**决策支持工具**而非工时记录系统。它回答管理者的三类核心问题：

| 问题 | 产品能力 |
|---|---|
| 「我的人现在都被压在哪些事上？」 | 实时 workload / 利用率可视化 |
| 「下一个项目/任务该派给谁？」 | AI 优化分配 + tag/skill 语义匹配 |
| 「这个季度人力配置合不合理？」 | 多项目聚合 + 报表导出 |

### 1.2 核心目标清单

按优先级排序，MVP 须达成 G1–G4，G5–G6 为进阶：

| 编号 | 目标 | 衡量指标 / 完成定义 |
|---|---|---|
| **G1** | 以 PD/PM 为统一计算单元，量化人→任务/项目的分配 | 单位可在设置中切换且全系统口径一致；同一份分配数据切换单位后数值符合 `PM = PD / N` |
| **G2** | 支持多项目、多任务、长期任务（可分段排期）的建模与分配 | 一个资源可同时出现在 ≥2 个项目的不同任务中，且时间不重叠 |
| **G3** | AI 给出可复现、可约束、可解释的优化分配方案 | 相同输入 + 固定随机种子 → 相同输出；硬约束（容量上限、时间窗、跨项目不冲突）100% 满足；每条建议附自然语言理由；**支持多目标权重可由 UI 调节** |
| **G4** | 以 Kanban / Gantt / 日历 三视角实时呈现 workload 与利用率 | 任一分配变更后，三视图在 ≤500ms 内完成重渲染（10 资源 / 50 任务规模） |
| **G5** | 一键导出人力配置 / 利用率报表（CSV/Markdown/Excel） | 报表含按人，按团队，按项目三种聚合维度 |
| **G6** | 完全离线运行，数据本地化 | 断网状态下除云端 LLM 外全部功能可用；本地 Ollama 模型可独立完成 AI 优化 |

### 1.3 目标用户

| 角色 | 核心诉求 | 在本产品中的高频操作 |
|---|---|---|
| **研发管理者 / 研发总监** | 掌握全组织人力分布、识别过载/闲置、做季度人力规划 | 多项目聚合 workload、利用率红绿灯、报表导出、What-if 重分配 |
| **项目经理 (PM)** | 把任务合理派给合适的人、跟踪交付进度与风险 | 任务拆解、技能声明、一键 AI 分配、Gantt 排期、冲突预警 |
| **Tech Lead / 技术负责人** | 保证任务的技术匹配度、平衡团队成员成长与负载 | 维护技能熟练度、tag 体系、复核 AI 建议、调整投入比例 |
| **（次要）资源/开发者本人** | 查看自己的任务与负载 | 只读查看个人 Kanban / 日历（只读视图，不打卡不考勤） |

> 说明：本产品面向**管理者视角**，不为开发者提供工时填报入口——workload 由分配计划推导，而非打卡数据累加。

### 1.4 关键功能要点

1. **统一计算单元（PD/PM 可切换，多级覆盖）**
   - 基本单位 PD（人日），默认 `1 PD = 8h`；PM（人月）默认 `1 PM = 20 PD`。
   - **覆盖层级**：换算常数支持 `全局 → 团队 → 组织（实例）` 多级覆盖，而非仅全局单一值。优先级为「团队级 > 组织级 > 全局默认」，未显式覆盖的层级回退到上级。
   - 全系统（schema 存储 PD、UI 展示、报表、AI 约束）以 PD 为存储口径，PM 仅作展示换算，避免双口径导致的不一致。
   - 数值示例：某任务估算 `16 PD`，在 PM 视图下显示为 `0.8 PM`（按 N=20）；若某团队将其 `pd_per_pm` 覆盖为 21，则该团队视图下同一任务显示为 `0.76 PM`。

   | 配置项 | 默认 | 覆盖层级（优先级高→低） | 作用域 |
   |---|---|---|---|
   | `hours_per_pd` | 8 | 团队级 → 全局 | 1 PD 折算的小时数，用于 capacity 与工时换算 |
   | `pd_per_pm`（即 N） | 20 | 团队级 → 全局 | PM 仅作展示换算的分母：`PM = PD / N` |

   > impl 期决策：团队级覆盖的存储形式（独立配置表 vs settings.metadata JSON）与 UI 落点待第 3/7 章细化。

2. **多项目 / 多任务 / 长期任务建模**
   - Project（顶层容器，含周期/优先级/总人力预算）→ Task（工作项，含所需 skill、工作量 PD、时间窗 `[start, due]`、依赖、状态）。
   - 长期任务支持**分段排期**：一个 Task 可拆为多个 Allocation 段，跨多个时间窗分阶段交付。
   - 依赖：Task 间可声明 `depends_on`，AI 排期与手动排期均需满足偏序约束。

3. **跨项目资源复用**
   - Resource（一名开发者）可被分配到多个项目的不同任务，受**投入比例（commitment %）** 与**时间不重叠**双重约束。
   - 示例：开发者 Alice 在 6/1–6/15 对项目 P1 投入 60%、对 P2 投入 40%，二者时段可并行但总比例 ≤100%。

4. **实时 Workload / 利用率（阈值按角色/团队分别设定）**
   - 任一时间窗内，资源/团队的 workload = Σ(分配 PD)；利用率 = workload / capacity。
   - Capacity 扣除节假日、请假、非工作日，并受投入比例影响。
   - 利用率红绿灯阈值**支持按角色 / 团队分别设定**，优先级为「角色级 > 团队级 > 全局默认」，未设定者回退到全局默认。

   | 档位 | 默认阈值 | 含义 | 颜色 |
   |---|---|---|---|
   | 闲置 | `< 70%` | 负载不足，可承接更多 | 蓝 |
   | 健康 | `70% – 100%` | 合理区间 | 绿 |
   | 过载 | `> 100%` | 超过容量，需重新分配 | 红 |

   > 示例：研发管理者可将「SRE 角色」过载阈值下调到 90%、将「外包团队」闲置阈值上调到 80%，以适配不同岗位的健康基线。
   > impl 期决策：角色级 / 团队级阈值的存储结构（独立表 vs 配置 JSON）与 UI 落点待第 3/7 章细化。

5. **Tag / Skill 驱动的多目标 AI 优化**
   - 资源带 tag 与 skill（含熟练度等级 1–5）；任务声明所需 skill。
   - AI = 经典优化器（硬约束：容量/时间窗/不冲突，用 good_lp ILP 或贪心+匈牙利匹配）+ LLM（tag↔skill 语义匹配打分 + 自然语言解释）。
   - **多目标权重可调**：用户在 AI 优化面板中可拖动 / 输入各优化目标的权重，目标函数为加权综合；未调节时使用均衡默认权重。

   | 优化目标 | 含义 | 默认权重 | 说明 |
   |---|---|---|---|
   | **均衡负载**（balance） | 最小化资源间利用率方差，避免过载/闲置 | 0.4 | 优先让 workload 在团队内均衡分布 |
   | **技能最优**（skill fit） | 最大化 tag↔skill 语义匹配得分 | 0.4 | 优先把任务派给最匹配的人 |
   | **预算约束**（budget） | 最小化超出项目 `budget_pd` 的分配 | 0.2 | 优先贴合人力预算 |

   - 权重经 UI 归一化后输入求解器，结果随权重组合不同而变化；切换权重重跑视为新的 run。
   - 结果**可复现**（固定种子）、**可约束**（管理者可锁定/排除）、**可解释**（每条建议附理由，如「Alice 具备 Rust(L5) 且 6 月利用率仅 55%；当前权重组合下技能匹配权重为 0.4，故优先派给 Alice」）。
   > impl 期决策：权重 UI 控件形态（滑块 vs 数值输入 vs 预设档位「均衡/技能优先/控预算」）与是否暴露更多目标（如「最少切换成本」「成长机会」）待第 5/7 章细化。

6. **Kanban / Gantt / 日历 三视角**
   - Kanban：按任务状态/按资源/按项目分组拖拽。
   - Gantt：时间轴上的任务条 + 资源泳道，直观展示并行与依赖。
   - 日历：按日/周/月查看资源占用与产能。

7. **报表导出**
   - 维度：按人 / 按团队 / 按项目。
   - 指标：已分配 PD、capacity、利用率、过载时段、技能缺口。
   - 格式：CSV / Markdown / Excel。

### 1.5 非目标（YAGNI / 明确不做）

| 不做项 | 原因 |
|---|---|
| 薪资 / 成本 / 绩效核算 | 超出人力资源配置范畴，属 HR 系统职责 |
| 考勤打卡 / 工时填报审批 | workload 由分配计划推导，非打卡累加；避免与现有 OA 冲突 |
| 多端实时协同 / 多人同时编辑 | 单用户本地桌面应用，无服务器，无需 OT/CRDT |
| 代码托管 / Issue / CI 集成 | 不替代 GitHub/Jira，可后续以只读导入方式对接，MVP 不做 |
| 自动化日报/周报推送 / 邮件通知 | 非核心决策能力 |
| 移动端 / Web 端 | Tauri 桌面优先 |
| 基于真实代码提交量反推工时 | 引入噪声大，与「计划导向」定位冲突 |

### 1.6 成功标准 / 验收信号

**MVP 规模上限（硬性验收边界）**：以下规模为本产品 MVP 的验收基线，超出该规模的场景不纳入 MVP 验收范围，需在后续版本验证。

| 维度 | MVP 上限 | 说明 |
|---|---|---|
| 资源数（Resource） | ≤ **10** | AI 优化、workload 重算、三视图重渲染的性能基线均按此规模量化 |
| 任务数（Task） | ≤ **50** | 含已分配 + 待分配任务总量 |
| 项目数（Project） | ≤ **5** | 多项目聚合与报表维度按此规模验收 |

| 类别 | 验收信号 |
|---|---|
| **规模边界** | 在 **资源 ≤ 10 / 任务 ≤ 50 / 项目 ≤ 5** 的标准测试集下完成下列全部验收；超出该上限的规模不构成 MVP 阻塞项 |
| **功能正确性** | 给定上述标准测试集，AI 产出的分配方案满足全部硬约束（无过载、无时间冲突、依赖有序），通过断言测试 |
| **多目标可调** | 调整 AI 权重（均衡负载 / 技能最优 / 预算）后，输出方案与权重意图一致，且不同权重组合产出可区分的差异方案 |
| **单位多级覆盖** | 团队级覆盖 `hours_per_pd` / `pd_per_pm` 后，该团队视图的 PM 展示与 capacity 换算正确生效，未覆盖团队维持全局值 |
| **阈值分级** | 角色/团队级利用率阈值覆盖后，红绿灯按覆盖值判定，未覆盖对象回退全局默认 |
| **可复现性** | 同一输入 + seed 重复运行 10 次，输出字节级一致 |
| **性能** | 上述 MVP 规模下，AI 优化端到端 ≤ 5s（本地 Ollama）；三视图重渲染 ≤ 500ms |
| **离线可用** | 拔网后，除云端 LLM 外全部功能正常；切到本地 Ollama 后 AI 优化仍可用 |
| **可解释性** | 用户调研中 ≥80% 的管理者能仅凭 AI 附带的自然语言理由理解「为何派给此人」 |
| **采用信号** | 试点团队连续 2 周每周打开 ≥3 次、至少导出 1 次报表 |

### 1.7 高层用例列表

| 编号 | 角色 (Actor) | 操作 (做什么) | 结果 (得到什么) |
|---|---|---|---|
| UC-01 | 研发管理者 | 新建项目并录入周期/优先级/人力预算 | 项目出现在项目列表，可被任务与分配引用 |
| UC-02 | PM | 在项目下创建任务，声明所需 skill（如 Rust L4+）与工作量 PD、时间窗 | 任务入库，AI 据此参与匹配 |
| UC-03 | Tech Lead | 维护资源档案：录入 skill 与熟练度（1–5）、tag、capacity/投入比例 | 资源可被 AI 用于技能匹配与容量计算 |
| UC-04 | PM | 选中一批未分配任务，点击「AI 一键分配」 | 得到满足硬约束的分配方案 + 每条建议的理由；可一键采纳或手动微调 |
| UC-04a | PM / 研发管理者 | 在 AI 面板调节优化目标权重（均衡负载 / 技能最优 / 预算）后重跑 | 得到与权重组合一致的差异化方案，权重随 run 记录可追溯 |
| UC-05 | 研发管理者 | 切换 PD / PM 计算单位 | 全系统数值同步换算，存储口径（PD）不变 |
| UC-05a | 研发管理者 | 为某团队 / 角色覆盖换算常数（`hours_per_pd` / `pd_per_pm`）或利用率阈值 | 该团队/角色视图按覆盖值换算与判定，未覆盖对象维持全局默认 |
| UC-06 | 研发管理者 | 打开「团队 workload」看板 | 看到每个资源/团队在选定时间窗的 workload、capacity、利用率（红绿灯按角色/团队阈值判定） |
| UC-07 | PM | 在 Gantt 视图拖拽调整任务时间/投入比例 | 实时检测冲突与过载并预警；workload 即时更新 |
| UC-08 | 研发管理者 | 执行 What-if：临时禁用某资源（如请假）后重跑 AI | 得到重分配建议方案（不立即生效），用于决策对比 |
| UC-09 | 研发管理者 | 导出「按项目人力配置」报表 | 生成 CSV/Markdown/Excel，含分配 PD、利用率、技能缺口 |
| UC-10 | 开发者（只读） | 查看个人 Kanban / 日历 | 看到自己的任务、时间安排与负载（无编辑、无打卡） |
| UC-11 | PM | 标记长期任务并分段排期（多 Allocation 段） | 任务在 Gantt 上显示为多个分段，各段独立计算 workload |
| UC-12 | Tech Lead | 锁定/排除某资源参与某任务 | AI 优化将该约束作为硬约束遵守，不出现在建议中 |

---

> 本节定义的产品边界、目标与用例，是后续数据模型（schema）、AI 引擎（ai-engine）、IPC 命令与前端视图设计的输入基线。

### 1.8 假设（本节）

1. 产品正式命名为 **Development Resource Kanban**（全称），简称 **DevResource Kanban**；对外文档用全称、UI/日志/代码可用简称，不并存第三种写法。
2. 默认工时换算常数为 `1 PD = 8h`、`1 PM = 20 PD`；二者均支持「全局 → 团队/组织」多级覆盖，覆盖优先级为团队级 > 全局。
3. 利用率红绿灯阈值默认 `70% / 100%`（闲置/健康/过载分界）；支持按角色 / 团队分别设定，优先级为角色级 > 团队级 > 全局默认，未设定者回退全局。
4. AI 优化在 **资源 ≤ 10 / 任务 ≤ 50 / 项目 ≤ 5** 规模下端到端 ≤ 5s（本地 Ollama）为 MVP 性能基线；该规模同时作为 MVP 验收的硬性边界，超出规模的场景不纳入 MVP 验收。
5. 存储口径统一为 PD，PM 仅作展示换算；若用户存在双口径存储需求需重新评估。
6. 开发者视图为只读、不提供工时填报入口；若后续需要打卡回填实际工时需扩展范围。
7. 报表导出格式 MVP 至少支持 CSV 与 Markdown，Excel 作为进阶。
8. AI 优化目标函数为「均衡负载 / 技能最优 / 预算」等目标的加权综合，权重由用户在 UI 调节（默认均衡权重 0.4/0.4/0.2），未调节时使用默认；切换权重重跑视为新 run。

### 1.9 开放问题（本节）

1. **[impl 期决策]** 团队级 / 角色级换算常数与利用率阈值的存储形式（独立配置表 vs `settings.metadata` JSON）与 UI 落点，待第 3 章（schema）与第 7 章（前端设置页）落地时确认。
2. **[impl 期决策]** AI 多目标权重的 UI 控件形态（滑块 vs 数值输入 vs 预设档位「均衡/技能优先/控预算」）与是否暴露更多目标（如「最少切换成本」「成长机会」），待第 5 章（AI 引擎）与第 7 章（优化面板）落地时确认。
3. **[impl 期决策]** 权重组合与利用率阈值覆盖是否随 `ai_optimization_runs` / 配置变更记入审计，便于方案可追溯——待第 3/5 章确认字段去向。

---

## 2. 系统架构 / System Architecture

### 2.1 分层概览

系统为「单进程、单用户本地桌面应用」，但在进程内部严格分层，使领域逻辑与桌面外壳、持久化、AI 引擎解耦。整体由五层构成：

| 层 | 实现 | 职责 | 依赖方向 |
|---|---|---|---|
| **表现层 (Presentation)** | Vue 3 + Pinia + Vite，运行在 WebView | Kanban / Gantt / 日历 / 报表 UI、本地交互状态、单位切换 | → 仅依赖 IPC 客户端 |
| **IPC 边界 (Tauri Command Layer)** | `#[tauri::command]` + JSON 序列化 | 命令路由、DTO ↔ 领域模型转换、错误映射、长任务进度回传 | → 调用服务层 |
| **核心服务层 (Core Service Layer)** | Rust，基于 `tokio` 的 async service | 用例编排（事务边界、权限/校验、跨模块协调）、向 UI 回传进度 | → 调用 domain + persistence + ai-engine |
| **领域模块层 (Domain Modules)** | Rust，纯逻辑 + 实体（**零 `serde` 依赖**） | resource/team/project/task/allocation/calendar/skill/tag/optimization/report 的领域规则与不变量 | ← 被 service 依赖，不反向依赖外壳 |
| **基础设施层 (Infrastructure)** | sqlx/SQLite(WAL)、rig、good_lp | 持久化、LLM/embedding 抽象、经典优化器 | ← 被 domain(接口) 与 service 调用 |

设计原则：**依赖单向向内**。domain 只定义 trait（如 `AllocationRepository`、`SkillMatcher`、`Explainer`），persistence 与 ai-engine 提供实现并在启动期注入。这样 domain 可脱离 Tauri 与 SQLite 单测。**领域层保持纯净：不依赖 `serde`、`DomainError` 不实现 `Serialize`**（见 §2.5 错误模型），所有序列化职责集中在 IPC 边界。

### 2.2 进程内分层与数据流（文字图）

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Tauri 主进程 (单个 OS 进程，tokio 多线程运行时)                              │
│                                                                          │
│  ┌─────────────────────┐   invoke('cmd', payload)     ┌────────────────┐ │
│  │  WebView (前端)      │ ───────────────────────────▶ │  IPC 命令层     │ │
│  │  Vue3 + Pinia        │ ◀─────────────── JSON / Chan │  tauri::command │ │
│  │  Kanban/Gantt/Cal    │   (结果 / 错误 / 进度事件)     │  + AppHandle    │ │
│  └─────────────────────┘                              └───────┬────────┘ │
│                                                              │          │
│                                              Service 调用     │          │
│                                                              ▼          │
│  ┌──────────────────────────────────────────────────────────────────┐    │
│  │                    Core Service Layer (async)                     │    │
│  │  ResourceService · ProjectService · AllocationService             │    │
│  │  OptimizationService · ReportService · CalendarService            │    │
│  │  (事务边界 / 校验 / 编排；持有 tokio::sync::RwLock 包裹的缓存)        │    │
│  └───────┬───────────────────────┬──────────────────────┬──────────┘    │
│          │                       │                      │               │
│          ▼                       ▼                      ▼               │
│  ┌──────────────┐      ┌─────────────────┐     ┌──────────────────┐    │
│  │  Domain 层    │      │  Persistence     │     │  AI Engine        │    │
│  │  (纯逻辑+trait)│      │  sqlx + SQLite   │     │  rig + good_lp    │    │
│  │  实体/不变量    │      │  WAL · migrate   │     │  Ollama(本地默认)  │    │
│  │  零 serde     │      │                  │     │  + 可选云 provider │    │
│  └──────────────┘      └─────────────────┘     │  经典优化器(ILP/贪心)│    │
│         ▲                        ▲              │  embedding/语义匹配 │    │
│         │ impl trait              │ sqlx::query  │  解释器: LLM/规则降级│   │
│         └────────────────────────┴─────────────▶└──────────────────┘    │
│                                                                  ▲       │
│  长/耗时任务（优化求解、批量重排、报表导出）经 tokio::spawn 派发到后台；     │       │
│  CPU 密集的 MILP 求解用 spawn_blocking 隔离，不另起 OS 进程；                │       │
│  进度通过 tauri::ipc::Channel 或 app.emit("progress", ...) 回传。 ─────┘       │
└──────────────────────────────────────────────────────────────────────────┘
                          ↑ 单文件本地存储 (SQLite, WAL)
                          ↑ 可选外部进程：Ollama (HTTP, localhost:11434)
```

> **不另起独立 OS 进程（决策）：** 优化求解（贪心 / 匈牙利 / MILP）始终保持在 tokio 运行时内完成——CPU 密集的 MILP 求解用 `tokio::task::spawn_blocking`（或专用阻塞线程池）在运行时内部隔离，**不下放到独立 OS 进程**。单进程本地应用无需跨进程 IPC 与子进程生命周期管理；`spawn_blocking` 已能避免饿死 IPC 命令槽，配合 `CancellationToken` / `tokio::time::timeout` 即可中止/超时。唯一可选的外部进程是 Ollama 的 HTTP 服务（LLM 推理，非优化求解）。

典型数据流示例（“运行一次 AI 优化”）：

1. 前端 `invoke('run_optimization', { project_ids, unit: 'PD', horizon })`。
2. IPC 层反序列化为 `RunOptimizationReq`（DTO），调用 `OptimizationService::run(...)`。
3. service 从 persistence 拉取 tasks/resources/skills → 组装 ILP 模型 → 调 ai-engine：经典器解硬约束，LLM 打 tag↔skill 语义分并产出解释（LLM 不可用时降级到 `TemplateExplainer`，见 §2.8 降级链）。
4. 解在后台 `tokio::spawn` 中迭代，每步经 `Channel` 回推 `{stage, progress: 0..1}`；MILP 段进一步经 `spawn_blocking` 隔离。
5. service 将最终 `Allocation[]` 写库（事务），返回 `RunOptimizationResp`（DTO）。
6. IPC 序列化为 JSON，前端 Pinia 更新看板。

### 2.3 模块边界与职责

| 模块 (domain crate 内) | 核心实体 | 关键职责 | 不做 |
|---|---|---|---|
| `resource` | Resource | 开发者主数据、容量日历引用、所属 team | 不算 workload |
| `team` | Team | 资源聚合、跨项目成员关系 | 不持有 allocation |
| `project` | Project | 顶层容器、周期/优先级/人力预算、状态机 | 不直接排任务 |
| `task` | Task | 工作项、所需 skill、工作量(PD/PM)、时间窗、依赖(DAG)、分段(long-term) | 不求解分配 |
| `allocation` | Allocation | 资源×任务×区间×投入% 的绑定、利用率计算入口 | 不调 LLM |
| `calendar` | Calendar / CapacityDay | 工作日/节假日/请假、容量折算（受投入比例与单位 PM↔PD 转换） | 不存任务 |
| `skill` | Skill | 能力定义、熟练度等级、资源↔skill 关联 | 不打分 |
| `tag` | Tag | 资源/任务自由标签、多对多 | 不参与硬约束 |
| `optimization` | OptimizationPlan / Solver | 混合求解（硬约束 ILP + 语义打分）、可复现 seed、约束校验 | 不直接写 UI |
| `report` | Report | workload/利用率/缺口 报表与导出(CSV/Excel/PDF) | 不改数据 |

跨模块规则：模块间通过 service 编排，禁止 domain 模块互相直接持有可变状态；`workload` 与 `utilization` 的计算归口在 `allocation` + `calendar` 协作（utilization = workload / capacity，单位统一为 PD 或 PM）。

### 2.4 为什么单用户本地仍用 tokio（优化在进程内运行）

虽无服务端，tokio 仍是必需：

1. **IO 并发不阻塞 UI**：sqlx 是 async；批量导入、跨多项目重排时并发查询互不阻塞，避免 WebView 卡顿。
2. **AI 调用天然长耗时**：Ollama 推理/embedding 可达数秒~数十秒；必须 detach 到运行时，否则 `#[tauri::command]` 会阻塞命令槽。
3. **经典求解器长任务**：ILP 在任务/资源规模大时（如 50 资源 × 200 任务）可能秒级以上，需后台执行 + 进度回传。CPU 密集的 MILP 段用 `spawn_blocking` 隔离到阻塞线程池（**不下放独立 OS 进程**，见 §2.2 决策）。
4. **统一的取消与超时**：`CancellationToken` / `tokio::time::timeout` 让用户可中止优化。
5. **Channel 友好**：tauri v2 的 `tauri::ipc::Channel<T>` 与 tokio 任务天然对接，流式回传进度/部分结果。

> 注：Tauri v2 命令默认在 `tauri::async_runtime`（基于 tokio）调度；标注为 `async fn` 的 command 会进入该运行时，CPU 密集的求解应再用 `tokio::task::spawn_blocking` 隔离，避免饿死 IPC。

### 2.5 IPC 边界设计原则

**命令粒度**：按“用例”而非“CRUD 表”切分。例如 `run_optimization`、`commit_allocations`、`recompute_workload`、`export_report`，而非暴露 30 个细粒度 setter。读路径可适度细，写路径强一致收敛在用例级。

**DTO 与领域模型分离**：IPC 层只接受/返回 DTO（`serde` + `Serialize`），domain 实体不实现 `Serialize`，强制经 mapper 转换，避免内部结构泄漏与版本耦合。

**错误映射（领域层零 serde 依赖，决策）：** domain 定义领域 `DomainError` 枚举（仅 `thiserror`，**不派生 `serde::Serialize`**——领域 crate 不依赖 `serde`，保持可脱离 Tauri/IPC 单测的纯净性）。IPC/service 层的 `AppError` **不以内嵌 `#[from] DomainError` 的方式携带领域错误**，而是在 mapper / 边界处把 `DomainError` 显式映射为前端可判别的结构化错误：参数取值/格式类 → `AppError::Validation`；业务不变量类（容量/技能/求解）→ `AppError::Domain`。这样 `AppError` 派生 `Serialize` 时不会反向要求 `DomainError` 派生 `Serialize`。与 §6.4 统一错误模型方向一致。

```rust
// domain/error.rs —— DomainError：领域错误，仅 thiserror，零 serde 依赖（领域 crate 不引入 serde）
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("容量不足: 资源 {resource_id} 在 {window:?} 缺口 {shortfall_pd} PD")]
    InsufficientCapacity { resource_id: i64, window: DateWindow, shortfall_pd: f64 },
    #[error("技能不匹配: 任务 {task_id} 缺 {missing:?}")]
    SkillMismatch { task_id: i64, missing: Vec<String> },
    #[error("无效投入比例: {0}")]
    InvalidRatio(f64),
    #[error("时间窗非法: start > end")]
    InvalidDateWindow,
    #[error("求解失败: {0}")]
    Solver(String),
}

// core-service/error.rs —— AppError：IPC 层统一错误，派生 Serialize；不内嵌 DomainError
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "code", content = "detail")]
pub enum AppError {
    #[error("validation failed: {0}")]
    #[serde(rename = "VALIDATION")]
    Validation(String),                   // 参数取值/格式校验失败（来自 DomainError 的 InvalidRatio/InvalidDateWindow 等）
    #[error("domain violation: {0}")]
    #[serde(rename = "DOMAIN_ERROR")]
    Domain(String),                       // 业务不变量违例（容量/技能/求解），来自 DomainError.to_string()
    #[error("optimization failed: {0}")]
    #[serde(rename = "OPTIMIZATION")]
    Optimization(String),
    #[error("database error: {0}")]
    #[serde(rename = "DB_ERROR")]
    Db(String),
    #[error("entity not found: {0}")]
    #[serde(rename = "NOT_FOUND")]
    NotFound(String),
    #[error("llm/embedding error: {0}")]
    #[serde(rename = "LLM")]
    Llm(String),
    #[error("config/secret error: {0}")]
    #[serde(rename = "CONFIG")]
    Config(String),
    #[error("io error: {0}")]
    #[serde(rename = "IO")]
    Io(String),
    #[error("internal error")]
    #[serde(rename = "INTERNAL")]
    Internal,                             // 兜底，不泄露内部细节
}

// ipc/mapper.rs —— 显式映射：DomainError → AppError（不依赖 #[from] 内嵌，故 DomainError 无需 Serialize）
impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        match e {
            // 取值/格式类 → Validation
            DomainError::InvalidRatio(_) | DomainError::InvalidDateWindow => {
                AppError::Validation(e.to_string())
            }
            // 领域语义类（容量/技能/求解）→ Domain
            DomainError::InsufficientCapacity { .. }
            | DomainError::SkillMismatch { .. }
            | DomainError::Solver(_) => AppError::Domain(e.to_string()),
        }
    }
}

#[tauri::command]
async fn run_optimization(req: RunOptimizationReq, state: State<'_, AppHandle>)
    -> Result<RunOptimizationResp, AppError> { /* ... */ }
```

> **与 §6.4 的对齐说明：** §6.4 当前写法为 `AppError::Domain(#[from] DomainError)` 内嵌方案（强制 `DomainError` 派生 `Serialize`，污染领域层）。本章按决策采用**领域层零 serde** 方案：`DomainError` 不派生 `Serialize`，`AppError` 仅以稳定字符串承载领域错误文本，mapper 负责分类映射。**以本章为准**——§6.4 的 `Domain(#[from] DomainError)` 内嵌写法需在实现期对齐到本方案（`Domain(String)` / `Validation(String)`），属 impl 期决策（见 §2.11）。

前端 `invoke` 捕获 `AppError` 后据 `error.code` 给出可操作提示（如“切换到 PM 重排”“补充技能标签”）；`DOMAIN_ERROR` / `VALIDATION` 的 `detail` 为稳定可展示文本。若某类领域错误需把结构化字段（如 `shortfall_pd`、`missing` 技能列表）透传前端做交互提示，由 mapper 在映射时构造 DTO 友好的 detail，而非让 `DomainError` 自身实现 `Serialize`。

**长任务进度**：用 `tauri::ipc::Channel<ProgressEvent>`（v2）而非轮询：

```rust
#[derive(Clone, serde::Serialize)]
#[serde(tag = "stage")]
pub enum ProgressEvent {
    Fetching { total_tasks: usize },
    Solving { iter: u32, progress: f64 },     // 0.0..1.0
    Scoring { done: usize, total: usize },     // LLM 语义打分（不可用时跳过/走规则）
    Explaining,                                // 生成方案解释（LLM 或 TemplateExplainer）
    Committing,
}
```

### 2.6 目录结构与 crates 划分

采用 Cargo workspace 多 crate，使 domain 可独立编译/测试，Tauri 外壳仅是薄组装。

```
devhr-kanban/                         # 仓库根
├─ Cargo.toml                         # [workspace]
├─ crates/
│  ├─ domain/                         # 纯领域：实体、值对象、trait、领域错误（零 serde）
│  │  ├─ src/
│  │  │  ├─ resource.rs  team.rs  project.rs  task.rs
│  │  │  ├─ allocation.rs  calendar.rs  skill.rs  tag.rs
│  │  │  ├─ optimization/             # 模型组装、约束定义、可复现 seed
│  │  │  ├─ report/                   # 报表口径计算
│  │  │  └─ lib.rs
│  │  └─ Cargo.toml                   # 无 Tauri/SQL/serde 依赖（DomainError 仅 thiserror）
│  ├─ persistence/                    # sqlx + SQLite 实现 domain 的 repo trait
│  │  ├─ migrations/                  # sqlx::migrate! 来源 (*.sql)
│  │  ├─ src/  repos/  connection.rs  (WAL, pragma)
│  │  └─ Cargo.toml                   # features = ["sqlite","runtime-tokio"]
│  ├─ ai-engine/                      # rig 抽象 + good_lp + 语义匹配 + 解释器
│  │  ├─ src/
│  │  │  ├─ provider.rs               # rig: Ollama 默认 + 云 provider 切换
│  │  │  ├─ embedding.rs              # skill/tag 向量化与缓存
│  │  │  ├─ matcher.rs                # tag↔skill 语义打分 (LLM，不可用走规则)
│  │  │  ├─ solver/                   # 经典优化: ilp.rs(good_lp+HiGHS) greedy.rs hungarian.rs
│  │  │  ├─ orchestrator.rs           # 混合策略编排 + 解释生成
│  │  │  └─ explain.rs                # 解释器: LlmExplainer + TemplateExplainer(规则降级)
│  │  └─ Cargo.toml                   # good_lp 显式 highs，见 §2.7
│  ├─ core-service/                   # 用例编排: 注入 repo + ai-engine；AppError + mapper
│  │  ├─ src/  services/  dto.rs  error.rs  state.rs
│  └─ tauri-app/                      # 桌面外壳 (lib.rs + main.rs)
│     ├─ src/
│     │  ├─ commands/                 # #[tauri::command], 按 module 分文件
│     │  ├─ ipc/                      # DTO <-> domain mapper, Channel 事件
│     │  ├─ setup.rs                  # 建 DB 连接、migrate、注入 state
│     │  └─ lib.rs
│     ├─ tauri.conf.json              # v2 配置
│     ├─ build.rs
│     └─ Cargo.toml                   # tauri = { version = "2", features=[...] }
├─ frontend/                          # Vite + Vue3 前端 (被 tauri-app 引为 webview 资源)
│  ├─ src/
│  │  ├─ views/   (Kanban.vue  Gantt.vue  Calendar.vue  Reports.vue)
│  │  ├─ components/  stores/  (Pinia)  composables/
│  │  ├─ api/     (invoke 封装 + 类型)
│  │  └─ main.ts
│  ├─ vite.config.ts  tsconfig.json  package.json
└─ docs/                              # 设计文档（本文件）
```

依赖关系图：`tauri-app → core-service → domain`；`core-service → persistence, ai-engine`；`persistence, ai-engine → domain`（实现 trait）。`frontend` 不参与 Cargo 编译，仅经 Tauri bundler 打包为静态资源。

> **领域 crate 零 serde**：`crates/domain/Cargo.toml` 不引入 `serde`；`DomainError` 仅 `#[derive(Debug, thiserror::Error)]`。`serde` 序列化职责落在 `core-service` / `tauri-app` 的 `AppError` 与 DTO 上（见 §2.5）。

### 2.7 构建 / 打包考量

**Tauri v2 bundler**：通过 `tauri build` 产出各平台安装包（macOS `.dmg`/`.app`、Windows `.msi`/`.exe`、Linux `.deb`/`.AppImage`）。`tauri.conf.json` 指定 `frontendDist`（指向 `frontend/dist`）、`bundle.targets`、应用元数据与签名。

**SQLite 静态链接**：推荐 `sqlx` 的 `sqlite` + `runtime-tokio` + `rustls`（避免 OpenSSL 动态依赖），配合 `bundled` 特性将 SQLite C 库静态编译进二进制，免去目标机器的系统库依赖：

```toml
# crates/persistence/Cargo.toml
[dependencies]
sqlx = { version = "0.8", default-features = false, features = [
  "runtime-tokio", "tls-rustls", "sqlite", "macros", "migrate", "chrono", "uuid"
] }
libsqlite3-sys = { version = "*", features = ["bundled"] }   # 静态链接 SQLite
```

启动期执行 `PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;`，并跑 `sqlx::migrate!("./migrations")`。

**HiGHS 静态链接（决策：二进制体积无上限 → 静态链接 HiGHS，不走子进程）：** 建模层用 **good_lp**，默认后端 **HiGHS**（高性能、开源、支持 LP/MILP）。HiGHS 经 `good_lp` 的 `highs` feature **静态链接进二进制**（`highs-sys` 随 Tauri 打包），求解在进程内完成（见 §2.4），**不**通过子进程调用外部 solver 二进制。本应用**对二进制体积不设上限**，因此静态链接 C++ 求解器引入的体积增量（`highs-sys` 约 +10–20 MB）视为可接受成本，换取单一可执行文件分发、零运行时外部依赖、确定的可分发性与跨平台一致性、求解器版本与二进制强绑定（利于可复现性）。

**good_lp Cargo 配置（与 §5.5.2 完全对齐）：** 关闭默认 feature，仅启用 `highs`。原因：good_lp 的 `default_solver` 常量是 `coin_cbc::coin_cbc` 的别名（见 docs.rs `pub use solvers::coin_cbc::coin_cbc as default_solver;`），由 `coin_cbc` / `default-solver` feature 提供。一旦 `default-features = false`，`default_solver` 符号不存在（编译失败）；若保留默认 feature 则拉回 `coin_cbc`，需系统级 C 库，正是桌面分发要避免的。因此**必须用显式求解器实例 `good_lp::highs::highs`**（见 §5.5.3），不要用 `default_solver`。`clarabel` 后端不支持整数变量，不可用于本场景。

```toml
# crates/ai-engine/Cargo.toml —— 与 §5.5.2 完全对齐（已删除不一致表述）
[dependencies]
good_lp = { version = "1", default-features = false, features = ["highs"] }
# 禁用 default-solver/coin_cbc（需系统级 C 库），仅静态链接 highs。
```

> **打包说明：** 目标机无需预装 HiGHS / coin_cbc / SQLite / OpenSSL 等任何系统库；构建期需 C/C++ 工具链（macOS Xcode CLT / Windows MSVC / Linux gcc + g++）以静态编译 `highs-sys`（经 `cc` 构建脚本）与 `libsqlite3-sys`。CI 应固定工具链版本以保证可复现。

**Ollama 依赖说明**：Ollama 为**可选外部进程**（默认 `http://localhost:11434`），不打包进应用。首次使用需用户本地安装并 `ollama pull <model>`。`ai-engine` 在启动时探测连通性：可用则默认走本地，不可用且配置了云 provider/key 则降级到云端，**本地与云均不可用时降级到基于规则的自然语言模板 `TemplateExplainer`（见 §2.8）**，经典优化与手工分配不受影响。配置（provider/model/key/单位偏好 `PD|PM`、`1 PM = N PD`，N 固定为 20，见 §2.9）存于 SQLite 的 `app_config` 表或本地配置文件。

**前端 UI 库推荐**：综合轻量、Vue3 支持与组件覆盖，推荐 **Naive UI**（TS 友好、按需引入、无强主题包袱）。Gantt 推荐 **自绘 SVG + 轻交互**（dhtmlx/frappe 授权与定制成本较高，长期任务分段排期自绘更可控）；日历用 **vue-cal**；Kanban 自绘（列=状态，卡片=task/allocation）即可。

**交叉编译/体积**：二进制体积**无上限约束**（决策）。Rust 侧静态链接 SQLite + HiGHS 后二进制体积通常 30–55 MB（含 HiGHS C++ 求解器静态库），属预期且可接受；前端 dist 经 Vite 产物内联。优先保证零外部依赖与可分发性。macOS 需处理 arm64/x86_64 双架构与公证（notarization），Windows 需注意 WebView2 运行时依赖（Win11 自带，老系统提示安装）。构建机需具备 C/C++ 工具链（`highs-sys` 的 `cc` 构建脚本依赖）。

**可复现性**：优化结果受 LLM 随机性影响，需将 LLM 调用设 `temperature=0` 并缓存 embedding；经典器为确定性。每个 `OptimizationPlan` 落库时记录 `solver_version`、`model_hash`、`seed`、`provider/model`，便于事后复现与对比。

### 2.8 AI 功能降级链（架构层面）

本章从架构层面声明 AI 功能的降级链（具体 trait 与模板规则见 §5.6）：

1. **本地 Ollama 可用** → 走本地 LLM（embedding + chat 解释，默认实现 `LlmExplainer`）。
2. **本地不可用、配置了云 provider/key** → 降级到云端 LLM。
3. **本地与云均不可用** → 经典优化（贪心 / 匈牙利 / MILP）照常执行；tag↔skill 语义打分退化为规则/词法匹配（`FallbackScorer`）；方案解释退化为 **基于规则的自然语言模板 `TemplateExplainer`**——按「匹配度 + 时间窗余量 + 容量」等可计算指标套用模板生成解释（如「资源 A 的 skills 与 task#12 匹配度最高(0.95)，且在 6/10–6/20 时间窗内余量充足(8 PD)，故优先分配」），**零 LLM 依赖、永远可用**。

架构上 `ai-engine` 的 `Explainer` trait 同时存在 `LlmExplainer` 与 `TemplateExplainer` 两个实现，由 `orchestrator` 在启动期按 provider 探测结果注入相应实现（或运行期在 LLM 失败时回退）。即：经典优化硬约束求解永远可用；LLM 仅影响语义打分精度与解释文本质量，不影响硬约束解的产出。

### 2.9 单位换算 N 值（固定，不做地区动态计算）

PM↔PD 的换算常数 **`1 PM = N PD`，N 固定取 20，且不做按地区工作日的动态计算**（决策）。`N` 是一个全局配置常量（默认 20，可在 `app_config` 整体覆盖），**不**根据资源所属地区的工作日数量推导（不区分「按 20 工作日折算」还是「按 22 工作日折算」）。理由：本系统为单一时区 / 单一工作日历地区（见 §1.5 非目标与假设 #24），跨时区 / 跨地区日历属非目标；引入地区动态 N 会波及 §4.3 / §4.4 全链路口径与 workload_cache 迁移，不值得。

> 单位换算仅在展示层进行：存储口径统一为 PD，PM 视图按 `pd / N`（N=20）显示。`day_factor`（逐日容量折算，受节假日/工作周模板影响）与 `N`（PM↔PD 折算）是两个独立概念，不可混淆：前者按日、按资源解析；后者是全局常量。

### 2.10 本章假设

1. §2  单进程运行：整个应用为单 OS 进程（Tauri 主进程 + tokio 多线程运行时）；唯一可选外部进程是 Ollama HTTP 服务，优化求解本身不下放到独立 OS 进程。
2. §2  领域层零 serde：`domain` crate 不依赖 `serde`、`DomainError` 不派生 `Serialize`；序列化与错误映射职责集中在 IPC / 服务层的 `AppError`。
3. §2  二进制体积无上限：静态链接 HiGHS（`highs-sys`）与 SQLite（`libsqlite3-sys bundled`）造成的体积增量可接受；优先零运行时外部依赖。
4. §2  求解器后端固定为 HiGHS：good_lp 启用 `highs` feature、关闭默认 feature，显式使用 `good_lp::highs::highs`，不引入 `coin_cbc` / `default_solver` / `clarabel`。
5. §2  SQLite 经 `libsqlite3-sys` 的 `bundled` 特性静态链接，无需目标机系统库。
6. §2  单位换算固定 N=20：`1 PM = 20 PD`，不按资源 / 地区动态计算（与 §4.2.1 单一时区 / 单一工作日历地区假设一致）。
7. §2  LLM 为可选增强：LLM 调用默认 `temperature=0` 以保证可复现；LLM 不可用时 `TemplateExplainer` 提供规则化解释，经典优化硬约束求解永远可用、不依赖 LLM。
8. §2  AppConfig（provider/model/key/单位 PM↔PD，N=20）存于 SQLite `app_config` 表。
9. §2  前端 UI 库默认推荐 Naive UI；Gantt 默认自绘 SVG；日历用 vue-cal。
10. §2  构建期 C/C++ 工具链可用：静态编译 HiGHS / SQLite 需要目标平台 C/C++ 编译器；CI 固定工具链版本以保证可复现。

### 2.11 本章开放问题

1. §2（impl 期决策）§6.4 AppError 对齐：§6.4 当前写法为 `AppError::Domain(#[from] DomainError)` 内嵌（要求 `DomainError` 派生 `Serialize`），与本章「领域层零 serde、`DomainError → AppError::Domain/Validation` 文本映射」方案不一致。需在实现期将 §6.4 对齐到本章方案（以本章为准）：`AppError` 的 `Domain` / `Validation` 变体改为承载稳定字符串，mapper 负责 `DomainError` 的分类映射。
2. §2（impl 期决策）`TemplateExplainer` 的模板库覆盖度与多语言（i18n）文案：MVP 先内置中文模板；英文 / 其他语言模板是否随 vue-i18n 接入而定，impl 期决定补全范围与模板资源（放 `ai-engine/explain.rs` 还是 `frontend` 本地化文件）。
3. §2（impl 期决策）`DomainError → AppError` 映射时，哪些领域错误需要把结构化字段（如 `shortfall_pd`、`missing` 技能列表）以 DTO 友好的 detail 透传前端做交互提示，哪些仅需 `to_string()` 文本——impl 期据前端实际交互需求逐类确认。
4. §2（impl 期决策）HiGHS 静态链接在各目标平台（macOS arm64 / x86_64、Windows MSVC、Linux glibc）的构建链与产物体积实测，确认 `cc` / C++ 工具链在 CI 构建机就绪。

## 3. 数据模型 / Data Model (SQLite)

本节定义本地 SQLite 单文件数据库的完整 schema、长期任务与跨项目复用的建模方式、关键聚合查询，以及单用户场景下的并发写入与迁移策略。

> **日历模型单一真相源（决议）**：本系统的「工作日 / 节假日 / 请假」日历统一采用 §4.2 的**三表模型**（`work_week_template` + `holiday` + `time_off`），并在此节正式收录其 DDL。`settings.workweek_mask` 字段与历史草案中的 `resource_unavailable` 单表模型**均已被取代、不再使用**（见 §3.3.1 与 §3.3.9 的取代说明）。所有 capacity/workload 计算的权威口径以 §4.2 ~ §4.4 的公式与 §4.9 的 Rust 计算核心为准；§3.6 的聚合 SQL 仅为概念示意，不可作为可执行的正确实现。

### 3.1 设计原则与通用约定

| 原则 | 说明 |
|---|---|
| **单一真相源** | 一切持久化状态进 SQLite；本地配置（provider/key/单位偏好）也落库到 `settings` 表，避免配置文件与 DB 双源漂移。日历模型也遵循此原则：`work_week_template` / `holiday` / `time_off` 三表是工作日/节假日/请假的唯一来源（见 §3.3.9），`settings` 中不再存工作周掩码。 |
| **软删除** | 业务实体统一用 `deleted_at TEXT`（ISO8601，NULL=未删除）；所有查询默认带 `WHERE deleted_at IS NULL`，便于撤销误删与审计。 |
| **时间戳** | `created_at` / `updated_at` 统一 `TEXT` 存储 ISO8601 UTC（如 `2026-06-27T08:30:00Z`），应用层格式化。SQLite 无原生 TIMESTAMPTZ。 |
| **主键** | 全局用 `INTEGER PRIMARY KEY AUTOINCREMENT`（rowid 别名），保证跨表引用稳定且可被外部日志追溯。 |
| **日期/时间窗** | 日期用 `TEXT`（`YYYY-MM-DD`，与 `DATE` 兼容）；时间窗 `start_date`/`end_date` 均含端点（闭区间 `[start, end]`）。`time_off` / `holiday` 采用**单日粒度**（`day` 列），而非闭区间，便于按日 fraction 折算。 |
| **外键** | `PRAGMA foreign_keys = ON`（连接时强制开启，见 3.7）。 |
| **单位** | schema 内部**只存 PD（人日）**；PM 仅是展示换算（`PM = PD / N`，N 来自 `settings.pm_workdays` 或团队级覆盖 `team_overrides.pm_workdays`）。预算字段存 PD，避免单位切换引发数据不一致。**任何 `budget_pm` 列表述均已废弃**（见 §3.3.10）。 |

**容量/工作量单位口径（贯穿 §3/§4/§5）**：

- **投入比例（percent）**：allocation 的 `percent ∈ (0,1]` 表「该资源在该区间对该任务的 commitment」；容量上限**始终以比例为单位**：任一资源任一有效工作日的 `Σ percent ≤ 1.0`（见 §3.8 硬约束 1）。
- **`daily_capacity_pd`**：表「该资源一个标准工作日折合多少 PD」，**默认 1.0**，仅用于：(a) 把比例折算成 PD 做展示与跨资源加总（`pd = Σ(percent × daily_capacity_pd × day_factor)`）；(b) 表达「兼职资源」的整体产出基准。它**不**作为过载检测的阈值（阈值恒为 1.0）。
- 这样过载检测与求解器约束统一在「比例」空间，避免「比例之和 vs PD 值」的单位混用。

**连接初始化（每次获取连接时执行）：**

```sql
PRAGMA journal_mode = WAL;       -- 读写并发
PRAGMA synchronous = NORMAL;     -- WAL 下安全且更快
PRAGMA foreign_keys = ON;        -- 默认关闭，必须显式开
PRAGMA busy_timeout = 5000;      -- 5s，配合 BEGIN IMMEDIATE
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;    -- 256MB 内存映射
```

> WAL 必须在事务外设置（sqlx 的 migration 默认包在事务里，故 WAL 在应用启动的连接初始化阶段执行，不放进 `.sql` 迁移文件）。

### 3.2 实体关系总览（ER 概览）

```
resources ──┬──< resource_tags >── tags
            ├──< resource_skills
            ├──< team_members >── teams ──< team_overrides (团队级常数/阈值覆盖)
            ├──< resource_project_rates (resource×project 维度费率)
            ├──< allocations >── tasks ──< task_tags >── tags
            └──< time_off (请假/不可用单日)

projects ──< tasks
projects ──< work_week_template (项目级工作周覆盖)
projects ──< holiday (项目级节假日)
tasks ──< task_skill_requirements
tasks ──(self-ref)── tasks (parent/child)
tasks ──< task_dependencies (predecessors)

ai_optimization_runs ──< allocations (run_id)
settings (单行配置表，含 secret_store)
```

> 工作日历的三个层级：`work_week_template`（L1 工作周模板，全局/项目级）、`holiday`（L2 节假日，全局/项目级，单日 fraction）、`time_off`（L3 资源请假，单日 fraction）。三者叠加即得 `day_factor(d, resource)`（见 §4.2）。

### 3.3 完整 DDL

> 以下 DDL 按迁移顺序排列（被依赖表在前）。建议每张表一个迁移文件，文件名遵循 sqlx 约定 `YYYYMMDDHHMMSS_<name>.sql`。

#### 3.3.1 settings（全局配置，单行表）

```sql
CREATE TABLE settings (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),  -- 强制单行
    -- 单位与工时
    default_unit        TEXT    NOT NULL DEFAULT 'PD'  CHECK (default_unit IN ('PD','PM')),
    pd_hours            REAL    NOT NULL DEFAULT 8.0    CHECK (pd_hours > 0),  -- 1 PD = 8h
    pm_workdays         REAL    NOT NULL DEFAULT 20.0   CHECK (pm_workdays > 0), -- 1 PM = 20 PD
    -- AI provider
    ai_provider         TEXT    NOT NULL DEFAULT 'ollama',  -- ollama | openai | anthropic | ...
    ai_base_url         TEXT,        -- 如 http://localhost:11434
    ai_api_key_enc      TEXT,        -- 加密存储（应用层 AES-GCM，存密文）
    -- 密钥存储后端：标识 ai_api_key_enc 的存储载体，决定 keychain 是否可用与降级路径（见 §6）
    secret_store        TEXT    NOT NULL DEFAULT 'keychain'
                        CHECK (secret_store IN ('keychain','encrypted_file')),
    ai_chat_model       TEXT    NOT NULL DEFAULT 'qwen2.5:7b',
    ai_embed_model      TEXT    NOT NULL DEFAULT 'nomic-embed-text',
    ai_embed_dim        INTEGER NOT NULL DEFAULT 768,
    -- 优化求解器
    solver_backend      TEXT    NOT NULL DEFAULT 'good_lp' CHECK (solver_backend IN ('good_lp','greedy','hungarian')),
    solver_timeout_ms   INTEGER NOT NULL DEFAULT 5000,
    -- 杂项
    locale              TEXT    NOT NULL DEFAULT 'zh-CN',
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
INSERT INTO settings (id) VALUES (1);  -- 初始化单行
```

> **已废弃字段**：历史草案中的 `workweek_mask TEXT`（7 位工作周掩码）已被删除。工作周的统一定义移至 `work_week_template` 表（见 §3.3.9a），因其支持项目级覆盖与逐日 fraction（如「周五半天」），表达力远超单值掩码。
>
> **`secret_store` 列（决议落地）**：标识密钥（`ai_api_key_enc`）的存储后端，取值 `'keychain'`（默认，走 OS keychain）或 `'encrypted_file'`（keychain 不可用时的降级，AES-256-GCM 加密落盘，主密钥由首次启动随机生成、机器绑定）。§6 的 settings_svc 读写此字段；与原假设 #41 / 开放问题 #29（已决议「新增 settings 列」）对齐。**不再使用 `settings.metadata` JSON 承载此值**，以保证该配置可被 SQL 直接索引/校验，并与 keychain 降级路径的开关语义一一对应。

#### 3.3.2 tags

```sql
CREATE TABLE tags (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,         -- 全局唯一，资源/任务共用
    color       TEXT,                            -- 可选，UI 展示
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_tags_name ON tags(name);
```

#### 3.3.3 skills（技能字典，可选但推荐）

```sql
CREATE TABLE skills (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,         -- 'Rust' / 'Frontend' / 'DevOps'
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_skills_name ON skills(name);
```

> 用独立 `skills` 字典而非自由文本，便于 AI 语义匹配时按 skill 检索 embedding、避免大小写/别名漂移。LLM 可在录入时把自由文本归一到字典。

#### 3.3.4 resources（开发者）

```sql
CREATE TABLE resources (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    name                TEXT    NOT NULL,
    email               TEXT,
    -- 可用性窗口（资源整体在职区间；NULL 表示不限）
    available_from      TEXT,   -- 'YYYY-MM-DD'
    available_to        TEXT,
    -- 状态
    status              TEXT    NOT NULL DEFAULT 'active'
                        CHECK (status IN ('active','inactive','archived')),
    -- 容量基准：每日可用 PD（多数人 = 1.0 PD = pd_hours；兼职可设 0.5）
    -- 仅用于把「投入比例」折算成 PD 做展示与跨资源加总；不参与过载阈值（阈值恒为 Σ percent ≤ 1.0）。
    daily_capacity_pd   REAL    NOT NULL DEFAULT 1.0 CHECK (daily_capacity_pd > 0),
    -- 人日单价（默认单价）：用于 R1/R7 成本估算；NULL = 不计成本。
    -- 注：本列为「同一资源的默认单价」；若需按项目/周期浮动单价，见 §3.3.17 resource_project_rates。
    daily_rate_pd       REAL,
    -- 资源级并行任务上限覆盖（NULL = 回落到项目级或全局默认，见 §3.8 硬约束 6）
    max_parallel_tasks_per_day INTEGER,
    metadata            TEXT,   -- JSON 扩展字段
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at          TEXT
);
CREATE INDEX idx_resources_status ON resources(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_resources_name ON resources(name);
```

> **字段说明与历史决策**：
> - 本表**不再**保留历史草案中的 `default_capacity` 字段。该字段（注释「该资源对全部分配的默认 commitment」）与本表的 `daily_capacity_pd` 在「兼职/部分投入」语义上重叠，而 §4.3 的 capacity 公式只引用 `daily_capacity_pd`，从不引用 `default_capacity`，二者并存会导致写入优先级与计算口径不明。已统一删除 `default_capacity`。
> - 新建 allocation 时 `percent` 的默认值取 **1.0**（即默认全职投入），不依赖资源级字段；若后续需要资源级默认投入比例，应命名为 `default_allocation_percent` 并明确**它不参与 capacity 计算**，仅作为 `allocation.percent` 的 UI 默认值。
> - `daily_rate_pd` 是该资源的**默认单价**，用于成本估算报表（`cost = allocated_pd × daily_rate_pd`）。NULL 表示该资源不计成本，报表相应单元格输出 `N/A`。**MVP 即补入 schema**（不再延后到 v1.2，使成本核算能力可前移到 MVP，对齐原开放问题 #39 的决议）；若有按项目/周期浮动的单价，落到 §3.3.17 的 `resource_project_rates` 覆盖本默认值。
> - `max_parallel_tasks_per_day`（可空）为资源级并行上限覆盖：非空时覆盖该资源在所有项目上的每日并行任务数上限；为空则回落到项目级（§3.3.10 `projects.max_parallel_tasks_per_day`）或全局 `ConstraintFlags` 默认值（见 §3.8 硬约束 6、§5）。对齐原开放问题 #26「按项目和资源配置」的决议。
> - 非均匀日容量（如「周一-周四全职、周五半天」）的周期性模板由 `work_week_template` 的项目级覆盖表达（见 §3.3.9a）；资源个人固定的非工作日则录 `time_off`。

#### 3.3.5 resource_skills（资源技能 + 熟练度）

```sql
CREATE TABLE resource_skills (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    skill_id        INTEGER NOT NULL REFERENCES skills(id)    ON DELETE CASCADE,
    -- 熟练度等级 1..5（1=了解,2=能用,3=熟练,4=精通,5=专家）
    proficiency     INTEGER NOT NULL CHECK (proficiency BETWEEN 1 AND 5),
    -- 可选：自评 vs 认证分；用于 AI 匹配加权
    evidence        TEXT,    -- JSON，如 {"cert":"AWS-SAA","years":3}
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, skill_id)
);
CREATE INDEX idx_resource_skills_skill ON resource_skills(skill_id);
```

#### 3.3.6 resource_tags

```sql
CREATE TABLE resource_tags (
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    tag_id      INTEGER NOT NULL REFERENCES tags(id)      ON DELETE CASCADE,
    PRIMARY KEY (resource_id, tag_id)
);
CREATE INDEX idx_resource_tags_tag ON resource_tags(tag_id);
```

#### 3.3.7 teams

```sql
CREATE TABLE teams (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL,
    description TEXT,
    created_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at  TEXT
);
CREATE UNIQUE INDEX idx_teams_name_active ON teams(name) WHERE deleted_at IS NULL;
```

#### 3.3.8 team_members（多对多）

```sql
CREATE TABLE team_members (
    team_id     INTEGER NOT NULL REFERENCES teams(id)     ON DELETE CASCADE,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    role        TEXT,    -- 'lead' | 'member' | ...
    joined_at   TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (team_id, resource_id)
);
CREATE INDEX idx_team_members_resource ON team_members(resource_id);
```

#### 3.3.8a team_overrides（团队级常数与阈值覆盖）

> **决议落地（响应开放问题 #2 / #3 / #46 / #47）**：PD/PM 换算常数（`pd_hours` / `pm_workdays`）与利用率阈值（红绿灯、过载、欠载）需要**按团队级**可配置，而非仅全局可配。新增 `team_overrides` 表承载这些团队级覆盖；未覆盖的团队回落到 `settings` 全局值。

```sql
CREATE TABLE team_overrides (
    team_id             INTEGER PRIMARY KEY REFERENCES teams(id) ON DELETE CASCADE,
    -- 团队级 PD/PM 常数覆盖（NULL = 回落到 settings 全局值）
    pd_hours            REAL    CHECK (pd_hours IS NULL OR pd_hours > 0),
    pm_workdays         REAL    CHECK (pm_workdays IS NULL OR pm_workdays > 0),
    -- 团队级利用率阈值覆盖（NULL = 回落到全局 settings 阈值；语义见 §4.5/§9）
    overload_threshold  REAL    CHECK (overload_threshold IS NULL OR overload_threshold > 0),
    underload_threshold REAL    CHECK (underload_threshold IS NULL OR underload_threshold >= 0),
    -- 红绿灯阈值（NULL = 回落全局）
    utilization_green   REAL    CHECK (utilization_green IS NULL OR (utilization_green >= 0 AND utilization_green <= 1.0)),
    utilization_yellow  REAL    CHECK (utilization_yellow IS NULL OR (utilization_yellow >= 0 AND utilization_yellow <= 1.0)),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
```

> **解析优先级（单位/阈值常量查询口径）**：
> ```
> effective_pd_hours(r)      = team_overrides.pd_hours      (若 r 所属 team 有覆盖)  else settings.pd_hours
> effective_pm_workdays(r)   = team_overrides.pm_workdays   (若有覆盖)              else settings.pm_workdays
> effective_overload(r)      = team_overrides.overload_threshold (若有覆盖)         else settings / 全局默认
> ```
> 资源所属 team 取 `team_members`；若资源归属多个 team，按 `role='lead'` 优先、否则取最近 joined 的 team 解析覆盖（impl 期决策：多 team 资源的具体归属选取规则）。
> §4 的 `UnitConfig` / 利用率计算在加载时按上式展开为「逐资源 effective 常数」，下游公式不变；§7 红绿灯、§8 报表的 PM 换算（`pd_to_pm = pd / effective_pm_workdays`）也读同一 effective 值。

#### 3.3.9 日历模型（工作周 / 节假日 / 请假）—— 三表，单一真相源

> **本节取代历史草案中的 `resource_unavailable` 单表模型**。`resource_unavailable`（字段 `scope`/`start_date`/`end_date`/`reason`/`deduction`，用闭区间和 `deduction` 表达节假日+请假）已被删除，原因是它无法表达项目级覆盖与工作周模板，且与 §4.2 的三表模型字段口径（`fraction` 而非 `deduction`、单日 `day` 而非闭区间、`project_id` 外键）互斥冲突。统一到三表后，§4.9 的 Rust `Calendar`、§3.6 的查询、§3.8 的约束全部读同一组表。

##### 3.3.9a work_week_template（L1 工作周模板）

```sql
CREATE TABLE work_week_template (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    scope       TEXT    NOT NULL CHECK (scope IN ('global','project')),
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,  -- NULL when scope='global'
    -- 逐日是否工作日（0/1）；支持「周五半天」用 fraction 字段细化（见下）
    mon         INTEGER NOT NULL DEFAULT 1 CHECK (mon IN (0,1)),
    tue         INTEGER NOT NULL DEFAULT 1 CHECK (tue IN (0,1)),
    wed         INTEGER NOT NULL DEFAULT 1 CHECK (wed IN (0,1)),
    thu         INTEGER NOT NULL DEFAULT 1 CHECK (thu IN (0,1)),
    fri         INTEGER NOT NULL DEFAULT 1 CHECK (fri IN (0,1)),
    sat         INTEGER NOT NULL DEFAULT 0 CHECK (sat IN (0,1)),
    sun         INTEGER NOT NULL DEFAULT 0 CHECK (sun IN (0,1)),
    -- 逐日 fraction（可选；缺省为对应 week 字段的 1.0）：
    -- 支持「周五固定半天」等周期性非均匀日容量（响应 §9.4 开放问题 #12）
    mon_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (mon_frac > 0 AND mon_frac <= 1.0),
    tue_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (tue_frac > 0 AND tue_frac <= 1.0),
    wed_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (wed_frac > 0 AND wed_frac <= 1.0),
    thu_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (thu_frac > 0 AND thu_frac <= 1.0),
    fri_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (fri_frac > 0 AND fri_frac <= 1.0),
    sat_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sat_frac > 0 AND sat_frac <= 1.0),
    sun_frac    REAL    NOT NULL DEFAULT 1.0 CHECK (sun_frac > 0 AND sun_frac <= 1.0),
    CHECK ((scope = 'global' AND project_id IS NULL) OR (scope = 'project' AND project_id IS NOT NULL))
);
-- 全局默认：周一-周五全天（含 fraction），种子 INSERT 见 §3.9 迁移（INSERT OR IGNORE）。
-- 全局唯一性约束（决议落地）：强制「scope='global' 全局仅一行」。
-- 旧版 idx_wwt_global 基于 id WHERE scope='global'（id 已是 PK，约束无意义），
-- 现改为基于常量 1 的 UNIQUE，真正保证全局行不可重复：
CREATE UNIQUE INDEX idx_wwt_global ON work_week_template((1)) WHERE scope='global';
CREATE UNIQUE INDEX idx_wwt_project ON work_week_template(project_id) WHERE scope='project';
```

> **全局唯一性约束变更（决议落地，响应开放问题 #15）**：原 `idx_wwt_global` 写成 `ON work_week_template(id) WHERE scope='global'`，因 `id` 本就是主键，该索引对「全局仅一行」毫无约束力（任何行 `id` 都互不相同）。现改为 `ON work_week_template((1)) WHERE scope='global'`——对 `scope='global'` 的所有行计算常量表达式 `1`，UNIQUE 即要求全局行至多一行，从而真正实现「全局仅一行」的 schema 层强约束。第二条索引 `idx_wwt_project` 同步改为 `UNIQUE`，保证每个 project 至多一条项目级模板。
>
> `day_factor(d, resource)` 解析顺序：若存在 `scope='project'` 且 `project_id = resource 所属项目的某 allocation` 对应的模板，用项目模板；否则回落到 `scope='global'` 模板。逐日工作日因子 = `is_workday(weekday) ? day_frac(weekday) : 0`（见 §4.2）。
>
> `*_frac` 列的存在使本表能表达「周期性非均匀日容量」（如某项目约定每周五为半天），从而把 §9.4 开放问题 #12 从「不支持」升级为「支持 via work_week_template 逐日 fraction」。

##### 3.3.9b holiday（L2 节假日）

```sql
CREATE TABLE holiday (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,  -- NULL = 全局
    day         TEXT    NOT NULL,            -- 'YYYY-MM-DD'（单日粒度）
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
                                            -- 1.0=全天节假日 / 0.5=半天
    name        TEXT,                        -- 节假日名称（如「国庆」）
    CHECK (length(day) = 10)
);
CREATE INDEX idx_holiday_day ON holiday(day);
CREATE INDEX idx_holiday_project_day ON holiday(project_id, day);
```

> `holiday` 按 `day` 单日存储而非闭区间，便于半天 fraction（`fraction=0.5`）。全局节假日（`project_id IS NULL`）作用于所有资源；项目级节假日（`project_id` 非空）覆盖该项目的 allocation。项目匹配规则：取 `day` 上「项目级行优先，否则全局行」的 `fraction`。

##### 3.3.9c time_off（L3 资源请假 / 调休）

```sql
CREATE TABLE time_off (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    day         TEXT    NOT NULL,            -- 'YYYY-MM-DD'（单日粒度）
    fraction    REAL    NOT NULL DEFAULT 1.0 CHECK (fraction > 0 AND fraction <= 1.0),
                                            -- 1.0=全天请假 / 0.5=半天请假
    reason      TEXT,                        -- 'leave'|'sick'|'other' 等
    note        TEXT,
    CHECK (length(day) = 10)
);
CREATE INDEX idx_time_off_res_day ON time_off(resource_id, day);
```

> 三层叠加得到资源 `r` 在日期 `d` 的日历因子（与 §4.2 公式一致）：
>
> ```
> day_factor(d, r, project):
>     base = work_week_template(scope='project', project_id=project) 优先，否则 global
>            的 weekday(d) 对应字段（0/1） × 该日 fraction
>     if holiday[d] exists (global or project, fraction=1.0)  → 0     (全天节假日)
>     if holiday[d] exists (half day, fraction=0.5)           → base × 0.5
>     if time_off[r][d] exists (full day)                     → 0
>     if time_off[r][d] exists (half day)                     → base × 0.5
>     otherwise                                                → base
> ```

#### 3.3.10 projects

```sql
CREATE TABLE projects (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    description     TEXT,
    start_date      TEXT,                       -- 项目周期起
    end_date        TEXT,                       -- 项目周期止
    priority        INTEGER NOT NULL DEFAULT 5  -- 1(最高)..9(最低)，用于 AI/求解器目标函数加权
                    CHECK (priority BETWEEN 1 AND 9),
    -- 人力预算（内部统一 PD；PM 仅展示换算）。R3 预算消耗报表统一读此列。
    -- 决议：本表只存 budget_pd；任何 budget_pm 列表述均已废弃（见下方说明）。
    budget_pd       REAL    NOT NULL DEFAULT 0 CHECK (budget_pd >= 0),
    -- 项目级并行任务上限覆盖（NULL = 回落到全局 ConstraintFlags 默认值，见 §3.8 硬约束 6）
    max_parallel_tasks_per_day INTEGER,
    status          TEXT    NOT NULL DEFAULT 'planning'
                    CHECK (status IN ('planning','active','on_hold','done','cancelled')),
    metadata        TEXT,                       -- JSON
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date)
);
CREATE INDEX idx_projects_status ON projects(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_projects_dates ON projects(start_date, end_date) WHERE deleted_at IS NULL;
```

> **预算字段统一为 `budget_pd`（决议强化）**：与「schema 只存 PD」原则一致，本表**只**有 `budget_pd`，**不存在也不新增任何 `budget_pm` 列**（历史草案与跨节引用中的 `budget_pm` 表述一律视为废弃）。§8.3 的 R3 预算消耗报表（`budget_usage = allocated_pd / budget_pd`）与本表对齐；原假设 #20 / #56 / 开放问题 #12 中残留的 `budget_pm` 歧义表述均已由本决议清理。展示层若需 PM，按 `budget_pm = budget_pd / effective_pm_workdays`（含团队级覆盖，见 §3.3.8a）在渲染时换算，不落库。
>
> **`max_parallel_tasks_per_day`（决议落地，响应开放问题 #26）**：项目级并行任务上限覆盖。非空时约束该项目下任一资源单日并行任务数 ≤ 该值；为空则回落到全局 `ConstraintFlags.max_parallel_tasks_per_day`。资源级覆盖见 `resources.max_parallel_tasks_per_day`（§3.3.4），三者解析优先级见 §3.8 硬约束 6。

#### 3.3.11 tasks

```sql
CREATE TABLE tasks (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id      INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    parent_task_id  INTEGER REFERENCES tasks(id) ON DELETE CASCADE,  -- 子任务/分段，NULL=顶层
    title           TEXT    NOT NULL,
    description     TEXT,
    -- 工作量估算（内部 PD）
    estimate_pd     REAL    NOT NULL DEFAULT 0 CHECK (estimate_pd >= 0),
    -- 时间窗（任务希望被执行的窗口；可与项目周期不同）
    start_date      TEXT,
    end_date        TEXT,
    -- 是否长期任务：影响 AI 分段排期策略
    is_long_term    INTEGER NOT NULL DEFAULT 0 CHECK (is_long_term IN (0,1)),
    -- 段类型（仅长期任务有意义）：milestone/phase/segment
    segment_kind    TEXT    CHECK (segment_kind IN ('milestone','phase','segment') OR segment_kind IS NULL),
    -- 依赖：用独立关系表 task_dependencies；此处 status 驱动工作流
    status          TEXT    NOT NULL DEFAULT 'todo'
                    CHECK (status IN ('todo','in_progress','blocked','review','done','cancelled')),
    -- 排序/看板列
    sort_order      INTEGER NOT NULL DEFAULT 0,
    metadata        TEXT,    -- JSON 扩展
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date IS NULL OR start_date IS NULL OR end_date >= start_date),
    -- 长期任务必须无 estimate（estimate 由各子段聚合），顶层段任务建议留空
    CHECK (NOT (is_long_term = 1 AND parent_task_id IS NOT NULL AND estimate_pd = 0 AND segment_kind = 'segment')
           OR estimate_pd >= 0)
);
CREATE INDEX idx_tasks_project ON tasks(project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_parent  ON tasks(parent_task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_status  ON tasks(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_dates   ON tasks(start_date, end_date) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_longterm ON tasks(is_long_term) WHERE is_long_term = 1 AND deleted_at IS NULL;
```

#### 3.3.12 task_dependencies（predecessors，有向无环依赖）

```sql
CREATE TABLE task_dependencies (
    task_id         INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,  -- 后继
    predecessor_id  INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,  -- 前置
    lag_days        INTEGER NOT NULL DEFAULT 0,   -- 依赖延迟（FS+lag）
    dep_type        TEXT    NOT NULL DEFAULT 'FS' CHECK (dep_type IN ('FS','FF','SS','SF')),
    PRIMARY KEY (task_id, predecessor_id),
    CHECK (task_id <> predecessor_id)
);
CREATE INDEX idx_deps_predecessor ON task_dependencies(predecessor_id);
-- 应用层在写入时做环检测（SQLite 递归 CTE 也可检测，但成本高，故应用层先行）
```

#### 3.3.13 task_skill_requirements（任务所需技能 + 最低熟练度）

```sql
CREATE TABLE task_skill_requirements (
    task_id             INTEGER NOT NULL REFERENCES tasks(id)  ON DELETE CASCADE,
    skill_id            INTEGER NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    min_proficiency     INTEGER NOT NULL CHECK (min_proficiency BETWEEN 1 AND 5),
    -- 是否硬约束：true=必须满足，false=软偏好（影响打分但不阻断）
    is_mandatory        INTEGER NOT NULL DEFAULT 1 CHECK (is_mandatory IN (0,1)),
    weight              REAL    NOT NULL DEFAULT 1.0 CHECK (weight >= 0), -- 软约束权重
    PRIMARY KEY (task_id, skill_id)
);
CREATE INDEX idx_task_req_skill ON task_skill_requirements(skill_id);
```

#### 3.3.14 task_tags

```sql
CREATE TABLE task_tags (
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id  INTEGER NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);
CREATE INDEX idx_task_tags_tag ON task_tags(tag_id);
```

#### 3.3.15 allocations（核心：资源↔任务↔时间窗↔投入比例）

```sql
CREATE TABLE allocations (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    task_id         INTEGER NOT NULL REFERENCES tasks(id)     ON DELETE CASCADE,
    -- 分配时间区间（闭区间）
    start_date      TEXT    NOT NULL,
    end_date        TEXT    NOT NULL,
    -- 投入比例 0.0~1.0（该资源在此区间对该任务的 commitment）
    -- 容量上限以「比例」为单位：任一有效工作日 Σ percent ≤ 1.0（见 §3.8 硬约束 1）
    percent         REAL    NOT NULL CHECK (percent > 0 AND percent <= 1.0),
    -- 分配的 PD 工作量（派生冗余字段：= 全程工作日数 × daily_capacity_pd × percent；
    -- 仅在该 allocation 区间全程被查询窗口完整覆盖时可直接 SUM，否则需 Rust 端按 overlap 重算）
    allocated_pd    REAL    NOT NULL DEFAULT 0 CHECK (allocated_pd >= 0),
    status          TEXT    NOT NULL DEFAULT 'planned'
                    CHECK (status IN ('planned','committed','in_progress','done','cancelled','locked')),
    -- 来源：manual=人工拖拽；ai=AI 优化产出（关联 run_id）
    source          TEXT    NOT NULL DEFAULT 'manual' CHECK (source IN ('manual','ai')),
    run_id          INTEGER REFERENCES ai_optimization_runs(id) ON DELETE SET NULL,
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    deleted_at      TEXT,
    CHECK (end_date >= start_date),
    -- AI 来源必须有 run_id
    CHECK (source <> 'ai' OR run_id IS NOT NULL)
);
-- 核心查询索引：按资源 + 日期范围聚合 workload
CREATE INDEX idx_alloc_resource_date ON allocations(resource_id, start_date, end_date)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_task ON allocations(task_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_alloc_run ON allocations(run_id) WHERE run_id IS NOT NULL;
CREATE INDEX idx_alloc_status ON allocations(status) WHERE deleted_at IS NULL;
```

> **`allocated_pd` 的语义与限制（重要）**：
> - `allocated_pd` 是**派生冗余字段**，写入时按**整条 allocation 区间**全程计算：`allocated_pd = 全程有效工作日数 × daily_capacity_pd × percent`。
> - 它**仅在查询窗口恰好完整覆盖该 allocation 全程**时可直接 `SUM`。当查询窗口只与 allocation 部分相交（跨周/跨月边界），直接 `SUM(a.allocated_pd)` 会把整条全量 PD 计入局部窗口，导致 workload **虚高、过载误报**。
> - **正确的窗口聚合必须按 overlap 天数 × percent × avg_day_factor 重算**（见 §4.4 公式 `alloc_pd(a,[start,end])` 与 §4.9 的 `alloc_pd()` Rust 实现）。该重算逻辑在 Rust 端的 `workload_pd()` 中完成，**无法用单一 SQL 冗余列表达**。
> - 因此 §3.6 中所有 `SUM(allocated_pd)` 的查询都标注为「概念示意」；真正落地的利用率/workload 聚合以 §4.9 的 Rust 计算核心为准，或通过 §4.7 的 `workload_snapshot` 物化快照表读取。

> **成本单价解析（决议落地）**：allocation 的成本以 `cost = allocated_pd × effective_daily_rate(resource, project)` 计算，其中 `effective_daily_rate` 解析顺序为：`resource_project_rates(resource, project)` 命中则取其费率，否则回落 `resources.daily_rate_pd`（§3.3.4）；二者皆 NULL 则该 allocation cost = `N/A`。见 §3.3.17。

#### 3.3.15a allocation 校验触发器（强制硬约束 2：时间窗落在 task/resource 交集内）

> §3.8 硬约束 2 要求 allocation 必须落在 task 的 `[start,end]` 与 resource 的 `[available_from,available_to]` 交集内。由于 SQLite `CHECK` 约束无法跨表引用，且「应用层先行」的口头承诺会被批量导入/AI apply/迁移脚本等旁路写入绕过（§9.4 开放问题 #14），此处用**数据库触发器**在 schema 层强制，保证所有写入路径都过校验。

```sql
-- AFTER INSERT 校验
CREATE TRIGGER trg_allocation_validate_insert
AFTER INSERT ON allocations
BEGIN
    -- 校验：allocation 区间必须落在所属 task 的时间窗内（若 task 有时间窗）
    SELECT RAISE(ABORT, 'allocation 越界：超出 task 时间窗')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    -- 校验：allocation 区间必须落在所属 resource 的可用区间内（若 resource 有区间）
    SELECT RAISE(ABORT, 'allocation 越界：超出 resource 可用区间')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    -- 校验：percent > 0 且 ≤ 1.0（与列 CHECK 重复，触发器兜底批量导入路径）
    SELECT RAISE(ABORT, 'allocation.percent 非法')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;

-- AFTER UPDATE 校验（同样校验被修改后的行）
CREATE TRIGGER trg_allocation_validate_update
AFTER UPDATE OF start_date, end_date, resource_id, task_id, percent ON allocations
BEGIN
    SELECT RAISE(ABORT, 'allocation 越界：超出 task 时间窗')
    FROM tasks t
    WHERE t.id = NEW.task_id
      AND t.start_date IS NOT NULL AND t.end_date IS NOT NULL
      AND (NEW.start_date < t.start_date OR NEW.end_date > t.end_date);
    SELECT RAISE(ABORT, 'allocation 越界：超出 resource 可用区间')
    FROM resources r
    WHERE r.id = NEW.resource_id
      AND r.available_from IS NOT NULL AND r.available_to IS NOT NULL
      AND (NEW.start_date < r.available_from OR NEW.end_date > r.available_to);
    SELECT RAISE(ABORT, 'allocation.percent 非法')
    WHERE NEW.percent <= 0 OR NEW.percent > 1.0;
END;
```

> 触发器作为 schema 层的兜底防线。**应用层仍须保证所有 allocation 写入经 `AllocationService::create` 单一入口**（见 §3.7 事务策略、§6.5 `with_tx` 封装），repo 层不暴露裸 insert，避免触发器频繁 ABORT 影响批量写入体验。两层防线互为冗余。

#### 3.3.16 ai_optimization_runs（优化运行记录，可复现）

```sql
CREATE TABLE ai_optimization_runs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 运行参数快照（可复现）
    seed                INTEGER NOT NULL,            -- 求解器随机种子
    objective           TEXT    NOT NULL DEFAULT 'balanced'
                        CHECK (objective IN ('balanced','min_makespan','max_utilization','fairness','skill_fit')),
    scope               TEXT    NOT NULL DEFAULT 'full'   -- 全量 / 增量
                        CHECK (scope IN ('full','incremental')),
    scope_project_ids   TEXT,    -- JSON [1,3,5]，本次优化的项目范围
    scope_from          TEXT,    -- 优化时间窗起
    scope_to            TEXT,
    -- 配置快照（约束开关、软目标权重、求解器配置；统一收敛在此表）
    config_json         TEXT NOT NULL,  -- 含 constraints_flags + weights + solver_config 的合并快照
    constraints_json    TEXT NOT NULL,  -- 硬约束快照（容量上限、不冲突、时间窗、最低熟练度）
    weights_json        TEXT NOT NULL,  -- 软约束权重快照
    -- 输入快照：参与求解的资源/任务/技能的序列化状态（保证可复现）
    input_snapshot_json TEXT NOT NULL,
    -- 输出：方案摘要 + 评分
    output_plan_json    TEXT,           -- 生成的 allocation 列表（亦落 allocations 表，run_id 关联）
    score_overall       REAL,           -- 综合评分 0..100
    score_skill_fit     REAL,
    score_utilization   REAL,
    score_fairness      REAL,
    -- LLM 解释（自然语言）
    explanation_md      TEXT,           -- LLM 生成的方案解释（Markdown）
    -- provider/model（可复现）
    provider            TEXT NOT NULL,  -- 'ollama' | 'openai' | ...
    chat_model          TEXT NOT NULL,
    embed_model         TEXT,
    solver_backend      TEXT NOT NULL,
    solver_status       TEXT NOT NULL   -- 'optimal' | 'feasible' | 'infeasible' | 'timeout' | 'error'
                        CHECK (solver_status IN ('optimal','feasible','infeasible','timeout','error')),
    -- 采纳状态（统一状态机：proposed→accepted/rejected）
    status              TEXT    NOT NULL DEFAULT 'proposed'
                        CHECK (status IN ('proposed','accepted','rejected')),
    applied             INTEGER NOT NULL DEFAULT 0 CHECK (applied IN (0,1)), -- 是否已写入 allocations
    started_at          TEXT NOT NULL,
    finished_at         TEXT,
    duration_ms         INTEGER,
    error_msg           TEXT,
    created_at          TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now'))
);
CREATE INDEX idx_runs_applied ON ai_optimization_runs(applied, created_at);
CREATE INDEX idx_runs_scope ON ai_optimization_runs(scope_from, scope_to);
CREATE INDEX idx_runs_status ON ai_optimization_runs(status, created_at);
```

> **表合并决议（确认，响应开放问题 #13）**：历史草案中 §5.7 另定义了一张 `ai_run(run_id TEXT PRIMARY KEY /*UUID*/)` 表，导致 `allocations.run_id` 在 §3 是 `INTEGER REFERENCES ai_optimization_runs(id)`、在 §5 是 `TEXT REFERENCES ai_run(run_id)`，外键类型与目标表冲突。现已**删除 §5.7 的 `ai_run` 表**，统一以本表 `ai_optimization_runs`（INTEGER PK）为优化运行的唯一表，并把 `ai_run` 独有的列（`config_json`、`scope`、`status`）并入本表。
>
> - `allocations.run_id` 类型保持 `INTEGER REFERENCES ai_optimization_runs(id)`，全链路一致。
> - `AllocationProblem.run_id`（§5.3 Rust 结构）由原 `Uuid` 改为 `i64`（落库后由 `ai_optimization_runs` 的 AUTOINCREMENT 回填），全链路一致。
> - §5.1/§5.3/§5.7/§5.8/§5.10 中所有对 `ai_run` 表与 `Uuid` 的引用，在实现时统一指向 `ai_optimization_runs` 与 `i64`。
> - 原开放问题 #25（若未来确需 UUID 作 run_id 的回改预案）仍保留为「未来选项」，但当前 INTEGER 自增口径已确认即最终口径，不再悬置。
>
> **可复现性**：`seed` + `constraints_json` + `weights_json` + `input_snapshot_json` + `provider/model` 五元组确定一次优化结果。重放时按相同种子与快照重跑 `good_lp`/贪心即可得到相同的硬约束解；LLM 解释带有非确定性，但解本身不依赖 LLM（LLM 仅做语义打分与解释）。

#### 3.3.17 resource_project_rates（resource×project 维度费率，决议落地）

> **响应开放问题 #38 / 假设 #61 的决议**：同一资源在不同项目/周期可有不同日单价（浮动费率）。原 §8「单一 `daily_rate_pd`」假设已升级为「默认单价 + 项目级覆盖」两层结构。新增 `resource_project_rates` 表承载 (resource, project) 维度的费率，可按周期浮动；`resources.daily_rate_pd` 保留为「全局默认单价」。

```sql
CREATE TABLE resource_project_rates (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    project_id      INTEGER NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
    -- 该资源在该项目的日单价（PD）；覆盖 resources.daily_rate_pd
    daily_rate_pd   REAL    NOT NULL CHECK (daily_rate_pd > 0),
    -- 可选：周期浮动（闭区间，均含端点）；NULL = 该 (resource,project) 长期有效
    valid_from      TEXT,   -- 'YYYY-MM-DD'
    valid_to        TEXT,   -- 'YYYY-MM-DD'
    note            TEXT,
    created_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    updated_at      TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (resource_id, project_id, valid_from),
    CHECK (valid_to IS NULL OR valid_from IS NULL OR valid_to >= valid_from)
);
CREATE INDEX idx_rpr_project ON resource_project_rates(project_id);
CREATE INDEX idx_rpr_res_date ON resource_project_rates(resource_id, valid_from, valid_to);
```

> **费率解析（`effective_daily_rate(resource, project, d)`）**：
> ```
> 1. 命中 resource_project_rates(resource, project) 且周期覆盖日期 d
>    （valid_from IS NULL 或 d >= valid_from，且 valid_to IS NULL 或 d <= valid_to）→ 取其 daily_rate_pd
> 2. 否则回落 resources.daily_rate_pd（§3.3.4 默认单价）
> 3. 二者皆 NULL → cost = N/A（该 allocation 不计成本）
> ```
> `valid_from` 为 NULL 表示「自资源入职/项目起有效」；`valid_to` 为 NULL 表示「长期有效」。同一 (resource, project) 可有多行（不同周期），但任意两行周期不得重叠（应用层在 `AllocationService`/rate service 写入时校验区间不相交，impl 期决策：是否用 SQLite 触发器或 `EXCLUDE` 类约束做硬性防重叠）。
>
> §8 R1/R2/R3/R7 的成本列（`include_cost=true`）一律走 `effective_daily_rate` 解析；`projects.budget_pd` 保持不变（预算字段与单价字段正交，不相互取代）。

### 3.4 长期任务的建模

长期任务采用**父任务 + 子段（segment/phase）**模型，而非单行宽区间：

```
长期任务 T (is_long_term=1, parent=NULL, estimate_pd 留空或为目标值)
 ├── 段 S1 (parent=T, segment_kind='phase',  start=06-01, end=06-20, estimate_pd=8)
 ├── 段 S2 (parent=T, segment_kind='phase',  start=06-21, end=07-15, estimate_pd=12, predecessors=[S1])
 └── 里程碑 M1 (parent=T, segment_kind='milestone', start=07-15, end=07-15, estimate_pd=0)
```

- **父任务** `estimate_pd` 可作为「目标总量」，实际进度由子段聚合：`SELECT COALESCE(SUM(estimate_pd),0) FROM tasks WHERE parent_task_id = :T`。
- **allocation 永远落在子段**（或可被独立排期的叶子任务），而非父任务——父任务只是容器，不直接消耗人力。这样工作量、利用率、Gantt 都自然按段展开。
- **依赖**：段间用 `task_dependencies` 表达先后；AI 求解时把段当作独立可调度单元，但受段内依赖约束。
- **分段策略**：长期任务的初始分段可由 LLM 辅助生成（按交付里程碑切分），写入后人工可调整；`segment_kind` 区分 `phase`（阶段）/`segment`（纯排期段）/`milestone`（零工作量节点）。

### 3.5 跨项目复用的建模

跨项目复用**不需要额外表**，由 `allocations` 自然表达：同一 `resource_id` 指向不同 `project` 下的 `task_id`，且每个 allocation 各自带 `start_date/end_date/percent`。例如开发者 A 在 6 月同时投入项目 P1 的任务 T1（50%）和项目 P2 的任务 T2（30%）：

```sql
INSERT INTO allocations (resource_id, task_id, start_date, end_date, percent, allocated_pd, source)
VALUES
  (1, 101, '2026-06-01','2026-06-30', 0.5, <计算后PD>, 'manual'),  -- task 101 属 project P1
  (1, 207, '2026-06-01','2026-06-20', 0.3, <计算后PD>, 'ai', <run_id>); -- task 207 属 project P2
```

容量上限由聚合查询保证：同一资源在同一**有效工作日**所有 allocation 的 `percent` 之和 ≤ **1.0**（以比例为单位的硬约束，见 §3.6 查询 (6) 与 §3.8 硬约束 1/3）。

跨项目复用时，该资源在 P1、P2 的成本单价可不同（由 `resource_project_rates` 覆盖，见 §3.3.17），但容量上限是**跨项目**统一计算（硬约束 3）。

### 3.6 关键查询示例

> **重要声明**：本节的聚合 SQL 仅为**概念示意**，不可直接作为可执行的正确实现。原因有二：
>
> 1. **窗口边界折算缺失**：`allocated_pd` 是按整条 allocation 全程计算的冗余字段（见 §3.3.15 说明）。下列查询 (2)/(3)/(4)/(5) 用区间相交 `a.start_date <= :W_TO AND a.end_date >= :W_FROM` 后直接 `SUM(a.allocated_pd)`，会**把跨窗口边界的 allocation 全额计入**局部窗口，导致 workload 虚高、过载误报（见 §9.3 风险 R10）。正确的窗口聚合必须按 `overlap_days × percent × avg_day_factor` 重算，由 §4.9 的 Rust `workload_pd()` 完成。
> 2. **capacity 不能折叠成全局工作日常量**：`capacity = Σ_d day_factor(d, resource)` 显式依赖资源（每个资源请假/节假日不同），不可用「全体资源相同的工作日数」作分母（见 §4.3）。
>
> 因此真正落地的 utilization/workload 聚合一律走 **Rust `workload_pd()` / `capacity_pd()`**，或在事件触发时预算到 §4.7 的 `workload_snapshot` 物化快照表供 UI/报表 O(1) 读取。下列 SQL 仅帮助读者理解「按谁 group、按谁 join」的聚合形状。

#### (1) `allocated_pd` 的计算（写入时由应用层调用）

工作日数 = allocation 全程区间内满足 `work_week_template` 且未被 `holiday`/`time_off` 全额扣除的日期数：

```sql
-- 伪逻辑（实际在 Rust Calendar 里实现，因为要逐日叠加 work_week_template / holiday / time_off 的 fraction）：
-- base_factor(d) = work_week_template(weekday(d)) 的 is_workday × day_frac
-- if holiday[d] exists → 折半或归零（fraction）
-- if time_off[resource][d] exists → 折半或归零（fraction）
-- working_factor_sum = Σ_d day_factor(d, resource, project)   over allocation 全程
-- allocated_pd = working_factor_sum × daily_capacity_pd × percent
```

> 注意 `allocated_pd` 是**全程全量** PD（基于 allocation 自身区间 `[start_date, end_date]`），与「查询窗口」无关。窗口内的部分 PD 须由 Rust 按 overlap 重算。

#### (2) 按人聚合 workload（指定时间窗 `[W_FROM, W_TO]`）—— 概念示意

```sql
-- 资源在窗口内的已分配 PD 与对应任务。
-- 警告：SUM(allocated_pd) 仅在窗口完整覆盖各 allocation 全程时正确；
--       通用窗口聚合须用 Rust workload_pd()（按 overlap × percent × avg_day_factor 重算）。
SELECT  r.id, r.name,
        SUM(a.allocated_pd) AS total_pd,        -- 见上述警告
        COUNT(DISTINCT a.task_id) AS task_count,
        COUNT(DISTINCT t.project_id) AS project_count
FROM resources r
JOIN allocations a ON a.resource_id = r.id AND a.deleted_at IS NULL
JOIN tasks t       ON t.id = a.task_id AND t.deleted_at IS NULL
WHERE r.deleted_at IS NULL
  AND a.status NOT IN ('cancelled')
  AND a.start_date <= :W_TO AND a.end_date >= :W_FROM  -- 区间相交（非窗口内折算）
GROUP BY r.id
ORDER BY total_pd DESC;
```

#### (3) 按团队聚合 workload —— 概念示意

```sql
-- 同 (2) 警告：SUM(allocated_pd) 未做窗口 overlap 折算。
SELECT tm.team_id, te.name AS team_name,
       SUM(a.allocated_pd) AS team_pd,
       COUNT(DISTINCT a.resource_id) AS members_busy
FROM team_members tm
JOIN allocations a ON a.resource_id = tm.resource_id AND a.deleted_at IS NULL
JOIN teams te      ON te.id = tm.team_id AND te.deleted_at IS NULL
WHERE a.start_date <= :W_TO AND a.end_date >= :W_FROM
  AND a.status <> 'cancelled'
GROUP BY tm.team_id
ORDER BY team_pd DESC;
```

#### (4) 按项目聚合 —— 概念示意

```sql
-- 预算消耗：R3 报表统一用 budget_pd（schema 只存 PD，无 budget_pm 列）。
-- PM 展示在渲染层按 effective_pm_workdays（含团队级覆盖）÷ 换算。
-- 同 (2) 警告：SUM(allocated_pd) 未做窗口 overlap 折算。
SELECT p.id, p.name, p.budget_pd,
       COALESCE(SUM(a.allocated_pd),0) AS allocated_pd,
       COALESCE(SUM(a.allocated_pd),0) / NULLIF(p.budget_pd,0) AS budget_usage
FROM projects p
LEFT JOIN tasks t       ON t.project_id = p.id AND t.deleted_at IS NULL
LEFT JOIN allocations a ON a.task_id = t.id AND a.deleted_at IS NULL
        AND a.status <> 'cancelled'
        AND a.start_date <= :W_TO AND a.end_date >= :W_FROM
WHERE p.deleted_at IS NULL
GROUP BY p.id;
```

#### (5) 时间窗内利用率（resource × 窗口）—— 概念示意

```sql
-- 警告：本查询的两个 CTE 仅为「聚合形状」示意，落地请走 Rust capacity_pd()/workload_pd()：
--   (a) capacity 必须「逐资源逐日」按 Σ day_factor(d, resource) 求和（day_factor 含 time_off/holiday），
--       不可用对全体资源相同的全局常量 :work_days_in_window 作分母——请假/节假日不同的资源会被算错。
--   (b) used_pd = SUM(allocated_pd) 未做窗口 overlap 折算（见 §3.3.15 警告）。
--   (c) 阈值/单位常数（pd_hours、pm_workdays、overload_threshold 等）须按 §3.3.8a 的 effective 值
--       （team_overrides 覆盖 → settings 全局）解析，不可一律取全局默认。
WITH
-- capacity 由 Rust 端对每个资源循环 Σ_d day_factor(d, resource, project) 得到后，作为参数 :cap_pd[r] 传入；
-- 或在 SQL 中对每个资源展开其 time_off/holiday 的日期维度（递归 CTE / calendar_day 维表）。
window_days AS (
  SELECT r.id AS resource_id,
         :cap_pd_r AS capacity_pd   -- 来自 Rust capacity_pd(cal, project, r, [W_FROM,W_TO])
  FROM resources r WHERE r.deleted_at IS NULL
),
used AS (
  SELECT resource_id, SUM(allocated_pd) AS used_pd   -- 警告：未做窗口 overlap 折算
  FROM allocations
  WHERE deleted_at IS NULL AND status <> 'cancelled'
    AND start_date <= :W_TO AND end_date >= :W_FROM
  GROUP BY resource_id
)
SELECT r.id, r.name,
       wd.capacity_pd                       AS net_capacity_pd,
       COALESCE(u.used_pd,0)                AS used_pd,
       COALESCE(u.used_pd,0) / NULLIF(wd.capacity_pd,0) AS utilization
FROM resources r
JOIN window_days wd ON wd.resource_id = r.id
LEFT JOIN used u    ON u.resource_id = r.id;
```

> 利用率公式：**utilization = used_pd / capacity_pd**（分子分母同口径，均经 `day_factor` 折算）。`> effective_overload_threshold(r)`（默认 1.0/100%，可被 team_overrides 覆盖）即过载，UI 标红。
> 对照 §4.10 的数值示例：Alice 因节假日 + 半天请假，其 `capacity = Σ day_factor = 3.5 PD`（而非「窗口工作日数 × 1.0」的全局常量值 5 PD），证明 capacity 必须逐资源逐日计算。

#### (6) 检测资源过载（硬约束校验）—— 比例口径

```sql
-- 以「比例」为单位的过载检测：找出某日 percent 之和 > 1.0 的资源。
-- 逐日展开由 Rust 做（按 day_factor 逐日累加 percent）；这里给某固定日 :D 的近似查询。
-- 注意：阈值恒为 1.0（不是 daily_capacity_pd），daily_capacity_pd 仅用于把比例折算成 PD 展示。
SELECT resource_id, :D AS day, SUM(percent) AS total_pct
FROM allocations
WHERE deleted_at IS NULL AND status NOT IN ('cancelled','done')
  AND :D BETWEEN start_date AND end_date
GROUP BY resource_id
HAVING SUM(percent) > 1.0;   -- 比例之和 > 1.0 即过载
```

> 单位口径决议（贯穿 §3.8 / §4 / §5）：过载检测与容量上限**统一在「比例」空间**——任一有效工作日 `Σ percent ≤ 1.0`。`daily_capacity_pd` 不参与过载阈值，仅用于把比例折算成 PD 做展示（`pd = percent × daily_capacity_pd × day_factor`）与跨资源加总。这样避免了「比例之和（0–N）vs PD 值（常 1.0）」的单位混用：例如兼职资源 `daily_capacity_pd=0.5` 时，仍按 `Σ percent ≤ 1.0` 判过载，其产出 PD 由 `0.5 × percent × day_factor` 自然折算。
>
> `team_overrides.overload_threshold` 覆盖的是**利用率百分比阈值**（用于 §4.5/§9 红绿灯与预警展示），**不改写**本硬约束的 `Σ percent ≤ 1.0` 比例上限——后者是求解器与运行时一致的硬约束，不可被配置放宽。

#### (7) 按项目/资源解析并行任务上限 —— 概念示意

```sql
-- 配合 §3.8 硬约束 6：检查某资源在某项目某日的并行 allocation 数是否越上限。
-- 解析 effective_max_parallel(r, p)：
--   resources.max_parallel_tasks_per_day        非空 → 取它
--   否则 projects.max_parallel_tasks_per_day    非空 → 取它
--   否则回落全局 ConstraintFlags.max_parallel_tasks_per_day（None=不限并行）
-- 下列 SQL 仅示意「项目级」覆盖的统计形状；资源级覆盖与全局回落由 Rust 拼装。
SELECT a.resource_id, t.project_id, :D AS day, COUNT(*) AS parallel_cnt,
       p.max_parallel_tasks_per_day AS proj_cap
FROM allocations a
JOIN tasks t       ON t.id = a.task_id
JOIN projects p    ON p.id = t.project_id
WHERE a.deleted_at IS NULL AND a.status NOT IN ('cancelled','done')
  AND :D BETWEEN a.start_date AND a.end_date
  AND p.max_parallel_tasks_per_day IS NOT NULL
GROUP BY a.resource_id, t.project_id
HAVING COUNT(*) > p.max_parallel_tasks_per_day;
```

### 3.7 并发写入与事务策略

虽然是单用户桌面应用，但**后台 AI 优化任务（tokio）与前端 UI 写入会并发**，故仍需严格的并发控制。

| 关注点 | 策略 |
|---|---|
| **写入并发模型** | SQLite WAL 下**同一时刻只允许一个写者**。使用单个共享的 `sqlx::SqlitePool`（`max_connections` 设较小，如 5；写连接串行化）。 |
| **事务隔离级别** | SQLite 默认 SERIALIZABLE（写时全库锁）。**所有写事务一律用 `BEGIN IMMEDIATE`**（而非默认 `DEFERRED`），在事务开始即获取写锁，避免「先读很久、提交时才撞锁」导致 `SQLITE_BUSY` 浪费工作。sqlx 用 `pool.begin().await?` 后立即执行一条 `BEGIN IMMEDIATE`，或用 `sqlx::query("BEGIN IMMEDIATE")`。 |
| **busy_timeout** | `PRAGMA busy_timeout = 5000`：写者冲突时等待最多 5s，覆盖典型 AI 任务持锁时长；仍超时则应用层捕获 `SQLITE_BUSY` 并重试（指数退避，最多 3 次）。 |
| **读并发** | WAL 允许读者与写者并发，UI 的只读聚合查询不阻塞 AI 写入。 |
| **大事务边界** | AI 优化产出可能涉及成百上千条 allocation。**整个「写入方案」作为单一 `BEGIN IMMEDIATE` 事务**：先标记旧 `ai` allocation（同 run 范围）为 cancelled，再批量插入新 allocation，最后更新 `ai_optimization_runs.applied=1` 与 `status='accepted'`。要么全成功，要么全回滚，避免半应用状态。 |
| **乐观锁** | 业务实体带 `updated_at`；前端编辑时携带原值，`UPDATE ... SET ... updated_at=:new WHERE id=:id AND updated_at=:expected`，受影响行数=0 即并发冲突，提示用户刷新。 |
| **迁移互斥** | 迁移在应用启动时**单连接**运行（`Migrator::new`），先于连接池对外服务，避免迁移与业务并发。 |
| **allocation 单一写入入口** | 所有 allocation 写入（含人工拖拽、AI apply、批量导入、迁移脚本）**必须经 `AllocationService::create` 单一入口**；repo 层不暴露裸 `insert into allocations`。该入口在事务内执行 §3.8 硬约束 1/2/4/6 的预检（容量上限、时间窗、技能、并行上限），与 §3.3.15a 的数据库触发器互为冗余防线，确保旁路写入（导入/AI apply）也过校验。 |

**事务模板（Rust/sqlx）：**

```rust
// 写事务统一封装：BEGIN IMMEDIATE + busy_timeout 兜底
async fn with_write_tx<F, T, E>(pool: &SqlitePool, f: F) -> Result<T, E>
where
    F: for<'c> FnOnce(&'c mut Transaction<'c, Sqlite>) -> BoxFuture<'c, Result<T, E>>,
    E: From<sqlx::Error>,
{
    let mut backoff = 50;
    for _ in 0..3 {
        let mut tx = pool.begin().await?;
        sqlx::query("BEGIN IMMEDIATE") // 显式升级为写事务
            .execute(&mut *tx).await?;  // 注：实际需在 begin 前或用 IMMEDIATE pool
        match f(&mut tx).await {
            Ok(v) => { tx.commit().await?; return Ok(v); }
            Err(e) => { return Err(e); }
        }
        // SQLITE_BUSY -> sleep(backoff); backoff *= 2;
    }
    unreachable!()
}
```

> 实践中推荐用 `sqlx::Transaction` + 在连接初始化即设 `busy_timeout`，并在 `sqlx::Error::Database` 的 `is_busy()` 上做重试封装。

### 3.8 容量/分配一致性约束（硬约束清单）

下列约束是 AI 经典优化器（good_lp/ILP）与运行时校验共同遵守的硬约束。**容量上限统一以「比例」为单位**（任一有效工作日 `Σ percent ≤ 1.0`），`daily_capacity_pd` 仅用于 PD 展示折算，不作为阈值（见 §3.6 查询 (6) 口径说明）。

1. **容量上限（比例口径）**：任一资源任一**有效工作日**（`day_factor(d, resource) > 0`）的 `Σ percent ≤ 1.0`。
   - 不写 `Σ percent ≤ daily_capacity_pd`（那是比例 vs PD 的单位混用，仅在 `daily_capacity_pd=1.0` 时偶然正确）。
   - `daily_capacity_pd`（如兼职 0.5）的作用是把比例折算成 PD：`pd = Σ(percent × daily_capacity_pd × day_factor(d))`，仅用于展示与跨资源加总。
2. **时间窗（schema 层强制）**：allocation 的 `[start,end]` 必须落在 task 的 `[start,end]` 与 resource 的 `[available_from, available_to]` 交集内。**由 §3.3.15a 的 `allocation_validate` 触发器在数据库层强制**（`RAISE(ABORT)`），同时由 `AllocationService::create` 入口在写入前预检；违反即拒绝。
3. **跨项目不冲突（比例口径）**：同一资源同一有效工作日所有 allocation（跨任意 project）的 `percent` 之和 ≤ 1.0——由聚合查询天然覆盖，无需特殊表。
4. **技能硬约束**：若 `task_skill_requirements.is_mandatory=1`，则被分配资源在 `resource_skills` 中对该 skill 的 `proficiency ≥ min_proficiency`，否则求解器视为不可分配。由 `AllocationService::create` 校验。
5. **依赖**：allocation 的 start 不早于所有 predecessor 的 end（`FS` 默认，可加 `lag_days`）。由 AI 求解器与 `AllocationService` 共同校验。
6. **并行任务上限（决议落地，响应开放问题 #26）**：`max_parallel_tasks_per_day` 支持「按资源」与「按项目」细粒度配置，解析优先级如下：
   ```
   effective_max_parallel(r, p):
       1. resources.max_parallel_tasks_per_day            非空 → 取它（资源级覆盖）
       2. 否则 projects.max_parallel_tasks_per_day         非空 → 取它（项目级覆盖）
       3. 否则 ConstraintFlags.max_parallel_tasks_per_day  非空 → 取它（全局）
       4. 否则 None（不限并行，仅受硬约束 1 的 Σ percent ≤ 1.0 约束）
   ```
   含义：某资源在某项目某有效工作日同时承担的（未取消/未完成）allocation 条数 ≤ `effective_max_parallel(r, p)`。`Some(1)` 即禁并行。由 `AllocationService::create` 写入前预检（与硬约束 1/2/4 同入口）；求解器在建模时把每个资源×项目×日的并行数作为整数变量上限注入（见 §5）。
   - 注意：并行上限与容量上限（硬约束 1）是**两个独立约束**，互不替代——前者限制「同时挂着的任务条数」，后者限制「投入比例之和」。二者均满足才放行。

> **与 §5 MILP 建模的对齐（单位闭合）**：§5.5.1 用 0/1 决策变量 `x_{r,t,d}`（表「资源 r 在日 d 是否对任务 t 有投入」）与连续变量 `y_{r,t}∈[0,1]`（表「投入比例」）共同建模。为使本节的「比例」上限能被求解器执行，二者须满足耦合约束 `y_{r,t} = Σ_d x_{r,t,d} × (1 / 该任务有效工作日数)`（或直接以 `percent_{r,t,d}` 连续变量建模），容量约束改写为连续形式 `Σ_t percent_{r,t,d} ≤ cap_{r,d}`，其中 `cap_{r,d} = day_factor(d,r)`（比例空间，≤1.0）。`x` 表「是否投入」，`y`/`percent` 表「投入比例」，语义分离避免单位不闭合。并行上限约束改写为 `Σ_t x_{r,t,d·project=p} ≤ effective_max_parallel(r, p)`（对每个 (r,p,d) 整数不等式）。

应用层在每次 `allocation` 写入时经 `AllocationService::create` 校验上述 1/2/4/6（违反则拒绝），并由 §3.3.15a 触发器在 schema 层兜底硬约束 2；AI 求解器在求解阶段即保证全部硬约束。

### 3.9 迁移策略（sqlx migrate）

- **目录**：仓库根 `migrations/`，每个迁移一个文件，命名 `YYYYMMDDHHMMSS_<snake_name>.sql`（sqlx 默认顺序执行、记录到 `_sqlx_migrations` 表）。
- **不可变**：已发布的迁移文件不再修改，新需求新增迁移（如 `ALTER TABLE ... ADD COLUMN`）。SQLite 3.35+ 支持 `ADD COLUMN` 与 `DROP COLUMN`（受限）。
- **编译期校验**：`sqlx::query!` / `query_as!` 宏在 `cargo build` 时连接数据库校验 SQL 与类型；CI 中用 `SQLX_OFFLINE=true` + `sqlx prepare` 生成的 `.sqlx/` 目录做离线校验，避免 CI 依赖运行中的 DB。
- **WAL/PRAGMA 处理**：WAL 与 `PRAGMA foreign_keys=ON` **不放进迁移**（sqlx 把迁移包在事务里，而 `journal_mode` 不能在事务内执行）。这些 PRAGMA 在**连接池构建时**通过 `SqliteConnectOptions::pragma("journal_mode","WAL")` 等设置，或 `connect_options.pragma(...)` 链式配置。
- **启动顺序**：`SqlitePool` 建立 → 应用 PRAGMA → `Migrator::new(pool).await?.run::<Sqlite>(&pool).await?` → 暴露 IPC 命令。迁移失败则应用拒绝启动并报错。
- **种子数据**：`settings` 单行（含 `secret_store='keychain'` 默认值）、`work_week_template` 全局默认行的 `INSERT` 放在第一个迁移里（幂等：`INSERT OR IGNORE`）。
- **回滚**：单用户本地应用不提供自动 down 迁移；通过「备份 DB 文件 + 前向迁移」策略：每次启动若检测到 schema 版本跳跃或迁移失败，提示从 `*.bak` 恢复。建议应用在迁移前自动复制一份 `kanban.db.bak`。

**迁移文件示例：**

```
migrations/
  20260627120000_init_settings.sql       -- settings（含 secret_store 列）
  20260627120001_tags_skills.sql
  20260627120002_resources.sql           -- resources（含 daily_rate_pd、max_parallel_tasks_per_day）
  20260627120003_teams_members.sql       -- teams + team_members + team_overrides
  20260627120004_calendar.sql            -- work_week_template（含修正后的 idx_wwt_global）+ holiday + time_off
  20260627120005_projects.sql            -- projects（含 budget_pd、max_parallel_tasks_per_day）
  20260627120006_tasks_deps_reqs_tags.sql
  20260627120007_allocations.sql         -- 含 §3.3.15a 校验触发器
  20260627120008_ai_optimization_runs.sql
  20260627120009_resource_project_rates.sql  -- resource×project 维度费率（MVP 即建表）
```

> **MVP schema 一次性补齐（决议）**：`resources.daily_rate_pd`、`resource_project_rates` 表均**在 MVP 阶段即补入 schema**（不再延后到 v1.2），使成本核算能力可前移；对应的成本报表（R7）是否纳入 MVP DoD 仍由 §8/§9 路线图决定，但 schema 不再是其前置阻塞。

### 3.10 数值示例（PD/PM 换算与利用率）

设 `settings.pd_hours=8`、`pm_workdays=20`（即 `1 PM = 20 PD = 160 h`）；若资源 A 属于 team T1 且 `team_overrides(T1).pm_workdays=22`，则对 A 的 PM 换算用 22（`1 PM = 22 PD`）。

| 场景 | 计算 | 结果 |
|---|---|---|
| 任务估算 1.5 PM | `1.5 × 20`（默认 team） | 30 PD |
| 资源 A 月容量（全勤） | `20 有效工作日 × day_factor(1.0) = 20` | 20 PD = 1.0 PM |
| A 已分配：P1.T1（50%, 全月）+ P2.T2（30%, 前10有效工作日） | `20×0.5 + 10×0.3 = 10 + 3` | 13 PD |
| A 请假 2 天（`time_off.fraction=1.0`） | 容量 `Σ day_factor = 20 − 2 = 18 PD`（逐日扣减） | net_capacity = 18 PD |
| A 利用率 | `13 / 18` | **72.2%**（健康） |
| 若 A 另接 P3.T3（60%, 全月）→ 某日 `Σ percent` 检查 | `0.5+0.3+0.6 = 1.4 > 1.0` | **过载（比例口径），硬约束拒绝** |
| A 在 P1 的成本（`resource_project_rates(A,P1)=1200`，P2 无覆盖回落 `resources.daily_rate_pd=1000`） | P1: `allocated_pd_P1 × 1200`；P2: `allocated_pd_P2 × 1000` | 按 `effective_daily_rate` 分别计价 |

> 本例的 capacity 扣减是**逐资源逐日**按 `day_factor` 求和（`time_off.fraction` 折算），而非全局工作日常量；过载判断用比例 `Σ percent > 1.0`，与 §3.8 硬约束 1 口径一致；PM 换算与成本单价均按 effective 值（team_overrides / resource_project_rates 覆盖）解析。更完整的逐日示例见 §4.10。

### 3.11 与 AI 引擎的接口契约（数据层）

- **输入快照**：`ai_optimization_runs.input_snapshot_json` 序列化 `Vec<ResourceState>` / `Vec<TaskState>` / `Vec<SkillRequirement>`，含当时 capacity（逐资源逐日 `day_factor` 序列）、已有 allocation（作为固定占用）、技能矩阵、effective 常数（pd_hours/pm_workdays/thresholds/max_parallel 的解析结果）。求解器与 LLM 评分均以此快照为输入，保证运行期间用户编辑不影响本次结果。
- **输出落库**：方案中的 allocation 写入 `allocations`（`source='ai'`、`run_id` 指回本次 `ai_optimization_runs.id`），旧 `ai` allocation 在事务内标记 cancelled，run 的 `status` 置 `accepted`、`applied=1`，实现「方案切换」的原子性。
- **run_id 类型对齐（全链路确认）**：`AllocationProblem.run_id` 为 `i64`（落库时由 `ai_optimization_runs` AUTOINCREMENT 回填并写回 `allocations.run_id`）。`ai_optimization_runs` 主键为 INTEGER 自增；§5.7 的 `ai_run(TEXT UUID)` 表已废弃删除，全链路不再引用。参见 §3.3.16 表合并决议。
- **语义匹配**：LLM 读取 `resource_skills` + `resource_tags` + `task_skill_requirements` + `task_tags`，对每个 (resource, task) 对输出 `skill_fit ∈ [0,1]`，作为求解器目标函数中 `skill_fit` 项的系数（与 `task_skill_requirements.weight` 相乘）。
- **并行上限注入**：求解器在构建 MILP 时，对每个 (resource, project, day) 三元组按 `effective_max_parallel(r, p)`（§3.8 硬约束 6）注入并行数整数不等式；全局/项目/资源三级的解析在快照阶段一次性算好并写入 `input_snapshot_json`，求解器与运行时读同一结果。

### 3.12 假设（本章）

1. 日历模型单一真相源为 §4.2 三表 `work_week_template` / `holiday` / `time_off`；`settings.workweek_mask` 与 `resource_unavailable` 已废弃删除。
2. capacity/workload 必须「逐资源逐日」按 Σ day_factor 计算，不可折叠成全局工作日常量；§3.6 聚合 SQL 仅为概念示意，真正口径以 §4.9 Rust 计算核心为准。
3. `allocated_pd` 是按 allocation 全程全量计算的冗余字段，仅在窗口完整覆盖全程时可直接 SUM；通用窗口聚合须 Rust 按 overlap×percent×avg_day_factor 重算。
4. 容量上限统一为「比例口径」Σ percent ≤ 1.0（任一有效工作日）；`daily_capacity_pd` 仅用于 PD 展示折算与跨资源加总，不参与过载阈值。
5. 优化运行表统一为 `ai_optimization_runs`（INTEGER 自增 PK），§5.7 的 `ai_run(TEXT UUID)` 已删除；`allocations.run_id` 为 INTEGER，`AllocationProblem.run_id` 为 i64，全链路一致（开放问题 #13 已解决）。
6. `resources` 表不再保留 `default_capacity`（与 `daily_capacity_pd` 重叠）；`daily_rate_pd`（可空）供成本估算；新建 allocation `percent` 默认 1.0。
7. `projects` 预算统一为 `budget_pd`，**不新增、不保留任何 `budget_pm` 列**（跨节 `budget_pm` 表述一律废弃）；§8 报表需对齐此字段名（开放问题 #12 已解决）。
8. allocation 时间窗硬约束由 §3.3.15a 数据库触发器在 schema 层强制，并由 `AllocationService::create` 单一写入入口预检，两层互为冗余。
9. `work_week_template` 的 `mon_frac..sun_frac` 列支持周期性非均匀日容量（如周五半天），§9.4 开放问题 #12 升级为「支持」（开放问题 #16 同步更新已落地）。
10. §5 MILP 的 0/1 变量 x 与连续变量 y/percent 通过耦合约束闭合单位，容量约束在比例空间 `Σ_t percent ≤ cap`（cap=day_factor≤1.0）表达。
11. `work_week_template` 全局唯一性由基于常量的 `UNIQUE INDEX idx_wwt_global ON work_week_template((1)) WHERE scope='global'` 强制「全局仅一行」；项目级模板由 `UNIQUE idx_wwt_project(project_id) WHERE scope='project'` 保证每项目至多一条（开放问题 #15 已解决）。
12. `max_parallel_tasks_per_day` 支持「按资源」与「按项目」细粒度配置：`resources.max_parallel_tasks_per_day`（资源级）→ `projects.max_parallel_tasks_per_day`（项目级）→ 全局 `ConstraintFlags`（默认 None=不限并行）三级回落（开放问题 #26 已解决）。
13. PD/PM 常数与利用率阈值支持团队级覆盖：新增 `team_overrides(team_id PK, pd_hours, pm_workdays, overload_threshold, underload_threshold, utilization_green, utilization_yellow)`，未覆盖项回落 `settings` 全局值；effective 常数在加载时按资源所属 team 展开为逐资源值（响应开放问题 #2/#3/#46/#47 的「团队级可配置」决议）。
14. `settings.secret_store ∈ {'keychain','encrypted_file'}` 作为独立 settings 列（非 metadata JSON）承载密钥存储后端，与 §6 keychain 降级路径对齐（开放问题 #29 已解决）。
15. 成本单价支持「默认 + 项目级覆盖」两层：`resources.daily_rate_pd`（默认单价）与 `resource_project_rates(resource, project[, period])`（项目级/周期浮动费率）；`effective_daily_rate(r,p,d)` 解析顺序为 resource_project_rates → resources.daily_rate_pd → N/A（响应开放问题 #38 的「按项目/周期浮动」决议）。
16. `resources.daily_rate_pd` 与 `resource_project_rates` 表均**在 MVP 阶段即补入 schema**，使成本核算能力可前移（开放问题 #39 已解决）；R7 成本报表是否纳入 MVP DoD 仍由 §8/§9 路线图决定。

### 3.13 开放问题（本章，需 impl 期决策）

1. **[impl 期决策]** `team_overrides` 对「归属多个 team 的资源」的 effective 常数解析规则：当前约定按 `role='lead'` 优先、否则取最近 `joined_at` 的 team；落地时需确认该规则是否满足产品预期，或是否需要在 `team_members` 增加显式 `is_primary` 标志。
2. **[impl 期决策]** `resource_project_rates` 同一 (resource, project) 多周期行的区间不相交校验：当前约定由应用层（rate service / `AllocationService`）写入时预检；是否额外用 SQLite 触发器或 `EXCLUDE`-类机制做硬性防重叠，待实现期评估。
3. **[impl 期决策]** 硬约束 1（逐日 Σ percent ≤ 1.0）与硬约束 4（技能）因需跨行/跨表聚合，仍依赖 `AllocationService::create` 预检，触发器层不强制；是否需要在 `workload_snapshot` 快照表层面补充一致性校验，待 §4.7 快照落地后决定（延续原开放问题 #17）。
4. **[impl 期决策]** 若未来确需 UUID 作 `run_id`，须同步回改 §3.3.15/§3.3.16 全链路类型（INTEGER→TEXT）；当前 INTEGER 自增口径已确认即最终口径，本项仅作未来预案保留（延续原开放问题 #25）。

## 4. 工作负载与容量模型 / Workload & Capacity

工作负载（Workload）与容量（Capacity）是本系统「以人为核心」进行人力配置优化的计算基础。所有利用率、过载检测、AI 配置建议与报表，都依赖这两个量的精确、可复现、可换算的计算。本节定义计量单位、日历模型、Capacity/Workload 的计算公式、聚合口径、实时性策略与单位切换规则。

> **口径总则（贯穿全章）**：workload 与 capacity 的**唯一权威计算源**是 §4.9 的 Rust 计算核心（按日 overlap + `day_factor`，无副作用、可复现）。`allocations.allocated_pd` 冗余列、§4.7 物化缓存表、§3.6 的 SQL 聚合均为**派生/缓存**，三者口径必须始终向 §4.9 对齐；任何跨时间窗的 workload/utilization 聚合**不得**直接 `SUM(allocated_pd)`（理由见 §4.4 与 §4.7）。

### 4.1 计量单位：PD 与 PM 的可配置换算

系统以**人日（Person-Day, PD）**为内部唯一存储与计算单位，PM（人月）仅为展示/报表层的派生单位。所有 schema 字段统一用 `REAL` 存 PD 浮点（精度到 0.1 PD），避免单位混存导致的换算误差。

| 单位 | 含义 | 默认值 | 可配置层级 | 配置键 |
|------|------|--------|-----------|--------|
| 1 PD = ? 小时 | 一名工程师一个工作日的有效工时 | **8 h** | 全局 → 项目级覆盖 | `hours_per_pd` |
| 1 PM = ? PD | 一个人月折算的工作日数 | **20 PD**（约 4 个工作周） | 全局 → 项目级覆盖 | `pd_per_pm` |
| 1 PM = ? 小时 | 派生 = `hours_per_pd × pd_per_pm` | 160 h | 不可直接配，派生 | — |

配置优先级（高 → 低）：**项目级 override → 全局默认**。查询时通过 `effective_unit_config(project_id)` 解析：若项目未设置则回落到全局。

```rust
/// 单位换算配置（解析后）
#[derive(Clone, Copy, Debug)]
pub struct UnitConfig {
    pub hours_per_pd: f64,   // 默认 8.0
    pub pd_per_pm:   f64,   // 默认 20.0
}

impl UnitConfig {
    pub fn pd_to_hours(self, pd: f64)  -> f64 { pd * self.hours_per_pd }
    pub fn hours_to_pd(self, h: f64)   -> f64 { h  / self.hours_per_pd }
    pub fn pd_to_pm(self, pd: f64)     -> f64 { pd / self.pd_per_pm }
    pub fn pm_to_pd(self, pm: f64)     -> f64 { pm * self.pd_per_pm }
}
```

> **决策**：内部一律按 PD 计算、存储；只有渲染层（看板/甘特卡片、报表单元格）按用户当前单位偏好做一次性 `pd_to_pm` / `pd_to_hours` 换算。这样 AI 求解器、过载检测、利用率公式都只面对一种单位，PM 切换不会产生重复折算。

### 4.2 日历模型 / Calendar Model

Capacity 的本质是「时间窗内的有效工作日数」，因此需要一个能表达**工作日/周末/法定节假日/资源请假**的日历模型。采用三张表叠加（从粗到细）：

| 层级 | 表 | 作用 | 粒度 | 来源 |
|------|----|------|------|------|
| L1 工作周模板 | `work_week_template` | 定义每周哪几天是工作日 | 周（Mon..Sun 布尔） | 全局默认 Mon–Fri，项目可覆盖 |
| L2 节假日 | `holiday` | 扣除非工作节假日 | 单日（可标注半天） | 内置 ICS 导入 / 手工维护 |
| L3 资源请假 | `time_off` | 扣除特定资源的请假/调休 | 单日（含半天比例） | 资源登记 |

判定某一天 `d` 对资源 `r` 是否为有效工作日（以及其有效比例 `day_factor ∈ [0,1]`）的逻辑：

```
is_work_day(d, project):
    if not work_week_template[project][weekday(d)]   → 0   (周末)
    if holiday[d] exists (full day)                   → 0
    if holiday[d] exists (half day)                   → 0.5
    otherwise                                          → 1.0

day_factor(d, resource):
    base = is_work_day(d, resource.project)
    if time_off[resource][d] exists (full day)         → 0
    if time_off[resource][d] exists (half day)         → base × 0.5
    otherwise                                           → base
```

关键 schema 片段（SQLite，与 sqlx migrate 对齐）：

```sql
CREATE TABLE work_week_template (
    id          INTEGER PRIMARY KEY,
    scope       TEXT NOT NULL CHECK (scope IN ('global','project')),
    project_id  INTEGER REFERENCES project(id),   -- NULL when scope='global'
    mon TINYINT DEFAULT 1, tue TINYINT DEFAULT 1, wed TINYINT DEFAULT 1,
    thu TINYINT DEFAULT 1, fri TINYINT DEFAULT 1, sat TINYINT DEFAULT 0, sun TINYINT DEFAULT 0
);

CREATE TABLE holiday (
    id          INTEGER PRIMARY KEY,
    project_id  INTEGER REFERENCES project(id),   -- NULL = 全局
    day         TEXT NOT NULL,                     -- 'YYYY-MM-DD'
    fraction    REAL NOT NULL DEFAULT 1.0,         -- 1.0 全天 / 0.5 半天
    name        TEXT
);

CREATE TABLE time_off (
    id          INTEGER PRIMARY KEY,
    resource_id INTEGER NOT NULL REFERENCES resource(id),
    day         TEXT NOT NULL,
    fraction    REAL NOT NULL DEFAULT 1.0,         -- 1.0 全天请假 / 0.5 半天
    reason      TEXT
);
CREATE INDEX idx_time_off_res_day ON time_off(resource_id, day);
```

> 半天（`fraction=0.5`）的存在使模型能表达半天请假、半天节假日等真实场景，避免「要么全天要么没有」的粗暴扣减。

#### 4.2.1 时区与跨地区日历（明确假设）

本模型的所有日期字段（`day` / `start_date` / `end_date`）均为无时区归属的纯日期 `TEXT 'YYYY-MM-DD'`，时间戳统一 ISO8601 UTC（见 §3.1）。本章做如下显式假设，并将其纳入 §1.5 非目标：

> **假设：单一时区 / 单一工作日历地区。** 本模型假设整个组织运行在同一时区、同一套工作日历语义下。所有「工作日 / 周末 / 节假日」的边界以该单一地区为准；资源的 `day_factor` 不按资源本人的所在地区分别解析。

由此带来的限制与设计取舍：

| 关注点 | 当前（单时区）处理 | 跨时区/跨地区（非目标，未来若启用）所需扩展 |
|--------|-------------------|----------------------------------------------|
| 「同一工作日」边界 | 全局同一日历日 | 需为每个资源/项目定义归属时区，按各地本地日切 |
| 节假日表 `holiday` | 全局共享一套节假日 | 需增加 `region TEXT` 列，按资源的归属地区解析 |
| 工作周模板 `work_week_template` | 仅 global/project 两级 | 需增加 `timezone TEXT`，避免「项目级覆盖」在跨国项目下错位 |
| §4.2「项目级工作周覆盖」 | 仅适用于同一地区内的不同排班约定 | 跨国项目下需按成员所在地拆分工作周，否则 capacity 会错算 |

**跨时区团队（如一个资源在上海、一个在硅谷）为非目标**，已加入 §1.5 非目标表，本章维持此非目标不动。若未来确需启用，所需变更清单（均属**重大版本升级**，非渐进迁移，不在 MVP 范围）：

1. 给 `holiday` 与 `work_week_template` 增加 `region TEXT` / `timezone TEXT` 列；
2. `day_factor(d, resource)` 改为按 `resource.region` 解析对应地区日历（而非全局统一）；
3. **`workload_cache` 全量 `engine_rev` 迁移**：跨地区解析会改变既有缓存行的口径，必须提升 `engine_rev`（如 `workload_engine@0.5.0`）并判定全表 stale、按地区重新回填；
4. 连带重评 §4.3/§4.4 全链路的 capacity/workload 口径（同一「工作日」边界按各资源本地日切判定）。

该清单为未来工作路标，MVP 阶段不预留相关列、不预留空解析分支。

### 4.3 容量计算 / Capacity

某资源 `r` 在时间窗 `[start, end]` 内的**有效可用 PD**：

```
capacity(r, [start, end]) = Σ_{d ∈ [start,end]} day_factor(d, r) × hours_per_pd_eqv
```

其中 `hours_per_pd_eqv` 把日历扣减后的「有效工时」折算回 PD：因为 `day_factor` 已经按比例折算（1.0 = 一个完整工作日），所以一行工作日即 1 PD（在默认 8h/PD 下）。若希望以小时驱动，等价写法：

```
capacity_pd(r, [start, end]) = Σ day_factor(d, r) × (1.0)        // 直接得 PD
capacity_hours(r, [start, end]) = capacity_pd × hours_per_pd
```

需进一步乘以该资源在该任务/项目上的**投入比例（allocation percent）**。Capacity 分两种语义，文档需明确区分：

| Capacity 类型 | 公式 | 用途 |
|--------------|------|------|
| **毛容量 Raw Capacity** | 不含投入比例，纯日历折算 | 评估「此人理论上本月有多少 PD 可卖」 |
| **任务容量 Allocated Capacity** | `Raw × allocation_percent` | 与同一 allocation 的 workload 直接对比 |

通常利用率公式使用**毛容量**做分母（因为过载检测要回答「此人在窗口内总投入是否超过他物理上能工作的量」），而单 allocation 的盈亏用任务容量。

### 4.4 工作负载计算 / Workload

某资源 `r` 在时间窗 `[start, end]` 内的 workload，= 所有覆盖该窗口的 allocation 按区间折算后的 PD 之和。单条 allocation 的折算公式：

```
alloc_pd(a, [start, end]) =
    overlap_days(a.start, a.end, start, end)
    × a.percent                    // 投入比例（0.0–1.0）
    × avg_day_factor(a.resource, overlap区间)   // 日历折算，与 capacity 口径一致
```

> **关键约定**：workload 与 capacity **共用同一套 `day_factor`**。即一名工程师即便被分配到某任务，遇到周末/节假日/请假同样不计 workload——否则会出现「分了但实际干不了」的虚假负载。`avg_day_factor` 取重叠区间内各有效工作日的平均日历因子（半天按 0.5 计入），保证分子分母同口径。

资源级 workload：

```
workload(r, [start, end]) = Σ_{a ∈ allocations(r)} alloc_pd(a, [start, end])
```

#### 4.4.1 与 `allocations.allocated_pd` 的关系（重要口径区分）

`allocations.allocated_pd`（见 §3.3.15）是**写入时按 allocation 自身区间 `[a.start, a.end]` 预算好的单值**，其语义严格等于：

```
allocated_pd(a) == alloc_pd(a, [a.start, a.end])
```

即「整条 allocation 在其自身完整区间上的总工作量」。它**不**包含以下两种折算，因此**不可**用于跨窗口聚合：

1. **overlap 折算**：聚合窗口 `[W_FROM, W_TO]` 通常 ≠ allocation 区间。`allocated_pd` 存的是全区间值，而 §4.4 的 workload 需要的是窗口内 overlap 部分。
2. **窗口内 `avg_day_factor`**：§4.4 的 `avg_day_factor` 取的是「重叠区间」内的平均日历因子；当聚合窗口截断了 allocation（例如只取某 allocation 的前半段），`allocated_pd` 里存的「全区间 avg」与窗口内实际因子不一致。

因此，**跨窗口的 workload/utilization 聚合禁止 `SUM(allocated_pd)`**。两条合法路径：

- **权威路径（默认）**：走 §4.9 Rust 计算核心，按日 overlap + `day_factor` 精算。数据量小（10 资源 / 50 任务规模），单次窗口计算 < 5ms。
- **加速路径（可选物化，impl 期决策）**：`allocation_daily` 是**可选加速路径**，**是否进 MVP 待 impl 期在 Dashboard 实测读延迟后决定**。若 Dashboard 跨窗口聚合在 §4.9 精算下的读延迟可接受（10 资源/50 任务规模单次 < 5ms），则**不引入**该表，统一走 §4.9 精算（无新增物化、无增量维护成本）；仅当实测延迟成为体验瓶颈时才引入，按「资源 × 日 × allocation」预存每日 PD，聚合时按窗口 SUM 该明细：

  ```sql
  -- 可选：按日明细物化，由领域事件增量维护（与 §4.7 workload_cache 同源）
  CREATE TABLE allocation_daily (
      allocation_id INTEGER NOT NULL REFERENCES allocations(id) ON DELETE CASCADE,
      resource_id   INTEGER NOT NULL,
      project_id    INTEGER NOT NULL,
      day           TEXT    NOT NULL,   -- 'YYYY-MM-DD'（仅有效工作日）
      pd            REAL    NOT NULL,   -- = day_factor(day) × percent（该日该 allocation 的贡献）
      PRIMARY KEY (allocation_id, day)
  );
  CREATE INDEX idx_alloc_daily_res_day ON allocation_daily(resource_id, day);
  ```

  `allocation_daily` 的每行 `pd` 即 §4.9 `alloc_pd` 在「单日」粒度的分解，SUM 后与 Rust 核心逐日累加结果**数学等价**，从而保证 SQL 聚合（Dashboard/报表）与 Rust 公式（优化器）给出同一数字。该表为可重建的派生物化，不引入新的真相源。

> **对 §3.3.15 / §3.6 的口径修订**：`allocated_pd` **仅用于单条 allocation 的工作量展示**（如 Gantt 条带长度、任务总投入、预算消耗粗算），**不可用于跨窗口 workload/utilization 聚合**。§3.6 查询 (2)/(3)/(4) 中出现的 `SUM(a.allocated_pd)` 在「聚合窗口 == 各 allocation 自身区间」的近似场景可用，但在任意自定义窗口下口径不正确，应改用 §4.9 核心或 `allocation_daily`。

### 4.5 利用率与过载检测 / Utilization & Overload

```
utilization(r, [start, end]) = workload(r, [start, end]) / capacity(r, [start, end])
```

- `utilization ≤ 1.0`（100%）：负载正常。
- `1.0 < utilization ≤ 阈值（默认 1.1）`：接近满载，黄色告警。
- `utilization > 阈值`：**过载（Overload）**，红色告警，AI 求解器需将其作为软约束惩罚项。

过载阈值 `overload_threshold` 全局可配（默认 110%）。注意：多 allocation 叠加时只要 `Σ percent > 100% × (有效工作日)` 即过载，与「跨项目不冲突」硬约束在求解器侧配合——硬约束保证时间窗不冲突，软约束惩罚接近满载。

### 4.6 两种聚合口径 / Aggregation Scopes

| 聚合口径 | 定义 | 公式 |
|----------|------|------|
| **以人为单位（Per-Resource）** | 单个资源在窗口内的总负载/容量 | 见 4.4 / 4.3 |
| **以团队为单位（Per-Team）** | 团队内所有成员的负载/容量之和 | `workload(team) = Σ_{r∈team} workload(r)`；`capacity(team) = Σ capacity(r)` |

团队利用率 = 团队 workload 总和 / 团队 capacity 总和。注意：同一资源可属于多个团队（跨项目），因此「团队负载」按成员关系展开求和时**不去重**资源——团队口径是「该团队负责的人力池被消耗了多少」，重复统计符合管理直觉（一个工程师被两个团队各算一次他的部分投入）。若需全局唯一视角，改用「以人为单位」并在看板上按资源去重展示。

### 4.7 实时性策略 / Real-time Strategy

桌面单用户、SQLite 本地库，数据量小，但利用率/甘特图在看板上是高频查询。推荐**三段式架构**（兼顾正确性与响应性）：

```
┌─ 领域事件（事件驱动写入） ─────────────────────────────┐
│  allocation/task/time_off/holiday 变更 → 发布事件          │
│        ↓ (async, tokio task)                                │
│  ┌─ 物化 workload 缓存表（materialized cache） ──────┐  │
│  │  workload_cache(scope, subject_id, period,         │  │
│  │                 capacity_pd, workload_pd, util,     │  │
│  │                 engine_rev, config_hash)            │  │
│  │  按「资源 × 周/月」粒度预算，事件触发增量重算         │  │
│  └─────────────────────────────────────────────────────┘  │
│        ↓                                                     │
│  视图层按需细查（on-demand drill-down）                    │
│  看板卡片读缓存；点开某资源时再算 [自定义区间] 的精确值     │
└────────────────────────────────────────────────────────────┘
```

#### 4.7.1 `workload_cache` 与 §8.9 `workforce_snapshots` 的关系

本节定义的 **`workload_cache`** 与 §8.9 的 **`workforce_snapshots`** 是**两张目标不同、互不替代**的表，文档在此明确二者的边界，避免命名混淆：

| 维度 | `workload_cache`（本节，原 `workload_snapshot`） | `workforce_snapshots`（§8.9） |
|------|--------------------------------------------------|-------------------------------|
| **定位** | 实时**热缓存**（hot cache） | 历史**冷归档**（cold archive） |
| **生命周期** | 短周期，可随时丢弃并全量重建 | 长周期，一次冻结永不修改 |
| **数据形态** | 聚合后的标量（capacity_pd / workload_pd / util） | payload 自包含的全量深拷贝 JSON |
| **自洽性** | 依赖当前引擎与配置；旧记录可由 `engine_rev`/`config_hash` 判定是否过期 | payload 自带 `engine_rev`，归档即不可变 |
| **触发** | 领域事件增量重算 | 定时/手工冻结 |
| **消费方** | Dashboard / 看板高频读 | R1–R7 报表的 `snapshot_id` 取数 |

> **命名变更说明**：原表名 `workload_snapshot` 与 §8.9 `workforce_snapshots` 极易混淆。本节将其更名为 **`workload_cache`**，强调其「可重建的实时热缓存」语义，与 `workforce_snapshots` 的「不可变冷归档」彻底区分。文档其余处对「§4.7 物化快照表」的引用，均指 `workload_cache`。

#### 4.7.2 `engine_rev` 与 `config_hash`：缓存自洽的关键

workload/capacity 的计算依赖 `work_week_template` / `holiday` / `time_off` / 单位配置等**会随版本演进的逻辑**。若缓存表只存结果而不存「它是用哪个版本的引擎与配置算出来的」，则引擎升级算法或配置变更后，旧缓存的 `utilization` 永远无法判定是否过期——这与 §4.7「增量重算」「视图读缓存」的自洽承诺直接冲突，也使 §9.4 开放问题 #19（失效范围算法）无从校验。

因此 `workload_cache` 必须记录两列指纹：

- **`engine_rev TEXT NOT NULL`**：workload 计算引擎的语义版本（如 `workload_engine@0.4.2`），对齐 §8.9 `workforce_snapshots.engine_rev` 与 §8.10 审计元数据。引擎升级后，`engine_rev` 变更，所有旧缓存行被判定为 stale，触发全量或增量重建。
- **`config_hash TEXT NOT NULL`**：影响计算的配置指纹 = 对 `(workweek_mask, holiday 集, 单位配置 pd_hours/pm_workdays, ...)` 做稳定哈希（如 SHA-256 over 规范化 JSON）。任一配置项变更 → `config_hash` 变 → 对应 subject+period 的缓存失效。
  > **impl 期锁定**：`config_hash` 的具体哈希输入规范化（**纳入字段集合**、**序列化顺序**、**是否含项目级 override**）需在实现时锁定，并固化为确定性规范：保证同一配置集合产出稳定哈希、不同配置集合产出不同哈希。锁定后写入 §4.7.5 注释与单测，不得随意改动（改动等同于一次 `engine_rev` 升级，会全表失效）。

读取缓存时的**过期判定**（伪逻辑）：

```
is_stale(row):
    return row.engine_rev  != CURRENT_ENGINE_REV
        or row.config_hash != current_config_hash(row.subject, row.period)
```

`is_stale` 为真的行在查询时直接回退到 §4.9 Rust 核心精算（并在后台异步重填缓存），从而保证「读缓存」永远返回与权威口径一致的结果。

#### 4.7.3 失效范围与增量重算（回应 §9.4 #19）

领域事件到达时，按**受影响的 (subject × period)** 粒度增量重算，而非按资源全量重算该周期，以控制写放大：

| 事件 | 受影响范围 | 重算动作 |
|------|-----------|----------|
| allocation 写入/删除 | 该 allocation 的 `resource_id` × 覆盖到的自然周/月 | 重算对应 `workload_cache` 行 |
| allocation `percent`/区间变更 | 旧区间与新区间分别覆盖的自然周/月并集 | 重算并集范围内行 |
| `time_off` 变更 | 该 `resource_id` × 变更日所在自然周/月 | 重算该资源该周期 |
| `holiday` 变更 | 所有 scope 命中项目下的全部资源 × 该日所在周期 | 批量重算（写放大较大，节假日变更低频，可接受） |
| 引擎/配置版本变更（`engine_rev`/`config_hash`） | 全表 | 标记全表 stale，后台分批重建（启动自检或手动「重建缓存」） |

全量重建作为兜底：启动自检或用户手动触发时，清空 `workload_cache` 后按当前 `engine_rev`+`config_hash` 全量回填，修复任何数据漂移。

> **写放大实测值待 impl 期量化（方案保留）**：上述按 `subject × period` 增量重算方案**保留**为最终实现方向，但其**实际写放大需在 impl 期实测并量化**，重点关注两类高频放大事件，以决定是否需做异步批量重算/合并：
>
> | 放大事件 | 影响面 | 量化目标 | impl 期决策动作 |
> |---------|--------|----------|----------------|
> | **allocation 区间变更（percent / start / end）** | 旧区间 ∪ 新区间覆盖到的所有自然周/月行的重算 | 单次变更触发的 `workload_cache` 重算行数（资源×周期） | 若行数过大，考虑合并同一事务内多次变更、或对该次变更走异步重算 |
> | **holiday 变更** | 全项目 × 全资源 × 该日所在周期（写放大最大） | 一次 holiday 变更触发的重算行数（项目数 × 资源数 × 周期数） | holiday 变更低频，若实测放大不可接受（如 > 数千行同步重算导致 IPC 卡顿），改为后台分批异步重算，读取时命中 stale 行回退 §4.9 精算 |
>
> 量化结果应回填到本节作为「写放大基线表」，并据此决定 holiday 变更是否走异步批处理。MVP 默认先按同步增量实现，待实测数据出来后再优化。

#### 4.7.4 机制对比与推荐组合

| 机制 | 何时触发 | 优点 | 缺点 |
|------|---------|------|------|
| **领域事件**（推荐主线） | allocation CRUD、time_off/holiday 变更 | 单一写入路径，可审计 | 需事件总线 |
| **物化缓存表 `workload_cache`**（推荐） | 事件到达时增量重算受影响 subject+period | 看板/报表 O(1) 读取，Tauri IPC 低延迟 | 需维护失效范围（见 §4.7.3） |
| 视图按需计算 | 用户切换区间/钻取 | 永远精确、无缓存陈旧 | 大区间计算耗时（仍 ms 级，可接受） |
| 全量重算 | 启动自检 / 手动「重建缓存」 | 简单、可修复数据漂移 | 不适合实时 |

**推荐组合**：`领域事件 → 增量更新 workload_cache（带 engine_rev/config_hash 自洽）→ 视图读缓存 + 钻取按需精算`。缓存表按「资源/团队 × 自然周/自然月」预聚合；任意自定义区间由视图层在 `[start,end]` 上即时计算（数据量小，单次 < 5ms）。

#### 4.7.5 `workload_cache` DDL

```sql
-- 实时热缓存（可重建）；与 §8.9 workforce_snapshots（冷归档）目标不同，详见 §4.7.1
CREATE TABLE workload_cache (
    id            INTEGER PRIMARY KEY,
    scope         TEXT NOT NULL CHECK (scope IN ('resource','team','project')),
    subject_id    INTEGER NOT NULL,
    period_start  TEXT NOT NULL,
    period_end    TEXT NOT NULL,
    capacity_pd   REAL NOT NULL,
    workload_pd   REAL NOT NULL,
    utilization   REAL NOT NULL,
    unit          TEXT NOT NULL DEFAULT 'PD',
    -- 自洽指纹：引擎版本 + 影响计算的配置指纹（见 §4.7.2）
    engine_rev    TEXT NOT NULL,                 -- 对齐 §8.9 workforce_snapshots.engine_rev
    config_hash   TEXT NOT NULL,                 -- workweek/unit/节假日 等配置的稳定哈希
    computed_at   TEXT NOT NULL,
    UNIQUE(scope, subject_id, period_start, period_end)
);
CREATE INDEX idx_wc_subject_period ON workload_cache(scope, subject_id, period_start);
CREATE INDEX idx_wc_rev_hash       ON workload_cache(engine_rev, config_hash);
```

> 读取层（Service / 视图）必须先做 §4.7.2 的 `is_stale` 判定：命中 stale 行时回退 §4.9 精算并异步回填，绝不直接返回过期数值。

### 4.8 单位切换的一致换算 / Unit Switching

用户在 UI 顶部切换 PD/PM 偏好后，全应用（看板、甘特、报表）需一致换算。规则：

1. **唯一数据源是 PD**：所有缓存表、allocation 字段存 PD。
2. **渲染层薄换算**：在 Vue 的一个全局 computed / Pinia getter `format(pd)` 里，按当前单位偏好调用 `pd_to_pm` 或原样输出，只做乘除一次。
3. **报表导出**：在表头标注单位列（`Unit: PM (1 PM = 20 PD)`），数值用同一 `format` 函数，保证 UI 与导出文件一致。
4. **阈值一致性**：过载阈值虽以百分比表达，但底层比较的是 PD，避免「切到 PM 后阈值看似变小」的错觉。

```ts
// 前端单位换算（Pinia store 片段）
const unit = useUnitStore()          // 'PD' | 'PM'
const cfg  = useUnitConfig()         // { hoursPerPd: 8, pdPerPm: 20 }

export function fmt(pd: number): string {
  if (unit.value === 'PM') return (pd / cfg.pdPerPm).toFixed(2) + ' PM'
  return pd.toFixed(1) + ' PD'
}
```

### 4.9 Rust 计算核心：签名与伪代码

将上述公式收敛为一个无副作用的纯计算核心，便于单测与可复现：

```rust
use chrono::{NaiveDate, Days};

pub struct Calendar {
    // 内含 work_week_template / holiday / time_off 的内存索引
    // key: (project_id, date) -> day_factor
}

impl Calendar {
    /// 某资源某一天的日历因子 ∈ [0,1]
    pub fn day_factor(&self, project_id: i64, resource_id: i64, d: NaiveDate) -> f64 { /* L1∧L2∧L3 */ }

    /// 重叠区间内各工作日因子的平均值（workload/capacity 同口径）
    pub fn avg_day_factor(&self, project_id: i64, resource_id: i64,
                          a_start: NaiveDate, a_end: NaiveDate,
                          w_start: NaiveDate, w_end: NaiveDate) -> f64 { /* .. */ }
}

pub struct Window { pub start: NaiveDate, pub end: NaiveDate }

/// 毛容量（不含投入比例），单位 PD
pub fn capacity_pd(cal: &Calendar, project_id: i64, resource_id: i64, w: Window) -> f64 {
    let mut sum = 0.0;
    let mut d = w.start;
    while d <= w.end {
        sum += cal.day_factor(project_id, resource_id, d); // 1.0=整天
        d = d.checked_add_days(Days::new(1)).unwrap();
    }
    sum // 直接即 PD（一天工作日 = 1 PD）
}

#[derive(Clone)]
pub struct Allocation {
    pub id: i64,
    pub resource_id: i64,
    pub project_id: i64,
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub percent: f64,          // 0.0–1.0
}

/// 单条 allocation 在窗口内折算的 PD
pub fn alloc_pd(cal: &Calendar, a: &Allocation, w: Window) -> f64 {
    let ov = overlap(a.start..=a.end, w.start..=w.end);     // (os, oe)
    let days = count_calendar_days(ov.0, ov.1) as f64;
    let avg = cal.avg_day_factor(a.project_id, a.resource_id, ov.0, ov.1, w.start, w.end);
    days * a.percent * avg
}

/// 资源在窗口的总 workload（跨项目叠加）
pub fn workload_pd(cal: &Calendar, allocs: &[Allocation], resource_id: i64, w: Window) -> f64 {
    allocs.iter()
        .filter(|a| a.resource_id == resource_id)
        .map(|a| alloc_pd(cal, a, w))
          .sum()
}

/// 利用率，>1.0 即过载
pub fn utilization(cal: &Calendar, allocs: &[Allocation],
                   project_id: i64, resource_id: i64, w: Window) -> f64 {
    let cap = capacity_pd(cal, project_id, resource_id, w);
    if cap <= 0.0 { return 0.0; }                            // 容量为 0 时定义 0
    let wl = workload_pd(cal, allocs, resource_id, w);
    wl / cap
}

/// 团队聚合
pub fn team_utilization(cal: &Calendar, allocs: &[Allocation],
                        members: &[i64], project_id: i64, w: Window) -> f64 {
    let (wl, cap) = members.iter()
        .map(|&r| (workload_pd(cal, allocs, r, w),
                   capacity_pd(cal, project_id, r, w)))
        .fold((0.0, 0.0), |(wl, cap), (w2, c2)| (wl + w2, cap + c2));
    if cap <= 0.0 { 0.0 } else { wl / cap }
}
```

> 该核心是 §4.7 `workload_cache` 的**唯一回填来源**：缓存行的 `capacity_pd`/`workload_pd`/`utilization` 必须由本节的 `capacity_pd`/`workload_pd`/`utilization` 产出，`engine_rev`/`config_hash` 由构建 `Calendar` 时的引擎与配置版本决定，确保缓存与精算永远同口径。

### 4.10 数值示例：某工程师跨两项目的一周

**设定（默认配置：1 PD = 8h，1 PM = 20 PD，Mon–Fri 工作周）**

- 资源 Alice（id=1），分析窗口 = 某自然周 Mon–Sun（2026-06-29 周一 ~ 2026-07-05 周日），共 5 个工作日（Wed 7-01 为法定节假日，全天；Thu 7-02 Alice 请半天假 `fraction=0.5`）。
- Alice 同时挂在两个项目：
  - **项目 A**：allocation 50%（`percent=0.5`），覆盖整周 Mon–Fri。
  - **项目 B**：allocation 60%（`percent=0.6`），覆盖 Tue–Fri。

**Step 1 — 各工作日的 `day_factor`（Alice 视角）**

| 日期 | 周几 | 工作周 | 节假日 | 请假 | `day_factor` |
|------|------|--------|--------|------|--------------|
| 06-29 | Mon | ✓ | — | — | 1.0 |
| 06-30 | Tue | ✓ | — | — | 1.0 |
| 07-01 | Wed | ✓ | 全天 | — | **0**（节假日） |
| 07-02 | Thu | ✓ | — | 半天 | **0.5** |
| 07-03 | Fri | ✓ | — | — | 1.0 |
| 07-04 | Sat | ✗ | — | — | 0 |
| 07-05 | Sun | ✗ | — | — | 0 |

有效工作日因子合计 = 1.0 + 1.0 + 0 + 0.5 + 1.0 = **3.5**。

**Step 2 — Capacity（毛容量，PD）**

`capacity = Σ day_factor = 3.5 PD`（= 3.5 × 8 = 28 工时）。注意 Alice 物理上本周只有 3.5 PD 可卖。

**Step 3 — Workload（按 allocation 折算，PD）**

逐日叠加两条 allocation：

| 日期 | `day_factor` | 项目A 50% | 项目B 60% | 当日 workload PD |
|------|-------------|-----------|-----------|------------------|
| Mon 06-29 | 1.0 | 1.0×0.5 = 0.5 | — | 0.5 |
| Tue 06-30 | 1.0 | 0.5 | 1.0×0.6 = 0.6 | 1.1 |
| Wed 07-01 | 0   | 0 | 0 | 0 |
| Thu 07-02 | 0.5 | 0.5×0.5 = 0.25 | 0.5×0.6 = 0.30 | 0.55 |
| Fri 07-03 | 1.0 | 0.5 | 0.6 | 1.1 |
| 周末 | 0 | 0 | 0 | 0 |

`workload = 0.5 + 1.1 + 0 + 0.55 + 1.1 = 3.25 PD`（= 26 工时）。

**Step 4 — Utilization**

`utilization = 3.25 / 3.5 = 0.9286 ≈ 92.9%` → 绿色，未过载（< 100%）。

**Step 5 — 单位切换展示**

- PD：capacity 3.5 PD，workload 3.25 PD。
- PM（÷20）：capacity 0.175 PM，workload 0.1625 PM。
- 工时（×8）：capacity 28 h，workload 26 h。

利用率与单位无关，始终 92.9%。

**Step 6 — 过载反例**

若项目 B 投入比例从 60% 提到 **100%**：重算 Thu/Fri/Tue 后 workload = 0.5 + (0.5+1.0) + 0 + (0.25+0.5) + (0.5+1.0) = 4.25 PD → `4.25/3.5 = 121.4% > 110%` → **红色过载**，AI 求解器将对该解施加软约束惩罚，提示「Alice 在 Thu/Fri 双项目叠加超出物理容量，建议下调项目 A 或 B 的投入比例，或将项目 B 部分窗口移至下周」。

此例验证了模型的三条核心性质：①日历扣减同时作用于分子分母（同口径）；②跨项目 allocation 直接相加即得过载判断；③单位切换不影响利用率，仅影响展示数值。

**Step 7 — 口径一致性校验（验证 §4.4.1）**

> 本步用于自检「Rust 精算 vs. `allocated_pd` 冗余列 vs. `allocation_daily` 物化」三者口径一致，避免 Dashboard 与优化器给出不同数字。

- **`allocation_daily` 物化（可选加速路径）**：把 Step 3 的逐日 workload 落到 `allocation_daily`，按窗口 `SUM(pd)` = `0.5+1.1+0+0.55+1.1 = 3.25 PD`，与 §4.9 `workload_pd` 逐日累加结果**完全一致**。
- **`allocations.allocated_pd`（自身区间全值）**：项目 A 的 `allocated_pd = alloc_pd(A, [Mon..Fri]) = 5 个日历日中 3 个整天 + 1 个半天（Thu）= (1+1+0+0.5+1)×0.5 = 3.5×0.5 = 1.75 PD`；项目 B 的 `allocated_pd = (Tue..Fri) 内 (1+0+0.5+1)×0.6 = 2.5×0.6 = 1.5 PD`。
  - 若错误地 `SUM(allocated_pd) = 1.75 + 1.5 = 3.25 PD`，**本例碰巧相等**——仅因为聚合窗口 == 两条 allocation 的自身区间。
  - **反例**：若聚合窗口缩小为 `Mon..Thu`（仅 4 天），正确 workload（§4.9）= `0.5+1.1+0+0.55 = 2.15 PD`；而 `SUM(allocated_pd)` 仍是 `3.25 PD`（因为它存的是全区间值，不含窗口 overlap 折算）→ **数值错误，高估 51%**。这正说明 `allocated_pd` 不可用于跨窗口聚合，必须走 §4.9 或 `allocation_daily`。

### 4.11 本章假设

> 本节为 §4 的假设内聚视图；如与文末「假设汇总」冲突，以文末汇总为准，文末汇总应在下次修订时同步对齐本节。

1. **单一时区 / 单一工作日历地区**（对应假设 #24）：整个组织运行在同一时区、同一套工作日历语义下，`day_factor` 不按资源归属地区分别解析；跨时区团队（如上海+硅谷）为非目标，已纳入 §1.5。若未来启用需重大版本升级（见 §4.2.1 四步变更清单，含 `region`/`timezone` 列、按 `resource.region` 解析日历、`workload_cache` 全量 `engine_rev` 迁移、§4.3/§4.4 口径重评）。
2. **唯一权威计算源是 §4.9 Rust 纯函数核心**（对应假设 #25）：按日 overlap + `day_factor`，无副作用、可复现；`allocated_pd`、`workload_cache`、`allocation_daily`、§3.6 SQL 聚合均为派生/缓存，口径须向 §4.9 对齐。
3. **`allocations.allocated_pd` 的语义边界**（对应假设 #26）：严格等于 `alloc_pd(a, [a.start, a.end])`，不含 overlap 折算与窗口内 `avg_day_factor`；仅用于单条 allocation 的工作量展示（Gantt 条带/任务总投入/预算消耗粗算），**不可**用于跨窗口 workload/utilization 聚合。
4. **`workload_cache` 是实时热缓存**（对应假设 #27）：短周期、可重建、依赖当前引擎与配置；与 §8.9 `workforce_snapshots`（冷归档、长周期、payload 自包含、不可变）目标不同、互不替代。
5. **`workload_cache` 双指纹自洽**（对应假设 #28）：通过 `engine_rev` + `config_hash` 两列指纹判定 stale；引擎升级或 workweek/holiday/单位配置变更即失效，读取时回退 §4.9 精算并异步回填。
6. **`allocation_daily` 数学等价性**（对应假设 #29）：每行 `pd = day_factor(day) × percent`，SUM 后与 §4.9 Rust 逐日累加数学等价；其**是否纳入 MVP 为 impl 期决策**（见 §4.12 开放问题），不引入则统一走 §4.9 精算。

### 4.12 本章开放问题

> 本节为 §4 当前仍需在落地/实现期解决的开放项。已被本轮决策收敛到正文「impl 期锁定/量化」标注的，不再列为开放问题（见文末汇总 #19/#20/#21 已解决）。

1. **（impl 期量化）`workload_cache` 写放大实测**：§4.7.3 的 `subject × period` 增量重算方案已保留为最终方向，但需在 impl 期量化 **allocation 区间变更（旧/新区间并集）** 与 **holiday 变更（全项目×全资源）** 两类事件的实测写放大行数，据此决定 holiday 变更是否走异步批量重算。量化结果回填为「写放大基线表」。此为本轮决策 #1 的落地尾项，**不再是「方案选型」级开放问题**。
2. **（跨节，§9 范围）利用率口径单一真相源的执行校验**：§9 假设 #65 已拍板「Dashboard/看板/报表三处强制复用 §4.9 聚合函数」，本章公式已对齐；落地时需在 §7 Dashboard / §8 报表实现中校验三处未各自另立公式（属 §7/§8 范围，本章仅提供权威函数签名）。

> **本轮已解决的开放问题（自文末汇总移出，不再追踪）**：
> - 原开放问题 #19（跨时区/跨地区日历）：维持非目标，未来启用需重大版本升级（§4.2.1 变更清单已落定）。
> - 原开放问题 #20（`allocation_daily` 是否进 MVP）：标为可选加速路径，**impl 期在 Dashboard 实测读延迟后决定**；不引入则统一走 §4.9 精算（§4.4.1 已标注）。
> - 原开放问题 #21（`config_hash` 哈希输入规范化）：**impl 期锁定**纳入字段/序列化顺序/是否含项目级 override（§4.7.2 已标注）。

## 5. AI 优化引擎 / AI Optimization Engine

AI 优化引擎是系统的「大脑」，负责把「候选资源 + 待分配任务 + 容量日历 + 已有分配」求解为一份可执行的人力配置建议。本章采用 **混合管线（Hybrid Pipeline）** 设计：经典优化器求解硬约束、LLM 负责语义打分与方案解释，二者解耦，可独立替换、降级与复现。

> **本章决策落定（贯穿全章）**：
> 1. **MILP 单位闭合（与 §3.8 对齐）**：统一采用「连续投入比例 `percent_{r,t,d} ∈ [0,1]` 为主变量 + 0/1 指示变量 `x_{r,t,d} ∈ {0,1}`」的耦合形式，见 §5.5.1。容量约束在**比例空间**表达：`Σ_t percent_{r,t,d} ≤ cap_{r,d}`，其中 `cap_{r,d} = day_factor(d, r) ∈ [0,1]`。
> 2. **`input_snapshot_json` 体积**：向量快照默认采用「向量内容哈希 + 向量旁表」或「压缩序列化」两种候选，**impl 期评估**后定其一（见 §5.8.1）。
> 3. **`run_id` 维持 `INTEGER` 自增**（即 `ai_optimization_runs.id`），本轮**不引入 UUID**。
> 4. **多目标权重 `ObjectiveWeights` 由用户在 UI 调节**，写入目标函数与 `weights_json`（见 §5.8.3）。
> 5. **降级链**：LLM 不可用 → `TemplateExplainer`（规则模板）；embedding 不可用 → `FallbackScorer`（关键词/熟练度）。
> 6. **ILP 后端默认采用 `good_lp` + `HiGHS`**（静态链接）。

> **表名约定（贯穿全章）**：本章统一使用 §3.3.16 定义的持久化表 `ai_optimization_runs`（`INTEGER PRIMARY KEY AUTOINCREMENT`，与 `allocations.run_id INTEGER REFERENCES ai_optimization_runs(id)` 类型一致）。本章内存中的运行标识 `run_id` 在落库时即对应 `ai_optimization_runs.id`（INTEGER）。本轮已**拍板维持 INTEGER 自增、不引入 UUID**，故本章与 §3.3.15 / §3.3.16 全链路类型保持一致，不另起 `ai_run` 表（详见 5.7）。若未来确需全局可追溯的 UUID，须回改 §3.3.15 / §3.3.16 全链路类型（INTEGER→TEXT）。

### 5.1 设计目标与原则

| 原则 | 含义 | 落地手段 |
| --- | --- | --- |
| **可复现 (Reproducible)** | 相同输入 + 相同配置产生相同结果 | 固定随机种子；记录 provider/model/输入快照到 `ai_optimization_runs` 表（§3.3.16） |
| **可约束 (Constrainable)** | 业务硬规则不可被 AI 破坏 | 硬约束由 MILP/匹配求解器强制；LLM 输出绝不直接改写容量与冲突结果 |
| **可解释 (Explainable)** | 用户能看懂「为什么这样分」 | LLM 产出自然语言说明 + 结构化风险点；超时降级为模板化解释 |
| **可干预 (Human-in-the-loop)** | 方案是「建议」而非「命令」 | 用户可接受/微调/锁定/拒绝；落库标注 `source=ai, run_id, status` |
| **离线优先 (Offline-first)** | 本地 Ollama 即可运行；云端可选 | rig 抽象 provider；LLM/embedding 均可切本地/云 |

> **可复现性的边界（与 §3.11 / §5.8.1 对齐）**：经典优化解（贪心/匈牙利/MILP）在固定 seed + 固定求解器版本下**确定可复现**。语义打分依赖 embedding 向量，属于「条件可复现」——详见 5.8.1 对 embedding 向量快照的约定。

### 5.2 混合管线总览

管线分 5 个阶段，前一阶段产物即后一阶段输入，每阶段都可缓存与单独重跑。

```
┌─────────────┐   ┌────────────────┐   ┌──────────────────┐   ┌──────────────┐   ┌───────────────┐
│ (a) 输入构建 │──▶│ (b) 语义匹配打分 │──▶│ (c) 经典优化求解  │──▶│ (d) LLM 解释 │──▶│ (e) 人机闭环  │
│ DB → Problem │   │ rig embedding   │   │ good_lp / 贪心   │   │ rig chat     │   │ 建议→落库     │
└─────────────┘   └────────────────┘   └──────────────────┘   └──────────────┘   └───────────────┘
        │                  │                     │                    │
        └──────── 输入快照/种子/profiler 记录到 ai_optimization_runs（贯穿全程，保证复现）────────┘
```

**降级链（本轮已确认）：**

- `(b) embedding 不可用 → 退化为 `FallbackScorer`（tag 关键词精确匹配 + 熟练度线性加权打分）`。触发条件见 5.4 的 `embed_batch` 契约（provider 报错 / 维度异常 / 空批量）。
- `(c) 超时 → 退化为贪心初解`；`MILP 返回 infeasible → 松弛重解（见 5.5.1 / 5.8.4）`。
- `(d) LLM 不可用 → 退化为 `TemplateExplainer`（规则模板，零 LLM 依赖，永远可用）`。

任一阶段失败都不阻塞用户拿到「可用的纯经典解 + 模板解释」。

### 5.3 阶段 (a)：输入构建

从 SQLite 读取候选数据，组装成纯数据结构 `AllocationProblem`。此阶段只做 **投影与快照**，不做业务判断，保证输入稳定可复现。

**读取范围：**

| 数据 | 来源表 | 用途 |
| --- | --- | --- |
| 候选 resources | `resources` JOIN `resource_skills` JOIN `resource_tags` | 语义匹配的「资源侧」向量源 |
| 待分配 tasks | `tasks` JOIN `task_skill_requirements` JOIN `task_tags` | 任务侧需求、时间窗、工作量 |
| 容量日历 | `resource_unavailable` / `work_week_template` / `holiday`（按 PD/日预计算，已扣除节假日/请假/非工作日） | 硬约束的容量上限（比例空间 `day_factor` 序列） |
| 已有 allocations | `allocations`（含已锁定项 `status=locked`） | 避免冲突；锁定项不可被重排 |

```rust
/// 一次优化求解的完整输入快照（不可变，用于复现）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct AllocationProblem {
    pub run_id: i64,                       // 落库对应 ai_optimization_runs.id（INTEGER 自增，本轮拍板维持 INTEGER）
    pub unit: EffortUnit,                 // PD 或 PM（影响数值换算）
    pub resources: Vec<CandidateResource>,
    pub tasks: Vec<CandidateTask>,
    pub capacity: CapacityMatrix,         // [resource_id][day_index] -> day_factor(d,r) ∈ [0,1]，比例空间容量
    pub locked: Vec<Allocation>,          // 已锁定分配，求解时视为固定占用
    pub config: SolverConfig,             // 约束开关、权重、种子
    pub snapshot: InputSnapshot,          // provider/model/种子/DB 行哈希（向量快照见 5.8.1）
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CandidateResource {
    pub id: i64,
    pub name: String,
    pub skills: Vec<SkillLevel>,          // { skill, level: 0..=5 }
    pub tags: Vec<String>,
    pub home_team_id: Option<i64>,
    pub max_projects: u32,                // 跨项目上限（软/硬可配）
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CandidateTask {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub required_skills: Vec<SkillRequirement>, // { skill, min_level }
    pub tags: Vec<String>,
    pub effort_pd: f64,                   // 工作量（人日），用于工作量软目标
    pub window: TimeWindow,               // { start, end } 天索引
    pub priority: u8,                     // 1=最高
    pub depends_on: Vec<i64>,             // 任务依赖
    pub segmentable: bool,                // 是否为长期任务（可分段排期）
}

#[derive(Debug, Clone, Copy)]
pub enum EffortUnit { PersonDay, PersonMonth }

/// 求解配置：所有可调旋钮集中在此，便于复现与用户偏好持久化。
/// ObjectiveWeights 由用户在 UI（约束面板滑块）调节，写入 weights_json（见 5.8.3）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct SolverConfig {
    pub seed: u64,
    pub strategy: SolveStrategy,          // GreedyMatch / Milp / Hybrid（默认）
    pub constraints: ConstraintFlags,
    pub weights: ObjectiveWeights,        // 多目标权重（UI 可调），落库到 weights_json
    pub timeouts: Timeouts,               // 各阶段超时（ms）
    pub milp_var_threshold: usize,        // R·T·D 低于此值才进 MILP（默认 20_000）
}

#[derive(Debug, Clone, Copy, serde::Serialize, Default)]
pub struct ConstraintFlags {
    pub enforce_capacity: bool,           // 默认 true（硬）：比例空间 Σ_t percent_{r,t,d} ≤ cap_{r,d}（=day_factor）
    pub enforce_time_window: bool,        // 默认 true（硬）
    pub enforce_no_conflict: bool,        // 默认 true（硬）：见 5.5.1，区分「并行数」与「容量」两种语义
    pub enforce_min_skill_level: bool,    // 默认 true（硬）
    pub enforce_dependencies: bool,       // 默认 true（硬）
    pub balance_workload: bool,           // 默认 true（软）
    pub prefer_cross_project_reuse: bool, // 默认 true（软）
    /// 每资源每日并行任务上限。None = 不限并行，仅受 enforce_capacity（比例之和 ≤ day_factor）约束，
    /// 与 §3.8 硬约束清单口径一致；Some(1) = 禁止同日多任务并行。默认 None。
    pub max_parallel_tasks_per_day: Option<u32>,
}
```

`CapacityMatrix` 存的是 `day_factor(d, resource) ∈ [0,1]`（比例空间容量），而非 PD 值。PD 仅在结果展示与跨资源加总时按 `pd = percent × daily_capacity_pd × day_factor` 折算（见 §3.8）。`InputSnapshot` 记录 `embedding_provider/model`、`chat_provider/model`、`seed`、关键表的行数与内容哈希，并持久化当时的 embedding 向量快照（见 5.8.1，向量体积方案 impl 期评估），写回 `ai_optimization_runs` 表，使一次 run 在未来可被完整重放。

### 5.4 阶段 (b)：语义匹配打分

用 rig 调 embedding 模型，把 resource 的 `skills + tags` 与 task 的 `required_skills + tags` 向量化算余弦相似度，产出 `match_score ∈ [0,1]`，再叠加 **熟练度等级** 与 **tag 精确命中** 的加成。

> **版本假设**：基于 `rig-core ≈ 0.24.x`（截至 2026-06 仍在活跃演进，README 明确警告后续会有 breaking change）。代码以 `EmbeddingModel` trait + `EmbeddingsBuilder` 为准；落地时以 `docs.rs/rig-core` 当前版本为准锁定一个次版本号。

> **Ollama embedding 已知坑（rig issue #1082）**：`rig::providers::ollama::Client` + `EmbeddingsBuilder` 组合在使用时有陷阱——必须 import `EmbeddingsClient`（具体 trait 类型）而非其 `dyn` 版本，且单次 batch 的向量维度必须与目标模型 `ai_embed_dim`（§3.3.1 settings）一致。正确路径：走 `EmbeddingsBuilder::new(model).documents(..).build().await`，或直接对单个文本调用 `EmbeddingModel::embed(text)`（更可控、便于自封装分批）。详见 5.4 的 `embed_batch` 契约。

**打分公式（可配置权重）：**

$$
\text{match\_score}(r, t) =
\underbrace{\cos(\vec{r}, \vec{t})}_{\text{语义相似度 } s_{\text{sem}}}
\cdot \big(1 + \alpha \cdot \overline{\text{level}}(r,t)\big)
+ \beta \cdot \frac{|\text{tags}_r \cap \text{tags}_t|}{|\text{tags}_t|}
$$

- $s_{\text{sem}} \in [-1,1]$，归一化到 $[0,1]$（`0.5 + 0.5*cos`）。
- $\overline{\text{level}}(r,t)$：resource 在 task 所需技能上的平均熟练度（0..5 归一化到 0..1）。
- $\beta$：tag 精确命中加权（默认 0.15）。
- 默认 $\alpha = 0.2$，全部可在 `ObjectiveWeights` / `ScoringWeights` 中配置。

**数值示例：** task 需求 `{Rust(min 3), 前端(min 2)}`，tags `{high-perf, web}`。
- resource A：skills `{Rust:4, 前端:3}`，tags `{high-perf, web, senior}`。
  - $s_{\text{sem}}=0.88$，$\overline{\text{level}}=(4+3)/2 /5 = 0.70$，tag 命中 $2/2=1.0$。
  - score $= 0.88 \times (1+0.2\times0.70) + 0.15\times1.0 = 0.88\times1.14 + 0.15 = 1.153$ → 截断归一化到 1.0。
- resource B：skills `{Rust:2（不达标）, 前端:4}`，tags `{web}`。
  - 因 `enforce_min_skill_level=true` 且 Rust=2 < 3，**硬约束直接排除**，不进入打分（`match_score = -∞`）。

```rust
/// 语义打分器：依赖 rig 的 EmbeddingModel（本地 Ollama 或云端）。
pub struct SemanticScorer<M: rig::embeddings::EmbeddingModel> {
    model: M,
    weights: ScoringWeights,
    cache: EmbeddingCache,           // 文本→向量 LRU，避免重复调用
}

#[async_trait]
pub trait Scorer: Send + Sync {
    /// 对所有 (resource, task) 对打分，返回评分矩阵。
    async fn score(
        &self,
        problem: &AllocationProblem,
    ) -> Result<ScoreMatrix, ScorerError>;
}

impl<M: rig::embeddings::EmbeddingModel> Scorer for SemanticScorer<M> {
    async fn score(&self, problem: &AllocationProblem) -> Result<ScoreMatrix, ScorerError> {
        // 1. 文本化：把 skills/tags 拼成语义串
        let res_texts: Vec<String> = problem.resources.iter()
            .map(textualize_resource).collect();
        let task_texts: Vec<String> = problem.tasks.iter()
            .map(textualize_task).collect();

        // 2. 批量嵌入（rig 统一 API，换 provider 只改 model 构造）
        //    embed_batch 内部按模型 max batch 分块，并对返回维度做校验（见下方契约）。
        let res_vecs = self.embed_batch(&res_texts).await?;
        let task_vecs = self.embed_batch(&task_texts).await?;

        // 3. 余弦相似度 + 熟练度/tag 加成 + 硬约束排除
        let mut scores = ScoreMatrix::default();
        for (i, r) in problem.resources.iter().enumerate() {
            for (j, t) in problem.tasks.iter().enumerate() {
                if !meets_min_skill(r, t) && problem.config.constraints.enforce_min_skill_level {
                    scores.set(i, j, f64::NEG_INFINITY); // 硬排除
                    continue;
                }
                let sem = cosine(&res_vecs[i], &task_vecs[j]).clamp(0.0, 1.0);
                let lvl = avg_level_norm(r, t);
                let tag_hit = tag_overlap(r, t);
                let s = sem * (1.0 + self.weights.alpha * lvl)
                       + self.weights.beta * tag_hit;
                scores.set(i, j, s.min(1.0).max(0.0));
            }
        }
        Ok(scores)
    }
}
```

**`embed_batch` 契约（自封装，必须遵守）：**

1. **分块**：单次 batch 受模型最大批量与 `ai_embed_dim` 限制；输入超过模型 `max_batch` 时必须分块调用 `EmbeddingModel::embed` 或分批 `EmbeddingsBuilder`，禁止一次性塞入全部文本（否则 provider 截断或报错）。
2. **维度校验**：每次返回的向量必须满足 `vec.len() == settings.ai_embed_dim`。**一旦不符**（如换了模型、provider 返回异常维度），**视为该 provider 不可用**，立即走降级链 → `FallbackScorer`。
3. **降级触发条件**（覆盖「软失败」）：(i) provider 调用报错；(ii) 返回向量维度 ≠ `ai_embed_dim`；(iii) 返回空或长度不齐的 batch。三者任一命中即降级，**不要把脏向量喂给余弦计算**。

```rust
impl<M: rig::embeddings::EmbeddingModel> SemanticScorer<M> {
    /// 把一批文本分块嵌入，校验维度后返回。任一批维度不符则返回 Err（由 score() 走 FallbackScorer）。
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f64>>, ScorerError> {
        let mut out = Vec::with_capacity(texts.len());
        for chunk in texts.chunks(self.model.max_batch().max(1)) {
            // 走 EmbeddingModel::embed 逐条或 EmbeddingsBuilder 分批；此处以逐条示例
            for t in chunk {
                let v = self.model.embed(t).await?;
                if v.len() != self.expected_dim {
                    return Err(ScorerError::DimMismatch {
                        expected: self.expected_dim, got: v.len(),
                    }); // 触发降级
                }
                out.push(v);
            }
        }
        Ok(out)
    }
}
```

**rig provider 切换要点（一行换后端）：**

```rust
// 本地优先：Ollama（embedding 模型如 nomic-embed-text / bge-m3）
let client = rig::providers::ollama::Client::from_env()?;
let emb_model = client.embedding_model("nomic-embed-text");

// 可选云端：OpenAI（换 provider 不改业务逻辑）
let client = rig::providers::openai::Client::from_env()?;
let emb_model = client.embedding_model(rig::providers::openai::TEXT_EMBEDDING_3_SMALL);
```

> 注意：Ollama 本地 embedding 的维度随模型变化（如 `nomic-embed-text` 为 768 维），落库缓存需按 `(provider, model, text_hash)` 作为 key，避免跨模型误用向量。

**降级（`FallbackScorer`，本轮确认）：** embedding 不可用（provider 报错 / 返回维度异常 / 批量报错）时，退化为「tag 关键词精确 Jaccard 相似度 + 熟练度线性加权」，公式为 $s = \beta\cdot\text{Jaccard} + (1-\beta)\cdot\overline{\text{level}}$，保证管线不中断。`FallbackScorer` 与 `SemanticScorer` 实现同一 `Scorer` trait，引擎侧 `Arc<dyn Scorer>` 透明替换。

### 5.5 阶段 (c)：经典优化求解

把「资源 × 任务 × 时间」建模为 **带硬约束的指派/装箱问题**。默认策略 `Hybrid`：先用贪心 + 匈牙利匹配求快速初解，小规模直接返回；规模超过阈值或用户要求「精化」时再用 MILP（**默认 good_lp + HiGHS**，静态链接）重解。

#### 5.5.1 变量与约束建模（MILP 形式，单位闭合已落定）

**决策变量（本轮拍板的耦合形式，与 §3.8「比例口径」对齐）：**

- 连续主变量 `percent_{r,t,d} ∈ [0,1]`：资源 $r$ 在第 $d$ 天对任务 $t$ 的**投入比例**（即 `allocation.percent` 的逐日值）。
- 0/1 指示变量 `x_{r,t,d} ∈ {0,1}`：资源 $r$ 在第 $d$ 天**是否**对任务 $t$ 有任何投入（`x=1 ⇔ percent_{r,t,d} > 0`）。
- 聚合变量 `y_{r,t} ∈ [0,1]`：资源 $r$ 对任务 $t$ 的**总投入比例**（用于目标函数中的「匹配质量」加权）。

> **为何采用「连续 `percent` + 0/1 `x`」而非纯 0/1 `x`**：§3.8 的容量上限是「任一有效工作日 `Σ percent ≤ 1.0`」的**比例口径**。若只用 0/1 变量 `x_{r,t,d}`，则容量约束退化为「同日并行任务数 ≤ 某整数」，无法表达「一个资源同日以 50% + 30% 分给两个任务」的合法且常见场景。故以连续 `percent_{r,t,d}` 承载投入强度、以 `x_{r,t,d}` 承载「是否投入」语义，二者经耦合约束闭合单位。

**耦合约束（单位闭合的核心，与 §3.8 对齐）：**

| 耦合关系 | 表达式 | 含义 |
| --- | --- | --- |
| 指示 ↔ 强度（下界） | $\text{percent}_{r,t,d} \le x_{r,t,d}$ | `x=0 ⇒ percent=0`（未投入则比例为 0） |
| 指示 ↔ 强度（上界） | $\text{percent}_{r,t,d} \ge \epsilon \cdot x_{r,t,d}$（$\epsilon$ 取投入比例最小步进，见开放问题 #50） | `x=1 ⇒ percent ≥ 最小步进`（投入则至少占一个步进） |
| 聚合总投入 | $y_{r,t} = \dfrac{\sum_{d} \text{percent}_{r,t,d}}{\lvert W_t \rvert}$，其中 $\lvert W_t \rvert$ 为任务 $t$ 时间窗内**有效工作日数**（`day_factor>0` 的天数） | 任务级总投入比例 = 日均投入比例（与 §3.8 的 `y_{r,t}` 语义一致） |

> 与 §3.8「单位闭合」说明完全一致：`y_{r,t} = Σ_d x_{r,t,d} × (1 / 该任务有效工作日数)` 是 `x`-only 的等价表达；本节采用更强的 `percent` 连续主变量形式（能表达部分投入），二者在「每日投入比例恒定」时退化等价。容量约束统一改写为连续形式（见下表）。

**硬约束（必须满足，违反即不可行）：**

| 约束 | MILP 表达（比例空间） | 含义 |
| --- | --- | --- |
| **容量上限（比例口径，与 §3.8 硬约束 1/3 一致）** | $\sum_{t} \text{percent}_{r,t,d} \le \text{cap}_{r,d} \quad \forall r,d$，其中 $\text{cap}_{r,d} = \text{day\_factor}(d, r) \in [0,1]$ | 单日投入比例之和不超过该日有效容量比例；非工作日 `cap=0` 天然禁止排期 |
| 时间窗 | $\text{percent}_{r,t,d}=0 \quad \forall d \notin [t.\text{start}, t.\text{end}]$ | 任务只能在窗内排期 |
| 并行上限 | $\sum_{t} x_{r,t,d} \le k \quad$（仅当 `max_parallel_tasks_per_day = Some(k)`） | 同日同资源并行任务数上限；**默认 None = 不限并行，仅受上一行「容量上限」约束** |
| 最低熟练度 | $\text{percent}_{r,t,d}=0 \text{ 若 } \text{level}(r,t.s)<t.\text{min}$ | 技能不达标不可分配（已在打分阶段排除） |
| 依赖顺序 | $\max_d(x_{r,t_2,d}) \ge \min_d(x_{r,t_1,d}) + \text{gap}$ | $t_1$ 先于 $t_2$（$t_1 \in t_2.\text{deps}$） |
| 锁定项不可动 | $\text{percent}_{r,t,d}=\text{locked\_percent} \quad \forall (r,t,d)\in\text{locked}$ | 已锁定分配固定不变（其 `percent` 进入容量占用） |

> **关于「并行上限」与「容量上限」的区分**：本节明确把两者拆开——「容量上限」是 §3.8 的硬约束（**比例之和** `Σ_t percent_{r,t,d} ≤ day_factor(d,r)`），始终启用；「并行上限」是可配置的 `max_parallel_tasks_per_day`（默认 `None` = 不限并行，仅受容量约束），仅在用户/项目明确要求「禁止同日多任务」时设为 `Some(1)`。默认值 `None` 与 §3.8 口径一致。

> **容量约束为何用 `day_factor` 而非 `daily_capacity_pd`**：`daily_capacity_pd` 是 PD 展示折算（如兼职 0.5），不是过载阈值（§3.8 已明确阈值恒为比例 1.0）。MILP 在比例空间求解：`cap_{r,d} = day_factor(d,r)`，兼职资源的产出 PD 由 `percent × daily_capacity_pd × day_factor` 在结果折算阶段自然得到，不进入求解器。这避免了「比例（0–N）vs PD（常 1.0）」的单位混用。

> **关于「工作量满足」（由硬约束改为带松弛的软目标）**：原写法把「$\sum_{r,d} \text{percent}_{r,t,d}\cdot \text{day\_factor}(d)\cdot 1\text{PD} \ge t.\text{effort\_pd}$」列为硬约束，与容量硬约束同处一个 MILP 时，若某任务在可用容量内根本无法满足（资源不足/窗太短），整问题无可行解，求解器返回 `infeasible`，整条管线给出「全空解」——这是真实数据上最容易塌掉的地方。故改为：引入松弛变量 $u_t\ge 0$（未排满工作量），把工作量满足建模为**最大化已排工作量**的软目标，缺口计入 `SolutionMetrics.unscheduled_tasks`：
>
> - 软目标项：$w_e \cdot \sum_t \min(\sum_{r,d} \text{percent}_{r,t,d}\cdot \text{day\_factor}(d,r)\cdot 1\text{PD},\ t.\text{effort\_pd}) / t.\text{effort\_pd}$（已排工作量比例，最大化）。
> - 缺口：$\text{gap}_t = \max(0,\ t.\text{effort\_pd} - \sum_{r,d} \text{percent}_{r,t,d}\cdot \text{day\_factor}(d,r)\cdot 1\text{PD})$，写入 metrics。
>
> 若出于业务需要坚持保留为硬约束，则 **MILP 返回 `infeasible` 时自动降级**：把该约束松弛为软目标重解（见 5.8.4），**绝不返回空解**。

**软目标（目标函数最大化，权重来自用户在 UI 调节的 `ObjectiveWeights`）：**

$$
\max \;
w_m \cdot \underbrace{\sum_{r,t} y_{r,t}\cdot \text{match\_score}_{r,t}}_{\text{匹配质量}}
\;+\; w_e \cdot \underbrace{\sum_t \frac{\min(\sum_{r,d} \text{percent}_{r,t,d}\cdot \text{day\_factor}_{d,r}\cdot 1\text{PD},\ t.\text{effort\_pd})}{t.\text{effort\_pd}}}_{\text{已排工作量比例（原硬约束松弛）}}
\;+\; w_b \cdot \underbrace{(-\text{Var}(\text{load}_r))}_{\text{负载均衡}}
\;+\; w_p \cdot \underbrace{\sum_t \mathbb{1}[\text{scheduled}_t]/t.\text{priority}}_{\text{优先级覆盖}}
\;+\; w_c \cdot \underbrace{\sum_r \mathbb{1}[\text{跨项目数}\ge 2]}_{\text{跨项目复用}}
$$

其中负载均衡用「各资源总 load 的方差取负」线性化为偏差绝对值之和（$-\sum_r|load_r-\overline{load}|$）以保持线性性。$w_m, w_e, w_b, w_p, w_c$ 由用户在 UI「约束面板」调节（滑块/预设），落库到 `ai_optimization_runs.weights_json`，下次 run 生效（见 5.8.3）。

#### 5.5.2 求解器选型对比与推荐

> **本轮拍板：ILP 后端默认采用 `good_lp` + `HiGHS`**（静态链接进 Tauri 二进制）。二进制体积不设上限（开放问题 #7 已确认「没限制」），故无需为体积牺牲求解质量。

**变量规模公式与适用边界：**

> MILP 决策变量数 $N \approx 2 \cdot R \cdot T \cdot D$（`percent` 与 `x` 各一份）+ $R\cdot T$（`y`）。「贪心+匈牙利」不展开到天，只处理「资源×任务」匹配对 $P = R\cdot T$，故其适用规模远大于 MILP。

| 规模量级 | 变量数（举例） | 推荐路径 |
| --- | --- | --- |
| 极小 | $R\cdot T < 500$（如 10 资源 × 50 任务，§1.6 验收规模） | 贪心 + 匈牙利匹配，ms 级出初解直接返回 |
| 中 | $R\cdot T\cdot D < $ `milp_var_threshold`（默认 20_000） | Hybrid：贪心初解后用 MILP 精化 |
| 大 | $R\cdot T\cdot D \ge 20\,000$（如 §2.4 的 50×200×65=650_000） | **不进单次 MILP**；按 project / team 分块，逐块贪心+匈牙利，跨块容量用 Lagrangian 松弛协调 |

> **规模与变量数（重要）**：MILP 决策变量规模为 $N \approx 2 \cdot R \cdot T \cdot D$（资源数 × 任务数 × 天数，`percent`+`x`）。以 §2.4 示例「50 资源 × 200 任务」按季度（≈65 工作日）展开 → $N \approx 1{,}300{,}000$ 个变量，远超 good_lp/HiGHS 在 §5.8.4 「60s（大）」预算内可解的量级。因此 **Hybrid 默认先用贪心 + 匈牙利，仅在 $R\cdot T\cdot D < $ 阈值（`SolverConfig.milp_var_threshold`，默认 20_000）时才进 MILP；超过阈值则按 project/team 分块（§5.5.2 第 3 点）**。§2.4 的「50×200」示例应理解为「需分块/分窗求解，而非单次 MILP」。

| 维度 | 贪心 + 匈牙利匹配 | good_lp (MILP, HiGHS 后端) |
| --- | --- | --- |
| 求解速度 | 极快（O(n³) 匈牙利，ms 级） | 中大规模较快（秒级）；小问题有建模开销 |
| 最优性 | 贪心次优；匈牙利对「一对一匹配」最优 | 全局最优（在建模精度内） |
| 硬约束保证 | 需手工剪枝，易遗漏 | 求解器天然保证 |
| 依赖 | 纯 Rust（如 `augmenting`/自实现） | 需 C 编译器（`highs-sys` 静态链接） |
| 离线/打包 | 无外部依赖，最易分发 | HiGHS 静态链接，可随 Tauri 打包 |
| 适用规模 | $R\cdot T$ 对，不展开到天 | $R\cdot T\cdot D < $ 阈值（默认 2 万变量）；更大需分块 |

**推荐（默认 `Hybrid`）：**
1. 小规模（$R\cdot T < $ 阈值，默认 500，且 $R\cdot T\cdot D < $ `milp_var_threshold`）：贪心 + 匈牙利匹配秒出初解，直接返回。
2. 中大规模或用户点「精化」（$R\cdot T\cdot D < $ 阈值）：用 MILP（good_lp + HiGHS）在贪心解邻域内重解，时限内取最优。
3. 超大规模（$R\cdot T\cdot D \ge $ 阈值）：按 project 或 team **分块** 求解 + Lagrangian 松弛协调跨块容量。

> **good_lp 要点（显式 HiGHS，禁用 default_solver）**：在 `crates/ai-engine/Cargo.toml` 写
> ```toml
> good_lp = { version = "1", default-features = false, features = ["highs"] }
> ```
> 即关闭默认 feature，仅启用 `highs`。原因：good_lp 的 `default_solver` 常量是 `coin_cbc::coin_cbc` 的别名（见 docs.rs `pub use solvers::coin_cbc::coin_cbc as default_solver;`），由 `coin_cbc` / `default-solver` feature 提供。一旦 `default-features = false`，`default_solver` 符号不存在（编译失败）；若保留默认 feature 则拉回 `coin_cbc`，需系统级 C 库，正是桌面分发要避免的。因此**必须用显式求解器实例 `good_lp::highs::highs`**（见 5.5.3），不要用 `default_solver`。`clarabel` 后端不支持整数变量，不可用于本场景。

#### 5.5.3 求解 trait 与 good_lp 用法片段

```rust
/// 求解器抽象：贪心/MILP/Hybrid 各自实现。
#[async_trait]
pub trait Solver: Send + Sync {
    async fn solve(
        &self,
        problem: &AllocationProblem,
        scores: &ScoreMatrix,
    ) -> Result<Solution, SolverError>;
}

/// MILP 求解器（默认 good_lp + HiGHS）。
pub struct MilpSolver { /* good_lp 配置 */ }

#[async_trait]
impl Solver for MilpSolver {
    async fn solve(&self, p: &AllocationProblem, s: &ScoreMatrix) -> Result<Solution, SolverError> {
        // 显式指定 HiGHS 求解器：good_lp::highs::highs（由 "highs" feature 提供）。
        // 注意：不要用 default_solver（它是 coin_cbc 别名，default-features=false 下不存在）。
        use good_lp::highs::highs as Solver;
        use good_lp::{constraint, variables, Solution as GlpSolution, SolverModel};
        // 伪代码：实际需按 day 维度展开变量并加入上表全部约束
        variables! {
            vars;
            // 连续主变量 percent_{r,t,d} ∈ [0,1] + 0/1 指示 x_{r,t,d} + 聚合 y_{r,t}
        }
        let model = vars
            .maximise(/* 加权目标表达式（含松弛后的工作量项，权重来自 ObjectiveWeights） */)
            .using(Solver)                       // 显式 HiGHS，编译期保证不拉 coin_cbc
            .with(/* 耦合约束：percent ≤ x、percent ≥ ε·x、y = Σ percent / |W_t| */)
            .with(/* 容量约束：Σ_t percent_{r,t,d} ≤ cap_{r,d}（=day_factor，比例空间） */)
            .with(/* 时间窗、并行上限、依赖、锁定约束 */);
        // 若 infeasible（仅当工作量被保留为硬约束时可能发生），由外层 run() 捕获并松弛重解（见 5.8.4）
        let sol = tokio::task::spawn_blocking(move || model.solve())
            .await.map_err(|_| SolverError::Panic)??;
        // 解出后回填 Allocation 列表（percent → allocation.percent）与 unscheduled 缺口
        Ok(decode_solution(sol, p))
    }
}
```

> **超时与降级**：MILP 包在 `tokio::task::spawn_blocking` + `tokio::time::timeout` 中；超时返回当前最优可行解（good_lp 经 `WithTimeLimit` 设 time-limit 选项）或回退贪心解，绝不卡死 UI。MILP 返回 `infeasible` 时的处理见 5.8.4（松弛重解 / 退化贪心，绝不返回空解）。

### 5.6 阶段 (d)：LLM 解释

把求得的 `Solution` + 关键指标喂给 rig 的 chat completion（本地 Ollama 模型，如 `qwen2.5`/`llama3.1`，可切云），产出三段式结构化说明：**(1) 为什么这样分配 (2) 风险点 (3) 可改进处**。

```rust
#[async_trait]
pub trait Explainer: Send + Sync {
    async fn explain(
        &self,
        problem: &AllocationProblem,
        solution: &Solution,
        metrics: &SolutionMetrics,
    ) -> Result<Explanation, ExplainerError>;
}

pub struct LlmExplainer<M: rig::completion::CompletionModel> {
    model: M,
    prompt_template: PromptTemplate,
}

impl<M: rig::completion::CompletionModel> Explainer for LlmExplainer<M> {
    async fn explain(&self, p: &AllocationProblem, s: &Solution, m: &SolutionMetrics)
        -> Result<Explanation, ExplainerError>
    {
        let prompt = self.prompt_template.render(p, s, m);   // 注入方案+指标 JSON
        // rig 的 Prompt trait：prompt(...) 返回 PromptResponse，其文本字段为 .content（单一 String）。
        // 注意：PromptResponse 没有 .choices 字段；.choices 只存在于底层 CompletionResponse。
        let text = self.model.prompt(&prompt)                 // rig completion（高层 Prompt API）
            .await.map_err(ExplainerError::from)?
            .content;                                          // PromptResponse 的文本字段
        // LLM 可能不守格式：先尝试解析 JSON，失败则整体作为 free_text
        Ok(parse_or_fallback(&text))
    }
}
```

> **rig 字段名随次版本变化**：`PromptResponse` 的文本字段在 rig-core 0.24.x 为 `.content`，但该框架活跃迭代，后续版本可能改名（如 `.0` 元组访问）。落地前以**锁定版本**的 `docs.rs/rig-core` 字段名为准，并在 `Cargo.toml` 用 `rig-core = "=0.24.x"` 锁死次版本。若需要 `choices` / `tool_call` 等底层字段，改用 builder：`self.model.completion_request().prompt(prompt).send().await?` 返回 `CompletionResponse`，其 `.choices: Vec` 可用。

**Prompt 设计要点：** 系统提示词约束 LLM「只解释、不修改分配结果」，输入为结构化 JSON（分配清单 + 利用率/过载/闲置统计），输出要求 JSON `{rationale, risks[], suggestions[]}`，解析失败降级为整段文本展示。

**降级（`TemplateExplainer`，本轮确认）：** chat/LLM 不可用（本地与云均不可达 / 超时 / 返回无法解析）时，按规则模板生成解释——例如「资源 A 的 skills 与 task#12 匹配度最高(0.95)，且在 6/10–6/20 时间窗内余量充足(8 PD)，故优先分配」——零 LLM 依赖，永远可用。`TemplateExplainer` 与 `LlmExplainer` 实现同一 `Explainer` trait，引擎侧透明替换。

### 5.7 阶段 (e)：人机闭环与落库

方案以 **建议** 形式呈现，最终落库为 `allocation` 记录，关键在于区分来源与可追溯性。

```rust
#[derive(Debug, Clone)]
pub struct Solution {
    pub run_id: i64,                     // 落库对应 ai_optimization_runs.id（INTEGER 自增）
    pub assignments: Vec<ScoredAssignment>,
    pub metrics: SolutionMetrics,
    pub explainer_text: Explanation,
    pub status: SolutionStatus,         // Proposed / Accepted / Rejected / PartiallyAccepted
}

#[derive(Debug, Clone)]
pub struct ScoredAssignment {
    pub resource_id: i64,
    pub task_id: i64,
    pub day_range: (usize, usize),      // 起止天索引
    pub allocation_pct: f64,            // 投入比例 %（来自 MILP 的 percent_{r,t,d} 聚合）
    pub effort_pd: f64,                 // 实际分配人日 = Σ percent × day_factor × daily_capacity_pd
    pub match_score: f64,               // 来自阶段 (b)
    pub locked: bool,                   // 用户锁定后不可被重排
    pub source: AllocationSource,       // Ai / Manual / Locked
}

#[derive(Debug, Clone, Default)]
pub struct SolutionMetrics {
    pub avg_utilization: f64,           // 平均利用率
    pub overloaded_resources: Vec<(i64, f64)>, // (resource_id, 超载比例 Σ percent - day_factor)
    pub idle_resources: Vec<i64>,       // 闲置资源
    pub unscheduled_tasks: Vec<i64>,    // 未排上（容量不足 / MILP 松弛后缺口）
    pub project_budget_usage: HashMap<i64, f64>, // 项目预算消耗
}
```

**落库映射（统一到 §3.3 的 schema，本章不再另起表）：**

> 本章早期草案曾定义一张 `ai_run` 表（`run_id TEXT PRIMARY KEY`）并称「allocation 表增加列 source/run_id」。为消除与 §3.3 的表名/类型漂移，**本章落库统一映射到 §3.3.16 的 `ai_optimization_runs` 与 §3.3.15 的 `allocations`**：
>
> - `ai_optimization_runs`：`id INTEGER PRIMARY KEY AUTOINCREMENT`（即本章 `run_id`），已含 `seed / objective / scope_* / input_snapshot_json / output_plan_json / explanation_md / provider / chat_model / embed_model / solver_backend / solver_status / applied` 等全部审计字段。本轮拍板**维持 INTEGER 自增、不引入 UUID**。
> - `allocations`：`source TEXT ('manual'|'ai')`、`run_id INTEGER REFERENCES ai_optimization_runs(id) ON DELETE SET NULL` 已在 §3.3.15 定义，**无需新增列**。`allocation.percent` 直接承接 MILP 解出的 `percent`（比例空间，`Σ percent ≤ day_factor` 已在求解阶段保证）。
>
> 因此本节删除原 `CREATE TABLE ai_run (...)` 片段。本轮决策已确认不引入 UUID，故未来若确需用 UUID 作 `run_id`，须同步回改 §3.3.15 / §3.3.16 全链路类型（INTEGER→TEXT），不可在两章各定义一张表。

用户操作映射：**接受** → `ai_optimization_runs.applied=1`、`allocations.source='ai'` 批量写入（事务内先标记旧 `ai` allocation 为 cancelled，再插新，最后置 `applied=1`，见 §3.7 大事务边界 / §6.5）；**微调** → 局部覆盖；**锁定** → 对应 allocation 标记，后续 run 视为硬约束；**拒绝** → 保留 run 记录（`applied=0`）供复盘。

> **事务边界一致性提示（与 §3.7 / §6.5 对齐）**：落库写事务（`apply_solution` / `persist_run`）必须统一走 §3.7 的 `with_write_tx`（`BEGIN IMMEDIATE` + `busy_timeout` + SQLITE_BUSY 退避重试），**不得**直接 `pool.begin()`（DEFERRED、无重试，会在 AI 写与 UI 写并发时高频撞 SQLITE_BUSY）。详见 §6.5「所有写事务统一走 `db::with_write_tx`」。

### 5.8 横切关注点

#### 5.8.1 可复现性与 `input_snapshot_json` 体积

- 所有随机源（贪心扰动、tie-break）统一用 `SolverConfig.seed` 初始化的 `StdRng`。
- `ai_optimization_runs.input_snapshot_json` + `seed` + `constraints_json` + `weights_json` + provider/model 保证：相同输入 + 相同配置 + 相同求解器版本 → 相同的**硬约束解**（MILP 求解器在相同版本下确定）。
- 提供「重放此 run」入口：载入 snapshot，强制相同 seed 与 provider，重跑并 diff。

**embedding 向量快照（保证语义打分可复现）+ 体积方案（本轮拍板方向，impl 期评估定稿）：**

> §5.4 的语义打分依赖 embedding 向量。若 `input_snapshot_json` 仅存文本、不存向量，则用户更换 `ai_embed_model` 后重放会得到不同的 `match_score`（甚至影响软目标最优解的 tie-break），与可复现承诺矛盾。故约定：
>
> 1. `input_snapshot_json` 内同时持久化一份**「文本→向量」向量快照**（或其内容哈希 + provider + model + 维度 `ai_embed_dim`）。该表是 run 时刻实际参与打分的向量快照。
> 2. **重放优先级**：重放时**优先使用快照中的向量**，仅当 provider/model/维度三者完全一致时才允许重算 embedding（用于校验）。
> 3. **provider/model 变更的复现等级**：若重放时 `embed_provider/embed_model` 与快照不一致，则该 run 标注为 **「部分可复现（硬约束解确定，语义 score 非确定）」**，并在 `ai_optimization_runs.error_msg` 或 UI 上写明。
> 4. LLM 解释（`explanation_md`）本身带非确定性（即便 `temperature=0`），重放时不要求字节级一致；解本身不依赖 LLM 文本（LLM 仅做打分系数与解释）。

**向量体积问题（开放问题 #24 已给出方向，本轮标注 impl 期评估）：** 完整向量表体积可达「数百资源×任务×768 维 float」级别（如 50×200×768×8B ≈ 60MB），直接塞进单行 JSON 会撑爆 `input_snapshot_json` 并拖慢 run 列表查询。两种候选落地方案，**impl 期实测后定其一**：

| 方案 | 做法 | 优点 | 代价 |
| --- | --- | --- | --- |
| **A：向量内容哈希 + 向量旁表** | `input_snapshot_json` 仅存「文本哈希 → 向量内容哈希」的映射 + provider/model/维度；向量本体写入独立旁表 `ai_run_embeddings(run_id, text_hash, vec BLOB)` | run 主行轻量，列表查询快；向量可跨 run 复用（同 text_hash） | 多一张表 + join；需在 run 删除时级联清理 |
| **B：压缩序列化** | 向量经列存压缩（如 f32 量化到 f16 + zstd）后作为 `input_snapshot_json` 的子字段或单独 `snapshot_vectors_zstd BLOB` 列 | 无新表，单行自包含 | 单行仍偏大（压缩后约 1/3–1/2）；列查询需惰性加载 |

> **impl 期决策项**：在 §1.6 验收规模（10 资源/50 任务）与 §2.4 示例规模（50/200）下分别实测两种方案的 run 行体积、run 列表查询延迟、重放开销，选定为默认；未定稿前两种 trait 抽象保持等价接口（`VectorSnapshotStore`），便于切换。

#### 5.8.2 增量优化

全量重算在大数据集下昂贵。增量模式只对「变更项」重算：
- 触发：新增/编辑 task、resource skills 变更、容量调整、用户锁定/解锁。
- 范围限定：仅纳入「受影响的 resource × task」子集（按依赖图与容量交集裁剪），其余沿用上次 `allocation(source=ai)`。
- 锁定项始终进入约束集但变量固定（`percent_{r,t,d}` 固定为 locked 值），保证不破坏已确认结果。
- `ai_optimization_runs.scope_*`（project_ids / from / to）标注优化范围，便于区分全量与增量结果。

#### 5.8.3 约束与偏好的可配置（ObjectiveWeights UI 可调）

所有硬/软开关集中在 `ConstraintFlags`，权重集中在 `ObjectiveWeights`，二者随 `ai_optimization_runs.constraints_json` / `weights_json` 持久化。**本轮拍板：`ObjectiveWeights`（多目标权重）由用户在 UI「约束面板」调节**——前端暴露开关切换 + 权重滑块（每个软目标一项：匹配质量/已排工作量/负载均衡/优先级覆盖/跨项目复用），用户拖动后即时写入 `SolverConfig.weights`，并落库到 `weights_json`，下次 run 生效。典型预设（一键应用，本质是预设权重组合）：
- **均衡模式**：`balance_workload` 权重高，降低过载。
- **匹配优先**：`match_score` 权重高，把最强的人放最匹配的活。
- **交付优先**：优先级与项目预算覆盖权重高，保证高优任务必排上。

> 用户调节的权重直接进入 §5.5.1 目标函数的 $w_m, w_e, w_b, w_p, w_c$，求解器在下次 run 用新权重重解；UI 提供权重归一化提示（各权重 ≥0，归一化后和为 1 便于对比，但不强制）。对应开放问题 #6（多目标权重 UI 调节）已确认「是」并由本节落地。

#### 5.8.4 超时与降级矩阵（降级链已确认）

| 阶段 | 超时阈值（默认） | 失败/超时行为 |
| --- | --- | --- |
| (b) 语义打分 | 30s | **退化为 `FallbackScorer`**（关键词 Jaccard + 熟练度线性加权）；维度异常/批量报错同样降级 |
| (c) 求解 | 10s（小）/ 60s（大） | MILP 超时返回当前最优或贪心解；贪心失败返回 `unscheduled` 标注 |
| (c) **MILP 返回 infeasible** | — | **自动把「工作量满足」约束松弛为软目标重解**（最大化已排工作量比例，按 `Σ percent × day_factor × 1PD` 计），缺口写入 `SolutionMetrics.unscheduled_tasks`；若松弛后仍 infeasible（极少见，通常是依赖成环/锁定冲突），退化贪心，**绝不返回空解** |
| (d) LLM 解释 | 20s | **退化为 `TemplateExplainer`**（规则化文本，零 LLM 依赖） |
| 全流程 | 90s | 截断在可接受阶段，UI 标注「部分降级」 |

核心不变量：**LLM 永不参与硬约束求解**，因此 LLM 完全不可用时，系统仍能给出合法（满足容量/窗/技能/依赖）的经典解 + 模板解释（`TemplateExplainer`）。第二条不变量：**MILP 不可行时绝不返回空解**——通过松弛重解或退化贪心保证始终产出「尽力而为」的可行方案 + `unscheduled_tasks` 缺口标注。第三条不变量：**embedding 不可用时绝不把脏向量喂给余弦计算**——直接切 `FallbackScorer`。

### 5.9 Tauri IPC 集成

引擎通过 `tauri::command` 暴露给前端，前端 `invoke()` 触发并订阅进度。

> **`run_optimization` 签名统一（与 §6.6 一致）**：本节 `run_optimization` 采用**带 `on_event: Channel<OptProgress>`、返回 `OptimizationSolutionDto`** 的签名，与 §6.6 一致；早期不带 Channel、直接返回 `Solution` 的旧签名已废弃，统一以本节为准。

```rust
use tauri::ipc::Channel;

#[tauri::command]
async fn run_optimization(
    state: tauri::State<'_, AppState>,
    scope: OptScope,                 // Full / Incremental / { task_ids: Vec<i64> }
    config: Option<SolverConfig>,    // 含用户在 UI 调节的 ObjectiveWeights
    on_event: Channel<OptProgress>,   // 进度回传（与 §6.6 一致）
) -> Result<OptimizationSolutionDto, AppError> {  // 返回 DTO，非内部 Solution
    let engine = state.ai_engine.clone();
    let dto = engine.run(scope, config, |p| { let _ = on_event.send(p); })
        .await.map_err(AppError::from)?;
    Ok(dto)
}

#[tauri::command]
async fn accept_solution(
    state: tauri::State<'_, AppState>,
    run_id: i64,                     // ai_optimization_runs.id（INTEGER）
    accepted: Vec<i64>,               // 接受的 assignment 索引
    locked: Vec<i64>,
) -> Result<(), AppError> { /* 落库 allocation（走 db::with_write_tx） */ }
```

长期求解（MILP 精化）通过 `Channel<OptProgress>` 推送阶段进度（`Started / Stage{name,at,of} / Done / Error`），前端展示「打分中 → 求解中 → 解释中」与可降级提示。`OptProgress` 的具体定义见 §6.6。

### 5.10 引擎编排总览（Engine）

```rust
pub struct OptimizationEngine {
    scorer: Arc<dyn Scorer>,           // 默认 SemanticScorer，不可用时换 FallbackScorer
    solver: Arc<dyn Solver>,           // 默认 HybridSolver（内含贪心+MILP/good_lp+HiGHS）
    explainer: Arc<dyn Explainer>,     // 默认 LlmExplainer，不可用时换 TemplateExplainer
    db: SqlitePool,
}

impl OptimizationEngine {
    pub async fn run(
        &self, scope: OptScope, cfg: Option<SolverConfig>,
        on_progress: impl Fn(OptProgress) + Send + Sync,
    ) -> Result<OptimizationSolutionDto, EngineError> {
        let problem = self.build_problem(&scope, cfg).await?;   // (a)
        let scores  = self.scorer.score(&problem).await
            .unwrap_or_else(|_| self.fallback_score(&problem)); // (b) + 降级（含维度异常 → FallbackScorer）
        let sol     = self.solver.solve(&problem, &scores).await?; // (c)（infeasible 由 solver 内部/此处松弛）
        let metrics = compute_metrics(&sol, &problem);
        // (d) LLM 解释：在 persist_run 之前完成、**且在写事务之外**（不在事务内调 LLM）
        let text    = self.explainer.explain(&problem, &sol, &metrics).await
            .unwrap_or_else(|_| self.template_explain(&sol, &metrics)); // (d) + 降级 → TemplateExplainer
        let solution = Solution { run_id: problem.run_id, assignments: sol.assignments,
                                  metrics, explainer_text: text, status: SolutionStatus::Proposed };
        // persist_run 仅写元数据/快照/解 JSON（短事务，BEGIN IMMEDIATE），不持大锁；
        // explanation_md 文本已在上面事务外算好，这里只是普通 INSERT，不阻塞 UI 写。
        self.persist_run(&solution).await?;                     // 落库 ai_optimization_runs
        Ok(solution.into_dto())                                 // (e) 由前端 accept/lock
    }
}
```

> **`persist_run` 的事务语义（与 §6.5 「事务内绝不调用 LLM」一致）**：
> 1. `persist_run` 只做**短写事务**：写入 `ai_optimization_runs` 的元数据 / `input_snapshot_json`（向量体积方案见 5.8.1） / `output_plan_json` / `explanation_md` / score 字段。它走 `db::with_write_tx`（`BEGIN IMMEDIATE` + 重试），但**不包含 LLM 调用、不包含大批量 allocation 插入**——后者发生在用户「接受」时的 `apply_solution`（另一事务）。
> 2. **LLM 调用在 `run()` 内于 `persist_run` 之前完成、且在事务之外**：解释文本算好后作为普通字段传入短事务，避免「大事务持锁 + LLM 长耗时」叠加。
> 3. 因此 `run_optimization` 一次 invoke 期间只有两次短写事务（`build_problem` 写快照、`persist_run` 写结果），中间的 LLM/求解均在事务外，符合 §6.5。

引擎对 `Scorer / Solver / Explainer` 三个 trait 持有 `Arc<dyn ...>`，使其可被任意替换（本地/云、贪心/MILP、LLM/模板、语义/关键词），是整条管线可测试、可降级、可演进的关键。

---

**参考与版本假设：**
- `rig-core ≈ 0.24.x`（0xPlaygrounds/rig）：`providers::ollama::Client::from_env()` / `openai::Client::from_env()`；`client.embedding_model(model)` → `EmbeddingModel`（直接 `EmbeddingModel::embed(text)` 或经 `EmbeddingsBuilder`，注意 issue #1082 的 import 路径与维度陷阱）；chat 经 `client.agent(model)` 或 `CompletionModel::prompt`，`prompt(...)` 返回 `PromptResponse`，文本字段为 `.content`（**非 `.choices`**），字段名随次版本变化，以锁定版 `docs.rs/rig-core` 为准。该框架活跃迭代，需锁定次版本号。
- `good_lp`（rust-or/good_lp）：`variables!` 宏 + `.maximise(..).using(good_lp::highs::highs).with(constraint!(..)).solve()`；**默认采用显式 `highs` 求解器**（`default-features = false, features = ["highs"]`），**不要用 `default_solver`**——后者是 `coin_cbc` 别名，禁用默认 feature 时不存在、启用默认 feature 时会拉回需要系统级 C 库的 `coin_cbc`，两者都不适合桌面分发。

---

**本章假设（需用户确认 / impl 期跟进）：**

1. §3.8 容量上限统一为「比例口径」`Σ_t percent_{r,t,d} ≤ day_factor(d,r)`，MILP 在比例空间求解；`daily_capacity_pd` 仅用于 PD 展示折算（`pd = percent × daily_capacity_pd × day_factor`），不参与过载阈值（见假设汇总 #17/#23）。
2. MILP 单位闭合采用「连续 `percent_{r,t,d}` 主变量 + 0/1 `x_{r,t,d}` 指示 + 聚合 `y_{r,t}`」的耦合形式：`percent ≤ x`、`percent ≥ ε·x`、`y = Σ percent / |W_t|`（与 §3.8 对齐，见假设汇总 #23）。
3. 工作量满足约束默认建模为带松弛的软目标（最大化已排工作量比例，按 `Σ percent × day_factor × 1PD` 计，缺口写入 `unscheduled_tasks`）；MILP 返回 infeasible 时自动松弛重解或退化贪心，绝不返回空解（见假设汇总 #33）。
4. 并行上限与容量上限是两个独立约束：`max_parallel_tasks_per_day` 默认 None（不限并行，仅受比例容量约束）；Some(1) 表示禁并行（见假设汇总 #34）。
5. 本章统一落库到 §3.3.16 的 `ai_optimization_runs`（**INTEGER 自增主键，本轮拍板不引入 UUID**）与 §3.3.15 的 `allocations`（`source`/`run_id` 已定义，`percent` 直接承接 MILP 解），不另起 `ai_run` 表（见假设汇总 #18/#35）。
6. 所有写事务（`apply_solution`/`persist_run`）统一走 §3.7 的 `with_write_tx`（BEGIN IMMEDIATE + busy_timeout + SQLITE_BUSY 退避重试），禁止直接 `pool.begin()`（见假设汇总 #36）。
7. `persist_run` 仅做短写事务（写元数据/快照/解 JSON），不含 LLM 调用与大批量 allocation 插入；LLM 调用在 `run()` 内于 persist 之前完成且在事务之外，符合 §6.5（见假设汇总 #37）。
8. `input_snapshot_json` 持久化「文本→向量」向量快照（或哈希 + provider/model/维度）；**向量体积方案（旁表 vs 压缩序列化）impl 期评估定稿**（见假设汇总 #38 / 开放问题 #24）。
9. 多目标权重 `ObjectiveWeights` 由用户在 UI「约束面板」调节（滑块/预设），写入目标函数与 `weights_json`，下次 run 生效（对应开放问题 #6「是」）。
10. 降级链：LLM 不可用 → `TemplateExplainer`（规则模板，零 LLM 依赖）；embedding 不可用 → `FallbackScorer`（关键词 Jaccard + 熟练度）（对应开放问题 #8「是」）。
11. ILP 后端默认采用 `good_lp` + `HiGHS`（静态链接，二进制体积无上限，对应开放问题 #7/#49 与假设汇总 #10/#31）。
12. `rig-core ≈ 0.24.x`：Prompt trait 的 `prompt()` 返回 `PromptResponse`，文本字段为 `.content`（非 `.choices`）；embed 走 `EmbeddingModel::embed` 或 `EmbeddingsBuilder`，Ollama+EmbeddingsBuilder 组合存在 issue #1082 的 import 路径与维度陷阱。字段名随次版本变化，落地前以锁定版 `docs.rs/rig-core` 为准，`Cargo.toml` 用 `rig-core = "=0.24.x"` 锁死次版本（见假设汇总 #30）。
13. `good_lp` 默认 `default_solver` 常量是 `coin_cbc::coin_cbc` 的别名；桌面分发必须 `default-features = false` 且仅启用 `highs` feature，并显式用 `good_lp::highs::highs` 求解器实例，禁止 `default_solver`（见假设汇总 #31）。
14. MILP 决策变量规模 $N \approx 2 \cdot R \cdot T \cdot D$，good_lp/HiGHS 在 60s 预算内可解量级约 < 2 万变量；超过则按 project/team 分块，§2.4 的 50×200 示例需理解为分块/分窗而非单次 MILP（见假设汇总 #32）。

**本章开放问题（需用户拍板 / impl 期决策）：**

1. [impl 期决策] **`input_snapshot_json` 向量体积方案定稿**：在 §1.6 验收规模（10/50）与 §2.4 示例规模（50/200）下实测「向量内容哈希 + 向量旁表（A）」vs「压缩序列化（B）」的 run 行体积、列表查询延迟、重放开销，选定默认（原开放问题 #24）。
2. [impl 期决策] `max_parallel_tasks_per_day` 是否需要支持「按项目/按资源」细粒度配置（当前是全局 `ConstraintFlags` 单值，开放问题 #26 已确认「按项目和资源配置」）——需在 `ConstraintFlags` 引入 per-project / per-resource 覆盖结构并同步 §3.8。
3. [impl 期决策] 投入比例最小步进 $\epsilon$（耦合约束 `percent ≥ ε·x` 的下界）取值：与开放问题 #50（allocation 投入比例粒度：小数 vs 5% 步进）联动，落地时锁定 $\epsilon$（如 0.05）。
4. [impl 期决策] `ObjectiveWeights` 在 UI 的权重归一化策略：是否强制归一化（和为 1）还是允许自由绝对值，以及预设模式的默认权重组合需产品确认。
5. [跨章，属第 6 章范围] §6.5 的 `with_tx` 封装（`pool: &PgPool` 笔误、普通 `pool.begin()` 无 IMMEDIATE/重试）需在修订第 6 章时统一改为 `db::with_write_tx`（原开放问题 #22）；本章已在 §5.7/§5.9/§5.10 加交叉一致性提示。
6. [跨章，属第 2 章范围] §2.7 的 Cargo 片段若也写了 `good_lp` 的 `default-features/features`，需确认与本章一致的 `highs` 配置（原开放问题 #23）。

## 6. 后端命令与服务层 / Backend Service & Tauri Commands

本节定义 Rust 侧的分层结构：**Domain Services（纯 Rust 业务逻辑，不依赖 Tauri，可脱离 GUI 单测）** 与 **Tauri Commands（IPC 薄封装，仅做参数反序列化、调用 service、错误透传）** 的职责分离，覆盖命令分组清单、DTO 映射、统一错误模型、异步进度回传、事务边界、预检校验层，以及本地密钥安全存储。

### 6.1 分层结构与目录约定

```
src-tauri/src/
├── main.rs                 # Tauri 入口，注册 invoke_handler
├── commands/               # Tauri command 薄封装（IPC 边界）
│   ├── mod.rs
│   ├── resource.rs
│   ├── team.rs
│   ├── project.rs
│   ├── task.rs
│   ├── allocation.rs
│   ├── calendar.rs
│   ├── optimization.rs
│   ├── report.rs
│   └── settings.rs
├── services/               # 纯 Rust domain services（可单测，零 Tauri 依赖）
│   ├── mod.rs
│   ├── resource_svc.rs
│   ├── team_svc.rs
│   ├── project_svc.rs
│   ├── task_svc.rs
│   ├── allocation_svc.rs
│   ├── workload_svc.rs     # recalculate_workload / 利用率
│   ├── optimization_svc.rs # run_optimization / apply_solution
│   ├── report_svc.rs       # export_report
│   └── settings_svc.rs
├── domain/                 # 领域模型（实体 + 值对象 + 领域错误）
│   ├── mod.rs
│   ├── model.rs            # Resource/Team/Project/Task/Allocation …
│   └── error.rs            # DomainError + AppError 枚举
├── dto/                    # 请求/响应 DTO + 与 domain 的映射
│   ├── mod.rs
│   └── mappers.rs
├── repo/                   # sqlx 数据访问（每表一 repo，返回 domain）
│   └── ...
├── db/                     # 连接池、迁移、事务封装
│   └── tx.rs               # with_write_tx（BEGIN IMMEDIATE + busy_timeout + 退避重试）
└── infra/                  # rig 封装、keyring 封装、配置加载
    ├── llm.rs
    ├── secrets.rs          # 按 settings.secret_store 选择后端
    ├── crypto.rs           # AES-256-GCM；主密钥来自用户口令派生（option b）
    ├── master_key.rs       # Argon2id 派生 / 首启引导 / 解锁 / 换口令
    └── config.rs
```

**核心原则（Dependency Rule）：** `commands → services → repo → domain`。`domain` 不依赖任何上层；`services` 不依赖 `tauri`（用 `tracing` 打日志，用领域错误返回）。这样 `services` 与 `domain` 可放进 `#[cfg(test)]` 用内存 SQLite 单测，无需启动 Tauri。

| 层 | 依赖 | 可否单测 | 职责 |
|---|---|---|---|
| `commands/*` | `tauri`、`services`、`dto` | 否（需 IPC 上下文） | 反序列化入参 → 调 service → 序列化出参/错误透传 |
| `services/*` | `repo`、`domain`、`infra` | **是** | 业务规则、校验、事务编排、跨 repo 协调 |
| `repo/*` | `sqlx`、`domain` | 是（内存 SQLite） | 纯数据访问，返回 domain 实体，无业务规则 |
| `domain/*` | 无（仅 `thiserror`/标准库；**不依赖 `serde`**） | **是** | 实体、值对象、领域错误、不变量 |

> **领域层零 serde 依赖（本轮决策）：** `domain/*` 只派生 `#[derive(Debug, thiserror::Error)]`，**不派生 `serde::Serialize`**。`serde` 序列化是 IPC 边界的关切，不应反向侵入领域层（领域层需可脱离 Tauri/serde 在任意 host 单测与复用）。`DomainError` 跨出领域层时由 `dto/mappers.rs` 统一映射为 `AppError::Validation` 或 `AppError::Domain`（见 §6.4）。

### 6.2 命令分组清单

下表列出全部 command 分组。命名约定：CRUD 用 `create_X / list_X / get_X / update_X / delete_X`；业务动作用动词短语。所有 command 均为 `async fn`，返回 `Result<T, AppError>`。

#### 6.2.1 resource（开发资源/人）

| Command | 入参（简） | 返回 | 说明 |
|---|---|---|---|
| `create_resource` | `CreateResourceDto` | `ResourceDto` | 新建一名开发者 |
| `list_resources` | `ResourceFilterDto` | `Vec<ResourceDto>` | 含 tag/skill 过滤、分页 |
| `get_resource` | `id: i64` | `ResourceDto` | 单条 |
| `update_resource` | `UpdateResourceDto` | `ResourceDto` | 含技能熟练度更新 |
| `delete_resource` | `id: i64` | `()` | 软删除（带进行中 allocation 时拦截，见 6.7） |
| `set_resource_skills` | `id, skills[]` | `Vec<SkillDto>` | 批量覆盖技能集 |
| `set_resource_capacity` | `id, window, capacity` | `CapacityDto` | 设定时间窗内的可用容量 |

#### 6.2.2 team（团队）

| Command | 返回 | 说明 |
|---|---|---|
| `create_team` / `list_teams` / `get_team` / `update_team` / `delete_team` | `TeamDto` / `Vec<TeamDto>` | 标准 CRUD |
| `add_team_members` / `remove_team_members` | `TeamDto` | 维护成员（资源可跨项目/跨团队） |
| `team_workload` | `TeamWorkloadDto` | 团队在时间窗内的 workload 聚合 |

#### 6.2.3 project（项目）

| Command | 返回 | 说明 |
|---|---|---|
| `create_project` / `list_projects` / `get_project` / `update_project` / `delete_project` | `ProjectDto` / `Vec<ProjectDto>` | 含周期/优先级/总人力预算 |
| `set_project_budget` | `ProjectDto` | 设定/调整项目人力预算（PM/PD） |

#### 6.2.4 task（任务）

| Command | 返回 | 说明 |
|---|---|---|
| `create_task` / `list_tasks` / `get_task` / `update_task` / `delete_task` | `TaskDto` / `Vec<TaskDto>` | 含所需技能、工作量、时间窗、依赖 |
| `set_task_skills` | `Vec<SkillDto>` | 声明任务所需技能 |
| `set_task_dependencies` | `Vec<i64>` | 任务依赖（前置/后置） |
| `split_long_task` | `Vec<TaskDto>` | 长期任务分段（按阶段/里程碑切分） |

#### 6.2.5 allocation（分配/指派，文档统一用词 Allocation）

| Command | 返回 | 说明 |
|---|---|---|
| `create_allocation` / `list_allocations` / `get_allocation` / `update_allocation` / `delete_allocation` | `AllocationDto` / `Vec<AllocationDto>` | 资源+投入比例%+时间区间绑定到任务 |
| `recalculate_workload` | `WorkloadReportDto` | 重算并回写 workload（见 6.6） |

#### 6.2.6 calendar（日历/视图）

| Command | 返回 | 说明 |
|---|---|---|
| `get_calendar_allocations` | `Vec<CalendarItemDto>` | 按 `[start,end]` 取所有 allocation，供日历/Gantt 渲染 |
| `get_gantt_data` | `GanttDto` | 项目→任务→allocation 的层级聚合 |
| `list_holidays` / `upsert_holiday` | `HolidayDto` | 节假日/非工作日（影响容量计算） |

#### 6.2.7 optimization（AI 优化，长任务，带进度）

| Command | 返回 | 说明 |
|---|---|---|
| `run_optimization` | `OptimizationSolutionDto` | 触发「经典优化器 + LLM 语义打分」混合求解，通过 Channel 推送进度（见 6.6） |
| `apply_solution` | `ApplyResultDto` | 将方案落库为 allocation（事务批量，见 6.5） |
| `explain_solution` | `String` | LLM 生成自然语言解释（可复现：固定 seed + 缓存） |

#### 6.2.8 report（报表）

| Command | 返回 | 说明 |
|---|---|---|
| `export_report` | `ReportRefDto` | 导出 workload/利用率/项目人力消耗报表（PDF/CSV/Excel），返回本地文件路径 |
| `preview_report` | `ReportDataDto` | 前端预览用的结构化数据 |

#### 6.2.9 settings（配置与单位）

| Command | 返回 | 说明 |
|---|---|---|
| `get_settings` / `update_settings` | `SettingsDto` | 单位偏好（PM/PD）、PD 工时、PM 折算系数、`secret_store` 策略等 |
| `get_ai_provider` / `set_ai_provider` | `AiProviderDto` | provider 选择（ollama/cloud）、baseURL；写云 key 时经 `infra::secrets` 按 `secret_store` 落地（见 §6.8） |
| `test_ai_connection` | `ConnectionResultDto` | 连通性自检（不存密钥） |
| `set_master_password` | `()` | 首启引导设置 / 运行期解锁 / 重置主口令（派生 `encrypted_file` 主密钥，option b，见 §6.8.2）。重置需提供旧口令或走「忘记口令」流程（清空已存密钥，重设） |

> **`secret_store` 字段对齐（与 §3.3.1 迁移一致）：** `settings.secret_store` 为 §3.3.1 settings 表后续迁移（`ALTER TABLE settings ADD COLUMN secret_store TEXT NOT NULL DEFAULT 'encrypted_file' CHECK (secret_store IN ('keychain','encrypted_file'))`）新增的列，由 `settings_svc` 直接读写（与其它非敏感项走同一 CRUD 路径），不放进 `settings.metadata` JSON。`SettingsDto` 透出该字段供 UI 显示当前密钥落点。默认值 `encrypted_file`（option b：用户口令派生主密钥，首启引导设置口令，零 keychain 依赖即可用）。

### 6.3 DTO 与领域模型映射

**规则：command 层只接收/返回 DTO，不泄露 domain 实体。** DTO 与 domain 之间用显式 mapper 转换，避免 `From` 泛滥导致的隐式耦合。domain 实体可携带不变量校验（如 `Allocation.ratio ∈ (0,1]`），DTO 只做传输。

```rust
// domain/model.rs —— 领域实体，含不变量（不派生 serde）
#[derive(Debug, Clone)]
pub struct Allocation {
    pub id: i64,
    pub resource_id: i64,
    pub task_id: i64,
    pub ratio: f64,        // 投入比例，0 < ratio <= 1.0
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub pd: f64,           // 已求得的 PD 工作量
}

impl Allocation {
    pub fn new(ratio: f64, /* ... */) -> Result<Self, DomainError> {
        if !(0.0..=1.0).contains(&ratio) || ratio == 0.0 {
            return Err(DomainError::InvalidRatio(ratio));
        }
        Ok(Self { ratio, /* .. */ })
    }
}
```

```rust
// dto/mod.rs —— 前端契约，snake_case 字段（serde 默认）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AllocationDto {
    pub id: i64,
    pub resource_id: i64,
    pub task_id: i64,
    pub ratio: f64,
    pub start: String,     // ISO-8601 date，前端友好
    pub end: String,
    pub pd: f64,
    pub overload: bool,    // 预检回填（见 6.7）
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateAllocationDto {
    pub resource_id: i64,
    pub task_id: i64,
    pub ratio: f64,
    pub start: String,
    pub end: String,
}
```

```rust
// dto/mappers.rs —— 显式映射；这里是 DomainError → AppError 的唯一映射点
pub fn to_domain(dto: CreateAllocationDto) -> Result<NewAllocation, AppError> {
    Ok(NewAllocation {
        ratio: dto.ratio,
        start: parse_iso_date(&dto.start)?,
        end: parse_iso_date(&dto.end)?,
        ..NewAllocation::from_dto(dto)
    })
}

pub fn to_dto(a: Allocation, overload: bool) -> AllocationDto {
    AllocationDto {
        id: a.id,
        resource_id: a.resource_id,
        task_id: a.task_id,
        ratio: a.ratio,
        start: a.start.to_string(),
        end: a.end.to_string(),
        pd: a.pd,
        overload,
    }
}
```

前端对应的 TypeScript 类型（与 DTO 字段一一对应，由 `specta` / `ts-rs` 自动生成更佳）：

```ts
export interface AllocationDto {
  id: number;
  resource_id: number;
  task_id: number;
  ratio: number;     // 0 < ratio <= 1
  start: string;     // "2026-06-27"
  end: string;
  pd: number;
  overload: boolean;
}
```

### 6.4 统一错误模型

用 `thiserror` 在 `domain/error.rs` 定义 `AppError`，覆盖数据库、领域、校验、AI/LLM、配置/密钥、IO 五类。`AppError` 实现 `serde::Serialize`，Tauri v2 会自动把 Err 序列化为前端可读 JSON（Tauri v2 command 仅要求 Err 类型实现 `serde::Serialize`，IPC 层负责序列化与透传，不涉及任何 `into_response`/axum/web 概念）；前端按 `code` 分支处理，而非解析字符串。

> **领域层零 serde 依赖（本轮决策，替代原「给 DomainError 派生 Serialize」方案）：** `DomainError` **不派生 `serde::Serialize`**，领域层不依赖 `serde`。`AppError` 不再以 `#[from] DomainError` 内嵌领域错误（那样会递归要求 `DomainError: Serialize`，把 serde 反向侵入领域层）。改为在 IPC 边界（`dto/mappers.rs`）显式把 `DomainError` 映射为 `AppError::Domain`（携带结构化 detail，仍可被 serde 序列化透传给前端）或 `AppError::Validation`（携带字符串 detail）。这样 `domain/*` 保持纯净、可脱离 Tauri/serde 单测与复用，而前端仍能拿到结构化字段（如 `shortfall_pd`）做交互提示。

```rust
// domain/error.rs —— DomainError（领域错误，零 serde 依赖，仅 thiserror）
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("容量不足: 资源 {resource_id} 缺口 {shortfall_pd} PD")]
    InsufficientCapacity { resource_id: i64, shortfall_pd: f64 },
    #[error("技能不匹配: 任务 {task_id} 缺 {missing:?}")]
    SkillMismatch { task_id: i64, missing: Vec<String> },
    #[error("无效投入比例: {0}")]
    InvalidRatio(f64),
    #[error("时间窗非法: start > end")]
    InvalidDateWindow,
    #[error("求解失败: {0}")]
    Solver(String),
}
```

```rust
// domain/error.rs —— AppError（IPC 层统一错误，派生 Serialize）
// 注意：AppError 不内嵌 DomainError（#[from]），而是在边界 mapper 里映射。
#[derive(Debug, Error, Serialize)]
#[serde(tag = "code", content = "detail")]
pub enum AppError {
    #[error("database error: {0}")]
    #[serde(rename = "DB_ERROR")]
    Db(String),

    #[error("entity not found: {0}")]
    #[serde(rename = "NOT_FOUND")]
    NotFound(String),

    /// 领域违反：detail 为可序列化的结构化对象（如 InsufficientCapacity 的
    /// {resource_id, shortfall_pd}），由 mapper 从 DomainError 转换而来。
    /// 前端按 detail 内的 kind 字段二次分支。
    #[error("domain violation")]
    #[serde(rename = "DOMAIN_ERROR")]
    Domain(DomainErrorDetail),

    /// 校验失败：detail 为人可读字符串（如 "ratio out of range: 1.5"）。
    /// 用于参数解析失败、格式错误、sqlx 唯一约束冲突等非领域语义校验。
    #[error("validation failed: {0}")]
    #[serde(rename = "VALIDATION")]
    Validation(String),

    #[error("optimization failed: {0}")]
    #[serde(rename = "OPTIMIZATION")]
    Optimization(String),

    #[error("llm/embedding error: {0}")]
    #[serde(rename = "LLM")]
    Llm(String),

    #[error("config/secret error: {0}")]
    #[serde(rename = "CONFIG")]
    Config(String),

    #[error("io error: {0}")]
    #[serde(rename = "IO")]
    Io(String),

    #[error("internal error")]
    #[serde(rename = "INTERNAL")]
    Internal, // 兜底，不泄露细节
}

/// IPC 边界可序列化的领域错误载体（domain 层不依赖 serde，故定义在此处）。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum DomainErrorDetail {
    InsufficientCapacity { resource_id: i64, shortfall_pd: f64 },
    SkillMismatch { task_id: i64, missing: Vec<String> },
    InvalidRatio(f64),
    InvalidDateWindow,
    Solver(String),
}
```

**DomainError → AppError 的映射规则（集中在 `dto/mappers.rs`）：**

| DomainError 变体 | 映射到 AppError | 前端拿到的 JSON |
|---|---|---|
| `InsufficientCapacity { .. }` | `AppError::Domain(DomainErrorDetail::InsufficientCapacity{..})` | `{ code:"DOMAIN_ERROR", detail:{ kind:"insufficient_capacity", data:{ resource_id, shortfall_pd } } }` |
| `SkillMismatch { .. }` | `AppError::Domain(DomainErrorDetail::SkillMismatch{..})` | `{ code:"DOMAIN_ERROR", detail:{ kind:"skill_mismatch", data:{ task_id, missing } } }` |
| `InvalidRatio(f64)` / `InvalidDateWindow` | `AppError::Domain(DomainErrorDetail::InvalidRatio/InvalidDateWindow)` | 结构化 detail |
| `Solver(String)` | `AppError::Optimization(String)` | `{ code:"OPTIMIZATION", detail:"求解失败: ..." }` |

> 参数解析失败、ISO 日期解析失败、格式错误等**非领域语义**的校验，直接返回 `AppError::Validation(string)`；领域不变量违反（ratio 范围、时间窗、容量、技能）映射为 `AppError::Domain(...)`，使前端能区分「输入格式错」与「业务规则违反」。

```rust
// dto/mappers.rs —— DomainError → AppError 的唯一映射点
impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InsufficientCapacity { resource_id, shortfall_pd } =>
                AppError::Domain(DomainErrorDetail::InsufficientCapacity { resource_id, shortfall_pd }),
            DomainError::SkillMismatch { task_id, missing } =>
                AppError::Domain(DomainErrorDetail::SkillMismatch { task_id, missing }),
            DomainError::InvalidRatio(r) =>
                AppError::Domain(DomainErrorDetail::InvalidRatio(r)),
            DomainError::InvalidDateWindow =>
                AppError::Domain(DomainErrorDetail::InvalidDateWindow),
            DomainError::Solver(s) =>
                AppError::Optimization(s),
        }
    }
}
```

**Tauri command 错误透传的关键约束：** Tauri v2 要求 command 返回的 Err 类型实现 `serde::Serialize`。由于上面已派生 `Serialize`，`Result<T, AppError>` 可直接作为 command 返回类型——前端 `invoke().catch()` 拿到的就是 `{ code: "VALIDATION", detail: "ratio out of range: 1.5" }` 或 `{ code: "DOMAIN_ERROR", detail: { kind: "insufficient_capacity", data: { ... } } }`，无需把错误塞进 `String`。

```rust
// commands/allocation.rs —— service 错误原样向上传递（DomainError 经 mapper From 自动转换）
#[tauri::command]
pub async fn create_allocation(
    state: State<'_, AppState>,
    dto: CreateAllocationDto,
) -> Result<AllocationDto, AppError> {
    let new_alloc = dto::to_domain(dto)?;
    let created = state.services.allocation.create(&state.pool, new_alloc).await?;
    Ok(dto::to_dto(created, false))
}
```

**sqlx 错误转换**（避免在每个 repo 重复样板）：

```rust
// repo/mod.rs
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("row not found".into()),
            sqlx::Error::Database(ref d) if d.is_unique_violation() => {
                AppError::Validation(format!("duplicate: {}", d.message()))
            }
            other => AppError::Db(other.to_string()),
        }
    }
}
```

### 6.5 事务边界（sqlx 封装）

跨多表写操作必须在单个事务内，由 service 编排。`apply_solution` 是典型多写场景：把一批 allocation 原子落库，失败整体回滚，避免出现「半套方案」。

> **本轮决策（与 §3.7 / §5 一致）：** 所有写事务**统一走 `db::with_write_tx(&SqlitePool, …)`**，即 `BEGIN IMMEDIATE` + `busy_timeout` 兜底 + `SQLITE_BUSY` 退避重试。**严禁**直接 `pool.begin()`（DEFERRED、无重试，会在 AI 写与 UI 写并发时高频撞 `SQLITE_BUSY`，浪费已完成的工作）。该封装与 §3.7 的 `with_write_tx` 是同一份实现（§6 调用、§3 定义模板）。

```rust
// db/tx.rs —— 写事务统一封装：BEGIN IMMEDIATE + busy_timeout + SQLITE_BUSY 退避重试
// 与 §3.7 的 with_write_tx 为同一实现（签名统一为 db::with_write_tx）。
// busy_timeout 由连接池 PRAGMA 兜底（5000ms），此处再叠加应用层重试覆盖更长持锁。
pub async fn with_write_tx<F, T>(pool: &SqlitePool, f: F) -> Result<T, AppError>
where
    F: for<'c> FnOnce(&'c mut Transaction<'c, Sqlite>) -> BoxFuture<'c, Result<T, AppError>>
        + Copy, // Copy 便于重试时多次调用
{
    let mut backoff_ms = 50u64;
    for attempt in 0..3u32 {
        let mut tx = pool.begin().await?; // DEFERRED
        sqlx::query("BEGIN IMMEDIATE")    // 显式升级为 IMMEDIATE，立即取写锁
            .execute(&mut *tx).await
            .or_else(|e| retryable_busy(&e).then(|| ()).ok_or_else(|| AppError::from(e)))?;
        match f(&mut tx).await {
            Ok(v) => {
                tx.commit().await?;
                return Ok(v);
            }
            Err(AppError::Db(ref msg)) if is_busy_msg(msg) && attempt < 2 => {
                tracing::warn!(attempt, backoff_ms, "SQLITE_BUSY, retrying write tx");
                drop(tx);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                backoff_ms *= 2;
                continue;
            }
            Err(e) => return Err(e), // 业务错误或非 busy 的 DB 错误：直接抛出（事务随 drop 回滚）
        }
    }
    Err(AppError::Db("write tx exhausted SQLITE_BUSY retries".into()))
}

fn retryable_busy(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(d) if d.is_busy())
}
fn is_busy_msg(msg: &str) -> bool {
    msg.contains("database is locked") || msg.to_ascii_uppercase().contains("SQLITE_BUSY")
}
```

> **实现注记（impl 期决策）：** 上述 `BEGIN IMMEDIATE` 在已 `begin()` 的连接上再发一条 `BEGIN` 会与 sqlx 内部事务状态冲突；生产实现推荐两种之一：① 用 `SqliteConnectOptions` 在 pool 层把写事务路径配为 IMMEDIATE，或 ② 自行管理 `acquire()` 连接 + 手动 `BEGIN IMMEDIATE ... COMMIT`（不走 sqlx `Transaction` 的自动 BEGIN）。MVP 采用方案 ② 的薄封装，并在连接初始化即设 `PRAGMA busy_timeout = 5000` 作为第一道防线，应用层重试作为第二道。

```rust
// services/optimization_svc.rs —— apply_solution 事务边界（统一走 db::with_write_tx）
pub async fn apply_solution(&self, sol: &OptimizationSolution) -> Result<ApplyResultDto, AppError> {
    db::with_write_tx(&self.pool, |tx| Box::pin(async move {
        // 1. 预检：在事务内对每个候选 allocation 校验过载/冲突（见 6.7）
        for alloc in &sol.allocations {
            self.validate_allocation(tx, alloc).await?;
        }
        // 2. 批量插入（单事务）
        let n = repo::allocation::bulk_insert(tx, &sol.allocations).await?;
        // 3. 重算受影响资源的 workload（同一事务，保证一致）
        self.workload.recalculate_for_resources(tx, &sol.affected_resource_ids()).await?;
        Ok(ApplyResultDto { applied: n, skipped: 0 })
    })).await
}
```

**事务粒度规则：**

- 单 repo 单写：无需显式事务（sqlx 单条语句本身原子）。
- 跨 repo / 跨表写（`apply_solution`、`create_allocation` + 回写 workload、`split_long_task`）：用 `db::with_write_tx` 包裹。
- 长事务（如 `run_optimization` 中的 LLM 调用）：**事务内绝不调用 LLM**（耗时、易超时持锁）。求解在事务外完成，仅 `apply_solution` 落库时开事务。
- **统一约束：** 凡涉及写库的事务，一律走 `db::with_write_tx(&SqlitePool, …)`，禁止 `pool.begin()` 裸开。

### 6.6 异步与进度回传（长任务）

`run_optimization` 与 `export_report` 是秒级~分钟级长任务。用 **Tauri v2 的 `ipc::Channel`** 把类型化进度推给前端，比手动 `emit`/`listen` 更安全（编译期类型校验、自动清理）。

```rust
use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "phase")]
pub enum OptProgress {
    Started { total_tasks: usize },
    Stage { name: String, at: usize, of: usize }, // classic_solver / llm_scoring / explain
    Done { solution_id: i64 },
    Error { message: String },
}

#[tauri::command]
pub async fn run_optimization(
    state: State<'_, AppState>,
    scope: OptimizationScopeDto,
    on_event: Channel<OptProgress>,
) -> Result<OptimizationSolutionDto, AppError> {
    on_event.send(OptProgress::Started { total_tasks: scope.task_ids.len() })?;
    let sol = state.services.optimization
        .run(&scope, |p| { let _ = on_event.send(p); }) // 内部回调进度
        .await?;
    Ok(sol.into_dto())
}
```

> **注：** `Channel<T>` 是 Tauri v2 推荐的请求级进度回传方式；它会随 invoke 调用生命周期自动管理，无需前端手动 `unlisten`。若需广播给多窗口，再叠加 `app_handle.emit("opt-progress", …)`。

**前端调用（listen Channel）：**

```ts
import { invoke, Channel } from '@tauri-apps/api/core'

const ch = new Channel<OptProgress>()
ch.onmessage = (msg) => {
  switch (msg.phase) {
    case 'started':  store.total = msg.total_tasks; break
    case 'stage':    store.setProgress(msg.name, msg.at, msg.of); break
    case 'done':     store.solutionId = msg.solution_id; break
    case 'error':    store.error = msg.message; break
  }
}

const solution = await invoke<OptimizationSolutionDto>('run_optimization', {
  scope: { project_ids: [1, 2], unit: 'PM' },
  onEvent: ch,
})
```

**取消机制：** 长任务在 service 层持有 `CancellationToken`（`tokio_util::sync::CancellationToken`），暴露 `cancel_optimization(job_id)` command 触发取消；前端关闭进度弹窗即调用取消。

### 6.7 校验层（预检）

校验分两层：① domain 构造期的不变量（轻量、同步）；② service 层基于数据库状态的**预检**（跨实体，需查库）。预检结果用「软/硬」区分：硬约束违反 → 返回 `AppError::Validation`（或领域违反经 mapper 映射为 `AppError::Domain`）阻断；软约束违反 → 在 DTO 回填标记（如 `overload: true`）供前端告警，但仍允许保存（用户可覆盖）。

| 预检项 | 类型 | 触发点 | 说明 |
|---|---|---|---|
| 投入比例范围 | 硬 | domain 构造 | `0 < ratio ≤ 1.0` |
| 资源过载 | 软（可覆盖） | `create/update_allocation`、`apply_solution` | 区间内累计 PD > 容量 → 标记 `overload` |
| 跨项目时间冲突 | 硬 | allocation 预检 | 同资源、同日已被全额占用且无空余容量 → 阻断 |
| 技能达标 | 软 | allocation 预检 | 任务所需技能 vs 资源技能/熟练度 → 缺口写入 `skill_gap` |
| 任务依赖 | 硬 | task 状态变更 | 后置任务开工前，前置必须完成/到状态 |
| 时间窗合法 | 硬 | domain 构造 | `start ≤ end`，且落在项目周期内 |
| 删除有进行中分配 | 硬 | `delete_resource/task` | 存在未结束 allocation → 阻断并提示 |

**过载预检的算例（PD 计，默认 1 PD = 8h，1 PM = 20 PD）：**

> 资源 R 在 `2026-06-01 ~ 2026-06-30`（22 个工作日）的容量 = 22 PD × `ratio=1.0` = 22 PD。若已分配任务 A（10 PD）+ 任务 B（10 PD）= 20 PD，新增任务 C 需 5 PD：累计 25 PD > 22 PD → **过载 3 PD**，`overload=true`，利用率 = 25/22 ≈ 113.6%。`apply_solution` 中硬约束优化器会拒绝该方案；手动创建时软告警放行。

```rust
// services/allocation_svc.rs —— 预检返回结构化结果（软约束回填，硬约束报错）
pub struct AllocationCheck {
    pub overload_pd: f64,     // >0 表示过载
    pub skill_gap: Vec<String>,
    pub conflicts: Vec<i64>,  // 冲突的 allocation id
}

impl AllocationCheck {
    pub fn is_hard_violation(&self) -> bool {
        !self.conflicts.is_empty()
    }
}
```

### 6.8 配置与密钥管理

两类配置区分存储：

| 配置类型 | 内容 | 存储 | 说明 |
|---|---|---|---|
| 非敏感 | 单位偏好（PM/PD）、PD 工时、PM 折算系数、默认 provider、Ollama baseURL、UI 偏好、`secret_store` 策略 | SQLite `settings` 表（含 §3.3.1 迁移新增的 `secret_store` 列） | 可明文、可同步、可导出 |
| 敏感（密钥） | 云端 provider API key（OpenAI/Anthropic 等） | 优先 **OS keychain**（macOS Keychain / Windows Credential Manager / Linux Secret Service）via `keyring` crate；keychain 不可用时**降级**为应用层 AES-GCM 加密落 `settings.ai_api_key_enc`（见下方「降级路径」） | **绝不落明文文件**；降级路径必须在 UI 明确告知用户 |

#### 6.8.1 密钥存储后端与降级路径

本应用主打「离线、可分发」，但 **`keyring` crate 在 Linux 上依赖 Secret Service（D-Bus，需运行 `gnome-keyring` / `kwallet` 并已解锁）**。在纯 headless、无桌面环境（无 DE / 无 D-Bus secret service）的 Linux 上，`Entry::new(...)` 或 `set_password(...)` 会直接返回平台错误。为此 `secret_store` 采用**用户显式选择策略**（不做自动降级写回，见下方决策）：

- `settings.secret_store ∈ {'keychain', 'encrypted_file'}`（§3.3.1 迁移新增列）：审计可见当前密钥落在哪里。默认 `encrypted_file`（option b，零 keychain 依赖即可在所有平台工作）。
- **写 key 时严格按当前策略落地，不做 keychain→encrypted_file 自动降级**（本轮决策）：`set_ai_provider` 写 key 时，若策略为 `keychain` 而 `keyring` 返回平台不可用错误（非 `NoEntry`，如 `NoStorageAccess` / 平台 backend 缺失），直接返回 `AppError::Config`，由 UI 提示用户在设置页把 `secret_store` 显式切换为 `encrypted_file` 后重试。**不在 MVP 做自动写回**（自动降级写回与双向迁移列为 §9 路线图最后实现项）。
- `encrypted_file` 分支：用应用层 AES-256-GCM 加密 key 后写入 §3.3.1 已有的 `settings.ai_api_key_enc` 列（密文）。**主密钥来源（本轮决策，采纳方案 b）：由用户口令派生**——首启引导用户设置一个口令，用 Argon2id（机器绑定 salt + app pepper）从口令派生 32B 主密钥；明文 key 经 AES-256-GCM 加密为 `nonce||ciphertext||tag`，base64 后写 `ai_api_key_enc`。读取时需用户解锁口令（应用运行期缓存派生密钥于内存，退出即清）。**全程不落明文文件、不进 DTO、不进日志。**
- **keychain→encrypted_file 的回切/迁移到钥匙串不进 MVP**（本轮决策）：作为 §9 路线图最后实现项。MVP 阶段 `secret_store` 由用户显式选择并固定，`encrypted_file` 一旦启用即视为该机器的长期落点，不在设置页提供「迁移到钥匙串」按钮；待 MVP 后最后补齐该迁移交互（含迁移成功后清空 `ai_api_key_enc` 的校验）。

```rust
// infra/secrets.rs —— 统一抽象，内部按 secret_store 策略选择后端
use keyring::Entry;
use crate::domain::error::AppError;

const SERVICE: &str = "dev-kanban";

/// 密钥存储后端策略（落 settings.secret_store 新增列，便于审计与 UI 提示）
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecretStore {
    /// 系统 keychain：macOS Keychain / Windows Credential Manager / Linux Secret Service
    Keychain,
    /// 应用层 AES-256-GCM 加密落 settings.ai_api_key_enc（主密钥由用户口令派生，option b）
    EncryptedFile,
}

/// 尝试写 key：严格按当前策略落地。
/// MVP 不做 keychain→encrypted_file 自动降级写回（本轮决策）：
/// keychain 平台不可用时返回 Config 错误，由 UI 引导用户显式切换 secret_store 后重试。
pub async fn store_api_key(
    provider: &str,
    key: &str,
    store: SecretStore,
    db: &SqlitePool,
    master: &crypto::MasterKey, // 由用户口令派生（option b），见 §6.8.2
) -> Result<(), AppError> {
    match store {
        SecretStore::Keychain => match try_keychain_set(provider, key) {
            Ok(()) => Ok(()),
            Err(KeychainErr::PlatformUnavailable) => {
                // MVP：不自动降级，提示用户改选 encrypted_file
                Err(AppError::Config(
                    "当前环境不支持系统钥匙串，请在设置中把 secret_store 切换为 encrypted_file 后重试".into(),
                ))
            }
            Err(KeychainErr::Other(e)) => {
                Err(AppError::Config(format!("keychain set failed: {e}")))
            }
        },
        SecretStore::EncryptedFile => {
            crypto::encrypt_and_store(provider, key, master, db).await?;
            Ok(())
        }
    }
}

/// 读 key：按策略读取。
/// keychain 读失败（平台不可用 / NoEntry）时，若 DB 中存在历史密文，回退尝试解密
/// （兼容用户从 keychain 切到 encrypted_file 后仍能读到旧密钥）；否则返回 None。
pub async fn load_api_key(
    provider: &str,
    store: SecretStore,
    db: &SqlitePool,
    master: &crypto::MasterKey,
) -> Result<Option<String>, AppError> {
    match store {
        SecretStore::Keychain => match try_keychain_get(provider) {
            Ok(Some(k)) => Ok(Some(k)),
            Ok(None) => crypto::decrypt_and_load(provider, master, db).await, // 兼容历史密文
            Err(KeychainErr::PlatformUnavailable) => crypto::decrypt_and_load(provider, master, db).await,
            Err(KeychainErr::Other(e)) =>
                Err(AppError::Config(format!("keychain get failed: {e}"))),
        },
        SecretStore::EncryptedFile => crypto::decrypt_and_load(provider, master, db).await,
    }
}

fn try_keychain_set(provider: &str, key: &str) -> Result<(), KeychainErr> {
    Entry::new(SERVICE, provider)
        .map_err(KeychainErr::from)?
        .set_password(key)
        .map_err(KeychainErr::from)
}

fn try_keychain_get(provider: &str) -> Result<Option<String>, KeychainErr> {
    match Entry::new(SERVICE, provider).and_then(|e| e.get_password()) {
        Ok(k) => Ok(Some(k)),
        Err(keyring::Error::NoEntry) => Ok(None),
        // 平台无 secret service（典型 Linux headless）→ 归类为可降级
        Err(e) => Err(KeychainErr::classify(e)),
    }
}

/// keychain 错误分类：区分「平台不可用（可降级）」与「其它真实错误」
enum KeychainErr {
    PlatformUnavailable,
    Other(String),
}
```

```rust
// infra/crypto.rs —— AES-256-GCM 加密落 settings.ai_api_key_enc（schema 已有该列）
// 主密钥来源（本轮决策，方案 b）：用户口令派生。
//   - 首启引导设置口令（不可跳过、不可为空），口令本身永不落盘；
//   - 用 Argon2id(password, salt=机器绑定, pepper=app 绑定) 派生 32B 主密钥（见 master_key 模块）；
//   - 应用运行期在内存中缓存派生密钥（受 OS 内存保护），退出/锁屏即清；
//   - 明文 key 经 AES-256-GCM 加密为 nonce||ciphertext||tag，base64 后写 ai_api_key_enc。
pub struct MasterKey([u8; 32]); // 仅驻内存，进程退出即丢

pub async fn encrypt_and_store(
    provider: &str, key: &str, master: &MasterKey, db: &SqlitePool,
) -> Result<(), AppError> {
    let cipher = aes_gcm_encrypt(&master.0, key.as_bytes())?; // nonce||ct||tag, base64
    settings_repo::set_ai_api_key_enc(db, provider, &cipher).await?;
    Ok(())
}

pub async fn decrypt_and_load(
    provider: &str, master: &MasterKey, db: &SqlitePool,
) -> Result<Option<String>, AppError> {
    match settings_repo::get_ai_api_key_enc(db, provider).await? {
        Some(cipher) => {
            let plain = aes_gcm_decrypt(&master.0, &cipher)?;
            Ok(Some(String::from_utf8(plain).map_err(|e| AppError::Config(e.to_string()))?))
        }
        None => Ok(None),
    }
}

// master_key 模块（infra/master_key.rs）
// - derive_from_passphrase(passphrase): Argon2id(m=64MiB, t=3, p=4) over (passphrase, machine_salt, app_pepper) -> MasterKey([u8;32])
//   · machine_salt: 安装时随机生成的 32B，落 app 私有目录（不随 DB 导出），使同口令在不同机器派生不同密钥
//   · app_pepper: 编译期常量，与机器 salt 一起喂入 Argon2id
// - 首启 set_passphrase / 后续 unlock_passphrase / change_passphrase
//   （换口令需用旧主密钥解密所有 ai_api_key_enc，再用新主密钥重加密后落库）
```

**首启引导（设置口令）流程：**

1. 首次启动检测到 `settings` 已初始化但「口令未设置」标志为真 → 弹出全屏引导，要求用户设置一个口令（含强度提示、二次确认）；口令仅用于派生主密钥，**不落盘、不进日志、不上报**。
2. 引导完成后立即用派生密钥加密一个已知 magic 串写入校验位，用于后续解锁时验证口令正确性（避免「错口令 → 解密出垃圾」）。
3. 后续每次启动 / 锁屏后恢复，若 `secret_store == 'encrypted_file'` 且检测到有 `ai_api_key_enc`，弹出解锁框；口令正确后派生密钥缓存于内存。
4. 默认 provider = Ollama（本地、无 key），即使用户不设置云端 key，口令也仅需设置一次（占位），不阻塞离线使用。

**关键设计点：**

- 密钥只在 `infra::llm` 构造 rig provider 时按需从 keychain（或降级加密存储）读取，**全程只存在于内存**，不进 DTO、不进日志、不进 `SettingsDto`。`get_ai_provider` 返回的 `AiProviderDto` 只含 `has_key: bool` 与 `secret_store` 策略，不返回明文。
- `test_ai_connection` 用内存中的 key 做一次性连通测试，结果只回 `ok/err`，不回显 key。
- `baseURL`、`model` 等非敏感项走普通 settings 表，可被 `update_settings` 修改。
- 本地优先策略：默认 provider = Ollama（本地，无 key），keychain 为空也能离线运行；只有用户显式切换到云端 provider 并存 key 时才依赖密钥。
- **策略审计**：`secret_store` 字段（新增列）使运维/用户可审计密钥实际落在何处。MVP 期内 keychain 平台不可用时不自动降级写回，而是返回 `AppError::Config` 引导用户显式切换到 `encrypted_file`；自动降级与双向迁移列为 §9 路线图最后实现项。
- **口令派生主密钥（方案 b）的安全收益**：相比「随机生成落 app 私有目录」（方案 a，零交互但主密钥文件可被同机其它进程读取），口令派生保证即便 DB 文件与 app 目录一同被拷走，没有口令也无法解密 `ai_api_key_enc`；代价是首启与解锁需用户交互，以及换口令需批量重加密。权衡后采纳方案 b。

### 6.9 典型 Command 签名与前端调用示例

下面给出 3 个有代表性的 command 完整签名 + 前端 invoke，覆盖：普通 CRUD（带事务）、长任务（带进度 Channel）、AI 业务动作（apply_solution）。

#### 示例 1：create_allocation（普通写入，含事务与预检）

```rust
#[tauri::command]
pub async fn create_allocation(
    state: State<'_, AppState>,
    dto: CreateAllocationDto,
) -> Result<AllocationDto, AppError> {
    let new_alloc = dto::to_domain(dto)?;
    let (created, check) = state.services.allocation.create_with_check(&state.pool, new_alloc).await?;
    Ok(dto::to_dto(created, check.overload_pd > 0.0))
}
```

```ts
try {
  const a = await invoke<AllocationDto>('create_allocation', {
    dto: { resource_id: 7, task_id: 42, ratio: 0.5, start: '2026-06-27', end: '2026-07-10' },
  })
  if (a.overload) notify.warn('该资源在此区间已过载，已保存但建议复核')
} catch (e) {
  // e 形如 { code: 'VALIDATION', detail: '...' } 或 { code: 'DOMAIN_ERROR', detail: { kind:'...', data:{...} } }
  notify.error((e as AppError).detail)
}
```

#### 示例 2：run_optimization（长任务 + Channel 进度）

签名见 6.6。前端调用（含取消）：

```ts
const jobId = crypto.randomUUID()
const ch = new Channel<OptProgress>()
ch.onmessage = (m) => progressStore.onProgress(m)
try {
  const sol = await invoke<OptimizationSolutionDto>('run_optimization', {
    scope: { project_ids: [1, 2], unit: 'PM', job_id: jobId },
    onEvent: ch,
  })
  solutionStore.set(sol)
} catch (e) {
  notify.error((e as AppError).detail)
}
// 用户关闭弹窗 → 取消
await invoke('cancel_optimization', { jobId })
```

#### 示例 3：apply_solution（批量事务落库）

```rust
#[tauri::command]
pub async fn apply_solution(
    state: State<'_, AppState>,
    solution_id: i64,
    options: ApplyOptionsDto, // { dry_run: bool, overwrite_existing: bool }
) -> Result<ApplyResultDto, AppError> {
    state.services.optimization.apply(solution_id, options).await
}
```

```ts
const res = await invoke<ApplyResultDto>('apply_solution', {
  solutionId: 12,
  options: { dry_run: true, overwrite_existing: false },
})
// res = { applied: 18, skipped: 2 } —— dry_run 预演后再正式 apply
```

### 6.10 Tauri 注册与 AppState

所有 command 在 `main.rs` 集中注册；`AppState` 通过 `tauri::Manager::manage` 注入，持有连接池、service 容器、rig provider 工厂，以及（option b）运行期驻内存的派生主密钥。

```rust
// main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default()) // 可选
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::resource::create_resource,
            commands::resource::list_resources,
            // … 全部 command …
            commands::optimization::run_optimization,
            commands::optimization::apply_solution,
            commands::allocation::recalculate_workload,
            commands::report::export_report,
            commands::settings::get_settings,
            commands::settings::set_master_password, // option b：首启/解锁/重置主口令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub struct AppState {
    pub pool: SqlitePool,
    pub services: Services, // 内含 resource_svc / allocation_svc / optimization_svc …
    // option b：由用户口令派生的主密钥，首启/解锁后驻内存；未解锁时为 None
    // （云 key 不可读，但本地 Ollama provider 仍正常工作）
    pub master_key: std::sync::OnceLock<crypto::MasterKey>,
}
```

```rust
// services/mod.rs —— service 容器，依赖注入便于测试替身
pub struct Services {
    pub resource: ResourceService,
    pub team: TeamService,
    pub project: ProjectService,
    pub task: TaskService,
    pub allocation: AllocationService,
    pub workload: WorkloadService,
    pub optimization: OptimizationService,
    pub report: ReportService,
    pub settings: SettingsService,
}
```

**测试策略：** `services/*` 因零 Tauri 依赖，可用 `sqlx::test`（内存/临时 SQLite 文件）+ trait 抽象的 repo 替身做单测；`commands/*` 仅做薄层集成测试（验证序列化/错误透传），业务正确性下沉到 service 测试。这保证核心规则（过载、技能匹配、事务原子性）在 CI 中可独立验证。

---

### 假设（本节）

1. `domain/*` 层零 `serde` 依赖：`DomainError` 不派生 `serde::Serialize`；IPC 边界由 `dto/mappers.rs` 的 `From<DomainError> for AppError` 把领域错误映射为 `AppError::Domain(DomainErrorDetail)`（结构化，仍可序列化）或 `AppError::Validation(String)`（字符串）。领域层保持纯净、可脱离 Tauri/serde 单测与复用。

2. 所有写事务统一走 `db::with_write_tx(&SqlitePool, …)`：`BEGIN IMMEDIATE` + `busy_timeout`（连接池 PRAGMA 5000ms）+ 应用层 `SQLITE_BUSY` 退避重试（指数退避 50/100/200ms，最多 3 次）。与 §3.7 / §5 的 `with_write_tx` 为同一份实现，严禁 `pool.begin()` 裸开。

3. `settings.secret_store ∈ {'keychain','encrypted_file'}` 为 §3.3.1 settings 表后续迁移（`ALTER TABLE ... ADD COLUMN`）新增列（非 settings.metadata JSON），由 `settings_svc` 直接读写，`SettingsDto` 透出该字段供 UI 显示当前密钥落点。

4. `encrypted_file` 降级路径的主密钥由**用户口令派生**（方案 b）：首启引导设置口令，Argon2id（机器绑定 salt + app pepper）派生 32B 主密钥；口令不落盘、不进日志、不上报。明文 key 经 AES-256-GCM 加密为 `nonce||ciphertext||tag` 后写 `settings.ai_api_key_enc`。换口令需用新旧主密钥重加密所有密文。

5. `settings.ai_api_key_enc` 列（§3.3.1 已定义）作为 keychain 降级路径的落库目标，应用层 AES-256-GCM 加密；主密钥由用户口令派生（见假设 4），机器绑定、不随 DB 导出。

6. keychain→encrypted_file 的「回切/迁移到钥匙串」交互**不进 MVP**，列为 §9 路线图的最后实现项；MVP 阶段 `encrypted_file` 一旦启用即视为该机器长期落点，不提供迁移按钮。

### 开放问题（本节）

1. **[impl 期决策]** `db::with_write_tx` 的 `BEGIN IMMEDIATE` 在 sqlx 已 `begin()` 的连接上存在事务状态冲突。生产实现需在 ① `SqliteConnectOptions` 把写路径配为 IMMEDIATE、② 自行 `acquire()` 连接 + 手动 `BEGIN IMMEDIATE ... COMMIT`（不走 sqlx `Transaction` 自动 BEGIN）之间二选一，并在 impl 期验证 `is_busy()`/消息匹配的跨 sqlx 版本稳定性。

2. **[impl 期决策]** 口令派生的 Argon2id 参数（m/t/p，建议起点 m=64MiB, t=3, p=4）与「机器绑定 salt」「app pepper」的具体来源（如机器指纹哈希 vs 安装时随机落 app 私有目录的 salt 文件）需在 impl 期定稿并加单测；需评估低配机器（如旧 Mac）上派生耗时是否影响首启/解锁体验。

3. **[impl 期决策]** 首启口令引导与运行期解锁的 UX 落点：是否提供「记住口令（Keychain 中存派生密钥，下次自动解锁）」选项以减少重复输入，以及锁屏/切后台多久后强制重新解锁。需与 §7 前端 UI 设计对齐。

## 7. 前端与 UI / Frontend & UI

本节定义前端技术选型、目录与模块结构、状态管理、与 Rust 后端的 IPC 交互层，以及七个核心视图的布局草图、交互契约与跨视图联动机制。所有视图共享同一套领域 store，确保「选中某资源/项目」可在 Dashboard / Kanban / Gantt / 日历之间高亮过滤。

---

### 7.1 技术选型确认

| 维度 | 选型 | 理由 | 备选 |
|---|---|---|---|
| 框架 | **Vue 3 `<script setup>` + 组合式 API** | 团队既有决策；TS 友好、编译产物小，契合 Tauri 单用户桌面包体约束 | React（已排除） |
| 构建 | **Vite 5+** | 官方 Tauri v2 模板默认；HMR 快，`frontendDist` 指向 `dist`，`devUrl` 指向 Vite dev server（默认 `http://localhost:1420`） | — |
| 语言 | **TypeScript（strict）** | 与 Rust 侧 `#[derive(Serialize)]` 产出对齐，IPC 层端到端类型安全 | — |
| 路由 | **Vue Router 4** | 视图较多（7 个核心视图 + 子路由），需声明式路由与懒加载 | — |
| 状态管理 | **Pinia** | 组合式 store、SSR 无关、TS 推导完善；按领域拆 store（见 7.3） | Vuex（已废弃） |
| UI 组件库 | **Naive UI**（推荐） | 原生 TS 编写、内置 tree-shaking、内置暗色主题、无 CSS 运行时依赖，最适合 Tauri 对包体与启动速度敏感的桌面应用；90+ 组件覆盖表格/树/抽屉/虚拟滚动 | Element Plus（中文生态成熟，若团队更熟悉可替换）、PrimeVue（组件最全，但 tree-shaking 不稳定，需验证产物体积） |
| Gantt | **dhtmlx-gantt**（推荐）+ 自绘 SVG 衬底 | 成熟的资源负载视图、依赖连线、拖拽改区间/投入比例、auto-scheduling；开源 GPL 版可满足本地单用户场景，商业版用于高级资源视图 | frappe-gantt（轻量但无资源维度、拖拽弱，仅适合纯时间轴）、Bryntum Gantt（商业、最强但成本高） |
| 日历 | **自绘 SVG/Canvas 日历**（推荐 MVP） | 人力占用/请假/节假日是领域特化需求，自绘可精确控制单元格热度色与容量阈值线；后续若需重型排期可引入 FullCalendar Premium `resource-timeline` | vue-cal（免费无依赖，但无真正 resource-timeline 行）、FullCalendar（Premium 才有 resource-timeline，付费） |
| Kanban | **自绘（flex 列 + 拖拽）** | 看板列与卡片为领域强定制（卡片须显示 workload 条、技能徽标、过载红点），自绘最灵活；拖拽用原生 HTML5 DnD 或 `vue-draggable-plus` | vuedraggable、trello-vue（约束多不推荐） |
| 图表 | **ECharts 5**（推荐） | Dashboard 的利用率柱状/折线、堆叠面积、雷达（技能覆盖）均原生支持；按需引入减少体积 | Chart.js（更轻但交互弱） |
| 图标 | **xicons**（`@vicons/*`） | 与 Naive UI 同体系，tree-shakeable | lucide-vue-next |
| IPC 封装 | **`@tauri-apps/api/core` 的 `invoke`**（v2 路径） | Tauri v2 中 `invoke` 迁移至 `@tauri-apps/api/core`，须注意不是旧版 `@tauri-apps/api/tauri` | — |
| 单位/数字 | **`decimal.js` 或自写 PD/PM 转换工具** | 避免浮点误差（0.1 PD + 0.2 PD）；统一 `{ value, unit }` 二元组 | bigint（对 PM 精度需求过度） |
| 桌面通知 | **Tauri v2 `notification` 插件**（`@tauri-apps/plugin-notification`） | 桌面原生通知能力，预警中心推送（仅桌面，不做邮件/Web Push；邮件/推送已在 §1.5 排除） | Web Notifications API（Tauri 桌面下建议走原生插件） |

> 选型结论一句话：**Naive UI + 自绘 Kanban/日历 + dhtmlx-gantt + ECharts + Tauri notification 插件**，全部 tree-shakeable，控制在桌面包增量约 1.2–1.8 MB（gzip 后）。

---

### 7.2 目录与模块划分

前端源码置于仓库 `src/`，与 `src-tauri/`（Rust）平级。按「领域模块 + 共享基础设施」分层：

```
src/
├── main.ts                      # 应用入口，挂载 Pinia/Router/Naive UI
├── App.vue                      # 根布局：左侧导航 + 顶栏 + <RouterView>
├── router/
│   └── index.ts                 # 路由表（7 视图懒加载）
├── api/                         # ← 与后端交互层（见 7.4）
│   ├── invoke.ts                # typed invoke 封装 + 错误归一化
│   ├── commands/                # 按领域拆分的命令模块
│   │   ├── resource.ts
│   │   ├── team.ts
│   │   ├── project.ts
│   │   ├── task.ts
│   │   ├── allocation.ts
│   │   └── optimization.ts
│   └── types.ts                 # 与 Rust Serialize 对齐的 DTO 类型
├── stores/                      # ← Pinia 领域 store（见 7.3）
│   ├── resource.ts
│   ├── team.ts
│   ├── project.ts
│   ├── task.ts
│   ├── allocation.ts
│   ├── optimization.ts
│   ├── alert.ts                 # 预警中心：预算超支/过载/技能缺口/依赖阻塞聚合 + 严重度排序 + 桌面通知
│   └── ui.ts                    # 全局 UI 状态：选中实体、单位、主题、过滤条件
├── composables/                 # 跨视图复用的组合式函数
│   ├── useSelection.ts          # 全局选中态（资源/项目）→ 触发跨视图联动
│   ├── useWorkload.ts           # workload/利用率 计算（PD/PM）
│   ├── useUnits.ts              # PD↔PM 转换（读取配置 N=20）
│   ├── useCapacity.ts           # 扣除节假日/请假算容量
│   ├── useDependency.ts         # 依赖图查询：后继受影响链（限深 1-2 层）/ blocked 态 / 环检测预览
│   ├── useDesktopNotify.ts      # 桌面通知（Tauri notification 插件）：按严重度路由
│   └── useOptimistic.ts         # 乐观更新通用包装
├── views/                       # ← 七大核心视图（见 7.5）
│   ├── DashboardView.vue
│   ├── KanbanView.vue
│   ├── GanttView.vue
│   ├── CalendarView.vue
│   ├── ResourceCenterView.vue
│   ├── OptimizationPanelView.vue
│   └── ReportCenterView.vue
├── components/                  # 视图内与跨视图复用组件
│   ├── common/                  # EntityTag, SkillBadge, WorkloadBar, OverloadDot, DependencyLock, ProjectCountBadge
│   ├── dashboard/
│   ├── kanban/
│   ├── gantt/
│   ├── calendar/
│   ├── resource/
│   └── optimization/
│       ├── ObjectiveWeightsPanel.vue   # 多目标权重滑杆/开关组（见 §7.5.6）
│       └── WhatIfScenarioStack.vue     # 多 What-if 场景叠加对比（见 §7.5.6）
├── utils/
│   ├── date.ts                  # 时间窗、工作日计算
│   └── units.ts                 # PD/PM 数值与格式化
├── styles/
│   ├── tokens.css               # 设计 token（颜色/间距/过载阈值色阶/依赖违反紫/待交接灰/影响链深度色阶）
│   └── naivue-theme.ts
└── assets/
```

模块边界原则：`api/` 只负责类型化调用与错误归一化，不持有状态；`stores/` 持有领域数据与派生计算；`composables/` 是无状态逻辑；`views/` 组装组件并订阅 store；`components/` 只通过 props/events 与父级通信，禁止直接读 store（除少数全局 UI 组件）。

---

### 7.3 状态管理设计（Pinia 按领域拆分）

按业务领域拆为 7 个 store，加全局 UI store 与预警中心 store。每个 store 遵循 `state / getters / actions` 三段式，action 内调用 `api/commands/*`。

#### Store 职责矩阵

| Store | 持有状态（state） | 关键 getters（派生） | 关键 actions |
|---|---|---|---|
| `useResourceStore` | `resources: Resource[]`、`skills: Skill[]`、`tags: Tag[]`、`holidays: Date[]`、`leaves: Leave[]` | `byId`、`resourceCapacity(id, window)`（扣请假/节假日×投入比例）、`skillCoverage(skillId)`、`crossProjectCount(id, window)`（该资源在窗口内跨项目数） | `fetchResources`、`upsertResource`、`setLeave`、`archiveResource(id, handover)` |
| `useTeamStore` | `teams: Team[]` | `teamWorkload(teamId, window)`（聚合成员）、`teamCapacity` | `fetchTeams`、`addMember` |
| `useProjectStore` | `projects: Project[]` | `activeProjects`、`projectHealth(id)`（预算 vs 已分配）、`budgetUsage(id, window)`、`budgetOverrun(id, window)`（usage > `projects.budget_alert_threshold`，该阈值**按项目细分**，默认回落全局 `overload_threshold`，见下「预算预警阈值口径」） | `fetchProjects`、`upsertProject`、`checkBudgetAlert(id)` |
| `useTaskStore` | `tasks: Task[]`、`taskTags: Tag[]` | `byProject`、`byStatus`、`overdueTasks`、`blockedTasks`（前置未完成，卡片置灰带锁标）、`dependents(taskId, depth=2)`（该任务的后继影响链，**限深 1-2 层**，见 §7.5.3） | `fetchTasks`、`moveStatus`（Kanban 拖拽） |
| `useAllocationStore` | `allocations: Allocation[]` | `workloadByResource(window)`、`workloadByTeam`、`conflicts`（跨项目时间窗重叠）、`dependencyViolations(preview)`（拖拽预览下 `start < max(predecessor.end)` 的条带） | `fetchAllocations`、`upsertAllocation`、`bulkApply(plan)`、`lockResource(id)`、`excludeResource(id, window)`、`simulateUnavailable(id, window)` |
| `useOptimizationStore` | `currentPlan: Plan \| null`、`candidatePlan: Plan \| null`、`whatIfStack: Plan[]`（多 What-if 场景叠加，见 §7.5.6）、`lastRun: RunMeta`、`diff: AllocationDiff[]` | `isOverloaded(plan)`、`coverageScore(plan)`、`mergedWhatIfPreview()`（把 `whatIfStack` 中 2+ 场景与当前方案叠加，输出供 Gantt 渲染的条带 union） | `runOptimization(constraints)`、`acceptCandidate`、`discardCandidate`、`pushWhatIf(plan)`、`removeWhatIf(id)`、`clearWhatIfStack()` |
| `useAlertStore` | `dismissed: Set<alertKey>`（**会话内状态，不持久化**，见「预警 dismiss 口径」）、`severityOrder: SeverityKey[]`（四类预警严重度排序，可配）、`desktopNotifyMinSeverity: Severity`（触发桌面通知的最低严重度阈值，可配） | `overloads`（来自 `useAllocationStore.conflicts`/过载段）、`budgetOverruns`（来自 `useProjectStore.budgetOverrun`）、`skillGaps`（来自 `optimization.diff` 未覆盖技能）、`dependencyBlocked`（来自 `useTaskStore.blockedTasks`）、`allActive`（**按 `severityOrder` 排序**的聚合列表）、`notifyable`（严重度 ≥ `desktopNotifyMinSeverity` 且本会话未通知过的条目） | `refresh()`、`dismiss(key)`（仅写入会话内 `dismissed`）、`setSeverityOrder(keys)`、`setDesktopNotifyMinSeverity(sev)`、`requestNotifyPermission()` |
| `useUiStore` | `selectedResourceId`、`selectedProjectId`、`selectedTaskId`、`unit: 'PD'\|'PM'`、`theme`、`globalFilter: { window, tags[], showOverloadOnly }` | `activeEntity` | `selectResource`、`selectProject`、`toggleUnit` |

> **统一用词**：分配求解结果一律称 **Allocation**（不再用 Assignment），前后端 DTO、store、组件命名一致。

> **预算预警阈值口径**（决策#1）：预算预警判定为 `budget_usage(id) > projects.budget_alert_threshold`，其中 `budget_alert_threshold` 是**项目级可配字段**（列定义在 §6.2.3 / §3.3 projects 表，`REAL`，单位与预算一致为 PD），**未显式设置时回落全局 `overload_threshold`**（默认 110%，§4.x / §9.2.1）。即默认口径下「预算超支」与「资源过载」共用同一阈值语义；对风控/强约束项目可在项目详情单独收紧（如设为 100%）。`useProjectStore.budgetOverrun(id, window)` getter 取值顺序：`projects.budget_alert_threshold ?? config.overload_threshold`。

> **预警 dismiss 口径**（决策#7）：`useAlertStore.dismissed` 为**会话内状态，不落库、不跨会话持久化**——每次打开应用预警中心按 store getter 全量重新计算，「忽略」仅让该条目在当前会话的角标计数与下拉列表中隐藏，直到触发数据变更使其消失/复发或用户重启应用。设计意图：避免「预警被永久忽略后问题恶化却无人察觉」；重要风险应常驻直到根因消除。会话内 dismiss 记录仅在 `useAlertStore` 的内存 `Set<alertKey>` 中（`alertKey` = `${type}:${entityId}:${window}`），不进 SQLite、不写 `localStorage`（Pinia 可选用易失 session storage 仅做页面刷新恢复，关闭应用即丢）。

> **预警中心数据源**：`useAlertStore` 不持久化业务数据，仅做跨 store 聚合与去抖。所有预警均由对应领域 store 的 getter 实时计算，store 任一字段变更即在下一 tick 刷新 `allActive`，保证「打开 Dashboard 即见预警、改一条 allocation 即重算」。预警检测为派生计算，目标延迟见 §9.2.1 性能表（与 workload 增量重算同口径 ≤ 200ms）。

> **严重度排序可配 + 桌面通知**（决策#5）：四类预警（预算超支 / 过载 / 技能缺口 / 依赖阻塞）在 `allActive` 中的排序由 `severityOrder` 控制（默认顺序：预算超支 > 过载 > 依赖阻塞 > 技能缺口，可在「设置 → 预警」拖拽重排并持久化到 `app_config`）。桌面通知（不做邮件/Web Push，§1.5 已排除）由 `useDesktopNotify` 组合式函数驱动：当 `notifyable` 集合非空且用户已授权（Tauri `notification` 插件 `requestPermission`）时，按 `desktopNotifyMinSeverity` 过滤后弹出系统通知，点击通知跳转对应视图（与下拉「处理」按钮同路由）。同一 `alertKey` 在一个会话内只通知一次（去重），避免抖动。

#### 单位与数值约定

PD/PM 在 store 层统一以 **PD** 为内部基准存储，展示层按 `useUiStore.unit` 转换。转换参数来自本地配置表（默认 `1 PD = 8h`，`1 PM = 20 PD`）：

```ts
// utils/units.ts
export const PM_TO_PD = 20 // 来自配置，可改
export function toDisplay(pd: number, unit: 'PD' | 'PM'): number {
  return unit === 'PM' ? pd / PM_TO_PD : pd
}
export function toPD(display: number, unit: 'PD' | 'PM'): number {
  return unit === 'PM' ? display * PM_TO_PD : display
}
```

> 浮点处理：所有 PD 运算四舍五入到 2 位小数；存储用整数厘 PD（`pd_x100`）可彻底避免误差，MVP 阶段先用 `Math.round(x*100)/100`。

#### Workload / 利用率计算（核心公式）

对资源 $r$、时间窗 $W=[W_s, W_e]$：

$$\text{Workload}(r, W) = \sum_{a \in \text{alloc}(r) \cap W} a.\text{effortPD} \times a.\text{ratio}$$

$$\text{Capacity}(r, W) = \text{workdays}(W) - \text{leaves}(r,W) - \text{holidays}(W)$$

$$\text{Utilization}(r, W) = \frac{\text{Workload}(r,W)}{\text{Capacity}(r,W)}$$

利用率色阶（写入 `tokens.css`）：

| Utilization | 颜色 | 含义 |
|---|---|---|
| < 70% | 绿 `#52c41a` | 空闲 |
| 70%–95% | 蓝 `#1890ff` | 健康 |
| 95%–100% | 琥珀 `#faad14` | 接近满载 |
| > 100% | 红 `#f5222d` + OverloadDot | 过载（硬约束违反） |

补充状态色（写入 `tokens.css`，与利用率色阶并列）：

| 状态 | 颜色 | 含义 |
|---|---|---|
| 依赖违反（drag 预览） | 紫 `#722ed1` 描边 | allocation `start < max(predecessor.end)`，硬约束#5 |
| 依赖影响链（直接后继） | 紫 `#9254de` 半透明闪烁 | 仅渲染直接后继 1-2 层，见 §7.5.3 / §7.9 性能 |
| 待交接（资源 archived） | 灰 `#bfbfbf` 描边 + 斜纹 | 归属资源已离职/调岗，allocation 待迁移 |

---

### 7.4 与后端交互层

#### 7.4.1 Typed invoke 封装

在 `api/invoke.ts` 对原生 `invoke` 做一层泛型封装，集中处理：序列化、错误归一化、调用日志、可选 abort。Rust 侧 `#[tauri::command]` 返回 `Result<T, AppError>`，前端统一为 `AppError` 形态。

```ts
// api/invoke.ts
import { invoke } from '@tauri-apps/api/core'

export interface AppError {
  code: string            // 'DB_CONFLICT' | 'OPT_INFEASIBLE' | 'LLM_TIMEOUT' | 'DEPENDENCY_CYCLE' | ...
  message: string
  details?: unknown
  // 是否可重试
  retryable: boolean
}

export class IpcError extends Error {
  constructor(public raw: AppError) { super(raw.message) }
}

/**
 * 类型化 invoke。T 为 Rust 命令返回的 Ok 类型。
 * 所有命令统一返回 Result<T, AppError>，此处只暴露 Ok 通道。
 */
export async function call<T>(
  cmd: string,
  args?: Record<string, unknown>,
  opts?: { signal?: AbortSignal },
): Promise<T> {
  // 可选：在 opts.signal abort 时抛出（Tauri 暂无原生 cancel，靠应用层标记）
  try {
    const res = await invoke<Result<T, AppError>>(cmd, args)
    if (res && typeof res === 'object' && 'Err' in res) {
      throw new IpcError((res as any).Err)
    }
    return (res as any).Ok ?? (res as T)
  } catch (e) {
    // Tauri 在命令返回 Err 时会 reject promise，payload 即 AppError
    if (e instanceof IpcError) throw e
    const raw: AppError = (e as any)?.code
      ? (e as AppError)
      : { code: 'UNKNOWN', message: String(e), retryable: false }
    throw new IpcError(raw)
  }
}
```

```ts
// api/commands/allocation.ts
import { call } from '../invoke'
import type { Allocation, AllocationInput, WorkloadWindow } from '../types'

export const allocationApi = {
  list: (w: WorkloadWindow) => call<Allocation[]>('allocation_list', { window: w }),
  upsert: (input: AllocationInput) =>
    call<Allocation>('allocation_upsert', { input }, /* optimistic id */ undefined),
  remove: (id: string) => call<void>('allocation_remove', { id }),
  applyPlan: (planId: string) => call<void>('allocation_apply_plan', { planId }),
}
```

#### 7.4.2 错误处理策略

| 错误类型 | code 示例 | UI 反馈 |
|---|---|---|
| 约束不可满足 | `OPT_INFEASIBLE` | 全局 toast + 优化面板高亮冲突条目，**不回滚乐观更新**（因无更新发生） |
| 数据库冲突 | `DB_CONFLICT` | toast + 回滚该条乐观更新 |
| LLM 超时 | `LLM_TIMEOUT` | 优化面板降级为「仅经典优化器结果」，标记解释列缺失 |
| 校验失败 | `VALIDATION` | 表单内联错误（Naive UI `n-form` 的 `validate`） |
| 依赖环 / 依赖违反 | `DEPENDENCY_CYCLE` / `DEPENDENCY_VIOLATION` | Gantt 拖拽 toast「依赖冲突：需先完成 #T-xxx」；条带置依赖违反态（紫色描边），不落库 |

错误归一化后由 `composables/useToast`（基于 Naive UI `useMessage`）统一展示；表单类错误走组件内联。

#### 7.4.3 乐观更新策略

以 Kanban 拖拽改状态、Gantt 拖拽改区间为例，统一用 `useOptimistic` 包装：

```ts
// composables/useOptimistic.ts
export function useOptimistic<T extends { id: string }>() {
  const optimistic = ref<Map<string, T>>(new Map())

  /** 乐观写入：先改本地 + 记快照，失败回滚 */
  async function apply(
    list: Ref<T[]>,
    temp: T,
    remote: () => Promise<T>,
  ): Promise<void> {
    const snapshot = list.value.find(x => x.id === temp.id)
    const idx = list.value.findIndex(x => x.id === temp.id)
    if (idx >= 0) list.value[idx] = temp
    else list.value.push(temp)
    try {
      const saved = await remote()
      const i = list.value.findIndex(x => x.id === temp.id)
      list.value[i] = saved
    } catch (e) {
      // 回滚
      if (snapshot && idx >= 0) list.value[idx] = snapshot
      else list.value = list.value.filter(x => x.id !== temp.id)
      throw e
    }
  }
  return { apply }
}
```

**冲突检测**：因为单用户本地 SQLite、无并发写，乐观更新主要用于掩盖 IPC 往返延迟，回滚主要应对校验失败。复杂批量（如接受 AI 方案、资源离职批量迁移、What-if 多场景叠加 diff 应用）不走乐观，直接 `loading` 态 + 全量刷新。

---

### 7.5 核心视图设计

每个视图给出 ASCII 布局草图与交互说明。统一顶栏含：单位切换 `PD ⇄ PM`、全局时间窗选择器、`仅看过载` 开关、主题切换、**预警中心铃铛**（角标显示 `useAlertStore.allActive` 数量，点击展开下拉；新增**桌面通知**能力，见 §7.5.1）。

---

#### 7.5.1 Dashboard 总览

职责：人/团队 workload 概览、过载预警、项目健康度、**预警中心**（聚合过载/预算超支/技能缺口/依赖阻塞四类）。打开应用默认落地页。

```
┌──────────────────────────────────────────────────────────────────────┐
│ 顶栏: [单位 PD▾] [窗口 本月▾] [☐仅看过载] [🔔预警(5) 🔔桌面通知] [🌙]  │
├──────────────────────────────────────────────────────────────────────┤
│ ┌─KPI 卡片─────────────────────────────────────────────────────────┐ │
│ │ 总人力 24   已分配 412 PD   利用率 78%   过载 3 人   缺口 56 PD  │ │
│ └──────────────────────────────────────────────────────────────────┘ │
│ ┌─预警中心(useAlertStore.allActive, 按 severityOrder 排序)────────┐  │
│ │ 🔴 风控项目预算超支 106.7%(预算6.0PM 阈值100%项目级 已耗6.4PM)[处理]│ │
│ │ 🔴 张三 利用率 112.5% 连续两周过载(本周22.5/20PD)           [处理][×]│ │
│ │ 🟡 技能缺口: #T-310 缺 DevOps L4(候选 0 人)                 [处理]│  │
│ │ 🟡 依赖阻塞: #T-301 部署 被前置 #T-204 阻塞                 [处理]│  │
│ │  (×=会话内 dismiss, 不持久化; 预算阈值按项目细分, 默认回落全局)    │  │
│ │  (严重度≥desktopNotifyMinSeverity 时弹桌面通知, 点击跳转)         │  │
│ └────────────────────────────────────────────────────────────────┘  │
│ ┌─资源利用率排行(ECharts 横向柱)──────┐ ┌─过载预警列表────────────┐ │
│ │ 张三 ████████████████ 102% 🔴       │ │ ⚠ 张三  ProjectA 超出 4PD │ │
│ │ 李四 ████████████ 88%  🔵           │ │ ⚠ 王五  ProjectC 超出 2PD │ │
│ │ 王五 █████████████████ 105% 🔴      │ │ ⚠ 团队Beta 聚合超容 8PD  │ │
│ │        (行尾徽标: 张三 [跨3项目] 🔶单点风险) │ └───────────────────────────┘ │
│ └─────────────────────────────────────┘                            │
│ ┌─项目健康度(预算 vs 已分配, 阈值=项目级 budget_alert_threshold)──┐ │
│ │ ProjectA ████████░░ 80% 预算200PD 阈值110% 已用160PD 进行中    │ │
│ │ ProjectC ██░░░░░░░░ 18% 预算300PD 阈值110%(默认) 已用54PD     │ │
│ │ 风控    ██████████ 107% 预算120PD 阈值100%(收紧) 已用128PD 🔴 │ │
│ └──────────────────────────────────────────────────────────────────┘ │
│ ┌─团队 workload 堆叠面积(按周)──────────────────────────────────┐  │
│ │   ▂▃▅▆▇█ 团队Alpha   ▁▂▄▅ 团队Beta                            │  │
│ └────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

交互：
- 点击 KPI「过载 3 人」→ 路由跳 Kanban 并预设 `globalFilter.showOverloadOnly=true`。
- 点击资源柱条 → `useUiStore.selectResource(id)` → 右侧抽屉展开该资源详情（容量、技能、当前分配），并触发各视图联动高亮。
- 点击项目健康度行 → 跳 Gantt 项目视角并聚焦该项目；项目健康度行的阈值标签显示该项目的 `budget_alert_threshold`（项目级收紧则标注「收紧」，否则标注「默认」）。
- 「技能缺口」标记来自 `optimization.diff` 中的未覆盖技能项。
- **预警中心**：`useAlertStore` 聚合四类（过载来自 `useAllocationStore` 过载段、预算超支来自 `useProjectStore.budgetOverrun`、技能缺口来自 `optimization.diff`、依赖阻塞来自 `useTaskStore.blockedTasks`），**按 `severityOrder` 排序**（默认预算超支 > 过载 > 依赖阻塞 > 技能缺口，可在「设置 → 预警」拖拽重排并持久化）。每条预警带「处理」按钮：预算超支 → 跳对应项目 Gantt；过载 → 跳 Kanban 过载过滤；技能缺口 → 跳资源中心筛选该技能；依赖阻塞 → 跳 Gantt 聚焦阻塞链（仅高亮直接后继 1-2 层，见 §7.5.3）。预算预警判定：`budget_usage > projects.budget_alert_threshold`（**项目级字段，默认回落全局 `overload_threshold`**，列与 `check_budget_alert` 命令见 §6.2.3；首版不引入轮询，由 store getter 在数据变更时派生，刷新策略见 §7.4.3 全量刷新场景）。
- **预警 dismiss**（决策#7）：每条预警右侧带 `[×]` 忽略按钮，点击调用 `useAlertStore.dismiss(key)`，仅从本会话的角标计数与下拉列表中移除（写入会话内 `dismissed: Set`），**不持久化、不跨会话**；下次打开应用或触发数据变更重算后，若预警仍成立则重新出现。
- **桌面通知**（决策#5）：顶栏铃铛旁新增桌面通知开关（首次开启调用 `useDesktopNotify.requestNotifyPermission()` 走 Tauri `notification` 插件授权）。当 `notifyable`（严重度 ≥ `desktopNotifyMinSeverity` 且本会话未通知过）非空时弹系统通知，标题为预警类型与严重度，正文为摘要，点击跳转对应视图路由。**不做邮件、不做 Web Push**（§1.5 已排除）。同一预警一个会话内只通知一次。
- **资源利用率排行**行尾增加「跨项目数」徽标（`ProjectCountBadge`，来自 `useResourceStore.crossProjectCount`，复用 §8.5 R5 的 cell 公式聚合）。`> 2` 时橙色高亮并提示「单点风险」（识别高频跨项目资源），补齐 §1.4 关键功能#3「跨项目资源复用」在 Dashboard 的实时可视化入口。

---

#### 7.5.2 Kanban

职责：按任务状态/项目分列，卡片显示负责人、技能、workload、**依赖阻塞态**。支持拖拽改状态。

```
┌────────────────────────────────────────────────────────────────────────┐
│ 分组: ◉ 按状态  ○ 按项目    项目: [全部▾]   搜索 [____]   单位 PD       │
├───────────────┬───────────────────┬───────────────────┬───────────────┤
│ 待办 (12)     │ 进行中 (8)        │ 评审 (3)          │ 完成 (24)     │
├───────────────┼───────────────────┼───────────────────┼───────────────┤
│ ┌───────────┐ │ ┌───────────────┐ │ ┌───────────────┐ │ ┌───────────┐ │
│ │#T-101 API │ │ │#T-204 重构    │ │ │🔒 #T-301 部署 │ │ │#T-099 登录│ │
│ │Rust React │ │ │Rust ▮▮▮▯▯     │ │ │  (依赖 #T-204 │ │ │Vue TS     │ │
│ │需 5PD     │ │ │张三 60% •🔴   │ │ │   未完成)     │ │ │已完成     │ │
│ │[未指派]   │ │ │12-15 →12-20   │ │ │灰显·不可拖拽  │ │ │           │ │
│ └───────────┘ │ └───────────────┘ │ └───────────────┘ │ └───────────┘ │
│  ↑ 拖到此处  │                   │                   │               │
└───────────────┴───────────────────┴───────────────────┴───────────────┘
```

卡片元素：
- 标题 `#T-204 重构`、所需技能徽标（`SkillBadge`，带熟练度 `▮▮▮▯▯`）、工作量 `5PD`（按当前单位显示）。
- 负责人头像 + 投入比例（多负责人则多个头像堆叠），负责人当前 workload 条（`WorkloadBar`），过载则 `OverloadDot` 🔴。
- 时间窗 `12-15→12-20`，逾期标红。
- **依赖阻塞态**：当 task 满足「前置未完成」（来自 `useTaskStore.blockedTasks`，前置任务 status 非 `done`/`cancelled`）时，卡片置灰（opacity 0.5）并在标题前加锁标 🔒，提示「依赖 #T-xxx 未完成」；blocked 卡片不可拖入「进行中」及之后列（拖动时 toast「需先完成前置 #T-204」），对应 §3.8 硬约束#5 与 §6.7 任务依赖预检。

交互：
- 拖卡片跨列 → `taskStore.moveStatus`（乐观更新），失败回滚原列并 toast；若目标列为 blocked 卡片则前端预检拦截（硬约束）。
- 卡片双击 → 打开任务详情抽屉（编辑技能/工作量/时间窗/依赖）。
- 列头点击「按状态/按项目」切换分组维度；按项目分组时每列是一个项目泳道。
- 顶部 `useUiStore.selectedResourceId` 生效时，非该资源负责的卡片置灰（透明度 0.3）。

---

#### 7.5.3 Gantt

职责：资源视角 vs 项目视角，allocation 条带，拖拽改区间/投入比例，长期任务分段，依赖连线，**依赖违反/影响链校验（限深 1-2 层）**。本视图是人力配置的核心编辑面。

```
视角: ◉ 资源视角  ○ 项目视角        窗口 2026-06 ──────►   缩放 [日|周|月]
┌─────────────┬──06/02──06/09──06/16──06/23──06/30──07/07──07/14──────────┐
│ 张三 [跨3]🔶│   ▓▓▓▓ProjectA#T101(60%)           ▓▓ProjectB#T150(100%) │
│  88% 🔵     │            ╲                                        │
│             │             ╲ dep                                   │
│ 李四 [跨1]  │   ░░░░ProjectC#T204(50%) ░░░░░░░░░░░                 │
│  70%        │       ┊seg1┊──seg2(长期任务分段)──┊                   │
│ 王五 [跨2]  │   ▓ProjectA#T310(40%)    ▒▒[依赖违反·紫描边]T320      │
│ 105% 🔴     │   [请假 ██ 06/05-06/07]     ↳ 直接后继 T321/T322 闪烁(限深1层)│
├─────────────┴──────────────────────────────────────────────────────────┤
│ 容量基线:    │ ────每资源每日容量上限（虚线）──────────────────────── │
└────────────────────────────────────────────────────────────────────────┘
   注: 影响链高亮仅渲染直接后继 1-2 层(默认 depth=2), 超出深度不闪烁,
       仅在 tooltip 中计数(N 个间接后继), 避免与虚拟滚动性能冲突(§7.9)
```

两种视角：
- **资源视角**：行为资源，列为时间；每条 allocation 是一条横带，高度=投入比例（60% 条比 100% 条矮），颜色按所属项目，过载段叠加红色描边。
- **项目视角**：行为项目/任务，列为时间；allocation 条带按资源着色，便于看「项目内谁在干什么」。

**资源行头「跨项目数」徽标**（`ProjectCountBadge`）：每行资源名后显示该资源在当前窗口内参与的项目数（来自 `useResourceStore.crossProjectCount`，聚合逻辑复用 §8.5 R5 矩阵的 cell 公式）。`> 2` 时橙色高亮并 tooltip「单点风险：同时跨 N 项目」，补齐 §1.4 关键功能#3 在 Gantt 的实时可视化入口。

长期任务分段：长期任务（跨度 > 配置阈值，默认 > 4 周）渲染为多个 `segment` 子条带，每段可独立有里程碑节点（`◆`），段间用浅色连接表示同一任务。

依赖连线：任务间 `depends_on` 渲染为贝塞尔箭头（finish-to-start 默认），dhtmlx-gantt 原生支持。

拖拽交互契约：
- 拖动条带左右 → 改 `startDate/endDate`（吸附到工作日，跳过节假日）。
- 拖动条带上下边缘 → 改工作量（条带长度变化触发 `effortPD` 重算）。
- 拖动条带高度手柄 → 改 `ratio`（10% 步进，范围 10%–100%）。
- **依赖校验（onBarDrag 期间实时）**：拖动导致 `allocation.start < max(predecessor.end + lag_days)` 时（硬约束#5，§3.8 / §3.3.12），条带即置「依赖违反」态（紫色 `#722ed1` 描边 + 半透明），并 toast「依赖冲突：需先完成 #T-xxx」；同时触发**依赖影响链高亮（限深）**——仅渲染该任务的**直接后继 1-2 层**（`useTaskStore.dependents(id, depth=2)`，沿 `task_dependencies` 反向闭包求出，**截断在 `depth` 层**）条带闪烁（紫色 `#9254de` 半透明），超出深度的间接后继不闪烁、仅在条带 tooltip 中显示计数「共 N 个间接后继受影响（未渲染）」。`onBarDragEnd` 时若仍违反则不落库（硬阻止），仅保留预览态。
- **影响链限深理由**（决策#4）：Gantt 在 5000 条 allocation 规模下启用虚拟滚动（§7.9），若影响链不做深度截断，一次拖动可能触发数百乃至数千条带的闪烁重绘，与虚拟滚动的「仅渲染可视区」策略直接冲突，导致掉帧。限深 1-2 层把重绘集合控制在 $O(\text{直接后继数})$（通常 < 10），与虚拟滚动共存。`depth` 默认 2，可在「设置 → Gantt」下调为 1 以进一步降负载。
- 拖动后实时校验（容量）：若使某资源在该日超过容量，条带即时变红 + toast「张三 06/10 将超载 1.2PD」，允许保存但标记为冲突（软约束），或按配置阻止（硬约束）。

```
Gantt 条带 drag 事件契约:
  onBarDrag({ allocationId, field: 'start'|'end'|'effort'|'ratio', newValue, preview })
    → allocationStore.previewChange(id, patch)        // 实时算冲突，不改库
    → useDependency.checkViolation(preview)           // start < max(predecessor.end)?
        ↳ 违反 → 条带置依赖违反态(紫描边) + toast
    → useTaskStore.dependents(taskId, depth=2)        // 限深影响链
        ↳ 直接后继 1-2 层条带闪烁(紫半透明); 超出深度仅 tooltip 计数
    → useAllocationStore.conflicts(workload 预览)     // 容量过载?
        ↳ 过载 → 条带变红
  onBarDragEnd({ allocationId, patch })
    → if dependencyViolations: 拒绝落库, toast, 保持预览态(硬阻止)
    → else allocationStore.upsertAllocation(merged)   // 乐观写 + IPC
    → 失败回滚 + 高亮冲突
```

> 选型注：dhtmlx-gantt 提供 `onAfterTaskDrag` 等事件，资源负载视图用其 `resourceCalculator`；依赖连线与 auto-scheduling 用其原生 `links`，但环检测与「拖到前置之前」的即时校验在前端 `useDependency` 内做（应用层环检测见 §3.3.12，避免依赖后端往返延迟）。若 GPL 约束不可接受，备选自绘 SVG（行=资源，条=`<rect>`，依赖=`<path>`），工作量约为 3–5 人日。影响链限深截断逻辑在两种实现下一致，落在 `useTaskStore.dependents(id, depth)`。

---

#### 7.5.4 日历

职责：按日/周显示人力占用、请假、节假日。MVP 自绘。

```
视图: [日][周◉][月]        资源: [全部▾ / 团队▾]        窗口 ◄ 2026-W24 ►
┌────────┬────────┬────────┬────────┬────────┬────────┬────────┐
│        │ 周一 02 │ 周二 03 │ 周三 04 │ 周四 05 │ 周五 06 │ 周六 07 │
│        │        │        │        │        │        │ 休     │
├────────┼────────┼────────┼────────┼────────┼────────┼────────┤
│ 张三   │██A 60% │██A 60% │██A 60% │请▼假   │██B 100%│        │
│ 8h/日  │B 30%   │B 30%   │B 30%   │        │        │        │
│        │0.9 利用│0.9     │0.9     │  休    │1.0     │        │
├────────┼────────┼────────┼────────┼────────┼────────┼────────┤
│ 李四   │░░C 50% │░░C 50% │░░C 50% │░░C 50% │  空闲   │        │
│        │0.5     │0.5     │0.5     │0.5     │0.0     │        │
├────────┼────────┼────────┼────────┼────────┼────────┼────────┤
│ 假/节  │        │        │        │        │ 端午   │        │
└────────┴────────┴────────┴────────┴────────┴────────┴────────┘
   单元格底色 = 利用率热度（绿→蓝→琥珀→红）
   顶部「假/节」行显示全局节假日（红色斜纹）
```

单元格语义：
- 每个资源×日单元格是一个堆叠条，按项目色块（`A/B/C`）+ 投入比例堆叠，下方显示当日利用率。
- 单元格背景为利用率热度色；>100% 整格红描边 + ⚠。
- 请假、节假日以斜纹覆盖并加文字标签。
- 周六/周日/节假日列整体置灰。

交互：
- 点击单元格 → 弹出该资源当日 allocation 明细浮层，可快速新增/编辑 allocation。
- 切换「日」视图显示单资源 8 小时时间轴；「月」视图按资源聚合为热力图。

---

#### 7.5.5 资源中心

职责：人员/团队/skill/tag 管理，**资源离职/调岗处理**。三 Tab 切换。

```
[ 人员◉ | 团队 | 技能与标签 ]
┌──搜索[____]───────────────── 筛选: tag[Rust▾] skill[前端▾] 状态▾[含已归档]──┐
│ 姓名     团队      技能(熟练度)            标签      本月容量 利用率  跨项目 │
│ 张三     Alpha     Rust▮▮▮▮▮ React▮▮▮▯▯  [核心][后端] 20PD  102%🔴 [3]🔶 │
│ 李四     Beta      DevOps▮▮▮▯▯           [运维]      20PD   88%🔵 [1]   │
│ 王五     Alpha     Vue▮▮▮▮▮ TS▮▮▮▮▯     [核心][前端] 18PD  105%🔴 [2]   │
│ 赵六     —(已归档) TS▮▮▮▯▯              [前端]      —     待交接 ⬜ —    │
└──────────────────────────────────────────────────────────────────────┘
  [+ 新增人员]   行双击 → 编辑抽屉(技能/容量/投入比例默认值/请假设置)
              [离职/调岗 ▾] → 启动向导
```

- **人员 Tab**：CRUD 资源、绑定团队、设置技能熟练度（1–5）、设置默认投入比例与每日容量、维护请假、查看「跨项目数」列（`crossProjectCount`，`>2` 高亮）；状态列支持筛选「含已归档」（`status='archived'`）。
- **离职/调岗向导**（资源即将离职/已离职时其进行中 allocation 的批量处置，对应 HR 场景最高频破坏性事件）：
  1. 选择交接人（`to_resource_id`，下拉过滤技能匹配度）。
  2. 选择迁移时间窗 `[from, to]`（默认从交接日起到各 allocation `end_date`）。
  3. 选择模式 `mode`：`transfer`（原 allocation 的 `resource_id` 批量改为交接人）/ `cancel`（标记 cancelled 后由 AI 重排）/ `transfer_then_reoptimize`（先迁移再触发 AI 重排）。
  4. 勾选「触发 AI 重排」时（即 `mode=transfer_then_reoptimize`），向导提交后调用 `run_optimization`（窗口限定为交接区间）。
  5. 确认 → `resourceApi.transferAllocations({ from_resource_id, to_resource_id, window, mode })`（对应后端命令 `transfer_allocations`，见 §6.2.1；命令在单一事务内迁移或取消并回写 workload 快照，避免半应用状态）；成功后该资源 `status` 置 `archived`，其残留 allocation 在 Gantt/看板标「待交接」灰描边（见 §7.6 联动）。
  6. 预检：若交接人在迁移窗内将过载，向导内联告警并允许「仍迁移（标记冲突）」或「改选交接人」（复用 §6.7 过载预检口径）。
  7. **AI 重排审计与过载策略**（决策#3）：`mode=transfer_then_reoptimize` 触发的 AI 重排**作为一次正式优化运行落库到 `ai_optimization_runs`**（§3.3.16），并在新增列 `trigger` 上标记为 `'handover'`（取值集 `'manual' | 'scheduled' | 'handover'`，与 §8.x `scheduled_snapshots.trigger` 同名同语义；列定义属 §3.3.16 范围，本节仅引用并约定 UI 审计入口展示该来源标签）。在「资源中心 → 已归档资源 → 操作历史」与「AI 优化面板 → 历史运行」两处均能追溯该 run，区分人工主动优化与离职触发的自动重排。交接人在迁移窗过载时的默认策略**与全局资源超载策略对齐**（§9.4 开放问题#51 / 资源超载策略可配：硬阻止 vs 软警告）——即若全局策略为「硬阻止」，向导在交接人过载时阻止提交并要求改选交接人或缩窗；若为「软警告」则允许「仍迁移（标记冲突）」。向导内联告警文案随该全局策略动态变化，不引入独立的交接人过载策略。
- **团队 Tab**：CRUD 团队、拖拽资源入队（资源可跨团队）、查看团队聚合 workload。
- **技能与标签 Tab**：维护 skill 字典（含别名，供 LLM 语义匹配）、tag 字典、设置 tag↔skill 的隐含映射（如 tag「后端」隐含 skill「Rust|Go」）。

---

#### 7.5.6 AI 优化面板

职责：运行优化、查看建议方案、对比当前、接受/微调、解释，**What-if 资源约束模拟（支持多版本叠加对比，UC-08 落点）**，**多目标权重调节 UI**。

```
┌─约束设置────────────────────────────────────────────────────────────┐
│ 时间窗 [2026-06-01 → 2026-08-31]  硬约束: ☑容量上限 ☑时间窗 ☑跨项目不冲突 │
│ ┌─多目标权重(ObjectiveWeightsPanel · 决策#8)──────────────────────┐ │
│ │ 目标开关组(可多选, 至少留 1 项):                                  │ │
│ │   ☑ 技能匹配   ☑ 负载均衡   ☐ 偏好(资源-项目)   ☐ 最小 makespan  │ │
│ │ 权重滑杆(0.0–1.0, 开启项可见, 自动归一化到 Σ=1):                 │ │
│ │   技能匹配  ●───────────── 0.6   (拖动调整)                      │ │
│ │   负载均衡  ●─────── 0.3          (拖动调整)                      │ │
│ │   [重置默认]  归一化后: 技能0.67 / 均衡0.33   → 写入本次 run       │ │
│ │   (映射 ObjectiveWeights, 随 ai_optimization_runs.weights_json 持久)│ │
│ └────────────────────────────────────────────────────────────────┘ │
│ ┌─资源约束(What-if / UC-08)──────────────────────────────────────┐ │
│ │ 锁定资源(从当前 allocation 锁, AI 不可重排):                    │ │
│ │   ☑ 张三→#T-101(60%)  ☐ 李四→#T-204(50%)   [+从 Gantt 锁]      │ │
│ │ 排除资源(模拟离职/不可用, AI 不分给此人):                       │ │
│ │   ☑ 王五   [清除]   (等效 resources.status=archived 临时态)     │ │
│ │ 临时不可用窗口(模拟请假/出差):                                  │ │
│ │   王五  06/10→06/14(请假)  [删除]   [+ 新增窗口]               │ │
│ │   → 映射 store action: lockResource / excludeResource /         │ │
│ │     simulateUnavailable (写入本次 run 的 ConstraintFlags,       │ │
│ │     不改库; 复用 §5.7 ScoredAssignment.locked 与 §5.8.3 锁定/  │ │
│ │     排除语义)                                                    │ │
│ └────────────────────────────────────────────────────────────────┘ │
│ LLM: ◉ Ollama(llama3) ○ 云端    [仅经典优化器(无解释) ☐]            │
│   [ ▶ 运行优化 ]   [ + 存为 What-if 场景并入栈(支持叠加 2+ 对比) ]   │
├─结果对比────────────────────────────────────────────────────────────┤
│ 指标            当前方案            候选方案         Δ              │
│ 总利用率        82%                 91%              +9%            │
│ 过载数          3                   0                -3 ✓          │
│ 技能覆盖分      0.72                0.88             +0.16         │
│ 负载均衡(方差)  412                 180              -232 ✓        │
├─多 What-if 场景栈(WhatIfScenarioStack · 决策#6)─────────────────────┤
│ ☑ S1: 王五请假(06/10-14)  候选利用率 91%  过载 0   [Gantt叠加] [×]│
│ ☑ S2: 张三锁#T-101        候选利用率 88%  过载 1   [Gantt叠加] [×]│
│ ☐ S3: 李四排除            候选利用率 84%  过载 2   [Gantt叠加] [×]│
│ [叠加预览到 Gantt(勾选项 union, 不落库)] [清空栈]  (最多 4 场景同屏)│
├─差异明细(AllocationDiff · 当前 vs 候选/选中场景 union)──────────────┤
│ 资源   任务          当前          建议          原因(LLM 解释)      │
│ 张三   #T-204 60%    →王五 40%     减负           "王五具备 Rust▮▮▮▮▮ │
│                                                  且当前负载仅 70%"     │
│ 李四   #T-310 未指派 →李四 50%     补缺口         "李四 DevOps 匹配    │
│                                                  时间窗 06/16-06/30"   │
├─操作─────────────────────────────────────────────────────────────────┤
│  [全部接受]  [逐条勾选接受 ☑☑☐]  [导出方案 JSON]  [重新运行]          │
└──────────────────────────────────────────────────────────────────────┘
```

交互流程：
1. 设置约束与权重 → `optimizationStore.runOptimization(constraints)`（耗时，显示进度，经典优化器先出结果，LLM 解释异步补充）。**多目标权重**（决策#8）通过 `ObjectiveWeightsPanel` 组件调节：上方为「目标开关组」（checkbox 多选，决定哪些目标进入本次优化），下方为每个开启目标的「权重滑杆」（0.0–1.0，Naive UI `n-slider`），滑杆变更时前端做归一化（使 Σ=1）并实时预览归一化后的分布；提交 run 时随 `weights_json` 持久化到 `ai_optimization_runs`（§3.3.16），保证可复现。映射 §5.x 的 `ObjectiveWeights` 与 §5.8.3 的软目标开关；关闭某目标等价于将其权重置 0（求解器侧跳过该项打分）。提供「重置默认」恢复 `balanced` 预设。
2. **What-if 模拟（UC-08，多版本对比）**（决策#6）：在「资源约束」子区设置锁定/排除/临时不可用窗口（不改库，仅写入本次 run 的 `ConstraintFlags`：`locked` 资源作为固定占用、`excluded`/`simulateUnavailable` 资源在窗内容量置 0），点击「运行优化」得到候选方案后，可点击「存为 What-if 场景并入栈」把该候选方案作为一个命名场景（`S1/S2/...`，自动命名 + 可重命名）压入 `optimizationStore.whatIfStack`（`Plan[]`）。**支持同时叠加 2+ 场景对比**：在「多 What-if 场景栈」子区勾选多个场景，点击「叠加预览到 Gantt」→ 调用 `mergedWhatIfPreview()` 把选中场景与当前方案做条带 union，以不同色相/透明度的半透明条带叠加在 Gantt 之上（每个场景一种色相，重叠条带加深 + tooltip 列出来源场景），用于多方案决策对比；不立即生效，关闭面板、切换 run 或「清空栈」即丢弃。为避免视觉过载，同屏叠加场景上限默认 4（可配）。逐条接受仅对「候选方案（最近一次 run）」生效，场景栈仅用于对比预览。
3. 候选方案与当前方案 diff，渲染对比表。
4. 「逐条勾选接受」→ 仅应用勾选的 diff 到 `allocationStore`（批量，非乐观，loading 态）。
5. 「全部接受」→ `allocationStore.bulkApply(planId)`，随后全量刷新 Gantt/日历/Kanban。
6. LLM 超时降级：解释列显示「（经典优化器结果，解释生成超时）」，不阻断接受。

可复现性：`lastRun` 记录 `{ seed, constraints, weights, provider, modelVersion }`，相同输入复跑结果一致（经典优化器确定性；LLM 解释可设 `temperature=0`）。

---

#### 7.5.7 报表中心

职责：输出报表。支持预览 + 导出（Rust 侧重渲染 PDF/Excel，前端仅传参与展示）。

```
报表类型: [◉ 人力利用率周报 ○ 项目人力成本 ○ 技能缺口 ○ 团队负载月报]
参数: 窗口[2026-W22→W24]  粒度[周▾]  分组[团队▾]   [👁 预览]  [⬇ 导出PDF/Excel]
┌─预览────────────────────────────────────────────────────────────────┐
│ 人力利用率周报  2026-W22~W24                                        │
│ ┌─团队Alpha──────────────────────────────────────────────────────┐ │
│ │ 资源   W22   W23   W24   平均   趋势                              │ │
│ │ 张三   78%   92%  102%  91%   ▁▃█ (W24 过载)                     │ │
│ │ 王五   70%   88%  105%  88%   ▂▃█                                │ │
│ └────────────────────────────────────────────────────────────────┘ │
│ 结论建议(LLM): 张三连续两周过载，建议将 #T-204 部分转移至王五。       │
│   (走 §5.6 TrendExplainer.explain_trend(problem, history_metrics);   │
│    history_metrics 输入保留 4 周; 脱敏模式同样作用于趋势 prompt;     │
│    与方案解释 LlmExplainer.explain(solution) 是两条独立 prompt 模板, │
│    降级同样走规则模板; 详见 §5.6 / §8.7)                              │
└──────────────────────────────────────────────────────────────────────┘
```

- 报表数据由 Rust 侧聚合 SQL 产出，前端用 ECharts 渲染预览；导出由 Rust 侧用 `printpdf` / `rust_xlsxwriter` 生成文件并通过 Tauri `save`/`open` 命令落地。
- 「结论建议」**不复用**方案解释的 `LlmExplainer.explain(solution)`（其输入是单个 `Solution + SolutionMetrics`，而报表建议需跨周期趋势对比，输入形态不同）；改走 §5.6 新增的 `TrendExplainer`（trait 方法 `explain_trend(problem, history_metrics)`），是独立的 prompt 模板，降级同样走规则模板。报表建议与方案解释为两条独立解释通道（详见 §5.6 / §8.7）。
- **趋势输入窗口与脱敏**（决策#2）：`TrendExplainer.explain_trend` 的 `history_metrics` 输入**保留最近 4 周**的工作量/利用率/过载指标（来自 §8.9 `workforce_snapshots` 冷归档或 `workload_cache` 回溯读取，4 周为默认值，可在「设置 → 报表/AI」调整范围 2–8 周）。**脱敏模式（redact）同样作用于趋势 prompt**：当全局脱敏开关开启时，趋势 prompt 中的资源姓名、项目名、tag 等可识别实体一律替换为占位符（`R-001` / `P-001`），与方案解释 `LlmExplainer` 走同一套脱敏管线（§5.x），确保趋势建议不因走第二条 prompt 通道而泄露身份信息。

---

### 7.6 跨视图联动

通过 `useUiStore` 的全局选中态实现。`useSelection` 组合式函数订阅选中态并广播。

```
选中张三 (selectResource)
   │
   ├── Dashboard: 资源柱条高亮 + 其余置灰，KPI 聚焦到该资源；预警中心过滤涉及该资源的条目
   ├── Kanban:    仅高亮张三负责的卡片，其余卡片 opacity 0.3
   ├── Gantt:     资源视角自动滚动定位到张三行，其余行折叠/淡化
   ├── 日历:      过滤为仅显示张三行
   ├── 资源中心:  人员表格选中该行并展开详情
   └── 优化面板:  差异明细过滤为涉及张三的条目

选中 ProjectA (selectProject)
   ├── Dashboard: 项目健康度行高亮
   ├── Kanban:    切换「按项目」分组并聚焦 ProjectA 列
   ├── Gantt:     切项目视角并聚焦 ProjectA
   ├── 日历:      单元格仅显示 ProjectA 色块
   └── 报表:      报表参数预填 ProjectA

场景联动（非选中态，由领域事件触发）
   ├── What-if 模拟 (§7.5.6, 支持多场景叠加) → Gantt 候选方案叠加预览:
   │     优化面板把「多 What-if 场景栈」中勾选的 2+ 场景与当前方案做条带 union，
   │     Gantt 以不同色相半透明条带叠加多场景（每场景一色），当前方案保留，
   │     供多方案决策对比；不落库，关闭/切换 run/清空栈即丢弃。
   ├── 资源 archived (§7.5.5 离职/调岗) → Gantt/看板标灰待交接:
   │     资源 status='archived' 后，其进行中 allocation 在 Gantt 标「待交接」灰描边、
   │     Kanban 卡片置灰并提示「归属资源已离职，待交接」；点击跳资源中心离职向导。
   └── 依赖违反/影响链 (§7.5.3) → Gantt 闪烁直接后继条带(限深 1-2 层) + toast。
```

实现要点：
- 选中态写入 URL query（`?resource=...&project=...`），支持深链与浏览器后退。
- `globalFilter`（时间窗、tag、`showOverloadOnly`）同样全局，所有视图响应。
- 场景联动不依赖选中态，由对应 store 的状态/预览变更驱动各视图组件 watch；提供顶栏「清除选择」按钮一键复位（仅清选中态，不清场景预览；What-if 场景栈需在优化面板内显式「清空栈」）。

---

### 7.7 关键组件清单与契约示例

#### 跨视图通用组件

| 组件 | 职责 | 关键 props | 关键 events |
|---|---|---|---|
| `WorkloadBar` | 渲染利用率条 + 色阶 | `utilization: number`, `capacity?: number`, `unit: 'PD'\|'PM'`, `showLabel: boolean` | — |
| `OverloadDot` | 过载红点 + tooltip | `overloadPD: number`, `reason?: string` | — |
| `ProjectCountBadge` | 跨项目数徽标（>2 高亮单点风险） | `count: number`, `threshold?: number`（默认 2） | `click(resourceId)` |
| `DependencyLock` | 依赖阻塞锁标 + tooltip（前置未完成） | `blockedBy: number[]`（前置 task id） | — |
| `DependencyChainBadge` | 依赖影响链徽标（直接后继计数 + 未渲染间接后继计数） | `directCount: number`, `indirectHidden: number`, `depth?: number`（默认 2） | `click(taskId)` |
| `SkillBadge` | 技能徽标 + 熟练度 | `skill: Skill`, `level: 1..5` | `click(skillId)` |
| `EntityTag` | 资源/项目/任务的可点击标签（点击触发选中联动） | `type: 'resource'\|'project'\|'task'`, `id: string`, `label: string` | `select(id)` |
| `UnitSwitch` | PD/PM 切换 | `modelValue: 'PD'\|'PM'` | `update:modelValue` |
| `WindowPicker` | 时间窗选择 | `modelValue: TimeWindow` | `update:modelValue` |
| `AlertCenter` | 预警中心铃铛 + 下拉列表（按 `severityOrder` 排序）+ 桌面通知开关 | `alerts: Alert[]`（来自 `useAlertStore.allActive`）、`desktopNotifyEnabled: boolean` | `handle(alert)`, `dismiss(key)`（会话内）, `toggleDesktopNotify(enabled)` |
| `ObjectiveWeightsPanel` | 多目标权重调节（开关组 + 滑杆 + 归一化预览） | `modelValue: ObjectiveWeights`, `availableObjectives: ObjectiveKey[]` | `update:modelValue(weights)` |
| `WhatIfScenarioStack` | 多 What-if 场景叠加对比（勾选 → union 预览到 Gantt） | `scenarios: WhatIfScenario[]`, `selectedIds: string[]`, `maxOverlay?: number`（默认 4） | `toggle(id)`, `overlayToGantt(ids)`, `clear()` |

#### 契约示例（Vue 3 props/emits 类型）

```ts
// components/common/WorkloadBar.vue
<script setup lang="ts">
interface Props {
  utilization: number          // 0.x 或 >1（过载）
  capacity?: number            // PD，用于显示分母
  unit?: 'PD' | 'PM'           // 默认取 useUiStore.unit
  showLabel?: boolean
}
const props = withDefaults(defineProps<Props>(), {
  unit: 'PD', showLabel: true,
})
// 色阶映射：<0.7 绿 / 0.7-0.95 蓝 / 0.95-1 琥珀 / >1 红
const color = computed(() => {
  const u = props.utilization
  if (u > 1) return 'var(--c-overload)'
  if (u > 0.95) return 'var(--c-warn)'
  if (u > 0.7) return 'var(--c-ok)'
  return 'var(--c-free)'
})
</script>
```

```ts
// components/common/EntityTag.vue —— 触发跨视图联动
<script setup lang="ts">
import { useUiStore } from '@/stores/ui'
const props = defineProps<{
  type: 'resource' | 'project' | 'task'
  id: string
  label: string
}>()
const ui = useUiStore()
function onSelect() {
  if (props.type === 'resource') ui.selectResource(props.id)
  else if (props.type === 'project') ui.selectProject(props.id)
  else ui.selectTask(props.id)
}
</script>
<template>
  <button class="entity-tag" @click="onSelect">{{ label }}</button>
</template>
```

```ts
// components/optimization/ObjectiveWeightsPanel.vue —— 多目标权重调节（决策#8）
<script setup lang="ts">
interface ObjectiveWeights {
  skill_fit: number      // 0..1
  fairness: number       // 0..1 (负载均衡)
  preference: number     // 0..1 (资源-项目偏好)
  min_makespan: number   // 0..1
}
type ObjectiveKey = keyof ObjectiveWeights
const props = defineProps<{
  modelValue: ObjectiveWeights
  availableObjectives: ObjectiveKey[]   // 当前启用的目标开关
}>()
const emit = defineEmits<{
  (e: 'update:modelValue', v: ObjectiveWeights): void
}>()
// 滑杆变更时归一化到 Σ=1（仅对 availableObjectives 中开启项），并 emit
function normalize(w: ObjectiveWeights): ObjectiveWeights {
  const keys = props.availableObjectives
  const sum = keys.reduce((s, k) => s + w[k], 0) || 1
  const out = { ...w }
  for (const k of keys) out[k] = Math.round((w[k] / sum) * 100) / 100
  return out
}
// 关闭某目标开关 → 其权重置 0 并重新归一化其余项
</script>
```

#### Gantt 条带组件契约（自绘 SVG 版，备选 dhtmlx 时的适配层）

```ts
// components/gantt/AllocationBar.vue
interface Props {
  allocation: Allocation
  row: number                  // 资源/项目行号
  startPx: number              // 时间轴像素起点
  widthPx: number              // 区间像素宽
  ratio: number                // 投入比例，决定条高
  color: string                // 项目色
  overloaded?: boolean         // 是否落在过载段（红描边）
  dependencyViolation?: boolean // 是否依赖违反（紫描边，onBarDrag 期间）
  impactChainActive?: boolean  // 是否为影响链直接后继（紫半透明闪烁，限深 1-2 层内）
  handoverPending?: boolean    // 是否待交接（灰描边+斜纹，归属资源 archived）
  whatIfScenarioHue?: number   // What-if 叠加预览时的色相（多场景对比，未定义则非预览态）
  draggable?: boolean
}
const emit = defineEmits<{
  (e: 'drag', p: { field: 'start'|'end'|'ratio'; newValue: number }): void
  (e: 'dragEnd', p: Partial<Pick<Allocation,'startDate'|'endDate'|'ratio'>>): void
  (e: 'click'): void
}>()
```

---

### 7.8 前端与 Rust 后端的 IPC 命令清单（摘要）

为便于 7.4 落地，列出本节涉及的主要命令（命名即 Tauri command 名，Rust 侧 `#[tauri::command]` 对应实现）：

| 领域 | 命令 | 入参（简） | 返回 |
|---|---|---|---|
| resource | `resource_list` / `resource_upsert` / `resource_remove` / `transfer_allocations` | filter / Resource / id / `{from_resource_id, to_resource_id, window, mode}` | `Resource[]` / `Resource` / `()` / `TransferResult{ run_id?, trigger? }` |
| team | `team_list` / `team_upsert` / `team_add_member` | — / Team / {teamId, resourceId} | `Team[]` / `Team` / `()` |
| project | `project_list` / `project_upsert` / `check_budget_alert` | — / Project(含 `budget_alert_threshold?`) / id | `Project[]` / `Project` / `BudgetAlert{usage, threshold, thresholdSource:'project'\|'global', overrun}` |
| task | `task_list` / `task_upsert` / `task_move_status` | filter / Task / {id, status} | `Task[]` / `Task` / `()` |
| allocation | `allocation_list` / `allocation_upsert` / `allocation_remove` / `allocation_apply_plan` | window / AllocationInput / id / planId | `Allocation[]` / `Allocation` / `()` / `()` |
| optimization | `optimization_run` / `optimization_diff` | Constraints（含 locked/excluded/unavailable_windows + ObjectiveWeights 权重组） | `Plan`（含 `run_id` 落 `ai_optimization_runs`，`trigger` 默认 `'manual'`） / `AllocationDiff[]` |
| report | `report_preview` / `report_export` | {type, window, groupBy, history_weeks?:4} / {…, format: 'pdf'\|'xlsx'} | `ReportData`（含 `trend_suggestion` 字段，走 4 周 history_metrics + 脱敏） / `{ filePath }` |
| config | `config_get` / `config_set` | key / {key, value}（含 `alert.severity_order`、`alert.desktop_notify_min_severity`、`gantt.impact_chain_depth`） | `ConfigValue` / `()` |

> 命令返回一律为 `Result<T, AppError>`，前端 `call<T>` 解包 `Ok`。`AppError` 的 `code` 枚举须与 Rust 侧 `thiserror` 派生的错误体一一对应（见后端错误处理节），新增 `DEPENDENCY_CYCLE` / `DEPENDENCY_VIOLATION` 码对应 §7.5.3 依赖校验。
>
> 跨节命令补注（不在本节定义，但前端会调用）：
> - `transfer_allocations` 命令组在 §6.2.1 resource 命令组登记；当 `mode=transfer_then_reoptimize` 时其返回的 `run_id` 指向 `ai_optimization_runs` 一条 `trigger='handover'` 记录（决策#3），`trigger` 列定义属 §3.3.16 范围。
> - `projects.budget_alert_threshold` 列（项目级，可空，空则回落全局 `overload_threshold`）与 `check_budget_alert` 命令在 §6.2.3 project 命令组登记（决策#1）。
> - 报表 LLM 建议走 §5.6 的 `TrendExplainer`（非方案解释通道），`history_metrics` 默认保留 4 周、脱敏同样作用于趋势 prompt（决策#2）。
> - `alert.severity_order` / `alert.desktop_notify_min_severity` / `gantt.impact_chain_depth` 为新增配置项，存 `app_config`，定义属 §3 / §9 范围。

---

### 7.9 前端性能与体积约束

| 项 | 目标 | 手段 |
|---|---|---|
| 首屏 JS（gzip） | < 350 KB | 路由懒加载 + Naive UI tree-shaking + ECharts 按需引入 |
| Gantt 大数据 | 5000 条 allocation 仍可拖拽 | 虚拟滚动（dhtmlx 原生 / 自绘仅渲染可视区行） |
| 依赖影响链渲染 | 一次拖动闪烁条带重绘集合 < ~10 条 | `dependents(id, depth=2)` **限深 1-2 层**（决策#4），仅直接后继闪烁，间接后继只 tooltip 计数，与虚拟滚动共存；`depth` 可在「设置 → Gantt」下调为 1 |
| What-if 叠加 | 同屏 ≤ 4 场景 union 仍可交互 | `mergedWhatIfPreview()` 输出条带 union，多色相半透明；超出 4 个时禁用勾选并提示「过多场景影响可读性」 |
| IPC 往返 | 单命令 < 30 ms（本地 SQLite） | 乐观更新掩盖延迟；列表查询走分页/窗口裁剪 |
| 重渲染 | 选中态变更不触发全树重渲染 | Pinia 细粒度 store + `computed` 派生；组件用 `shallowRef` 持有大数组 |
| 预警检测延迟 | ≤ workload 增量重算阈值（200ms，见 §9.2.1） | `useAlertStore` 全派生（getter 聚合），随领域 store 变更去抖刷新，不轮询 |
| 桌面通知去抖 | 同一预警一会话内只通知 1 次 | `notifyable` 经 `alertKey` 去重（会话内 Set），避免抖动 |

---

### 7.10 MVP 分阶段建议（仅前端范围）

| 阶段 | 视图 | 说明 |
|---|---|---|
| P0 | 资源中心 + Kanban + 基础 Dashboard | 打通 CRUD → IPC → 展示闭环，验证 typed invoke 与乐观更新；Kanban 含依赖阻塞态 |
| P1 | Gantt（资源视角，只读 + 拖拽改区间 + 依赖校验/影响链限深 1-2 层） | 引入 dhtmlx-gantt，验证 allocation 编辑、冲突检测、依赖违反预警与限深影响链渲染 |
| P2 | 日历 + AI 优化面板（含 What-if 资源约束模拟 + 多目标权重调节 UI + 单场景 Gantt 叠加） | 日历自绘；优化面板接入 rig（Ollama 默认），资源约束子区映射 store action，`ObjectiveWeightsPanel` 滑杆/开关组随 run 持久化权重 |
| P3 | 报表中心（走 TrendExplainer，4 周 history + 脱敏） + 完整跨视图联动 + 项目视角 Gantt + 长期任务分段 + 资源离职/调岗向导（`transfer_then_reoptimize` 记 `trigger=handover`） + Dashboard 预警中心（严重度排序可配 + 桌面通知 + 会话内 dismiss） + What-if 多场景叠加对比 | 补齐导出、What-if→Gantt 多场景叠加、archived 待交接联动、预算预警项目级阈值闭环与桌面通知能力 |

---

### 7.11 假设（本节）

1. §7.1 桌面通知经 Tauri v2 `notification` 插件实现；macOS/Windows/Linux 三端原生通知能力可用，授权交互由插件封装。**不做邮件、不做 Web Push**（§1.5 已排除）。
2. §7.3 预算预警阈值 `projects.budget_alert_threshold` 为项目级可空字段，空值回落全局 `overload_threshold`；该列与 `check_budget_alert` 命令的 schema 定义属 §3 / §6.2.3 范围，本节仅消费。
3. §7.3 `useAlertStore.dismissed` 为会话内内存状态，不落 SQLite、不写 `localStorage`（可选易失 session storage 仅做页面刷新恢复，关闭应用即丢）。
4. §7.3 `severityOrder` 与 `desktopNotifyMinSeverity` 持久化到 `app_config`（定义属 §3 / §9），默认 `severityOrder=['budget','overload','dependency','skill_gap']`、`desktopNotifyMinSeverity='high'`。
5. §7.5.3 影响链限深 `depth` 默认 2（可配 1），间接后继只 tooltip 计数；`gantt.impact_chain_depth` 存 `app_config`。
6. §7.5.5 `ai_optimization_runs.trigger` 取值集为 `'manual' | 'scheduled' | 'handover'`；该列定义属 §3.3.16 范围，本节仅约定离职重排写入 `'handover'`。
7. §7.5.5 交接人过载策略与全局资源超载策略（§9.4 #51）对齐，不引入独立的交接人过载策略。
8. §7.5.6 What-if 同屏叠加场景上限默认 4（可配），超出禁用勾选。
9. §7.5.7 `TrendExplainer.explain_trend` 的 `history_metrics` 默认保留最近 4 周（可配 2–8），脱敏模式同样作用于趋势 prompt（与方案解释共用同一脱敏管线）。
10. §7 前端默认 UI 库为 Naive UI；Gantt 默认 dhtmlx-gantt；日历默认自绘 SVG（与全局假设 #9 对齐，本节不再展开备选争议）。

### 7.12 开放问题（本节）

1. **[impl 期决策]** `alert.severity_order` 配置 UI 的具体形态（拖拽重排 vs 下拉排序）与 `desktopNotifyMinSeverity` 的档位命名（`high/critical` vs 数字 1-3），待 §9「设置」页原型落地时定稿。
2. **[impl 期决策]** What-if 多场景叠加在 Gantt 上的色相分配策略（固定调色板 vs HSL 等距），以及重叠条带加深阈值的视觉调参，需在 P3 阶段做可读性实测。
3. **[impl 期决策]** 桌面通知在应用处于前台 vs 后台时的策略差异（前台是否仍弹系统通知，还是仅更新铃铛角标），需产品确认默认行为。
4. **[impl 期决策]** `ai_optimization_runs.trigger='handover'` 记录在 UI 的展示位置（资源中心操作历史 vs AI 面板历史运行 vs 两处都展示），待 P3 两视图落地后定稿。
5. **[跨节]** `ai_optimization_runs` 新增 `trigger` 列、`projects.budget_alert_threshold` 列、`app_config` 新增三项预警/Gantt 配置键的 schema 迁移脚本，归属 §3.3 / §3.x 迁移章范围，需在修订第 3 章时落定并补交叉引用。

## 8. 报表与导出 / Reporting & Export

本节定义「开发人力资源看板」的报表体系：覆盖报表类型清单、数据口径、导出格式、参数化、生成方式分工、模板与可配置性，以及人力快照存档机制。所有报表共享第 4 节的 workload / capacity 计算模型，单位遵循全局可切换的 PD / PM 约定。

> **本轮修订口径（决策清单对齐）**：
> - 成本/预算字段统一引用 `resources.daily_rate_pd`、`projects.budget_pd`；**不存在 `budget_pm` 列**（与 §3 决议一致）。
> - 人天单价支持按 resource × project [, period] 维度浮动：通过 `resource_project_rates` 表解析 `effective_daily_rate(r,p,d)`，回落 `resources.daily_rate_pd` → `N/A`。
> - `daily_rate_pd` 与 `resource_project_rates` 均在 **MVP 即补入 schema**（§3.3.4 / §3.3.4a），使 **R7 成本估算可前移到 MVP**（但报表**导出**本身仍整体延后，见 §8.11 / §9.1 Phase 5+）。
> - R4「AI 决策记录」报表**只保存结构化的约束+打分摘要**（取自 `ai_optimization_runs` 的 `constraints_json` / `weights_json` / `score_*` / `config_json` / `output_plan_json`），**不存 LLM prompt/response 原文**（隐私与体积）。
> - HTML→PDF 通路（`headless_chrome`）为**可选特性**，由用户在设置中显式开启，不默认引入其重依赖。
> - 报表导出**整体延后到最后阶段（Phase 5+）**，不进核心 MVP；首版导出格式优先级 **CSV > Excel > PDF**；本期 **PDF 不套公司模板**。
> - TrendExplainer 趋势报表的历史输入保留 **4 周**，**脱敏模式同样作用于趋势 prompt**（与 §7 / 假设 #32 对齐，本章仅引用）。

### 8.0 数据模型依赖（与第 3 节 schema 的对齐说明）

> 本节报表依赖若干 schema 字段。下表逐项声明其来源与处置。**所有成本/费率字段已在 MVP schema 中落地**（`daily_rate_pd` 见 §3.3.4 第 627 行；`resource_project_rates` 见 §3.3.4a）；`budget_pd` 见 §3.3.10。

| 报表字段 / 指标 | 来源表.字段 | schema 状态 | 处置 |
|---|---|---|---|
| 容量 / 已分配 / 利用率 | `resources.daily_capacity_pd`、`allocations.allocated_pd`（派生冗余） | 已存在（§3.3.4 / §3.3.15） | 直接引用，经 workload_engine 聚合 |
| 项目预算 | `projects.budget_pd` | 已存在（§3.3.10，**唯一预算字段**，内部统一 PD） | 直接引用；PM 视图由 `pd_to_pm` 派生展示，**不引用 `budget_pm`**（schema 不存在 PM 列） |
| 资源标准人天单价 | `resources.daily_rate_pd` | **已存在（§3.3.4，MVP 即在）** | 作为 `effective_daily_rate` 的回落基准；NULL → 该行 cost 为 `N/A` |
| 项目/周期浮动单价 | `resource_project_rates`（resource × project [, period] 维度费率） | **已存在（§3.3.4a，MVP 即在）** | 优先解析；命中则覆盖 `resources.daily_rate_pd`（见 §8.3 公式） |
| R4 决策摘要 | `ai_optimization_runs.constraints_json` / `weights_json` / `score_*` / `config_json` / `output_plan_json` | 已存在（§3.3.16） | **仅取结构化字段**；不引用 LLM prompt/response 原文（见 §8.4a） |

#### 8.0.1 resource_project_rates 表（§3.3.4a 落地，本章引用）

```sql
-- resource × project [, period] 维度的浮动人天单价（PD 口径）
CREATE TABLE resource_project_rates (
    resource_id     INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    project_id      INTEGER NOT NULL REFERENCES projects(id)  ON DELETE CASCADE,
    -- 可选周期覆盖：NULL 表示「该项目全程」；非空则仅在该 [valid_from, valid_to] 生效
    valid_from      TEXT,   -- 'YYYY-MM-DD'，NULL = -∞
    valid_to        TEXT,   -- 'YYYY-MM-DD'，NULL = +∞
    daily_rate_pd   REAL    NOT NULL CHECK (daily_rate_pd IS NULL OR daily_rate_pd >= 0),
    -- 货币单位由 settings.locale_currency 决定（默认 CNY ¥）
    note            TEXT,
    created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    -- 同一 (resource, project) 在任意时间点最多一条生效记录
    PRIMARY KEY (resource_id, project_id, valid_from)
);
CREATE INDEX idx_rpr_resource ON resource_project_rates(resource_id, valid_from, valid_to);
CREATE INDEX idx_rpr_project  ON resource_project_rates(project_id);
```

> **与 §3.3.4 一致性**：`resources.daily_rate_pd` 是「资源在所有项目共用的基准单价」（可为 NULL）；`resource_project_rates` 是「针对特定项目/周期的覆盖单价」。二者通过 §8.3 的 `effective_daily_rate(r,p,d)` 解析合并，**报表/成本计算统一走解析函数**，不直接硬编码引用任一列。这同时解决了原开放问题 #38（按项目浮动单价）与 #39（按周期浮动单价）。

---

### 8.1 设计原则

1. **单一数据源**：报表不另起计算逻辑，统一调用第 4 节 `workload_engine` 的聚合函数（`capacity_in_window`、`allocated_in_window`、`utilization`），保证「看板所见即报表所出」。
2. **单位一致性**：报表内部一律以 PD 为最小存储/计算单位，渲染时按用户偏好（PD/PM）做展示层换算；换算因子 `pm_workdays`（默认 20，即 `1 PM = 20 PD`）来自本地配置表 `settings`。PM 仅作派生展示，不反向写入任何字段。
3. **可复现**：每次生成报表时把「报表参数 + 数据快照版本号 + 引擎版本」写入导出文件的元数据/页脚，便于审计与回溯。
4. **离线优先**：所有导出在本地 Rust 侧或前端 WebView 内完成，不依赖任何网络服务。
5. **字段先于报表**：任何报表引用的 schema 字段必须先在 §3 落地（见 §8.0）。本轮决策后，`daily_rate_pd` / `resource_project_rates` / `budget_pd` 均已在 MVP schema 中就位，**R7 成本估算不再被字段缺失阻塞**（报表**导出**仍按 §8.11 延后）。
6. **结构化优先（隐私）**：涉及 AI 的报表（R4）只持久化结构化的约束/打分/方案摘要，**不存 LLM prompt/response 原文**，避免 PII 与体积膨胀。

### 8.2 报表类型清单

| ID | 报表名称 | 核心问题 | 主维度 | 默认频率 | 推荐导出格式 |
|----|----------|----------|--------|----------|--------------|
| R1 | 资源利用率报表（Resource Utilization） | 每个人在时间窗内被用了多少、利用率多少 | Resource × Time | 周/月 | Excel / PDF |
| R2 | 团队 workload 报表（Team Workload） | 某团队整体负载与瓶颈 | Team × Time | 周/月 | Excel / PDF |
| R3 | 项目人力消耗 vs 预算（Project Budget Burn） | 项目花了多少人力、是否超预算 | Project × Time | 月 | Excel / PDF |
| R4 | AI 优化决策记录（AI Allocation Log） | AI 为什么这样分配、约束与打分依据 | Allocation 决策 | 每次求解后 | JSON / PDF（**仅结构化摘要**） |
| R5 | 跨项目资源复用矩阵（Cross-project Reuse Matrix） | 哪些人跨了哪些项目、占比多少 | Resource × Project | 月/季度 | Excel / PDF |
| R6 | 过载/欠载名单（Overload / Underload List） | 谁超载了、谁闲置了 | Resource | 周 | Excel / PDF |
| R7 | 周期人力成本估算（Period Cost Estimate） | 这段时间的人力成本是多少 | Resource/Team/Project × Period | 月 | Excel / PDF |
| R8 | 人力快照存档（Workforce Snapshot） | 某时刻全量人力状态归档 | 全系统 | 手动/定时 | JSON / Excel |

> R1–R7 为「查询型报表」（基于实时数据计算），R8 为「快照型报表」（冻结某一时刻数据）。两者底层均经 workload_engine，区别见 8.7。
>
> **R7 成本估算前移说明**：因 `daily_rate_pd` / `resource_project_rates` 已在 MVP schema 中，R7 的**成本数值计算**可在 MVP 期作为 Dashboard 内联视图/前端实时渲染交付（无需导出文件）；R7 的**文件导出**仍随报表导出整体落在 Phase 5+（见 §8.11）。

### 8.3 数据口径

所有报表的数值字段统一由以下口径推导（与第 4 节对齐）：

- **容量 `capacity(res, [t0,t1])`**：`Σ 有效工作日 × 日容量`，扣除节假日 `holidays`、请假 `time_off`，受资源在该任务上的投入比例 `allocation_ratio` 影响。
- **已分配 `allocated(res, [t0,t1])`**：`Σ allocation.workload`（PD），按时间窗裁剪；跨项目任务不重复计数（一份 allocation 只属于一个 task/project）。
- **利用率 `utilization = allocated / capacity`**，结果为百分比。`capacity = 0` 时利用率记为 `N/A`（如整段请假）。
- **过载阈值 `overload_threshold`**（默认 100%，可配）、**欠载阈值 `underload_threshold`**（默认 60%，可配），来自 `settings`。
- **单位**：底层一律 PD；展示时若 `unit = PM` 则 `value_pm = value_pd / pm_workdays`（`pm_workdays` 默认 20），保留 2 位小数。
- **预算（R3）**：取自 `projects.budget_pd`（PD）；PM 视图下预算与已消耗列同口径经 `pd_to_pm` 换算展示。**不引用 `projects.budget_pm`（schema 不存在该列）。**
- **成本估算（R1/R2/R3/R7，`include_cost=true` 时）**：通过 `effective_daily_rate(r,p,d)` 解析单价，再乘以 allocated_pd：

```
// 1) 解析资源 r 在项目 p、日期 d 的生效人天单价（PD 口径）
effective_daily_rate(r, p, d):
    rate = SELECT daily_rate_pd FROM resource_project_rates
           WHERE resource_id = r AND project_id = p
             AND (valid_from IS NULL OR valid_from <= d)
             AND (valid_to   IS NULL OR valid_to   >= d)
           ORDER BY valid_from DESC LIMIT 1;     -- 命中：取周期覆盖单价
    if rate is None:
        rate = SELECT daily_rate_pd FROM resources WHERE id = r;   -- 回落资源基准单价
    return rate;                                  -- 仍为 NULL → 该行 cost 输出 N/A

// 2) 成本聚合（货币单位由 settings.locale_currency 决定，默认 ¥ CNY）
cost_pd  = allocated_pd × effective_daily_rate(r, p, d)      // 按 allocation 的 project 归属逐条解析
```

  > 解析优先级：**`resource_project_rates`（项目/周期覆盖）> `resources.daily_rate_pd`（资源基准）> `N/A`**。同一资源在不同项目可有不致单价；同一项目在不同周期也可浮动（通过 `valid_from/valid_to`）。该机制取代了「同一资源在所有项目共用一个 `daily_rate_pd`」的旧表述，原开放问题 #38/#39 由本设计解决。

```rust
// 报表共享的口径结构（与 workload_engine 对齐）
pub struct ReportMetric {
    pub resource_id: i64,
    pub capacity_pd: f64,
    pub allocated_pd: f64,
    pub utilization: Option<f64>, // None 表示 capacity=0
    pub cost: Option<f64>,        // None 表示 effective_daily_rate 解析后仍为 NULL（未维护任何单价）
}

// 单价解析（在 exporter / workload_engine 共享）
pub fn effective_daily_rate(
    db: &AppState,
    resource_id: i64,
    project_id: i64,
    date: NaiveDate,
) -> Result<Option<f64>, ReportError> {
    // 1) 优先查 resource_project_rates 命中记录
    if let Some(rate) = db.query_rpr_rate(resource_id, project_id, date).await? {
        return Ok(Some(rate));
    }
    // 2) 回落 resources.daily_rate_pd（可能为 NULL）
    Ok(db.query_resource_base_rate(resource_id).await?)
}
```

### 8.4 报表参数

所有查询型报表（R1–R7）共享统一参数对象，前端通过 Tauri IPC `invoke("generate_report", { kind, params })` 触发：

```rust
#[derive(serde::Deserialize, Clone)]
pub struct ReportParams {
    /// 时间窗（必填）；含端点，左闭右闭
    pub window: DateWindow,        // { start: NaiveDate, end: NaiveDate }
    /// 项目过滤（None=全部项目）
    pub project_ids: Option<Vec<i64>>,
    /// 团队过滤（None=全部团队）
    pub team_ids: Option<Vec<i64>>,
    /// 单个资源过滤（None=全部资源）
    pub resource_ids: Option<Vec<i64>>,
    /// Tag 过滤（资源 tag 或任务 tag，AND/OR 语义见 8.6）
    pub tags: Option<TagFilter>,
    /// 展示单位："PD" | "PM"
    pub unit: Unit,
    /// 时间分桶粒度（用于 R1/R2/R3 的按周期列）："day"|"week"|"month"
    pub bucket: Bucket,
    /// 是否含成本列（R1/R2/R3/R7）；含成本列时经 effective_daily_rate 解析单价（§8.0/§8.3）
    pub include_cost: bool,
    /// 是否在成本解析时启用 resource_project_rates 浮动单价（默认 true；false 则只走 resources.daily_rate_pd）
    pub use_project_rates: bool,
}

pub enum Unit { PD, PM }
pub enum Bucket { Day, Week, Month }
pub struct TagFilter { pub tags: Vec<String>, pub mode: TagMode } // All=AND, Any=OR
```

参数校验规则：
- `window.end >= window.start`，否则返回 `ReportError::InvalidWindow`。
- `project_ids` / `team_ids` / `resource_ids` 任一非空时取交集（如同时指定项目与团队，则只统计「既在该项目又在该团队」的资源）。
- `unit` 仅影响展示，不影响底层计算。
- `include_cost = true` 且存在任意被统计资源在 `effective_daily_rate` 解析后仍为 NULL 时，成本列对该资源输出 `N/A`，并在报表页脚注明「N/A 行表示未维护人力单价」。
- `use_project_rates = false` 时跳过 `resource_project_rates` 查询，仅用资源基准单价（用于「忽略项目浮动、看基准成本」的对比场景）。

### 8.4a R4「AI 决策记录」报表的内容边界（决策 4）

R4 是「AI 优化决策记录」报表，**只保存/导出结构化的约束+打分摘要**，**不存 LLM prompt/response 原文**：

| R4 字段 | 来源（`ai_optimization_runs` 列） | 说明 |
|---|---|---|
| run 元信息 | `id` / `seed` / `objective` / `scope` / `scope_project_ids` / `scope_from` / `scope_to` / `provider` / `chat_model` / `embed_model` / `solver_backend` / `solver_status` | 求解器与 provider/model 快照，保证可复现 |
| 约束快照 | `constraints_json` | 硬约束（容量上限、不冲突、时间窗、最低熟练度）|
| 权重快照 | `weights_json` | 软约束权重 |
| 合并配置 | `config_json` | `ConstraintFlags` + `ObjectiveWeights` + `solver_config` 合并快照 |
| 评分摘要 | `score_overall` / `score_skill_fit` / `score_utilization` / `score_fairness` | 综合与分项评分 0..100 |
| 方案摘要 | `output_plan_json` | 生成的 allocation 列表摘要（亦可通过 `allocations.run_id` 关联明细）|
| 自然语言解释 | `explanation_md` | LLM 生成的方案解释 Markdown（**这是 LLM 输出，但不属于 prompt/response 原文**；为生成结果的精炼文本，体积可控）|
| 时长/状态 | `started_at` / `finished_at` / `duration_ms` / `status` / `applied` / `error_msg` | 运行状态机 |

> **不持久化的内容**：发往 LLM 的原始 `prompt`、LLM 返回的原始 `response`（含任何 thinking/工具调用 trace）。理由：(1) 体积（单次 prompt+response 可达数十 KB～MB）；(2) 隐私（即便脱敏，原始 prompt 仍可能含可逆 PII）；(3) 可复现性已由 `seed` + `input_snapshot_json` + `constraints_json` + `weights_json` + provider/model 保证，无需原文。如需调试 LLM 行为，应在 dev 模式下经独立日志通道（不入 `ai_optimization_runs` 表）。

### 8.5 报表表头结构示例

#### 示例 1：R1 资源利用率报表（按月分桶，单位 PD）

| 资源 | 团队 | 容量(PD) | 已分配(PD) | 利用率 | 状态 | 成本(¥) |
|------|------|----------|------------|--------|------|---------|
| 张三 | 平台组 | 20.0 | 22.5 | 112.5% | 🔴 过载 | 9,450 |
| 李四 | 平台组 | 20.0 | 13.0 | 65.0% | 🟢 正常 | 5,200 |
| 王五 | 应用组 | 16.0 | 4.0 | 25.0% | 🟡 欠载 | 1,700 |
| 赵六 | 应用组 | 0.0 | 0.0 | N/A | ⚪ 休假 | N/A |

> 「状态」列由阈值判定：`util > overload_threshold` → 过载；`util < underload_threshold` 且 `capacity>0` → 欠载；`capacity=0` → 休假/不可用；其余正常。成本列示例按 `cost = allocated_pd × effective_daily_rate(r,p,d)`：张三 14 PD 在项目 A（`resource_project_rates` 覆盖单价 ¥450）+ 8.5 PD 在项目 B（基准 `daily_rate_pd=400`）→ `14×450 + 8.5×400 = 6300+3400 = 9700`（上表为示意取整）；未维护任何单价的资源该列为 `N/A`。

#### 示例 2：R5 跨项目资源复用矩阵（行=资源，列=项目，值=该资源在该项目的已分配 PD）

| 资源 \ 项目 | 项目A (支付) | 项目B (风控) | 项目C (中台) | 合计(PD) | 项目数 |
|-------------|--------------|--------------|--------------|----------|--------|
| 张三 | 14.0 | 8.5 | 0.0 | 22.5 | 2 |
| 李四 | 0.0 | 13.0 | 0.0 | 13.0 | 1 |
| 王五 | 2.0 | 0.0 | 2.0 | 4.0 | 2 |
| **列合计** | **16.0** | **21.5** | **2.0** | **39.5** | — |

矩阵单元格公式：`cell[res,proj] = Σ allocation.workload where resource=res and task.project=proj and allocation overlaps window`。「项目数」列用于识别高频跨项目资源（潜在瓶颈/单点风险）。

#### 示例 3：R3 项目人力消耗 vs 预算（单位 PM，pm_workdays=20）

> 数值换算示例：项目「支付」预算存储为 `projects.budget_pd = 240 PD`，PM 视图展示为 `240 / 20 = 12.0 PM`；已消耗 170 PD → `8.5 PM`。

| 项目 | 预算(PM) | 已消耗(PM) | 剩余(PM) | 消耗率 | 状态 |
|------|----------|------------|----------|--------|------|
| 支付 | 12.0 | 8.5 | 3.5 | 70.8% | 🟢 正常 |
| 风控 | 6.0 | 6.4 | -0.4 | 106.7% | 🔴 超预算 |
| 中台 | 15.0 | 2.1 | 12.9 | 14.0% | 🟡 低于预期 |

> 「预算」列存储来源为 `projects.budget_pd`，PM 视图下与「已消耗」「剩余」同口径经 `pd_to_pm = pd / pm_workdays`（默认 ÷20）换算展示。**报表不引用 schema 中不存在的 `projects.budget_pm`。**「消耗率 = 已消耗 / 预算」（PD 口径与 PM 口径下比值一致，因分子分母同因子换算）；状态阈值复用 `overload_threshold`（超预算）与 `underload_threshold`（远低于预算，提示排期风险）。

### 8.6 导出格式

| 格式 | 适用报表 | 实现方式 | 说明 |
|------|----------|----------|------|
| **CSV** | 全部查询型（R1–R7） | Rust 侧 `csv` crate | 单表扁平化，便于二次处理/导入 BI；不支持多 sheet。**首版优先级最高（见 §8.11）** |
| **Excel (.xlsx)** | R1, R2, R3, R5, R6, R7, R8 | Rust 侧 `rust_xlsxwriter`（0.83+） | 多 sheet、公式、条件格式、冻结表头；**支持数据透视表（pivot table）与原生图表（chart）**（见 §8.6a），推荐为结构化表格默认格式 |
| **PDF** | R1, R2, R3, R4, R6, R7, R8 | Rust 侧 `printpdf`（0.9+）生成；或 HTML→PDF（可选特性，见 §8.6b） | 适合归档/分享；含页眉页脚、报表参数、生成时间。**本期 PDF 不套公司模板**（见 §8.11 决策 9） |
| **JSON** | R4, R8，以及所有报表的「数据层」 | Rust 侧 `serde_json` | 机器可读；R4 决策记录（仅结构化摘要，见 §8.4a）与 R8 快照的**唯一完整格式** |

#### 8.6a Excel 数据透视表与图表（决策 7）

`rust_xlsxwriter` 支持 pivot table 与原生 chart，因此 Excel 导出**不仅静态表格**：

- **R5 跨项目复用矩阵** → 导出为带 pivot table 的 sheet，用户可在 Excel 内自行拖拽维度（resource / project / tag）重新切片。
- **R1/R2 趋势** → 在数据 sheet 之外附加 chart sheet（利用率折线图、负载堆叠柱状图），数据与图同源。
- **R3 预算消耗** → 附加消耗率条形图 + 阈值参考线（`overload_threshold`）。
- 透视/图表的**字段绑定由 `report_templates.columns_json` 驱动**（见 §8.8），用户自定义模板时可声明 pivot 行/列/值维度与 chart 类型。

> 实现约束：pivot table 在 `rust_xlsxwriter` 中需显式构建 `ExcelPivotTable` 并关联到源数据 range；chart 需 `ExcelChart` + 数据序列引用。两者均在 Rust 侧渲染期一次性写入，不依赖 Excel 宏。

#### 8.6b PDF 生成路径选型（决策 5：HTML→PDF 为可选特性）

```
                           ┌─ 结构化表格型报表(R1/R3/R5)
                           │   → rust_xlsxwriter 先出 xlsx
PDF 来源 ──────────────────┤
                           │   → 同时 printpdf 直绘表格(无字体依赖、跨平台一致)   ← 默认通路
                           │
                           └─ 仪表盘/图文型报表(含 Gantt 缩略图)
                               → 前端 Vue 渲染为 HTML
                               → Rust 侧 headless 渲染 HTML→PDF（可选特性，见下）  ← 用户显式开启
```

> **关键约束**：Tauri v2 的 WebView 原生 `print_to_pdf` 在稳定 API 中尚未一等支持（WebView2/WKWebView 能力不一致）。因此本系统 **不依赖 webview 原生打印**，PDF 统一在 Rust 侧生成：
> - **默认通路（结构化）** → `printpdf` 直接绘制（推荐，无外部进程依赖，离线纯净）。
> - **可选通路（图文型，复用前端已渲染的图表）** → 内置一个轻量 headless 渲染（`headless_chrome` crate，绑定本地 Chromium）。**该通路为可选特性（Cargo feature `html-to-pdf`），默认不启用**；用户需在「设置 → 高级 → 启用 HTML→PDF（需本地 Chromium）」中显式开启，开启时才安装/绑定 Chromium，**不默认引入其重依赖**。未开启时图文型报表降级为 `printpdf` 纯表格版 + 提示「图表请截图」。
> - 应急通路：前端 `window.print()` 调用系统打印对话框「另存为 PDF」（用户手动），作为兜底。

**Cargo 依赖片段（默认不引入 headless_chrome）**：

```toml
[dependencies]
rust_xlsxwriter = "0.83"   # Excel .xlsx（含 pivot table / chart）
csv = "1"                  # CSV（首版优先级最高）
printpdf = "0.9"           # PDF 直绘（默认通路）
serde_json = "1"           # JSON
# 可选（图文型 HTML→PDF，默认不启用；用户在设置中显式开启时才需此 feature）
headless_chrome = { version = "1", optional = true }

[features]
default = []
html-to-pdf = ["headless_chrome"]   # 仅在用户启用 HTML→PDF 时编译进二进制
```

> 与设置页的联动：`settings` 表新增布尔列 `enable_html_to_pdf INTEGER NOT NULL DEFAULT 0`（impl 期决策：列名与默认值）。前端「设置 → 高级」展示该开关，开启时前端提示「将下载/绑定本地 Chromium，体积约 ~150MB」，用户确认后 Rust 侧首次调用时拉起/绑定 Chromium 实例并缓存。

### 8.7 生成方式分工（前端渲染 vs Rust 端生成）

| 场景 | 谁生成 | 理由 |
|------|--------|------|
| 复杂多 sheet 表格 / 大数据量（>1000 行） | **Rust 端** | 性能稳定、内存可控、跨平台一致；直接落盘文件 |
| 含图表/交互的仪表盘（看板内的「报表视图」，含 R7 成本估算的 MVP 内联视图） | **前端渲染** | 复用 ECharts/Gantt 组件，交互体验好；不入文件 |
| PDF 归档（结构化） | **Rust 端 `printpdf`** | 无字体渲染差异，离线确定性强 |
| PDF 归档（图文仪表盘） | Rust 端 HTML→PDF（**可选特性**，见 §8.6b） | 复用前端布局；不可用时降级 |
| CSV / JSON 导出 | **Rust 端** | 序列化简单、需保证与 xlsx 同源 |
| Excel 含 pivot/chart 导出 | **Rust 端 `rust_xlsxwriter`** | pivot table 与 chart 在 Rust 侧一次性写入（见 §8.6a） |
| 快照存档（R8） | **Rust 端** | 需写入 `workforce_snapshots` 表 + 落盘文件，事务性要求 |

**推荐默认**：
- 用户在看板内点「查看报表」→ 前端实时渲染（参数可调，所见即所得）。**R7 成本估算在 MVP 期即以此形态交付**（字段已就位，无需导出文件）。
- 用户点「导出」→ 弹出格式选择（CSV/Excel/PDF/JSON）→ 调用 Rust 侧 `generate_report` 命令 → **落盘到用户选定路径（系统文件保存对话框，见 §8.7a）** → 返回文件路径并 toast 提示。

#### 8.7a 导出文件落盘策略（决策 8：不引入应用内文档库）

导出文件**落盘到用户通过系统文件保存对话框选定的路径**（Tauri `dialog::save`），**不引入应用内「文档库」管理**：

- 用户点「导出」→ 调用 `tauri::plugin_dialog::DialogExt::save()`，预填文件名（如 `R1_utilization_2026-06.xlsx`），由用户选择目录与文件名。
- 导出完成后**只记审计元数据**（`export_audit` 表，见 §8.10），**不复制/接管文件本身**：`export_audit.file_path` 仅记录用户选择的绝对路径，便于「重新生成」与审计回溯；文件的实际存储、备份、清理由用户/OS 负责。
- 不做「文档库 / 最近文件列表 / 应用内打开」等管理 UI（避免文件丢失、版本混乱、跨设备同步等复杂问题）。「导出历史」视图（§8.10）仅展示「何时导出了什么参数的报表到哪个路径」，并提供「重新生成到新路径」按钮。
- 若用户选择的路径不可写（权限/磁盘满），返回 `ReportError::SaveFailed(path, reason)` 并 toast 提示重选路径。

**IPC 命令签名**：

```rust
#[tauri::command]
async fn generate_report(
    state: tauri::State<'_, AppState>,
    kind: ReportKind,            // R1..R8
    params: ReportParams,
    format: ExportFormat,        // Csv | Xlsx | Pdf | Json  （首版优先级 CSV > Excel > PDF）
    out_path: Option<String>,    // None → 弹系统 save 对话框由用户选定；Some(path) → 直接写该路径（用于「重新生成」）
) -> Result<ReportResult, String> {
    // 1. workload_engine 聚合数据（成本经 effective_daily_rate 解析）
    let rows = state.workload.aggregate(kind, &params).await?;
    // 2. 解析最终输出路径（None → dialog::save）
    let path = match out_path {
        Some(p) => p,
        None => state.dialog.save(default_filename(kind, &params, format))?.ok_or("cancelled")?,
    };
    // 3. 按 format 渲染并落盘（Excel 含 pivot/chart；PDF 默认走 printpdf，html-to-pdf feature 关闭时不走 headless）
    let path = state.exporter.render(kind, &params, &rows, format, &path).await?;
    // 4. 记录导出审计（仅元数据，不接管文件）
    state.audit.log_export(kind, &params, format, &path).await?;
    Ok(ReportResult { path, row_count: rows.len(), generated_at: now() })
}
```

### 8.8 报表模板与可配置

报表模板以**数据 + 渲染规范**分离存储，便于用户自定义：

```sql
-- 报表模板表（本地 SQLite）
CREATE TABLE report_templates (
    id            INTEGER PRIMARY KEY,
    code          TEXT NOT NULL UNIQUE,         -- 如 "R1_default"
    report_kind   TEXT NOT NULL,                -- 'R1'..'R8'
    name          TEXT NOT NULL,
    params_json   TEXT NOT NULL,                -- 预置 ReportParams（时间窗可留占位符）
    columns_json  TEXT NOT NULL,                -- 列定义：可见列、顺序、别名、格式；可含 pivot/chart 声明（§8.6a）
    thresholds    TEXT,                         -- 覆盖默认过载/欠载阈值
    is_default    INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL DEFAULT (datetime('now'))
);
```

- 内置模板：每类报表预置一个 `_default` 模板（如 R1 默认按月分桶、PD 单位、含成本列）。
- 用户模板：用户可在「报表设置」页基于内置模板「另存为」，调整列可见性、阈值、单位，保存为 `report_templates` 行。
- 占位符：`params_json` 中时间窗支持占位 `"window": "@current_month"` / `"@last_week"` / `"@custom"`，生成时由 Rust 侧 `resolve_window` 解析为具体日期，实现「一键生成本月报表」。
- 列定义示例（`columns_json`，含 pivot/chart 声明）：

```json
{
  "columns": [
    { "key": "resource_name", "label": "资源", "visible": true },
    { "key": "team_name",     "label": "团队", "visible": true },
    { "key": "capacity_pd",   "label": "容量", "visible": true, "format": "pd_pm" },
    { "key": "utilization",   "label": "利用率", "visible": true, "format": "percent1" },
    { "key": "cost",          "label": "成本", "visible": false }
  ],
  "excel": {
    "pivot": { "rows": ["team_name"], "cols": ["project_name"], "value": "allocated_pd" },
    "charts": [
      { "type": "line", "title": "利用率趋势", "x": "bucket", "y": "utilization" }
    ]
  }
}
```

### 8.9 人力快照存档（R8）

快照用于「冻结某一时刻全量人力状态」，解决实时数据持续变化导致的历史报表不可复现问题。

```sql
-- 快照主表
CREATE TABLE workforce_snapshots (
    id            INTEGER PRIMARY KEY,
    code          TEXT NOT NULL UNIQUE,         -- 如 "snap_2026_06_27"
    name          TEXT NOT NULL,
    window        TEXT NOT NULL,                -- 快照时间窗
    taken_at      TEXT NOT NULL DEFAULT (datetime('now')),
    engine_rev    TEXT NOT NULL,                -- workload_engine 版本，用于复现
    trigger       TEXT NOT NULL,                -- 'manual' | 'scheduled'
    payload_json  TEXT NOT NULL,                -- 完整快照数据（JSON）；>1MB 时外迁（见 §8.9a）
    file_path     TEXT,                         -- 关联落盘的 .xlsx/.json 文件（外迁 payload 时指向分表/独立文件）
    payload_size  INTEGER NOT NULL DEFAULT 0    -- payload 字节数，用于触发外迁判断
);
CREATE INDEX idx_snapshots_taken ON workforce_snapshots(taken_at);
```

**快照内容（`payload_json`）**：在 `taken_at` 时刻，对当前所有 resource / team / project / task / allocation 做一次只读深拷贝序列化，外加预计算的 R1 全量利用率矩阵。体积通常 KB 级，直接存 JSON 列；**超过阈值时外迁**（见 §8.9a）。

#### 8.9a 大 payload 外迁策略（决策 6）

当 `payload_json` 体积 **> 1 MB** 时，从 JSON 列迁移到独立文件/分表存储，避免 SQLite 单行膨胀导致的全表扫描与 VACUUM 开销：

| payload 体积 | 存储位置 | 记录方式 |
|---|---|---|
| ≤ 1 MB | `workforce_snapshots.payload_json`（JSON 列） | `file_path = NULL`，`payload_size` 为实际字节数 |
| > 1 MB | 独立文件 `<app_data>/snapshots/<snapshot_id>.json.gz`（gzip 压缩）或 `workforce_snapshot_blobs` 分表 | `payload_json` 置为占位 `{"_external": true, "format": "json+gz"}`，`file_path` 指向独立文件/blob_id |

**阈值与策略**：
- 阈值 `SNAPSHOT_INLINE_MAX_BYTES = 1_048_576`（1 MB），可在 `settings` 中调整（impl 期决策：是否暴露给用户 UI）。
- 外迁文件命名：`<app_data>/snapshots/snap_<id>_<taken_at_epoch>.json.gz`，与 `workforce_snapshots.id` 一一对应。
- 删除快照时（用户手动删 / 过期清理），若 `file_path` 非空，**先删独立文件再删表行**（事务内记录删除意图，文件删除失败不阻塞表行删除，但写 `audit` 警告「孤立文件残留」）。
- 读取时：`payload_json` 顶层含 `{"_external": true}` 则从 `file_path` 解压加载；否则直接解析 JSON 列。

> 可选分表方案（impl 期决策）：若不希望管理独立文件，可新增 `workforce_snapshot_blobs(snapshot_id INTEGER PK, payload BLOB, encoding TEXT)` 分表，外迁时把 payload 写入该表并在主表 `payload_json` 置占位。文件方案利于备份/迁移，分表方案利于事务一致性，二选一在 impl 期定。

**定时快照**：在「设置」中可配置 cron-like 触发（如「每月 1 日 09:00 自动快照」，`trigger='scheduled'`）。Tauri 侧用 `tokio::time` + 持久化的 `scheduled_snapshots` 配置驱动；应用未运行时段错过的快照在下次启动时按 `next_run` 补一次（仅补一次，避免堆积）。

**快照复用**：任意查询型报表（R1–R7）的 `ReportParams` 可附加 `snapshot_id`，引擎命中时从 `payload_json`（或外迁文件）取数而非实时表，从而「以 6 月 27 日的快照数据生成 R1 报表」，结果完全可复现。

### 8.10 审计与可复现元数据

每次导出在文件内（xlsx 自定义属性 / PDF 页脚 / JSON 顶层字段）写入：

```json
{
  "_meta": {
    "report_kind": "R1",
    "params": { "window": {"start":"2026-06-01","end":"2026-06-30"}, "unit":"PD", "bucket":"Month", "include_cost": true, "use_project_rates": true },
    "engine_rev": "workload_engine@0.4.2",
    "snapshot_id": null,
    "generated_at": "2026-06-27T16:42:03+08:00",
    "row_count": 24,
    "out_path": "/Users/.../R1_utilization_2026-06.xlsx"
  }
}
```

同时写入本地 `export_audit` 表，供「导出历史」视图检索与重新生成（**仅元数据，不接管文件**，见 §8.7a）：

```sql
CREATE TABLE export_audit (
    id            INTEGER PRIMARY KEY,
    report_kind   TEXT NOT NULL,
    params_json   TEXT NOT NULL,
    format        TEXT NOT NULL,
    file_path     TEXT NOT NULL,                 -- 用户选定路径（绝对路径）；文件由用户管理，应用不复制
    snapshot_id   INTEGER,                       -- 可空，关联快照
    generated_at  TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (snapshot_id) REFERENCES workforce_snapshots(id)
);
```

### 8.11 实现优先级与路线对齐（决策 9）

> **核心决策**：报表**导出整体延后到最后阶段（Phase 5+）实现，不进核心 MVP**。MVP 期 R1/R6/R7 的**数值计算与前端实时渲染**已可用（复用 workload_engine，R7 成本字段已就位），但**文件导出能力（CSV/Excel/PDF/JSON）属于 Phase 5 交付物**。本期 PDF **不套公司模板**。与 §9.1 Phase 5 口径保持一致（Phase 5 = 报表与导出，Phase 6 = 打磨）。

| 阶段 | 报表 | 格式 | 备注 |
|------|------|------|------|
| **MVP（Phase 0–4 期，前端内联视图）** | R1 资源利用率、R6 过载/欠载名单、**R7 周期成本估算**（`daily_rate_pd` / `resource_project_rates` 已在 schema） | 无文件导出；前端 ECharts 实时渲染 | 数值复用 workload_engine；R7 经 `effective_daily_rate` 解析单价；**不落文件** |
| **Phase 5（报表导出首版）** | R1, R6, R3（项目预算消耗） | **CSV（优先）> Excel > PDF** | 首版导出格式优先级 CSV > Excel > PDF；CSV 最快落地，Excel 含 pivot/chart（§8.6a），PDF 默认走 `printpdf`（**不套公司模板**） |
| **Phase 5（续）** | R2 团队 workload、R5 复用矩阵、R8 手动快照 | Excel、CSV、JSON | 团队维度 + 跨项目视角；R8 快照为大 payload 外迁打基础（§8.9a） |
| **Phase 5（续）/ Phase 6** | R4 AI 决策记录（仅结构化摘要，§8.4a） | JSON、PDF | 配合 ai-engine 输出，可解释性闭环；**不存 LLM prompt/response 原文** |
| **Phase 6 / 后续** | 定时快照、模板自定义、HTML→PDF 图文报表（可选特性，§8.6b） | 全格式 | 完整体验；HTML→PDF 由用户在设置显式开启 |

> **DoD 前置条件（更新）**：
> - R3 进入 Phase 5 的前置条件：`projects.budget_pd` 已存在（已在 §3.3.10 落地，无需新增）。✅
> - R7 成本估算**计算**进入 MVP 内联视图的前置条件：`resources.daily_rate_pd`（§3.3.4）与 `resource_project_rates`（§3.3.4a）已在 MVP schema。✅（**本轮决策前移**，原「R7 阻塞至 v1.2」的表述作废）
> - R7 成本估算**文件导出**进入 Phase 5 的前置条件：CSV/Excel 导出通路就绪（与 R1/R6 同批）。
> - R4 进入 Phase 5/6 的前置条件：`ai_optimization_runs` 的 `constraints_json` / `weights_json` / `score_*` / `config_json` / `output_plan_json` 已存在（§3.3.16）。✅ 报表只读这些结构化字段，**不依赖 LLM prompt/response 原文存储**。
> - 报表引用的 schema 字段缺失时，该报表不计入对应阶段交付物。本轮决策后，所有报表依赖字段均已在 MVP schema 就位，无字段级阻塞。

### 8.12 与 §7 TrendExplainer 的引用对齐

报表中心的「结论建议（LLM 趋势解释）」走 §5.6 的 `TrendExplainer.explain_trend(problem, history_metrics)`，本章仅引用，不在本章重复定义：

- **历史输入保留 4 周**（假设 #32）：`history_metrics` 取最近 4 周的 workload/利用率/成本趋势，超期数据不进入趋势 prompt。
- **脱敏模式同样作用于趋势 prompt**：当 `settings.privacy_mode = on` 时，发往云端 LLM 的趋势 prompt 中资源姓名、项目代号等 PII 同样替换为占位符（`RES_001` / `PROJ_A`）；本地 Ollama 不受此开关影响。与 §7 / §5.6 脱敏链一致。
- 趋势建议**不写入报表导出文件**（它是 Dashboard 内联视图的 LLM 建议，非报表数据列）；如需归档，用户可单独导出该文本为 Markdown（随 CSV/Excel 一并由 Rust 侧生成，不入 `export_audit` 的结构化字段）。

### 8.13 假设 / Assumptions

> 本节假设沿用全章假设编号；已被本轮决策解决的开放问题见 §8.14 顶部说明。

- **#14 / #28（沿用）**：全系统只存 PD；PM 仅作派生展示。`projects.budget_pd` 是唯一预算列，不存在 `budget_pm`。
- **#32（沿用，对齐 §7）**：TrendExplainer 趋势报表的历史输入保留 **4 周**；**脱敏模式同样作用于趋势 prompt**。
- **A8.1（新增）**：`resource_project_rates` 的 `(resource_id, project_id, valid_from)` 主键足以表达「同一资源在同一项目的不同周期单价」；不引入「项目阶段/milestone 维度」的更细粒度（如需，impl 期再评估）。
- **A8.2（新增）**：快照 payload 外迁阈值 `SNAPSHOT_INLINE_MAX_BYTES = 1 MB` 为默认值；实际可由 `settings` 调整。
- **A8.3（新增）**：HTML→PDF（`headless_chrome`）依赖用户机器存在可用 Chromium；应用不强制捆绑 Chromium（避免体积膨胀），由用户在设置中显式开启时才绑定。
- **A8.4（新增）**：导出文件由用户/OS 管理生命周期；应用不提供「文档库」、不做跨设备同步、不负责文件备份。`export_audit.file_path` 仅作审计回溯，不保证文件持续存在。

### 8.14 开放问题 / Open Questions

> **本轮决策已解决的开放问题**（从开放问题清单中移除，不再悬置）：
> - ~~#38 资源按项目浮动单价~~ → 由 `resource_project_rates` + `effective_daily_rate` 解决（§8.0.1 / §8.3）。
> - ~~#39 资源按周期浮动单价~~ → 由 `resource_project_rates.valid_from/valid_to` 解决（§8.0.1 / §8.3）。
> - ~~原 §8.0「daily_rate_pd 需在 §3 补列」~~ → 已在 §3.3.4 落地，MVP 即在；R7 成本估算前移到 MVP 内联视图。
> - ~~报表导出是否进 MVP / 首版格式优先级 / PDF 是否套公司模板~~（原开放问题 #45 / §9 #53 / §9 #8）→ 决策 9：报表导出延后到 Phase 5+，首版 CSV > Excel > PDF，本期 PDF 不套公司模板。
> - ~~Excel 是否支持 pivot/chart~~（隐含开放问题）→ 决策 7：支持，由 `rust_xlsxwriter` 提供。
> - ~~HTML→PDF 是否默认引入重依赖~~（隐含开放问题）→ 决策 5：可选特性，用户显式开启。
> - ~~导出文件是否做应用内文档库~~（隐含开放问题）→ 决策 8：不做，落盘到用户选定路径。
> - ~~R4 是否存 LLM prompt/response 原文~~（隐含开放问题）→ 决策 4：不存，仅结构化约束+打分摘要。
> - ~~快照大 payload 如何存储~~（隐含开放问题）→ 决策 6：>1MB 外迁独立文件/分表。

**本章剩余开放问题（impl 期决策 / 待验证）**：

- **O8.1（impl 期决策）**：`resource_project_rates` 是否需要暴露 UI 编辑入口，还是仅作为「数据导入/管理员脚本」维护？若需 UI，需在「资源详情 → 项目费率」页增加 CRUD（与 §6.2 资源命令组对齐）。
- **O8.2（impl 期决策）**：快照大 payload 外迁采用「独立文件 `.json.gz`」还是「`workforce_snapshot_blobs` 分表」？前者利于备份迁移，后者利于事务一致性（见 §8.9a）。
- **O8.3（impl 期决策）**：`SNAPSHOT_INLINE_MAX_BYTES = 1 MB` 是否暴露到用户 UI 让其调整，还是作为隐藏常量？
- **O8.4（impl 期决策）**：Excel pivot table 在 `rust_xlsxwriter` 不同版本间的字段绑定 API 是否稳定？需在 Phase 5 起步时锁定 `rust_xlsxwriter` 版本并验证 pivot/chart 在 Excel/LibreOffice/WPS 中的兼容性。
- **O8.5（impl 期决策）**：HTML→PDF 开启时 Chromium 的获取方式——引导用户系统已装的 Chrome/Edge 路径，还是应用内下载独立 Chromium？前者体积友好但路径探测复杂，后者简单但 +150MB。
- **O8.6（待验证）**：`effective_daily_rate` 在「跨周期 allocation（一个 allocation 跨越多个 `resource_project_rates` 周期记录）」时，是否需要按日拆分加权计算成本？当前公式按 allocation 的代表日期 `d` 解析单价，跨周期场景的精度需在 R7 实测时验证（若误差可接受则保持现状，否则改为按日加权）。
- **O8.7（impl 期决策）**：R4 报表的 `explanation_md` 是否随 JSON 导出？当前设计含该字段（属 LLM 输出的精炼文本，非 prompt/response 原文），但若团队认为任何 LLM 文本都不应入报表，可在导出时剔除该字段，仅保留结构化 score/constraints/weights。

## 9. 路线图 / 非功能 / 风险 / 开放问题

### 9.1 分阶段 MVP 路线

整体采用「垂直切片 + 增量可演示」的策略：每个 Phase 结束都是可独立运行、可演示的产物，且前一阶段的 schema 不被后阶段推翻（新需求通过新增表/列 + 迁移扩展，而非破坏性变更）。各阶段估算工作量以「人日」(PD) 标注，仅供参考（默认 1 PD = 8h）。

**计算单位与默认配置**（决策 #1/#2）：系统内部一律以 PD 为存储基准，默认**展示单位**为可配置项（`settings.default_unit ∈ {'PD','PM'}`，默认 `PD`），并允许**项目级覆盖**（`projects.display_unit` 覆盖全局默认）。PM 换算常数 `N` 可自定义（`settings.pm_workdays`，默认 **20**）。各阶段涉及的默认配置项见 §9.1.1。

| Phase | 主题 | 关键交付物 | 验收标准（DoD） | 估时 |
|---|---|---|---|---|
| **Phase 0** | 脚手架与数据层 | ① Tauri v2 + Vite + Vue 3 工程骨架（`pnpm tauri dev` 可起空壳）<br>② Rust workspace 分 crate：`domain` / `infra` / `ai-engine` / `app`(tauri lib)<br>③ SQLite + sqlx 接入，开启 WAL，`sqlx migrate` 机制就位<br>④ 初始迁移含全部核心表（resources / teams / projects / tasks / skills / tags / allocations / holidays / settings），即使字段尚未全用<br>⑤ IPC 通信样板（一个 `ping` → `pong` 的 `tauri::command`）<br>⑥ 单位换算配置项落地（`settings` 表：`pd_hours=8`、`pm_workdays=20`、`default_unit='PD'`；`projects.display_unit` 列允许覆盖）<br>⑦ **DB 加密默认开启**（决策 #10）：用户首次启动设置口令，派生主密钥，SQLite 经 SQLCipher 打开 | 冷启动 ≤ 2s；DB 文件生成在用户数据目录并已加密；迁移可正向执行无错；`PRAGMA journal_mode=WAL` 生效；前端能 `invoke('ping')` 收到响应 | 5–8 PD |
| **Phase 1** | 核心 CRUD + Kanban | ① Resource / Team / Project / Task / Skill / Tag 的完整 CRUD（前端表单 + 后端 `command` + sqlx 查询）<br>② Tag 与 Skill 的多对多关联（资源技能带熟练度 1–5 级，见决策 #12）<br>③ Kanban 看板视图（按 task 状态列拖拽，自绘或轻量库）<br>④ Pinia store 与 IPC 调用封装层（`useResources()` / `useTasks()` 等）<br>⑤ 单位换算在 UI 全局生效（输入 PD 自动显示 PM；展示单位遵从 `default_unit`/项目级覆盖） | 任一实体可增删改查并持久化；看板拖拽改变状态后刷新仍保持；列表/详情双向绑定无脏读；技能等级 1–5 可编辑；项目可指定本项目的展示单位 | 12–16 PD |
| **Phase 2** | Allocation + Workload/容量 + Dashboard | ① Allocation CRUD（资源 × 任务 × 时间窗 × 投入比例，**粒度 0.01 小数、步进可配**，决策 #5）<br>② Capacity 计算引擎（扣除 holidays / 非工作日 / 请假，乘投入比例）<br>③ Workload 聚合计算（按资源 / 按团队 / 按时间窗）+ 利用率公式<br>④ 冲突检测（同一资源跨任务时间窗重叠且总投入 > 100%）；**资源超载策略可配**（决策 #6）：`settings.overload_policy ∈ {'soft_warn','hard_block'}`，默认 `soft_warn`（标红警告，允许保存），可选 `hard_block`（阻止保存）<br>⑤ Dashboard：个人/团队利用率柱状图、超载预警色带<br>⑥ 时间窗内已分配人力与容量的对比可视化<br>⑦ **长期任务强制分段排期**（决策 #7）：单条 allocation 跨度超过阈值（`settings.long_task_span_limit`，默认 8 周）时强制拆段；长期任务须先拆为阶段（phase）再排期 | 给定资源+月份，workload 与容量对比准确；超载时按策略告警或拦截；利用率公式与口径一致（见 §4.3/§4.5/§4.10 及 §9.2.1）；调整一条 allocation 后 Dashboard 数字在阈值延迟内更新；长期任务分段保存后每段为独立 allocation | 14–18 PD |
| **Phase 3** | Gantt + 日历 + 跨项目 | ① Gantt 视图（任务时间轴 + allocation 条 + 依赖箭头）<br>② 日历视图（按日/周/月展示资源分配热力）<br>③ 跨项目视图（多项目泳道、资源跨项目排期一览）<br>④ 时间维度的缩放与虚拟滚动（性能保障）<br>⑤ 资源视角 vs 任务视角切换<br>⑥ 对外暴露「Gantt 叠加渲染」能力：支持在现有 Gantt 之上叠加一组「候选条带」（半透明、可区分色），供 Phase 4 的 AI 候选方案预览复用 | Gantt 拖拽改时间窗后回写 allocation；1000 个 allocation 条渲染不掉帧（见 §9.2.1 目标）；日历与 Gantt 数据同源；候选条带叠加层可被外部（Phase 4）调用渲染 | 14–18 PD |
| **Phase 4** | AI 优化引擎 | ① embedding 生成（资源技能/标签 ↔ 任务所需技能），通过 rig 接 Ollama 默认 provider；**默认模型清单见决策 #3 与 §9.1.1**（chat=qwen2.5:7b、embed=nomic-embed-text）<br>② 语义相似度矩阵 + 经典求解器（**good_lp + HiGHS 做 ILP，静态链接**，决策 #4）求解硬约束；**超载作软约束带罚值、可违反**（决策 #6）；**技能匹配权重可配**（决策 #12）<br>③ 求解结果落库为 allocation（草稿态，待用户确认）<br>④ LLM 自然语言解释方案（为何这样分配、冲突说明）<br>⑤ AI provider/key 可在设置页切换；脱敏开关生效<br>⑥ **What-if 叠加预览后置补齐**（决策 #13）：Phase 4 必须晚于 Phase 3（Gantt）完成；What-if「候选方案 diff → Gantt 叠加预览」在 Phase 3 完成后补齐，**Phase 4 期间以纯 diff 表（候选方案对比表）降级**，不含 Gantt 叠加形态 | 在 50 资源 × 100 任务规模下，10s 内出可行解；硬约束（容量上限、时间窗、不冲突、must-have 技能）100% 满足；解释文本引用到具体技能匹配依据；相同输入+固定 seed 结果可复现；**降级期内**可展示候选方案纯 diff 对比表（叠加预览待 Phase 3 完成后补齐） | 18–24 PD |
| **Phase 5** | 报表与导出 | ① 报表模板：项目人力消耗、资源利用率、团队负载、技能覆盖度<br>② **导出格式首版优先级**（决策 #8）：**CSV 优先、Excel 次之、PDF 可选**；**本期不套公司模板**（PDF 走通用排版）<br>③ 数据全量导出（JSON / SQLite 文件，用于备份迁移）<br>④ 数据导入（从导出的 JSON/CSV 恢复，含校验） | 报表数值与 Dashboard 一致；CSV/Excel 导出可读；导出文件可在另一空库导入还原 | 8–12 PD |
| **Phase 6** | 打磨：性能 / UX / 离线降级 / i18n | ① workload 重算的性能优化（物化/缓存/增量）<br>② 大数据量分页与延迟加载<br>③ AI 不可用时的降级路径（纯经典求解 / 手动分配仍可用）<br>④ **i18n 首版**（决策 #9）：**zh-CN + en 双语**；日期/货币按 locale 处理（走 `Intl`）<br>⑤ 错误处理、空状态、加载态、键盘快捷键 | §9.2 全部非功能指标达标；拔网线后除云端 AI 外全功能可用；首屏到可交互时间达标；中英双语切换生效，日期/货币随 locale 变化 | 10–14 PD |

#### 9.1.1 默认配置项汇总（决策 #1/#2/#3/#5/#6/#10/#11/#12）

以下默认值在 Phase 0/1 落入 `settings` 表（或 `projects` 表的项目级覆盖列），后续阶段直接读取：

| 配置项 | 默认值 | 归属 | 决策 |
|---|---|---|---|
| `default_unit` | `'PD'`（可配为 `'PM'`） | `settings` 全局 | #1 |
| `projects.display_unit` | NULL（继承全局，可设为 `'PD'`/`'PM'`） | `projects`（项目级覆盖） | #1 |
| `pd_hours` | `8`（1 PD = 8h） | `settings` | #2 |
| `pm_workdays`（换算常数 N） | `20`（1 PM = 20 PD，可自定义） | `settings` | #2 |
| `allocation_percent_step` | `0.01`（投入比例最小步进，可配为 0.05 等） | `settings` | #5 |
| `overload_policy` | `'soft_warn'`（可选 `'hard_block'`） | `settings` | #6 |
| `overload_soft_penalty` | 罚值（求解器软约束，可配） | `settings`/求解器配置 | #6 |
| `overload_threshold` | `1.10`（110%，与 §4.5/假设#24 一致） | `settings` | #6 |
| `long_task_span_limit` | `8 周`（超过强制分段） | `settings` | #7 |
| `db_encryption_enabled` | `true`（DB 加密默认开启，主密钥由用户口令派生） | `settings`/首启向导 | #10 |
| `backup_frequency` | `'daily'`（可配 hourly/weekly/off） | `settings` | #11 |
| `backup_keep_count` | `7`（保留份数） | `settings` | #11 |
| `backup_dir` | OS 用户数据目录下 `backups/`（可配置） | `settings` | #11 |
| `skill_level_max` | `5`（熟练度 1–5 级） | 常量 | #12 |
| `skill_match_weight` | 可配（must-have / nice-to-have 区分，影响 AI 打分） | `settings`/求解器配置 | #12 |
| Ollama chat 模型 | `qwen2.5:7b`（备选 `qwen2.5:14b` / `llama3.1:8b`） | `settings.ai_chat_model` | #3 |
| Ollama embed 模型 | `nomic-embed-text`（备选 `bge-m3`） | `settings.ai_embed_model` | #3 |
| 求解器后端 | `good_lp + HiGHS`（ILP，静态链接） | 常量 | #4 |

**本地 Ollama 默认模型 `ollama pull` 清单**（决策 #3，README 与设置页引导）：

```bash
# 推荐默认（首版）
ollama pull qwen2.5:7b           # chat 默认
ollama pull nomic-embed-text     # embedding 默认

# 备选（显存充裕 / 需要更好语义时）
ollama pull qwen2.5:14b          # chat 备选
ollama pull llama3.1:8b          # chat 备选
ollama pull bge-m3               # embedding 备选（多语种更强）
```

**技能熟练度与匹配**（决策 #12，与 §3.3.5 `resource_skills.proficiency`(1–5) / §3.3 `task_skill_requirements.min_proficiency` + `is_mandatory`(must-have/nice-to-have) 对齐）：

- 熟练度等级固定 **1–5**（`skill_level_max=5`）。
- `task_skill_requirements.is_mandatory` 区分 **must-have**（`is_mandatory=1`，求解器硬约束，资源对该技能 `proficiency ≥ min_proficiency`）与 **nice-to-have**（`is_mandatory=0`，参与 AI 匹配打分但非硬约束）。
- AI 匹配权重 `skill_match_weight` 可配：nice-to-have 命中按配置权重计入目标函数系数（与 §5 语义匹配 `skill_fit × weight` 口径一致）。

> **依赖与并行说明**：Phase 0 → 1 → 2 为强串行（数据模型是地基）。**Phase 3（Gantt/日历）先于 Phase 4（AI）**（决策 #13）：Phase 4 不得早于 Phase 3 完成；Phase 4 的「求解结果回写」依赖 Phase 2 的 allocation 落地，Phase 4 的「Gantt 叠加预览」依赖 Phase 3 的 Gantt 叠加渲染能力（Phase 3 交付物 ⑥）。**What-if「候选方案 diff → Gantt 叠加预览」在 Phase 3 完成后后置补齐；Phase 4 期间以纯 diff 对比表（无 Gantt 叠加）作为降级形态。** Phase 5、6 依赖前面所有阶段稳定。
>
> 与 §7.5.6 一致性：§7.5.6 交互流程第 2 步「候选方案 diff + Gantt 叠加预览（候选条带半透明叠在当前之上）」的交付归属明确为 **Phase 4**，依赖 **Phase 3** 的 Gantt 渲染能力；Phase 4 先行完成时叠加预览后置补齐、期间降级为纯 diff 表，不存在「DoD 不含叠加预览但交互流程含」的口径错配。

---

### 9.2 非功能需求

#### 9.2.1 性能

| 指标 | 目标值 | 说明/测算口径 |
|---|---|---|
| 冷启动到可交互 | ≤ 3s | Tauri 启动 + DB 打开（含 SQLCipher 解密）+ 首页首屏；默认空库场景 |
| 单条 CRUD IPC 往返 | ≤ 50ms | 含 sqlx 写入；WAL 下不阻塞读 |
| Workload 重算延迟（增量） | ≤ 200ms | 改一条 allocation 后，受影响资源/团队的 workload 重算 |
| Workload 全量重算（首次/换时间窗） | ≤ 1.5s @ 5000 allocation | 全表扫描 + 聚合；超出则走物化/缓存 |
| Gantt/日历渲染 | 60fps @ 1000 allocation 条；≤ 1s 首帧 @ 5000 条 | 虚拟滚动 + canvas/SVG 分层 |
| AI 求解 | ≤ 10s 出可行解 @ 50 资源 × 100 任务 | 经典求解器（good_lp + HiGHS）；超时回退贪心 |
| Embedding 批量生成 | ≤ 5s @ 100 段文本（本地 Ollama，视模型） | 增量缓存，避免重复计算 |

**实现策略**：

- **Workload 物化**：对「资源 × 月（或周）」粒度建物化结果（一张 `workload_snapshot` 表或视图），allocation 写入时触发增量更新（`AFTER INSERT/UPDATE/DELETE` 触发器或应用层 dirty 标记 + 后台重算）。查询 Dashboard 直接读快照。
- **分页与延迟加载**：列表类查询统一带 `LIMIT/OFFSET`（或 keyset 分页），前端按需加载；Gantt 仅渲染可视区间。
- **求解器规模保护**：ILP 求解前做问题规模评估，变量数超阈值（如 > 2000 整数变量）时自动降级为「分批贪心 + 局部匈牙利匹配」，并在 UI 明示降级。

**利用率口径**（Dashboard / 看板 / 报表三处必须共用同一套口径，避免过载判定翻转）：

> 利用率的统一口径见 **§4.3 / §4.4 / §4.5 / §4.10**，本节不再另立公式。核心约定如下：
> - **分母 = 毛容量 Raw Capacity（不含投入比例）**：`capacity(r,[start,end]) = Σ_{d∈[start,end]} day_factor(d, r) × 1.0`（PD），即窗口内各日的有效工作日因子之和（半天按 0.5、节假日/请假/周末按 0），**不乘** allocation 的 `percent`。投入比例只用于评估「单 allocation 盈亏」时的「任务容量 Allocated Capacity = Raw × percent」（见 §4.3 的两种 Capacity 语义区分）。
> - **分子 = Workload（含投入比例）**：`workload(r,[start,end]) = Σ_{a} overlap_days(a) × a.percent × avg_day_factor`（PD），即各 allocation 按区间折算并叠加后的总量。
> - **公式**：`utilization = workload / capacity`，分子分母同口径共用 `day_factor`。
> - **阈值（与 §4.5 / 假设#24 一致，全局可配）**：`≤ 100%` 正常；`100% < utilization ≤ 110%` 黄色告警（接近满载）；`utilization > 110%`（默认过载阈值 `overload_threshold`）**红色过载**，AI 求解器将其作为软约束惩罚项（决策 #6，罚值 `overload_soft_penalty` 可配、可违反）。
>
> ⚠️ 历史版本曾在此处给出 `Capacity = 工作日数 × 投入比例 × (1 − 请假扣除)` 并把过载阈值写为 `> 100%`，与 §4.3（毛容量不含投入比例）、§4.5 / 假设#24（阈值 110%）矛盾，已删除。**任何新增视图/报表严禁再另立利用率公式**，一律引用上述口径。

**数值示例**（与 §4.10 同口径重算，供 Dashboard / 报表对齐验证）：

资源 Alice 在 2026-07：该月工作日 23 天（已扣周末），其中 1 天全天请假（`day_factor=0`）、1 天半天请假（`day_factor=0.5`）；被分配到任务 T1（全月投入 60%）+ T2（全月投入 30%）。按毛容量口径逐日折算（为简化展示，设其余 21 天均为全工作日 `day_factor=1.0`）：

- `capacity（毛容量，不含投入比例）= Σ day_factor = 21 × 1.0 + 1 × 0.0 + 1 × 0.5 = 21.5 PD`
- `workload（含投入比例，简化为全月均匀）= Σ day_factor × Σ percent = 21.5 × (0.6 + 0.3) = 21.5 × 0.9 = 19.35 PD`
- `utilization = workload / capacity = 19.35 / 21.5 ≈ 90.0%` → **绿色，未过载**（< 100%）

对照 §4.10 的逐日示例（周窗口，capacity = Σ day_factor = 3.5 PD，workload = 3.25 PD，utilization = 92.9%），二者分母均为「毛容量不含投入比例」、分子均为「workload 含投入比例」，口径一致、可直接比较。若误把投入比例乘进分母，则上例 capacity 会变成 `21.5 × 0.9 = 19.35 PD`，utilization 恒为 100%，过载判定失效——这正是必须统一口径的原因。

#### 9.2.2 隐私与安全

| 维度 | 要求 | 实现方式 |
|---|---|---|
| 本地数据 | 所有业务数据存本地 SQLite 单文件，不上传任何服务器 | 默认无网络出口；DB 文件置于 OS 用户数据目录 |
| 密钥存储 | AI provider 的 API key（如 OpenAI/Anthropic）不得明文落盘 | 通过 `tauri-plugin-keyring`（封装 Rust `keyring` crate）写入 OS 原生钥匙串：macOS Keychain / Windows Credential Manager / Linux Secret Service；DB 配置表只存「provider 名称 + 是否启用」，不存 key 本身 |
| AI 数据脱敏 | 云端 AI 调用前可选脱敏 | 设置页提供「脱敏模式」开关：开启后，发往云端 LLM/embedding 的 prompt 中资源姓名、项目代号等 PII 字段被替换为占位符（`RES_001` / `PROJ_A`）；本地 Ollama 不受此开关影响 |
| **数据库加密（默认开启）**（决策 #10） | DB 文件整体加密，**首版默认开启** | 经 SQLCipher；**主密钥由用户口令派生**（首次启动引导设置口令，经 KDF 派生主密钥，主密钥落 OS keychain；口令本身不落盘）。不再列为 Phase 6+ 可选项；性能影响由 WAL + SQLCipher 分页缓存缓解 |
| 自动备份（决策 #11） | 应用内自动定时备份 | 频率/份数/目录可配置（默认每日/保留 7 份/用户数据目录下 `backups/`），见 §9.2.4；启用加密时备份文件继承同一加密 |
| 供应链 | 第三方 crate 须经 `cargo audit`；前端依赖定期 `pnpm audit` | CI 中加入安全扫描步骤 |

> **口令丢失风险**（决策 #10）：DB 加密默认开启后，主密钥派生自用户口令，**口令丢失将导致数据不可恢复**。须在首启引导与设置页明确警示，并提供「导出恢复密钥/备份口令」的引导（Phase 0/1 落地基础提示，Phase 6 强化 UX）。建议同时提示用户配置 OS 账户恢复路径。

#### 9.2.3 可离线

- 全功能离线：除「云端 AI provider」外，CRUD、Kanban、Gantt、日历、workload 计算、经典求解器（good_lp + HiGHS）、报表导出（CSV/Excel/PDF）均可在无网环境运行。
- AI 降级链：`本地 Ollama(默认)` → 不可用时 → `云端 provider(若配置且联网)` → 不可用时 → `纯经典求解器(无 LLM 解释)` → 仍可手动分配。每一级降级在 UI 显式提示当前模式。
- Ollama 健康检查：应用启动时探测 `http://localhost:11434`，未运行则在设置页提示安装指引（含默认模型 `qwen2.5:7b` / `nomic-embed-text` 的 `ollama pull` 命令），但不阻断主流程。

#### 9.2.4 可迁移性

| 关注点 | 方案 |
|---|---|
| Schema 迁移 | `sqlx migrate` 版本化迁移；每条迁移不可变（已发布的不改，只追加新迁移）。迁移文件命名 `_NNN_描述.sql`，NNN 单调递增 |
| 数据导出（备份） | 提供 `导出全部` command：① 物理备份 `VACUUM INTO 'backup-<ts>.db'`（推荐，整库一致性快照；启用加密时继承加密）；② 逻辑导出 JSON（按表，带 schema 版本号） |
| **自动备份**（决策 #11） | 可配置自动 `VACUUM INTO` 快照：`backup_frequency`（默认 `daily`）/ `backup_keep_count`（默认 `7`）/ `backup_dir`（默认用户数据目录下 `backups/`，可配置）；超过保留份数自动滚动删除最旧；调度基于 Tauri 进程存活，错过的任务下次启动补一次 |
| 数据导入 | 从 JSON 导入时校验 schema 版本，版本不匹配则提示先升级应用；导入采用事务，失败全回滚 |
| 跨设备迁移 | 单用户场景下，用户可手动复制 DB 文件（含加密）+ 在新设备 keychain 配置 key/重新输入口令；不做自动云同步 |
| 单位偏好迁移 | `pd_hours` / `pm_workdays` / `default_unit` 存 settings 表，随 DB 导出；历史 PD/PM 数值本身不随单位变更重算（单位仅影响「展示换算」，存量 allocation 的 PD 绝对值不变） |

#### 9.2.5 可测试性

| 层 | 测试类型 | 覆盖目标 |
|---|---|---|
| `domain`（Rust） | 单元测试 | workload/capacity 计算函数、单位换算、冲突检测；覆盖率 ≥ 80% |
| `infra`（Rust，sqlx） | 集成测试 | 用内存 SQLite（或临时文件 DB）跑迁移 + CRUD；验证触发器/物化正确性 |
| `ai-engine`（Rust） | 黄金用例(golden) | 固定输入 + 固定 seed → 期望分配方案快照；embedding 相似度矩阵与求解结果可复现断言；求解器硬约束满足性 100%（must-have 技能、容量、时间窗） |
| 前端（Vue） | 组件测试(Vitest) | 表单校验、Kanban 拖拽状态机、Pinia store 与 IPC mock；关键组件覆盖率 ≥ 70% |
| E2E | 端到端(WebdriverIO/Playwright + Tauri 测驱动) | 至少覆盖「建项目→建任务→分配资源→看 workload→导出 CSV」主流程一条 |

**优化器黄金用例示例结构**（`ai-engine/tests/golden/`）：

```rust
// golden/case_001_basic_balance.toml
seed = 42
[resources]
# 3 资源，各自技能与容量
[skills_match] # 期望: 资源技能命中任务所需技能
[[expected.allocations]]
resource = "R1"
task = "T1"
# 断言: 不超载、技能匹配分 ≥ 阈值、must-have 硬约束满足
```

CI 中跑全部 golden case，结果与快照不一致即失败（除非显式更新快照并说明原因）。

#### 9.2.6 国际化预留（决策 #9）

- **i18n 首版语言范围：zh-CN + en**（Phase 6 落地 `vue-i18n`，文案抽到 `locales/zh-CN.json` / `en.json`）。首版不引入第三语言，但键结构保持可扩展（扁平 key + 命名空间，不硬编码复数逻辑以便后续追加）。
- 从 Phase 1 起，前端**禁止硬编码中文文案**，统一走 `t('key')`（即使初期只有中文）。
- **日期/货币格式化按 locale 处理**：日期/数字走 `Intl` API（`Intl.DateTimeFormat` / `Intl.NumberFormat`），货币（如成本估算列）按 locale 货币符号与精度处理；避免单位（PD/PM）和日期在不同 locale 下错位。
- DB 中 enum/状态值用英文常量存储（如 `status='in_progress'`），仅展示层做 i18n 映射。

---

### 9.3 主要风险与缓解

| # | 风险 | 影响 | 概率 | 缓解措施 |
|---|---|---|---|---|
| R1 | **Ollama 未安装 / 模型未拉取** | Phase 4 的 embedding 与 LLM 解释无法本地运行 | 高 | 启动健康检查 + 设置页引导安装与 `ollama pull qwen2.5:7b` / `nomic-embed-text`（决策 #3 默认清单）；降级到云端或纯经典求解器；在 README 与应用内提供一键检测脚本 |
| R2 | **经典求解器规模上限**（ILP 在大变量数下求解时间爆炸） | AI 优化在大数据量下卡死或超时 | 中 | 求解前评估变量数，超阈值自动降级为贪心+匈牙利匹配分批求解；设硬超时（如 10s）；UI 明示「已降级」 |
| R3 | **前端 Gantt/日历性能**（大量 DOM 节点卡顿） | 大数据量下交互掉帧、首帧白屏 | 中 | 虚拟滚动 + 仅渲染可视区间；canvas 分层绘制时间轴；allocation 条数 > 阈值时聚合（按周/按资源聚合显示） |
| R4 | **AI 结果不可解释 / 不可复现** | 用户不信任分配方案，或重算结果漂移 | 中 | 固定 seed + 确定性求解器（good_lp + HiGHS）保证硬约束可复现；LLM 解释强制引用技能匹配分数与冲突明细；提供「方案 diff」对比（Phase 4 降级期为纯 diff 表） |
| R5 | **rig 抽象对 provider 切换不充分** | 切换 Ollama↔云端时 embedding 维度/接口不一致 | 中 | 在 ai-engine 层封装统一 embedding trait，做维度归一化（投影或选统一维度模型，如 nomic-embed-text / bge-m3）；provider 切换有冒烟测试 |
| R6 | **sqlx 编译期校验与迁移耦合** | 迁移未跑时编译失败；CI 环境需 DB | 中 | 离线模式用 `sqlx::query!` 的 `cargo sqlx prepare` 生成 `.sqlx` 缓存；CI 依赖缓存而非实时连库 |
| R7 | **Tauri v2 插件生态变动**（keyring/sql 等 plugin 版本不稳定） | 升级破坏构建 | 中 | 锁定插件次版本；对关键插件（keyring/SQLCipher）做薄封装层，便于替换实现 |
| R8 | **数据丢失**（误删 / DB 损坏 / 加密口令丢失） | 用户历史数据不可恢复 | 中 | **自动备份可配**（决策 #11，默认每日/保留 7 份/用户目录）；关键删除做软删除（`deleted_at`）；启用加密时（决策 #10，默认开启）强制引导导出恢复密钥/记录备份口令 |
| R9 | **单位换算口径不一致**（PD/PM 切换时存量数据语义混乱） | 报表/利用率数字出错 | 中 | 单位仅作「展示换算」，存量 allocation 的 PD 绝对值固定不变；所有计算内部统一用 PD，仅 UI 层做 PM 折算（N=`pm_workdays` 默认 20，可配，决策 #2） |
| R10 | **跨项目资源冲突漏检**（时间窗跨月/跨周边界） | 排期出现隐性超载 | 中 | 冲突检测按「日」粒度逐日核算总投入比例，避免按月聚合掩盖峰值；提供逐日负载明细视图 |
| R11 | **利用率口径被各视图各自实现而漂移**（Dashboard / 看板 / 报表各取一套公式，过载判定翻转） | 同一资源在三处显示不同利用率，过载红绿灯互相矛盾 | 中 | 利用率公式单一真相源：分母=毛容量（不含投入比例）、分子=workload（含投入比例）、过载阈值 110%，统一见 §4.3/§4.5/§4.10；新增视图/报表强制复用 `workload_engine` 的聚合函数，禁止本地重算公式；CI 增加口径一致性断言 |
| R12 | **长期任务跨数月导致 Gantt/workload 失真** | 单条 allocation 跨度过长、分段缺失 | 中 | 长期任务强制分段排期（决策 #7，`long_task_span_limit` 默认 8 周超限拆段） |
| R13 | **DB 加密默认开启带来首启体验/性能负担**（决策 #10） | 用户遗忘口令、冷启动变慢 | 中 | 首启引导强制设口令并提示备份；性能经 SQLCipher 分页缓存 + WAL 调优；口令丢失风险在 R8 备份策略中覆盖 |
| R14 | **What-if 叠加预览在 Phase 4 降级期内用户体验割裂**（决策 #13） | 纯 diff 表无 Gantt 叠加，用户难直观对比 | 中 | 降级期 diff 表信息充分（资源/任务/比例/容量逐行对比）；Phase 3 完成后立即补齐叠加预览；UI 明示「当前为降级预览，叠加视图待补」 |

---

### 9.4 开放问题清单

下列为遗留/impl 期待定项。本轮决策（§9.1.1 决策 #1–#14）已解决的开放问题已从本清单移除。

1. **DB 加密口令恢复机制细节**（决策 #10 衍生）：口令派生主密钥的 KDF 选型（Argon2id / PBKDF2）、是否支持多设备/多口令、恢复密钥导出格式 —— **impl 期决策**。
2. **求解器软约束罚值标定**（决策 #6 衍生）：`overload_soft_penalty` 的默认量级与 nice-to-have 技能权重 `skill_match_weight` 的默认取值，需在 golden 用例上调参确认 —— **impl 期决策**。
3. **长期任务分段算法**（决策 #7 衍生）：超过 `long_task_span_limit` 时的自动拆段策略（按周等分 / 按里程碑节点 / 按阶段依赖）与拆段后依赖箭头处理 —— **impl 期决策**。
4. **PM 换算常数 N 在组织/团队级（而非全局/项目级）可配置**（决策 #1/#2 衍生，对应开放问题汇总 #2）：当前已支持全局 `settings.pm_workdays` + 项目级 `projects.display_unit`；是否需要「团队级」或「组织级」覆盖层 —— 待后续需求确认。
5. **利用率阈值按角色/团队分别设定**（对应开放问题汇总 #3）：当前 `overload_threshold` 全局可配，是否细化为按角色/团队多套阈值 —— 待后续需求确认。
6. **依赖环检测的边界**（§7.5.3，应用层环检测已在前端落地）：复杂 DAG 下环检测的告警粒度与自动断环建议 —— impl 期决策。
7. **跨设备自动云同步**（非目标，本期不做）：是否在后续版本引入端到端加密的云同步 —— 待产品规划。
8. **第三语言扩展**（决策 #9 衍生）：首版仅 zh-CN + en，键结构已预留扩展，是否在后续加入日文等第三语言 —— 待市场反馈。
9. **报表公司模板**（决策 #8 衍生）：本期 PDF 不套公司模板；后续是否提供模板可插拔能力 —— 待市场/客户反馈。
10. **DB 加密性能基准**（决策 #10 衍生）：SQLCipher 在 5000 allocation 规模下的冷启动与查询延迟实测是否满足 §9.2.1 目标 —— impl 期验证。

---

> **已支持的非均匀日容量**（决策 #14，更新原开放问题 #12）：work_week_template 已支持**非均匀日容量**（`mon_frac..sun_frac` 逐日 fraction，如周五半天 0.5）；原 §9.4 开放问题 #12「日容量是否支持非均匀」已升级为「已支持」，不再列为开放问题。详见 §3 work_week_template 与假设#22。


---

## 假设汇总（需用户确认 / impl 期）

1. §1  产品正式命名为 Development Resource Kanban（全称），简称 DevResource Kanban；对外文档用全称、UI/日志/代码可用简称，不并存第三种写法。

2. §1  默认工时换算常数为 1 PD = 8h、1 PM = 20 PD；二者均支持「全局 → 团队/组织」多级覆盖，覆盖优先级为团队级 > 全局。

3. §1  利用率红绿灯阈值默认 70% / 100%（闲置/健康/过载分界）；支持按角色 / 团队分别设定，优先级为角色级 > 团队级 > 全局默认，未设定者回退全局。

4. §1  AI 优化在 资源 ≤ 10 / 任务 ≤ 50 / 项目 ≤ 5 规模下端到端 ≤ 5s（本地 Ollama）为 MVP 性能基线；该规模同时作为 MVP 验收的硬性边界，超出规模的场景不纳入 MVP 验收。

5. §1  存储口径统一为 PD，PM 仅作展示换算；若用户存在双口径存储需求需重新评估。

6. §1  开发者视图为只读、不提供工时填报入口；若后续需要打卡回填实际工时需扩展范围。

7. §1  报表导出格式 MVP 至少支持 CSV 与 Markdown，Excel 作为进阶。

8. §1  AI 优化目标函数为「均衡负载 / 技能最优 / 预算」等目标的加权综合，权重由用户在 UI 调节（默认均衡权重 0.4/0.4/0.2），未调节时使用默认；切换权重重跑视为新 run。

9. §2  §2  单进程运行：整个应用为单 OS 进程（Tauri 主进程 + tokio 多线程运行时）；唯一可选外部进程是 Ollama HTTP 服务，优化求解本身不下放到独立 OS 进程。

10. §2  §2  领域层零 serde：`domain` crate 不依赖 `serde`、`DomainError` 不派生 `Serialize`；序列化与错误映射职责集中在 IPC / 服务层的 `AppError`。

11. §2  §2  二进制体积无上限：静态链接 HiGHS（`highs-sys`）与 SQLite（`libsqlite3-sys bundled`）造成的体积增量可接受；优先零运行时外部依赖。

12. §2  §2  求解器后端固定为 HiGHS：good_lp 启用 `highs` feature、关闭默认 feature，显式使用 `good_lp::highs::highs`，不引入 `coin_cbc` / `default_solver` / `clarabel`。

13. §2  §2  SQLite 经 `libsqlite3-sys` 的 `bundled` 特性静态链接，无需目标机系统库。

14. §2  §2  单位换算固定 N=20：`1 PM = 20 PD`，不按资源 / 地区动态计算（与 §4.2.1 单一时区 / 单一工作日历地区假设一致）。

15. §2  §2  LLM 为可选增强：LLM 调用默认 `temperature=0` 以保证可复现；LLM 不可用时 `TemplateExplainer` 提供规则化解释，经典优化硬约束求解永远可用、不依赖 LLM。

16. §2  §2  AppConfig（provider/model/key/单位 PM↔PD，N=20）存于 SQLite `app_config` 表。

17. §2  §2  前端 UI 库默认推荐 Naive UI；Gantt 默认自绘 SVG；日历用 vue-cal。

18. §2  §2  构建期 C/C++ 工具链可用：静态编译 HiGHS / SQLite 需要目标平台 C/C++ 编译器；CI 固定工具链版本以保证可复现。

19. §3  日历模型单一真相源为 §4.2 三表 work_week_template / holiday / time_off；settings.workweek_mask 与 resource_unavailable 已废弃删除。

20. §3  capacity/workload 必须「逐资源逐日」按 Σ day_factor 计算，不可折叠成全局工作日常量；§3.6 聚合 SQL 仅为概念示意，真正口径以 §4.9 Rust 计算核心为准。

21. §3  allocated_pd 是按 allocation 全程全量计算的冗余字段，仅在窗口完整覆盖全程时可直接 SUM；通用窗口聚合须 Rust 按 overlap×percent×avg_day_factor 重算。

22. §3  容量上限统一为「比例口径」Σ percent ≤ 1.0（任一有效工作日）；daily_capacity_pd 仅用于 PD 展示折算与跨资源加总，不参与过载阈值。

23. §3  优化运行表统一为 ai_optimization_runs（INTEGER 自增 PK），§5.7 的 ai_run(TEXT UUID) 已删除；allocations.run_id 为 INTEGER，AllocationProblem.run_id 为 i64，全链路一致（开放问题 #13 已解决）。

24. §3  resources 表不再保留 default_capacity（与 daily_capacity_pd 重叠）；daily_rate_pd（可空）供成本估算；新建 allocation percent 默认 1.0。

25. §3  projects 预算统一为 budget_pd，不新增、不保留任何 budget_pm 列（跨节 budget_pm 表述一律废弃）；§8 报表需对齐此字段名（开放问题 #12 已解决）。

26. §3  allocation 时间窗硬约束由 §3.3.15a 数据库触发器在 schema 层强制，并由 AllocationService::create 单一写入入口预检，两层互为冗余。

27. §3  work_week_template 的 mon_frac..sun_frac 列支持周期性非均匀日容量（如周五半天），§9.4 开放问题 #12 升级为「支持」（开放问题 #16 同步更新已落地）。

28. §3  §5 MILP 的 0/1 变量 x 与连续变量 y/percent 通过耦合约束闭合单位，容量约束在比例空间 Σ_t percent ≤ cap（cap=day_factor≤1.0）表达。

29. §3  work_week_template 全局唯一性由基于常量的 UNIQUE INDEX idx_wwt_global ON work_week_template((1)) WHERE scope='global' 强制「全局仅一行」；项目级模板由 UNIQUE idx_wwt_project(project_id) WHERE scope='project' 保证每项目至多一条（开放问题 #15 已解决）。

30. §3  max_parallel_tasks_per_day 支持「按资源」与「按项目」细粒度配置：resources.max_parallel_tasks_per_day（资源级）→ projects.max_parallel_tasks_per_day（项目级）→ 全局 ConstraintFlags（默认 None=不限并行）三级回落（开放问题 #26 已解决）。

31. §3  PD/PM 常数与利用率阈值支持团队级覆盖：新增 team_overrides(team_id PK, pd_hours, pm_workdays, overload_threshold, underload_threshold, utilization_green, utilization_yellow)，未覆盖项回落 settings 全局值；effective 常数在加载时按资源所属 team 展开为逐资源值（响应开放问题 #2/#3/#46/#47 的「团队级可配置」决议）。

32. §3  settings.secret_store ∈ {'keychain','encrypted_file'} 作为独立 settings 列（非 metadata JSON）承载密钥存储后端，与 §6 keychain 降级路径对齐（开放问题 #29 已解决）。

33. §3  成本单价支持「默认 + 项目级覆盖」两层：resources.daily_rate_pd（默认单价）与 resource_project_rates(resource, project[, period])（项目级/周期浮动费率）；effective_daily_rate(r,p,d) 解析顺序为 resource_project_rates → resources.daily_rate_pd → N/A（响应开放问题 #38 的「按项目/周期浮动」决议）。

34. §3  resources.daily_rate_pd 与 resource_project_rates 表均在 MVP 阶段即补入 schema，使成本核算能力可前移（开放问题 #39 已解决）；R7 成本报表是否纳入 MVP DoD 仍由 §8/§9 路线图决定。

35. §4  单一时区 / 单一工作日历地区（对应假设#24）：整个组织运行在同一时区、同一套工作日历语义下，day_factor 不按资源归属地区分别解析；跨时区团队为非目标，已纳入 §1.5。若未来启用需重大版本升级（见 §4.2.1 四步变更清单：region/timezone 列、按 resource.region 解析日历、workload_cache 全量 engine_rev 迁移、§4.3/§4.4 口径重评）。

36. §4  唯一权威计算源是 §4.9 Rust 纯函数核心（对应假设#25）：按日 overlap + day_factor，无副作用、可复现；allocated_pd、workload_cache、allocation_daily、§3.6 SQL 聚合均为派生/缓存，口径须向 §4.9 对齐。

37. §4  allocations.allocated_pd 的语义边界（对应假设#26）：严格等于 alloc_pd(a, [a.start, a.end])，不含 overlap 折算与窗口内 avg_day_factor；仅用于单条 allocation 工作量展示，不可用于跨窗口 workload/utilization 聚合。

38. §4  workload_cache 是实时热缓存（对应假设#27）：短周期、可重建、依赖当前引擎与配置；与 §8.9 workforce_snapshots（冷归档、不可变）目标不同、互不替代。

39. §4  workload_cache 双指纹自洽（对应假设#28）：通过 engine_rev + config_hash 判定 stale；引擎升级或 workweek/holiday/单位配置变更即失效，读取时回退 §4.9 精算并异步回填。

40. §4  allocation_daily 数学等价性（对应假设#29）：每行 pd = day_factor(day) × percent，SUM 后与 §4.9 Rust 逐日累加数学等价；其是否纳入 MVP 为 impl 期决策（见 §4.12），不引入则统一走 §4.9 精算。

41. §5  §3.8 容量上限统一为「比例口径」Σ_t percent_{r,t,d} ≤ day_factor(d,r)，MILP 在比例空间求解；daily_capacity_pd 仅用于 PD 展示折算（pd = percent × daily_capacity_pd × day_factor），不参与过载阈值（见假设汇总 #17/#23）。

42. §5  MILP 单位闭合采用「连续 percent_{r,t,d} 主变量 + 0/1 x_{r,t,d} 指示 + 聚合 y_{r,t}」的耦合形式：percent ≤ x、percent ≥ ε·x、y = Σ percent / |W_t|（与 §3.8 对齐，见假设汇总 #23）。

43. §5  工作量满足约束默认建模为带松弛的软目标（最大化已排工作量比例，按 Σ percent × day_factor × 1PD 计，缺口写入 unscheduled_tasks）；MILP 返回 infeasible 时自动松弛重解或退化贪心，绝不返回空解（见假设汇总 #33）。

44. §5  并行上限与容量上限是两个独立约束：max_parallel_tasks_per_day 默认 None（不限并行，仅受比例容量约束）；Some(1) 表示禁并行（见假设汇总 #34）。

45. §5  本章统一落库到 §3.3.16 的 ai_optimization_runs（INTEGER 自增主键，本轮拍板不引入 UUID）与 §3.3.15 的 allocations（source/run_id 已定义，percent 直接承接 MILP 解），不另起 ai_run 表（见假设汇总 #18/#35）。

46. §5  所有写事务（apply_solution/persist_run）统一走 §3.7 的 with_write_tx（BEGIN IMMEDIATE + busy_timeout + SQLITE_BUSY 退避重试），禁止直接 pool.begin()（见假设汇总 #36）。

47. §5  persist_run 仅做短写事务（写元数据/快照/解 JSON），不含 LLM 调用与大批量 allocation 插入；LLM 调用在 run() 内于 persist 之前完成且在事务之外，符合 §6.5（见假设汇总 #37）。

48. §5  input_snapshot_json 持久化「文本→向量」向量快照（或哈希 + provider/model/维度）；向量体积方案（旁表 vs 压缩序列化）impl 期评估定稿（见假设汇总 #38 / 开放问题 #24）。

49. §5  多目标权重 ObjectiveWeights 由用户在 UI「约束面板」调节（滑块/预设），写入目标函数与 weights_json，下次 run 生效（对应开放问题 #6「是」）。

50. §5  降级链：LLM 不可用 → TemplateExplainer（规则模板，零 LLM 依赖）；embedding 不可用 → FallbackScorer（关键词 Jaccard + 熟练度）（对应开放问题 #8「是」）。

51. §5  ILP 后端默认采用 good_lp + HiGHS（静态链接，二进制体积无上限，对应开放问题 #7/#49 与假设汇总 #10/#31）。

52. §5  rig-core ≈ 0.24.x：Prompt trait 的 prompt() 返回 PromptResponse，文本字段为 .content（非 .choices）；embed 走 EmbeddingModel::embed 或 EmbeddingsBuilder，Ollama+EmbeddingsBuilder 组合存在 issue #1082 的 import 路径与维度陷阱。字段名随次版本变化，落地前以锁定版 docs.rs/rig-core 为准，Cargo.toml 用 rig-core = "=0.24.x" 锁死次版本（见假设汇总 #30）。

53. §5  good_lp 默认 default_solver 常量是 coin_cbc::coin_cbc 的别名；桌面分发必须 default-features = false 且仅启用 highs feature，并显式用 good_lp::highs::highs 求解器实例，禁止 default_solver（见假设汇总 #31）。

54. §5  MILP 决策变量规模 N ≈ 2·R·T·D，good_lp/HiGHS 在 60s 预算内可解量级约 < 2 万变量；超过则按 project/team 分块，§2.4 的 50×200 示例需理解为分块/分窗而非单次 MILP（见假设汇总 #32）。

55. §6  domain/* 层零 serde 依赖：DomainError 不派生 serde::Serialize；IPC 边界由 dto/mappers.rs 的 From<DomainError> for AppError 把领域错误映射为 AppError::Domain(DomainErrorDetail)（结构化，仍可序列化）或 AppError::Validation(String)（字符串）。领域层保持纯净、可脱离 Tauri/serde 单测与复用。

56. §6  所有写事务统一走 db::with_write_tx(&SqlitePool, …)：BEGIN IMMEDIATE + busy_timeout（连接池 PRAGMA 5000ms）+ 应用层 SQLITE_BUSY 退避重试（指数退避 50/100/200ms，最多 3 次）。与 §3.7 / §5 的 with_write_tx 为同一份实现，严禁 pool.begin() 裸开。

57. §6  settings.secret_store ∈ {'keychain','encrypted_file'} 为 §3.3.1 settings 表后续迁移（ALTER TABLE ... ADD COLUMN）新增列（非 settings.metadata JSON），DEFAULT 'encrypted_file'，由 settings_svc 直接读写，SettingsDto 透出该字段供 UI 显示当前密钥落点。

58. §6  encrypted_file 降级路径的主密钥由用户口令派生（方案 b）：首启引导设置口令，Argon2id（机器绑定 salt + app pepper）派生 32B 主密钥；口令不落盘、不进日志、不上报。明文 key 经 AES-256-GCM 加密为 nonce||ciphertext||tag 后写 settings.ai_api_key_enc。换口令需用新旧主密钥重加密所有密文。

59. §6  settings.ai_api_key_enc 列（§3.3.1 已定义）作为 keychain 降级路径的落库目标，应用层 AES-256-GCM 加密；主密钥由用户口令派生（见假设 4），机器绑定、不随 DB 导出。

60. §6  keychain→encrypted_file 的「回切/迁移到钥匙串」交互不进 MVP，列为 §9 路线图的最后实现项；MVP 阶段 encrypted_file 一旦启用即视为该机器长期落点，不提供迁移按钮；MVP 也不做 keychain→encrypted_file 自动降级写回，keychain 平台不可用直接返回 AppError::Config 引导用户显式切换 secret_store。

61. §7  §7.1 桌面通知经 Tauri v2 `notification` 插件实现；macOS/Windows/Linux 三端原生通知能力可用，授权交互由插件封装。不做邮件、不做 Web Push（§1.5 已排除）。

62. §7  §7.3 预算预警阈值 `projects.budget_alert_threshold` 为项目级可空字段，空值回落全局 `overload_threshold`；该列与 `check_budget_alert` 命令的 schema 定义属 §3 / §6.2.3 范围，本节仅消费。

63. §7  §7.3 `useAlertStore.dismissed` 为会话内内存状态，不落 SQLite、不写 `localStorage`（可选易失 session storage 仅做页面刷新恢复，关闭应用即丢）。

64. §7  §7.3 `severityOrder` 与 `desktopNotifyMinSeverity` 持久化到 `app_config`（定义属 §3 / §9），默认 `severityOrder=['budget','overload','dependency','skill_gap']`、`desktopNotifyMinSeverity='high'`。

65. §7  §7.5.3 影响链限深 `depth` 默认 2（可配 1），间接后继只 tooltip 计数；`gantt.impact_chain_depth` 存 `app_config`。

66. §7  §7.5.5 `ai_optimization_runs.trigger` 取值集为 `'manual' | 'scheduled' | 'handover'`；该列定义属 §3.3.16 范围，本节仅约定离职重排写入 `'handover'`。

67. §7  §7.5.5 交接人过载策略与全局资源超载策略（§9.4 #51）对齐，不引入独立的交接人过载策略。

68. §7  §7.5.6 What-if 同屏叠加场景上限默认 4（可配），超出禁用勾选。

69. §7  §7.5.7 `TrendExplainer.explain_trend` 的 `history_metrics` 默认保留最近 4 周（可配 2–8），脱敏模式同样作用于趋势 prompt（与方案解释共用同一脱敏管线）。

70. §7  §7 前端默认 UI 库为 Naive UI；Gantt 默认 dhtmlx-gantt；日历默认自绘 SVG（与全局假设 #9 对齐，本节不再展开备选争议）。

71. §8  **#14 / #28（沿用）**：全系统只存 PD；PM 仅作派生展示。`projects.budget_pd` 是唯一预算列，不存在 `budget_pm`。

72. §8  **#32（沿用，对齐 §7）**：TrendExplainer 趋势报表的历史输入保留 **4 周**；**脱敏模式同样作用于趋势 prompt**。

73. §8  **A8.1（新增）**：`resource_project_rates` 的 `(resource_id, project_id, valid_from)` 主键足以表达「同一资源在同一项目的不同周期单价」；不引入「项目阶段/milestone 维度」的更细粒度（如需，impl 期再评估）。

74. §8  **A8.2（新增）**：快照 payload 外迁阈值 `SNAPSHOT_INLINE_MAX_BYTES = 1 MB` 为默认值；实际可由 `settings` 调整。

75. §8  **A8.3（新增）**：HTML→PDF（`headless_chrome`）依赖用户机器存在可用 Chromium；应用不强制捆绑 Chromium（避免体积膨胀），由用户在设置中显式开启时才绑定。

76. §8  **A8.4（新增）**：导出文件由用户/OS 管理生命周期；应用不提供「文档库」、不做跨设备同步、不负责文件备份。`export_audit.file_path` 仅作审计回溯，不保证文件持续存在。

77. §9  §9 计算单位与默认配置：内部一律以 PD 为存储基准；默认展示单位可配(settings.default_unit ∈ {'PD','PM'}，默认 PD)，允许项目级覆盖(projects.display_unit)；PM 换算常数 N 可自定义(settings.pm_workdays，默认 20)；历史 PD 绝对值不随单位变更重算。

78. §9  §9 求解器与超载：默认采用 good_lp + HiGHS(ILP，静态链接)；超载作软约束带罚值(overload_soft_penalty 可配、可违反)；超载阈值 overload_threshold 默认 1.10(110%，与 §4.5/假设#24 一致)。

79. §9  §9 投入比例与分段：allocation 投入比例粒度为 0.01 小数、步进可配(allocation_percent_step)；单条 allocation 跨度超过 long_task_span_limit(默认 8 周)强制分段。

80. §9  §9 AI 默认模型：本地 Ollama 默认 chat=qwen2.5:7b(备选 qwen2.5:14b/llama3.1:8b)、embed=nomic-embed-text(备选 bge-m3)；README/设置页给出 ollama pull 清单。

81. §9  §9 技能熟练度：熟练度固定 1–5 级(skill_level_max=5)；task_skill_requirements.is_mandatory 区分 must-have(硬约束)与 nice-to-have(参与 AI 打分)；AI 匹配权重 skill_match_weight 可配，与 §5 skill_fit×weight 口径一致。

82. §9  §9 DB 加密默认开启：首版经 SQLCipher 整库加密，主密钥由用户首启口令经 KDF 派生并落 OS keychain(口令本身不落盘)；不再列为 Phase 6+ 可选项；口令丢失数据不可恢复，须强制引导导出恢复密钥。

83. §9  §9 自动备份：频率/份数/目录可配(默认每日/保留 7 份/用户数据目录下 backups/)；调度基于 Tauri 进程存活，错过的任务下次启动补一次；启用加密时备份继承加密。

84. §9  §9 i18n 首版：仅 zh-CN + en，键结构保持可扩展；日期/货币按 locale 经 Intl API 处理；DB enum 用英文常量存储，仅展示层 i18n 映射。

85. §9  §9 报表首版：导出优先级 CSV 优先、Excel 次之、PDF 可选；本期 PDF 不套公司模板。

86. §9  §9 排期依赖：Phase 3(Gantt)先于 Phase 4(AI)；What-if「候选方案 diff → Gantt 叠加预览」在 Phase 3 完成后后置补齐，Phase 4 期间以纯 diff 对比表降级(不含 Gantt 叠加)。

87. §9  §9 利用率口径单一真相源(保留既有假设#65)：分母=毛容量 Raw Capacity(不含投入比例 = Σ day_factor×1.0)、分子=workload(含投入比例)、过载阈值默认 110% 全局可配；Dashboard/看板/报表三处严禁另立公式，强制复用 workload_engine 聚合函数。

88. §9  §9 work_week_template 已支持非均匀日容量(mon_frac..sun_frac 逐日 fraction，对应假设#22 与决策#14)；原开放问题#12「日容量是否支持非均匀」已升级为「已支持」。


---

## 开放问题汇总（impl 期决策为主）

1. §1  [impl 期决策] 团队级 / 角色级换算常数与利用率阈值的存储形式（独立配置表 vs settings.metadata JSON）与 UI 落点，待第 3 章（schema）与第 7 章（前端设置页）落地时确认。

2. §1  [impl 期决策] AI 多目标权重的 UI 控件形态（滑块 vs 数值输入 vs 预设档位「均衡/技能优先/控预算」）与是否暴露更多目标（如「最少切换成本」「成长机会」），待第 5 章（AI 引擎）与第 7 章（优化面板）落地时确认。

3. §1  [impl 期决策] 权重组合与利用率阈值覆盖是否随 ai_optimization_runs / 配置变更记入审计，便于方案可追溯——待第 3/5 章确认字段去向。

4. §2  §2（impl 期决策）§6.4 AppError 对齐：§6.4 当前写法为 `AppError::Domain(#[from] DomainError)` 内嵌（要求 `DomainError` 派生 `Serialize`），与本章「领域层零 serde、`DomainError → AppError::Domain/Validation` 文本映射」方案不一致。需在实现期将 §6.4 对齐到本章方案（以本章为准）：`AppError` 的 `Domain` / `Validation` 变体改为承载稳定字符串，mapper 负责 `DomainError` 的分类映射。

5. §2  §2（impl 期决策）`TemplateExplainer` 的模板库覆盖度与多语言（i18n）文案：MVP 先内置中文模板；英文 / 其他语言模板是否随 vue-i18n 接入而定，impl 期决定补全范围与模板资源（放 `ai-engine/explain.rs` 还是 `frontend` 本地化文件）。

6. §2  §2（impl 期决策）`DomainError → AppError` 映射时，哪些领域错误需要把结构化字段（如 `shortfall_pd`、`missing` 技能列表）以 DTO 友好的 detail 透传前端做交互提示，哪些仅需 `to_string()` 文本——impl 期据前端实际交互需求逐类确认。

7. §2  §2（impl 期决策）HiGHS 静态链接在各目标平台（macOS arm64 / x86_64、Windows MSVC、Linux glibc）的构建链与产物体积实测，确认 `cc` / C++ 工具链在 CI 构建机就绪。

8. §3  [impl 期决策] team_overrides 对「归属多个 team 的资源」的 effective 常数解析规则：当前约定按 role='lead' 优先、否则取最近 joined_at 的 team；落地时需确认该规则是否满足产品预期，或是否需要在 team_members 增加显式 is_primary 标志。

9. §3  [impl 期决策] resource_project_rates 同一 (resource, project) 多周期行的区间不相交校验：当前约定由应用层（rate service / AllocationService）写入时预检；是否额外用 SQLite 触发器或 EXCLUDE-类机制做硬性防重叠，待实现期评估。

10. §3  [impl 期决策] 硬约束 1（逐日 Σ percent ≤ 1.0）与硬约束 4（技能）因需跨行/跨表聚合，仍依赖 AllocationService::create 预检，触发器层不强制；是否需要在 workload_snapshot 快照表层面补充一致性校验，待 §4.7 快照落地后决定（延续原开放问题 #17）。

11. §3  [impl 期决策] 若未来确需 UUID 作 run_id，须同步回改 §3.3.15/§3.3.16 全链路类型（INTEGER→TEXT）；当前 INTEGER 自增口径已确认即最终口径，本项仅作未来预案保留（延续原开放问题 #25）。

12. §4  （impl 期量化）workload_cache 写放大实测：§4.7.3 的 subject × period 增量重算方案已保留为最终方向，但需在 impl 期量化 allocation 区间变更（旧/新区间并集）与 holiday 变更（全项目×全资源）两类事件的实测写放大行数，据此决定 holiday 变更是否走异步批量重算；量化结果回填为「写放大基线表」。此为本轮决策#1 落地尾项，不再是方案选型级开放问题。

13. §4  （跨节，§9 范围）利用率口径单一真相源的执行校验：§9 假设#65 已拍板 Dashboard/看板/报表三处强制复用 §4.9 聚合函数，本章公式已对齐；落地时需在 §7 Dashboard / §8 报表实现中校验三处未各自另立公式（属 §7/§8 范围）。

14. §4  [本轮已解决，自追踪移除] 原开放问题#19 跨时区/跨地区日历：维持非目标，未来启用需重大版本升级（§4.2.1 四步变更清单已落定）。

15. §4  [本轮已解决，自追踪移除] 原开放问题#20 allocation_daily 是否进 MVP：标为可选加速路径，impl 期在 Dashboard 实测读延迟后决定；不引入则统一走 §4.9 精算（§4.4.1 已标注）。

16. §4  [本轮已解决，自追踪移除] 原开放问题#21 config_hash 哈希输入规范化：impl 期锁定纳入字段/序列化顺序/是否含项目级 override（§4.7.2 已标注）。

17. §5  [impl 期决策] input_snapshot_json 向量体积方案定稿：在 §1.6 验收规模（10/50）与 §2.4 示例规模（50/200）下实测「向量内容哈希 + 向量旁表（A）」vs「压缩序列化（B）」的 run 行体积、列表查询延迟、重放开销，选定默认（原开放问题 #24）。

18. §5  [impl 期决策] max_parallel_tasks_per_day 是否需要支持「按项目/按资源」细粒度配置（当前是全局 ConstraintFlags 单值，开放问题 #26 已确认「按项目和资源配置」）——需在 ConstraintFlags 引入 per-project / per-resource 覆盖结构并同步 §3.8。

19. §5  [impl 期决策] 投入比例最小步进 ε（耦合约束 percent ≥ ε·x 的下界）取值：与开放问题 #50（allocation 投入比例粒度：小数 vs 5% 步进）联动，落地时锁定 ε（如 0.05）。

20. §5  [impl 期决策] ObjectiveWeights 在 UI 的权重归一化策略：是否强制归一化（和为 1）还是允许自由绝对值，以及预设模式的默认权重组合需产品确认。

21. §5  [跨章，属第 6 章范围] §6.5 的 with_tx 封装（pool: &PgPool 笔误、普通 pool.begin() 无 IMMEDIATE/重试）需在修订第 6 章时统一改为 db::with_write_tx（原开放问题 #22）；本章已在 §5.7/§5.9/§5.10 加交叉一致性提示。

22. §5  [跨章，属第 2 章范围] §2.7 的 Cargo 片段若也写了 good_lp 的 default-features/features，需确认与本章一致的 highs 配置（原开放问题 #23）。

23. §6  [impl 期决策] db::with_write_tx 的 BEGIN IMMEDIATE 在 sqlx 已 begin() 的连接上存在事务状态冲突。生产实现需在 ① SqliteConnectOptions 把写路径配为 IMMEDIATE、② 自行 acquire() 连接 + 手动 BEGIN IMMEDIATE ... COMMIT（不走 sqlx Transaction 自动 BEGIN）之间二选一，并在 impl 期验证 is_busy()/消息匹配的跨 sqlx 版本稳定性。

24. §6  [impl 期决策] 口令派生的 Argon2id 参数（m/t/p，建议起点 m=64MiB, t=3, p=4）与「机器绑定 salt」「app pepper」的具体来源（如机器指纹哈希 vs 安装时随机落 app 私有目录的 salt 文件）需在 impl 期定稿并加单测；需评估低配机器（如旧 Mac）上派生耗时是否影响首启/解锁体验。

25. §6  [impl 期决策] 首启口令引导与运行期解锁的 UX 落点：是否提供「记住口令（Keychain 中存派生密钥，下次自动解锁）」选项以减少重复输入，以及锁屏/切后台多久后强制重新解锁。需与 §7 前端 UI 设计对齐。

26. §7  [impl 期决策] `alert.severity_order` 配置 UI 的具体形态（拖拽重排 vs 下拉排序）与 `desktopNotifyMinSeverity` 的档位命名（`high/critical` vs 数字 1-3），待 §9「设置」页原型落地时定稿。

27. §7  [impl 期决策] What-if 多场景叠加在 Gantt 上的色相分配策略（固定调色板 vs HSL 等距），以及重叠条带加深阈值的视觉调参，需在 P3 阶段做可读性实测。

28. §7  [impl 期决策] 桌面通知在应用处于前台 vs 后台时的策略差异（前台是否仍弹系统通知，还是仅更新铃铛角标），需产品确认默认行为。

29. §7  [impl 期决策] `ai_optimization_runs.trigger='handover'` 记录在 UI 的展示位置（资源中心操作历史 vs AI 面板历史运行 vs 两处都展示），待 P3 两视图落地后定稿。

30. §7  [跨节] `ai_optimization_runs` 新增 `trigger` 列、`projects.budget_alert_threshold` 列、`app_config` 新增三项预警/Gantt 配置键的 schema 迁移脚本，归属 §3.3 / §3.x 迁移章范围，需在修订第 3 章时落定并补交叉引用。

31. §8  **O8.1（impl 期决策）**：`resource_project_rates` 是否需要暴露 UI 编辑入口，还是仅作为「数据导入/管理员脚本」维护？若需 UI，需在「资源详情 → 项目费率」页增加 CRUD（与 §6.2 资源命令组对齐）。

32. §8  **O8.2（impl 期决策）**：快照大 payload 外迁采用「独立文件 `.json.gz`」还是「`workforce_snapshot_blobs` 分表」？前者利于备份迁移，后者利于事务一致性（见 §8.9a）。

33. §8  **O8.3（impl 期决策）**：`SNAPSHOT_INLINE_MAX_BYTES = 1 MB` 是否暴露到用户 UI 让其调整，还是作为隐藏常量？

34. §8  **O8.4（impl 期决策）**：Excel pivot table 在 `rust_xlsxwriter` 不同版本间的字段绑定 API 是否稳定？需在 Phase 5 起步时锁定 `rust_xlsxwriter` 版本并验证 pivot/chart 在 Excel/LibreOffice/WPS 中的兼容性。

35. §8  **O8.5（impl 期决策）**：HTML→PDF 开启时 Chromium 的获取方式——引导用户系统已装的 Chrome/Edge 路径，还是应用内下载独立 Chromium？前者体积友好但路径探测复杂，后者简单但 +150MB。

36. §8  **O8.6（待验证）**：`effective_daily_rate` 在「跨周期 allocation（一个 allocation 跨越多个 `resource_project_rates` 周期记录）」时，是否需要按日拆分加权计算成本？当前公式按 allocation 的代表日期 `d` 解析单价，跨周期场景的精度需在 R7 实测时验证（若误差可接受则保持现状，否则改为按日加权）。

37. §8  **O8.7（impl 期决策）**：R4 报表的 `explanation_md` 是否随 JSON 导出？当前设计含该字段（属 LLM 输出的精炼文本，非 prompt/response 原文），但若团队认为任何 LLM 文本都不应入报表，可在导出时剔除该字段，仅保留结构化 score/constraints/weights。

38. §9  DB 加密口令恢复机制细节(决策#10 衍生)：KDF 选型(Argon2id/PBKDF2)、多设备/多口令、恢复密钥导出格式 —— impl 期决策

39. §9  求解器软约束罚值标定(决策#6 衍生)：overload_soft_penalty 默认量级与 nice-to-have 技能权重 skill_match_weight 默认取值，需在 golden 用例调参 —— impl 期决策

40. §9  长期任务分段算法(决策#7 衍生)：超 long_task_span_limit 的拆段策略(按周等分/按里程碑/按阶段依赖)与依赖箭头处理 —— impl 期决策

41. §9  PM 换算常数 N 在团队/组织级(而非全局/项目级)可配置(对应开放问题汇总#2) —— 待后续需求确认

42. §9  利用率阈值按角色/团队分别设定(对应开放问题汇总#3) —— 待后续需求确认

43. §9  依赖环检测边界(§7.5.3 应用层环检测已落地)：复杂 DAG 环检测告警粒度与自动断环建议 —— impl 期决策

44. §9  跨设备自动云同步(本期非目标)：后续是否引入端到端加密云同步 —— 待产品规划

45. §9  第三语言扩展(决策#9 衍生)：键结构已预留，后续是否加入日文等 —— 待市场反馈

46. §9  报表公司模板(决策#8 衍生)：本期不套模板，后续是否提供可插拔能力 —— 待市场/客户反馈

47. §9  DB 加密性能基准(决策#10 衍生)：SQLCipher 在 5000 allocation 规模下冷启动与查询延迟实测是否满足 §9.2.1 —— impl 期验证


---

## 用户决策记录（2026-06-27，58 项已拍板）

> 以下为用户对本设计 58 项开放问题的决策，已逐章落地到正文（字段/表/算法/UI/路线图）。


### §1 用户决策落地
1. 产品正式命名改为 "Development Resource Kanban"（全称），可保留 DevResource Kanban 作简称；更新本章产品定位/名称。
2. PD/PM 换算常数（8h/PD、N PD/PM）支持团队/组织级覆盖（非仅全局）。
3. 利用率红绿灯阈值（默认 70%/100%）支持按角色/团队分别设定。
4. 明确 MVP 规模上限作为验收边界：资源≤10、任务≤50、项目≤5；写入验收信号。
5. AI 优化支持多目标权重（均衡负载 / 技能最优 / 预算 等）由用户在 UI 调节。

### §2 用户决策落地
1. 二进制体积无上限 → 静态链接 HiGHS（不走子进程）；更新打包说明。
2. §2.7 Cargo 的 good_lp 配置对齐 §5.5.2（启用 highs feature），删除不一致。
3. §2.5 DomainError 不派生 serde；改为映射到 AppError::Validation/Domain（领域层零 serde 依赖），与 §6 一致。
4. 降级：无 LLM 时用基于规则的自然语言模板（TemplateExplainer）提供解释；本章架构提及。
5. 优化保持在 tokio 任务内（不下放独立 OS 进程）；N=20 固定、不做地区动态计算。

### §3 用户决策落地
1. 新增 settings.secret_store 列（标识密钥存储后端）。
2. resources.daily_rate_pd 列在 MVP 即补入 schema（使成本核算可前移）。
3. 新增 resource_project_rates 表（resource×project 维度费率，支持按项目/周期浮动 daily_rate_pd）；projects.budget_pd 保留，删除任何 budget_pm 表述。
4. work_week_template 全局唯一性：idx_wwt_global 改为基于常量的 UNIQUE（约束"全局仅一行"），而非基于 id。
5. max_parallel_tasks_per_day 支持"按项目"与"按资源"细粒度配置（projects/resources 覆盖列或子表）。
6. PD/PM 常数与利用率阈值支持团队级覆盖：新增 team_overrides（team 级 pd_hours/pm_workdays/thresholds）。
7. 确认 ai_optimization_runs（INTEGER PK）/ run_id i64 全链路一致；ai_run(UUID) 已废弃。

### §4 用户决策落地
1. §4.7.3 失效范围（按 subject×period 增量重算）方案保留；标注"写放大实测值待 impl 期量化（allocation 区间并集、holiday 全项目全资源）"。
2. config_hash 规范化（纳入字段/序列化顺序/是否含项目级 override）标注"impl 期锁定"。
3. allocation_daily 标注"可选加速路径，Dashboard 实测读延迟后决定是否进 MVP；不引入则统一走 §4.9 精算"。
4. 跨时区/跨地区日历维持非目标（§4.2.1）；若未来启用需重大版本升级（region/timezone 列、day_factor 按 region 解析、engine_rev 迁移）。

### §5 用户决策落地
1. MILP 单位闭合：落定 x(0/1) 与连续 percent 的耦合约束形式（y=Σx/有效工作日 或直接连续 percent_{r,t,d}，容量约束比例空间 Σ_t percent≤cap），与 §3.8 对齐。
2. ai_optimization_runs.input_snapshot_json 体积：采用"向量内容哈希 + 向量旁表"或"压缩序列化"，标注 impl 期评估。
3. run_id 维持 INTEGER 自增（不引入 UUID）。
4. 多目标权重 ObjectiveWeights 由用户在 UI 调节，写入目标函数与配置。
5. 降级链确认：LLM 不可用→TemplateExplainer（规则模板）；embedding 不可用→FallbackScorer（关键词/熟练度）。
6. 默认采用 good_lp + HiGHS 作 ILP 后端。

### §6 用户决策落地
1. §6.5 with_tx 统一为 db::with_write_tx(&SqlitePool, BEGIN IMMEDIATE + busy_timeout + 退避重试)；修正 PgPool 笔误为 SqlitePool。
2. DomainError 不派生 serde；IPC 边界映射为 AppError::Validation/Domain。
3. encrypted_file 主密钥来源：用户口令派生（option b，更强）；首启引导设置口令。
4. settings.secret_store 新增列（§3 迁移与 §6.2.9 settings_svc 对齐）。
5. keychain→encrypted_file 回切/迁移不进 MVP，列为最后实现项。

### §7 用户决策落地
1. 预算预警阈值 budget_alert_threshold 按项目细分（默认复用全局 overload_threshold）。
2. TrendExplainer 历史输入保留 4 周；脱敏模式同样作用于趋势 prompt。
3. 资源离职/调岗向导 mode=transfer_then_reoptimize 的 AI 重排记入 ai_optimization_runs 并标 trigger=handover；交接人过载策略与超载策略(可配)对齐。
4. 依赖影响链高亮限制渲染深度（仅直接后继 1-2 层），避免与虚拟滚动性能冲突。
5. 预警中心：四类预警严重度排序可配置；新增桌面通知（不做邮件/推送）。
6. What-if 支持多版本对比（同时叠加 2+ 场景）。
7. Dashboard 预警 dismiss 为会话内状态（不持久化）。
8. AI 面板新增多目标权重调节 UI（滑杆/开关组）。

### §8 用户决策落地
1. 字段对齐：resources.daily_rate_pd、projects.budget_pd；删除 budget_pm 表述。
2. 支持人天单价按项目/周期浮动（resource_project_rates 表）。
3. daily_rate_pd 在 MVP 即补入 schema，使 R7 成本估算可前移。
4. R4 AI 决策记录仅保存结构化约束+打分摘要（不存 LLM prompt/response 原文）。
5. HTML→PDF（headless_chrome）作为可选特性，用户显式开启。
6. 快照 payload >1MB 时从 JSON 列迁移到独立文件/分表存储。
7. Excel 导出支持数据透视表/图表（rust_xlsxwriter）。
8. 导出文件落盘到用户选定路径（不引入应用内文档库）。
9. 报表导出整体延后到最后阶段实现（不进核心 MVP）；首版优先级 CSV > Excel > PDF。

### §9 用户决策落地
1. 默认计算单位可配（PD/PM），允许项目级覆盖。
2. PM 换算常数 N 可自定义（默认 20）。
3. 本地 Ollama 默认模型清单：chat=qwen2.5:7b（备选 qwen2.5:14b / llama3.1:8b）、embed=nomic-embed-text（备选 bge-m3）；写入默认值。
4. 默认采用 good_lp + HiGHS（ILP 后端），静态链接。
5. allocation 投入比例粒度：小数 0.01（可配置步进）。
6. 资源超载策略可配（默认软警告、可选硬阻止）；求解器可将超载作软约束（带罚值可违反）。
7. 长期任务强制分段排期。
8. Phase 5 报表格式首版：CSV 优先、Excel 次之、PDF 可选；不套公司模板（本期）。
9. i18n 首版：zh-CN + en；日期/货币按 locale 处理。
10. DB 加密：首版默认开启（用户口令派生主密钥）。
11. 自动备份：频率/份数/目录可配置（默认每日/保留 7 份/用户目录）。
12. 技能熟练度等级（1-5）+ must-have/nice-to-have 区分 + AI 匹配权重可配。
13. 排期：Phase 3（Gantt）先于 Phase 4（AI）；What-if 叠加预览在 Phase 4 后置补齐，期间以纯 diff 表降级。
14. 更新 §9.4 开放问题 #12 为"已支持非均匀日容量（work_week_template 逐日 fraction）"。


### 释义待复核（我据简短回答做的推断，请确认）
- #45/11/53 报表导出：你答"是/否/最后实现"——我释为"报表导出延后到最后阶段，不进核心 MVP；实现时 CSV>Excel>PDF"。
- #50 投入比例粒度：你答"是"——我释为"默认小数 0.01，可配置步进（如 5%）"。
- #51 超载策略：你答"是"——我释为"默认软警告、可选硬阻止；求解器把超载当软约束（带罚值可违反）"。
- #47 PM 常数 N：你答"是"——我释为"可自定义，默认 20"。
- #55 DB 加密：你答"是"——我释为"首版默认开启、用户口令派生主密钥"。


---

## 修订记录

- **v1（初稿合并）**：9 章并行深写 → 合并。

- **v2（对抗式评审）**：3 路评审 36 条 findings（7 blocker/18 major/11 minor），按章修订；统一日历三表模型、容量比例口径 Σ percent≤1.0、ai_optimization_runs(INTEGER)、allocation_validate 触发器等。

- **v3（用户决策落地，本轮）**：用户拍板 58 项开放问题，9 章全量修订；新增 Development Resource Kanban 命名、团队级覆盖(team_overrides)、浮动费率(resource_project_rates)、daily_rate_pd 进 MVP、多目标权重 UI、db::with_write_tx、DomainError→AppError 映射、用户口令派生主密钥(SQLCipher 默认开启)、桌面通知、What-if 多版本对比、报表导出延后(CSV>Excel>PDF)等。


---

> **文档说明**：本设计经 多 agent 深写 → 合并 → 对抗式评审 → 用户决策落地 三轮迭代生成。术语统一 Allocation；存储口径统一 PD（PM 仅展示换算）。文末「释义待复核」「开放问题」为剩余讨论项。
