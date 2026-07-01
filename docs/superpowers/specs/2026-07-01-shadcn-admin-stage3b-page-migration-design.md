# Spec — 阶段3b：逐域迁 page（store → composable）

> 套用 shadcn-vue-admin 模板到 kanban 前端的**第 3 个子项目的第 2 段**（3b）。
> 依赖阶段3a（PR #3，已合入 main）：`services/api/` 12 域 composable + ofetch + vue-query 已就位。
>
> 决策已确认：**保留 snake_case**（types.ts/template 字段读取不改，只换数据获取方式）；**保留瘦 store**（projects.current/unit 退化为纯 UI 状态容器，数据查询走 vue-query）；其余 11 个数据 store + refresh 总线在 **3c 删除**（本期保留不删，双轨收尾）。

## 1. 背景与定位

3a 建好了全新 vue-query 数据层（`services/api/`），但 **13 page + 12 component 仍走旧 Pinia store**。3b 把每个 page 的数据获取从 `useXxxStore()` 迁到 `useXxxQuery()`/`useXxxMutation()`，同时：
- `projects.current` / `unit` 退化为瘦 store（保留，纯 UI 状态）；
- `AppLayout` 的 bootstrap 手写重试循环改为 vue-query 自动重试；
- 删每个 page 里 watch refresh bus 的逻辑（vue-query 的 invalidateQueries 自动刷新，3a 已配好）。

**这是改动面最大的执行阶段**：触及 13 page + 12 component 的数据消费层。但**不改 types.ts、不改 template 字段读取（snake_case 透传）、不改 page 的 UX 逻辑**——只换数据从哪来。

## 2. 目标

阶段结束后：
- 13 page + 12 component 的数据获取走 `services/api/` composable（不再调 store 的 load/items/create 等）；
- `useProjectsStore` 瘦身为 `current` + `select`；`useUnitStore` 瘦身为 `unit` + `formatPd` + `applyTeamOverride`；
- `AppLayout` bootstrap 走 vue-query（删手写重试循环）；
- 所有 watch refresh bus 逻辑删除；
- 11 个数据 store + refresh store 变成无消费者（3c 删除，本期保留）。

**应用行为零变化**：UI/交互/字段读取全不变，只是数据获取路径换了。每个 page 迁完后 build+test 全绿。

## 3. 非目标（推迟）

- **snake→camel 全量转换**：types.ts 不改，template 的 `p.budget_pd` 不改。推迟到阶段6 或专项。
- **删 11 个数据 store + refresh store + 旧 api/index.ts**：3c。本期保留（双轨收尾）。
- **page 内部 UX 重写**（BasicPage / data-table / zod validators / vee-validate）：阶段6。本期 page 的 UI 结构/组件用法不动。
- **i18n 抽取**：阶段6。
- **`projects.current` / `unit` 抽成 composable**（彻底删 store）：3c 或更后。本期保留瘦 store 形态。

## 4. 详细设计

### 4.1 迁移单元与顺序（4 批，每批可独立验证）

按**依赖深度从浅到深**迁移。每个域一个 commit，每批结束 build+test 全绿。

| 批次 | page/component | 迁移内容 | 难点 |
|---|---|---|---|
| **1（无依赖）** | catalog, settings | catalog: useListSkills/Tags + ensure mutations；settings: useGetSettings/useUpdateSettings | settings 有 ~45 snake v-model（**不动**，只换 load/save） |
| **2（依赖 projects.current）** | projects, resources, kanban, TaskForm, ProjectForm, ResourceForm, TaskCard | 各自 query/mutation；projects.current 走瘦 store | kanban 读 tasks.columns（store 派生）→ 需在 page 内计算 |
| **3（依赖 refresh bus）** | allocations, dashboard, gantt, calendar, calendar-grid, AllocationForm, GanttTimeline, HolidayList, TimeOffList, WorkWeekEditor, OccupancyGrid | 删 watch refreshBus，靠 invalidateQueries 自动刷新 | dashboard 6 store 交织最复杂 |
| **4（特殊）** | teams, ai, reports, PlanReview, WeightsPanel, AppLayout | teams CRUD；ai optimization；reports 删直接 api import；AppLayout bootstrap 改 vue-query | AppLayout bootstrap 改造影响全局 |

**重要**：每个 page 迁移时，它依赖的 component 必须一并迁（否则编译失败——component 读 store 而 page 不再传 store 数据）。component 迁移随其宿主 page 归入对应批次。

**灵活度**：若某 page 过于复杂（如 dashboard），可拆为 3b.1 / 3b.2 子提交。不强求一个 PR 装完所有 page——若 PR 过大，可拆多个 PR（如 3b-批次1+2 一个 PR，3b-批次3+4 一个 PR）。本 spec 覆盖全部 4 批，实现时按需拆 PR。

### 4.2 瘦 store 改造（批次 2 / 4）

**`useProjectsStore`**（瘦身后）：
```ts
import { defineStore } from "pinia";
import { ref } from "vue";

export const useProjectsStore = defineStore("projects", () => {
  const current = ref<number | null>(null);

  function select(id: number) {
    current.value = id;
  }

  return { current, select };
});
```
- 删 `load()`、`items`、内部 `api` import。
- `current` 的初始化**不在 store 内做**——vue-query 的 `useQueryClient`/`useQuery` 依赖 `hasInjectionContext()`（vue-query 源码确认：在无注入上下文时 throw）。pinia setup store 在组件 setup 外实例化，**不能调 useQuery**。故 current 的"首次有数据自动选第一个"逻辑移到 **AppLayout**（它是组件，有注入上下文）：AppLayout 内 `watch(projectsQuery.data, items => { if (projects.current == null && items?.length) projects.select(items[0].id) })`。见 §4.3。

**`useUnitStore`**（瘦身后）：
```ts
import { defineStore } from "pinia";
import { ref } from "vue";

export const useUnitStore = defineStore("unit", () => {
  const unit = ref<"PD" | "PM">("PD");

  function formatPd(pd: number | null | undefined): string {
    if (pd == null) return "—";
    return unit.value === "PM" ? (pd / 20).toFixed(1) : pd.toFixed(1);
  }

  function applyTeamOverride(pmWorkdays: number | null): number {
    return pmWorkdays ?? 20;
  }

  return { unit, formatPd, applyTeamOverride };
});
```
- 删 `loadGlobal()`、`pd_hours`/`pm_workdays` 读取、内部 `api` import。
- `formatPd` 的 `20` 硬编码：原 store 从 `unit.loadGlobal()` 读 `pm_workdays`。瘦身后该数据走 `useGetUnitConfigQuery()`——但 `formatPd` 是纯 UI 格式化，**本期保留硬编码 20**（与现状 `applyTeamOverride` 默认值一致），3c 或阶段6 再接 `useGetUnitConfigQuery` 的真实值。这是**已知简化**，记录在风险表。

### 4.3 AppLayout bootstrap 改造（批次 4）

当前（`src/layouts/default.vue`）：
```ts
onMounted(async () => {
  for (let i = 0; i < 40; i++) {
    try {
      await projects.load();
      await catalog.load();
      await unit.loadGlobal();
      ready.value = true;
      return;
    } catch { await new Promise(r => setTimeout(r, 100)); }
  }
});
```
改为：
```ts
const projectsQuery = useListProjectsQuery();
const skillsQuery = useListSkillsQuery();
const tagsQuery = useListTagsQuery();
const unitConfigQuery = useGetUnitConfigQuery();

const ready = computed(() =>
  projectsQuery.isSuccess && skillsQuery.isSuccess && tagsQuery.isSuccess && unitConfigQuery.isSuccess
);

// projects.current 初始化（从瘦 store §4.2 移到此——store 不能调 useQuery）
watch(() => projectsQuery.data.value, (items) => {
  if (projects.current == null && items && items.length > 0) {
    projects.select(items[0].id);
  }
});
```
- vue-query 自动重试（`retry: 1`，若 bootstrap 需更激进可临时调高）。
- 删 `onMounted` 重试循环 + 手写 `ready` ref。
- `projects.current` 初始化：AppLayout watch `projectsQuery.data`，首次有数据时选第一个（替代旧 bootstrap 的隐式初始化）。
- 顶栏 project `<Select>` 的 options 改读 `projectsQuery.data.value`（不再 `projects.items`）。
- Skeleton 占位保留（`v-if="!ready"`）。

### 4.4 refresh bus 处理（批次 3）

每个 watch refreshBus 的 page（allocations/dashboard/gantt/kanban/calendar-grid）：
- **删除** `import { useRefreshStore }` + `watch/watchEffect(() => refreshBus.version.X)`。
- 数据查询改用对应 `useXxxQuery()`——vue-query 在 mutation onSuccess 调 `invalidateQueries`（3a 已配）时自动 refetch，无需手动 watch。
- 例：allocations 页当前 `watchEffect` 读 `refreshBus.version.allocations` 触发 `allocations.load(projects.current)` → 改为 `useListAllocationsQuery(projects.current)`，allocations mutation 的 onSuccess 已 invalidate `["allocations"]`，自动刷新。

refresh store 本期保留（3c 删），但迁完所有 watcher 后无消费者。

### 4.5 直接 api import 处理（批次 3/4）

3 个文件直接 import `{ api } from "@/api"`：
- `calendar-grid/index.vue` → `api.dailyOccupancy` → 改 `useDailyOccupancyQuery`（但该 query 需 start/end 参数，用 `enabled` 控制何时查）。
- `reports/index.vue` → `api.getReportCatalog` + `api.exportReport/Snapshot` → 改 `useGetReportCatalogQuery` + `exportReport(apiFetch, ...)` helper（3a 已建，apiFetch 从 `useApiFetch()` 取）。同时 `reportKinds/ReportKind/ReportCatalogEntry` 改从 `@/services/api/reports.api` import。
- `AllocationForm.vue` → `api.listTasks` + `api.resourceSummary` → 改 `useListTasksQuery` + `useResourceSummaryQuery`。

### 4.6 snake_case 边界（本期不变）

- `types.ts`：不改。
- template 字段读取：`p.budget_pd`、`r.available_from`、`settings.pd_hours` 等**全部保留 snake**。
- composable 返回类型：沿用 snake（3a 已定）。
- mutation 入参：camel（3a 已定），body 内转 snake。
- **settings 的 ~45 个 v-model**：`v-model="draft.pd_hours"` 等**全部不动**——只把 `settings.load()` → `useGetSettingsQuery`、`settings.save(draft)` → `useUpdateSettingsMutation`。

### 4.7 派生数据（store 内计算逻辑）迁移

某些 store 有派生逻辑（非纯数据），迁 page 时需在 page 内复现：
- `useTasksStore.columns` / `byStatus(status)`：kanban 列分组逻辑 → 在 kanban page 内用 `computed` 从 `useKanbanTasksQuery().data` 计算。
- `useWorkloadStore.band(utilization)`：dashboard 颜色带 → 在 dashboard 或 OccupancyGrid 内 computed。
- `useUnitStore.formatPd`：保留在瘦 store（§4.2）。

每个迁 page 时识别其依赖的派生逻辑，搬到 page 的 `<script setup>`。

## 5. 验证标准（Definition of Done）

1. `pnpm build` 通过。
2. `pnpm test` 通过（旧 store 测试在 3c 删；本期若 store 仍存在则测试仍跑）。
3. `pnpm dev` 13 page 全部可达，行为零变化（UI/交互/数据一致）。
4. 13 page + 12 component 不再 import 任何数据 store 的 load/items/create（除瘦 projects/unit store 的 current/select/unit/formatPd）。
5. 不再有 `import { api } from "@/api"`（3 个直接 importer 已迁）。
6. 不再有 `import { useRefreshStore }` 的 watcher（5 个 watcher 已删）。
7. AppLayout bootstrap 走 vue-query（无手写重试循环）。
8. `grep -r "useAllocationsStore\|useCatalogStore\|useTasksStore\|useWorkloadStore\|useGanttStore\|useCalendarStore\|useOptimizationStore\|useTeamsStore\|useResourcesStore\|useSettingsStore" src/pages/ src/components/` → 仅剩 projects/unit 瘦 store 的合理用法（或为空，若 projects.current 也已迁）。

## 6. 风险与对策

| 风险 | 对策 |
|---|---|
| 瘦 store 内调 useQuery 会 throw（vue-query 依赖 hasInjectionContext） | **已解决**：current 初始化移到 AppLayout（组件有注入上下文），store 只留 current ref + select |
| settings 的 v-model 双向绑定与 useGetSettingsQuery 的只读 data 冲突 | page 内保留 `draft` ref，`watch(settingsQuery.data)` 种子 draft，save 时调 mutation（与旧 store 的 settings.settings→draft 模式一致） |
| dashboard 6 store 交织，迁移易错 | 单独作为批次3 的最后一个，仔细对照；必要时拆 3b.2 |
| component 随 page 迁移遗漏 | 每个 page 迁移时 grep 其 import 的所有 component，确认 component 也迁 |
| refresh bus 删除后某 page 数据不自动刷新 | 该 page 的 mutation onSuccess invalidate 必须覆盖（3a 已配，验证 invalidate key 匹配） |
| vue-query 的 enabled 参数（如 calendar-grid 的 occupancy 需 start/end） | 用 `enabled: !!start && !!end` 控制 |
| 瘦 store 的 formatPd 硬编码 20 与真实 pm_workdays 不符 | 已知简化，3c/阶段6 接 useGetUnitConfigQuery |

## 7. 不在本 spec 解决的问题（显式记录）

- snake→camel 全量转换 —— 阶段6 或专项。
- 删 11 数据 store + refresh store + 旧 api/index.ts —— 3c。
- projects.current/unit 抽成 composable —— 3c 或更后。
- page UX 重写（BasicPage/data-table/zod）—— 阶段6。
- formatPd 接真实 unit config —— 3c/阶段6。
