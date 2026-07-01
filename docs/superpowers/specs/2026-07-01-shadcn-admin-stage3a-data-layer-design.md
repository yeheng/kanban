# Spec — 阶段3a：数据层基建（services/api/ + ofetch + vue-query 激活）

> 套用 shadcn-vue-admin 模板到 kanban 前端的**第 3 个子项目的第 1 段**（3a）。
> 依赖阶段0+1（PR #1）+ 阶段2（PR #2，已合入 main）。ofetch + vue-query 已在阶段0+1 装好并注册，但**完全没在用**。
>
> 决策已确认：**全量迁移**（最终删 13 store + refresh 总线）+ **分 3a/3b/3c 渐进**。本 spec 只覆盖 **3a（基建，双轨期开始）**；3b（逐域迁 page）/ 3c（删 store + refresh）等 3a 落地后基于真实代码再设计。

## 1. 背景与定位

当前数据层：单文件 `src/api/index.ts`（251 行、55 方法、原生 `fetch` + 手写 `request<T>()`）+ 13 个 Pinia store（薄包 api）+ 自研 `stores/refresh.ts` 版本号总线（6 scope，view `watchEffect` 读 version 触发 reload）。**无查询缓存层**。

模板用的是 `services/api/<domain>.api.ts`（按域拆分）+ `useApiFetch()`（ofetch）+ TanStack Vue Query（`useQuery`/`useMutation` + `invalidateQueries`）。

3a 建立**与旧并存的全新数据层骨架**：
- `services/api/*.api.ts` 11 个域文件（vue-query composable）
- `services/fetch.ts`（ofetch 客户端）
- `services/types/response.type.ts`（响应封装类型）
- QueryClient 配置调优

**不删任何 store、不改任何 page**。完成后新旧并存：旧 `api/index.ts` + 13 store 继续驱动 app；新 `services/api/` 就位，供 3b 逐域切换。

## 2. 目标

阶段结束后：
- `src/services/api/` 11 个域文件就位，每个导出 `useXxxQuery()` / `useXxxMutation()` composable；
- `src/services/fetch.ts` 提供 `useApiFetch()`（ofetch，baseURL 来自 `@/utils/env`）；
- `src/services/types/response.type.ts` 提供响应封装类型；
- QueryClient 配置调优（`refetchOnWindowFocus: false`、`retry: 1`）；
- 全部新文件类型检查通过，可被 import（但本期无消费者——3b 才接入 page）。

**应用行为零变化**：旧 `api/index.ts` + 13 store + refresh 总线全部保留不动，app 仍走旧路径。

## 3. 非目标（推迟）

- **任何 page 改动**（3b）：14 page 仍用 `useXxxStore()`。
- **删 store / 删 refresh 总线**（3c）：13 store 与 `stores/refresh.ts` 原样保留。
- **snake→camel 转换**：`types.ts` 不改，page 不改。本期新 composable 的**返回类型沿用 snake_case**（与现有 types.ts 一致），仅**入参**用 camel（与现有 store 方法签名一致），序列化时在 api 方法内转 snake（与现有 `api/index.ts` 做法一致）。全量 camel 化推迟到 3b/3c 或阶段6 决定。
- **删旧 `api/index.ts`**（3c）：本期保留。
- **auth 拦截器**：kanban 无 auth，`onRequest` 留空。

## 4. 详细设计

### 4.1 依赖与 QueryClient 调优

ofetch + `@tanstack/vue-query` 已装（阶段0+1）。无需新依赖。

修改 `src/plugins/tanstack-vue-query/setup.ts`，调优默认配置（桌面 app 不需要 window-focus 重取；retry 1 次够）：
```ts
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});
```
（其余不变：`VueQueryPlugin` 注册、devtools 注释保留。）

### 4.2 services/fetch.ts（ofetch 客户端）

```ts
import { ofetch } from "ofetch";
import env from "@/utils/env";

/**
 * ofetch 客户端。baseURL 来自 @/utils/env（相对路径，dev 走 vite proxy /api）。
 * 拦截器：onRequest 预留 auth header（kanban 无 auth，本期空）；
 * onResponseError 统一打日志。
 */
export function useApiFetch() {
  return {
    apiFetch: ofetch.create({
      baseURL: env.VITE_API_BASE,
      onRequest(_ctx) {
        // 预留：auth header 注入（kanban 当前无 auth）
      },
      onResponseError(_ctx) {
        console.error("[api] request failed:", _ctx.error);
      },
    }),
  };
}
```

关键点：
- `baseURL` 用 `env.VITE_API_BASE`（阶段0+1 已建的 `@/utils/env`，本期首次真正消费它）。值为相对路径（dev 下空串 → 同源 → vite proxy `/api` → `localhost:3000`）。
- ofetch 自动 JSON 编解码、204 处理、错误抛出（替代手写 `request<T>()`）。
- `onResponseError` 的回调接收 ofetch 的 `FetchContext`（含 `.error`、`.response`、`.request` 等），与模板 `use-fetch.ts` 的签名一致（`onResponseError: (_ctx) => {}`）。

### 4.3 services/types/response.type.ts

```ts
/**
 * 后端响应封装类型（对齐模板）。
 * kanban 后端目前直返裸数据（如 Project[]，无 { data, code, message } 包裹），
 * 故本期 IResponse 为可选工具类型，composable 暂不强制用它包裹。
 * 将来后端统一响应格式时启用。
 */
export interface IResponse<T, E = Record<string, unknown>> {
  data: T;
  extra: E;
  code: number;
  message: string;
  success: boolean;
}

export interface IPaginationRequestQuery {
  page?: number;
  pageSize?: number;
}

export type IRequestQuery<T extends Record<string, unknown>> = {
  page?: number;
  pageSize?: number;
} & {
  [K in keyof T]?: T[K];
};
```

### 4.4 services/api/*.api.ts（11 个域文件）

每个文件按域导出 `useXxxQuery()` / `useXxxMutation()` composable。**55 方法全量迁移**（与旧 `api/index.ts` 一一对应，保证 3b 切换时无遗漏）。

**域划分**（对应后端路由组 + 现有 api 方法聚类）：

| 文件 | 方法（来自旧 api/index.ts） |
|---|---|
| `projects.api.ts` | listProjects, createProject, updateProject, setProjectStatus, deleteProject |
| `catalog.api.ts` | listSkills, ensureSkill, listTags, ensureTag |
| `tasks.api.ts` | createTask, updateTask, deleteTask, setTaskStatus, addDependency, kanbanTasks, listTasks |
| `resources.api.ts` | listResources, createResource, updateResource, deleteResource, getResourceSkills, setResourceSkills, getResourceTags, setResourceTags |
| `workload.api.ts` | resourceSummary, teamSummary, overloads, projectBurn |
| `config.api.ts` | getThresholds, getUnitConfig, getSettings, updateSettings |
| `allocations.api.ts` | createAllocation, deleteAllocation, listAllocations, updateAllocation |
| `calendar.api.ts` | setGlobalWorkWeek, listWorkWeeks, addHoliday, listHolidays, addTimeOff, listTimeOff, deleteHoliday, deleteTimeOff |
| `teams.api.ts` | listTeams, createTeam, deleteTeam, listTeamMembers, addTeamMember, removeTeamMember, setTeamOverride, getTeamOverride |
| `gantt.api.ts` | ganttProject, ganttResource, dependenciesForProject, dailyOccupancy |
| `optimization.api.ts` | runOptimization, listOptimizationRuns, applySolution, rejectSolution |

**`projects.api.ts`**（完整示例，其余同构）：
```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/vue-query";
import { useApiFetch } from "../fetch";
import type { Project } from "@/types";

export function useListProjectsQuery() {
  const { apiFetch } = useApiFetch();
  return useQuery<Project[]>({
    queryKey: ["projects"],
    queryFn: () => apiFetch<Project[]>("/api/projects"),
  });
}

export function useCreateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<number, Error, { name: string; priority: number; budgetPd: number }>({
    mutationFn: (args) =>
      apiFetch<number>("/api/projects", {
        method: "POST",
        body: { name: args.name, priority: args.priority, budget_pd: args.budgetPd },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useUpdateProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, {
    id: number;
    name: string; priority: number; budgetPd: number;
    description?: string | null; start?: string | null; end?: string | null;
  }>({
    mutationFn: (args) =>
      apiFetch<void>(`/api/projects/${args.id}`, {
        method: "PATCH",
        body: {
          name: args.name, description: args.description ?? null,
          start: args.start ?? null, end: args.end ?? null,
          priority: args.priority, budget_pd: args.budgetPd,
        },
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useSetProjectStatusMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: number; status: string }>({
    mutationFn: (args) => apiFetch<void>(`/api/projects/${args.id}/status`, { method: "POST", body: { status: args.status } }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}

export function useDeleteProjectMutation() {
  const { apiFetch } = useApiFetch();
  const queryClient = useQueryClient();
  return useMutation<void, Error, number>({
    mutationFn: (id) => apiFetch<void>(`/api/projects/${id}`, { method: "DELETE" }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}
```

**约定**（所有 11 文件遵循）：
- **queryKey**：`["<domain>"]`（列表）或 `["<domain>", id]`（详情）。mutation 的 `onSuccess` 用 `invalidateQueries({ queryKey: ["<domain>"] })` 失效该域所有查询。这**取代 refresh 总线的 scope 机制**——3c 删 refresh 后，跨域失效靠一个 mutation 的 onSuccess 调多个 invalidate（如 allocation 写入失效 allocations+workload+gantt+kanban+calendar，对应旧 `bump("allocations","workload","gantt","kanban","calendar")`）。
- **snake_case 边界**：返回类型沿用 `types.ts` 的 snake（如 `Project.budget_pd`）；入参 camel（如 `budgetPd`），body 内转 snake（与旧 api 一致）。
- **路径**：与旧 api 完全一致（`/api/projects`、`/api/projects/${id}` 等），保证行为零差异。
- **HTTP 方法**：ofetch 用 `method` 字段（`"GET"` 默认可省略，但显式写更清晰——mutation 显式写，query 省略）。

### 4.5 旧 api/index.ts + 13 store + refresh 总线

**全部保留不动**。本期是纯增量。新 `services/api/` 与旧并存（双轨）。3b 迁 page 时逐步切到新 composable，3c 删旧。

## 5. 验证标准（Definition of Done）

1. `pnpm build` 通过（vue-tsc -b + vite build）。
2. `pnpm test` 16/16 通过（旧测试不动；新 composable 可加 1-2 个轻量单测验证 queryKey/路径，但非必须——它们本期无消费者）。
3. `pnpm dev` app 行为零变化（仍走旧 store/api）。
4. `src/services/api/*.api.ts` 11 文件就位，`useApiFetch` + `IResponse` 可 import，类型检查通过。
5. QueryClient 配置含 `refetchOnWindowFocus: false`、`retry: 1`。
6. `@/utils/env` 首次被真正消费（services/fetch.ts）。

## 6. 风险与对策

| 风险 | 对策 |
|---|---|
| ofetch baseURL 为相对路径时行为与原生 fetch 不一致 | ofetch 支持相对 baseURL（空串=同源）；dev 下同源 → vite proxy。验证 dev 起服务后请求能打到 localhost:3000 |
| 55 方法逐一迁移易遗漏/签名走样 | 以旧 `api/index.ts` 为权威，逐方法对照路径/body/方法；build+typecheck 会捕获类型偏差 |
| vue-query composable 在无消费者时被 tree-shake 掉 | 不影响正确性（3b 接入后自然保留）；本期只要求类型检查通过 |
| queryKey 约定与 3b 实际失效需求不匹配 | 3b 迁移时会暴露跨域失效需求；3a 建立 `["<domain>"]` 基础约定，3b 按需细化（如 `["allocations", projectId]`）|

## 7. 不在本 spec 解决的问题（显式记录）

- page 数据消费改造（store → composable）—— 3b。
- 删 store + refresh 总线 + 旧 api/index.ts —— 3c。
- snake→camel 全量转换 —— 3b/3c 或阶段6。
- auth 拦截器、token 注入 —— kanban 无 auth，将来需要时加。
- 错误 toast 统一处理（onResponseError 只打日志，不弹 toast）—— 可在 3b 接 page 时加全局错误边界。
