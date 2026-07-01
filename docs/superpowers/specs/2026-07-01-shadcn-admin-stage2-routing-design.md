# Spec — 阶段2：目录与路由迁移

> 套用 shadcn-vue-admin 模板到 kanban 前端的**第 2 个子项目**（7 阶段第 2 个）。
> 依赖阶段0+1（PR #1，`feat/shadcn-stage-0-1` 分支）已落地的基建：`src/plugins/router/setup.ts`、`@/utils/env`、auto-import、tsconfig references。
>
> 决策已确认：**保留 hash history**（web + Tauri prod 双安全）；**全套文件路由**（vue-router 内置的 `VueRouter` vite 插件 + vite-plugin-vue-layouts，对齐模板）。

## 1. 背景与定位

阶段0+1 装好了构建基建与插件化入口，但路由仍是 kanban 原有的手写扁平 `src/router.ts`（hash history，13 路由，3 eager + 11 lazy，无 guard/meta），页面散落在扁平的 `src/views/`。模板用的是**约定式文件路由**（`src/pages/<route>.vue` 自动生成路由）+ **布局插件**（layout 自动套用）。

阶段2 把路由体系与目录结构对齐模板：
- 手写 `router.ts` → 文件路由（vue-router 内置 `VueRouter` 插件按文件约定生成）+ layouts 插件
- `views/` 扁平 14 文件 → `pages/<feature>/index.vue`（每功能一文件夹，为阶段6 的 components/data/validators 预留位置）
- `AppLayout.vue` → `layouts/default.vue`
- 加 common guard 接通阶段0+1 配好但未用的 nprogress

**这是 7 阶段中改动面最大的一个**：会移动 14 个 view 文件、删除 router.ts、改 vite.config.ts、改 plugins/router/setup.ts、改 App.vue。

## 2. 目标

阶段结束后：
- 路由由 vue-router 内置的 `VueRouter` 插件按文件约定自动生成，`vite-plugin-vue-layouts` 自动套 `default` 布局；
- 13 路由 + `/`→`/kanban` 重定向全部正常；
- `pages/<feature>/index.vue` 目录结构就位；
- `layouts/default.vue` 接管 shell（功能与现 `AppLayout.vue` 一致）；
- common guard 接通 nprogress，路由切换有进度条；
- `src/types/route-map.d.ts` 生成入库；
- 旧 `src/router.ts` 与 `src/views/` 删除。

**应用行为零变化**：所有 page 内部代码不动（只移动文件），顶栏 project/unit 选择器、bootstrap 轮询、侧边栏 active 态、13 路由可达性全部与改造前一致。唯一新增可见行为是路由切换时出现 nprogress 进度条。

## 3. 非目标（推迟）

- **auth guard / 登录流**：kanban 无 auth，YAGNI。将来需要时再加（配 `meta.auth`）。
- **navItems 抽到 composable**（模板的 `useSidebar()`）：阶段5 随 shell 增强一起做。
- **theme toggle / command panel / 用户菜单 / team switcher**：阶段5。
- **i18n 抽 nav 标签**：阶段6。
- **page 内部重写**（BasicPage / data-table / zod validators / vee-validate）：阶段6。本期 page 内部代码**完全不动**，只移动文件位置 + 可能的组件名调整。

## 4. 详细设计

### 4.1 vite.config.ts 新增两个插件

在阶段0+1 的 vite.config.ts 基础上，`plugins` 数组**头部**插入两个（文件路由必须在 vue() 之前或紧邻）：

```ts
import VueRouter from 'vue-router/vite'           // 新增
import Layouts from 'vite-plugin-vue-layouts'      // 新增

plugins: [
  VueRouter({
    exclude: ['**/components/**', '**/layouts/**', '**/data/**', '**/types/**', '**/validators/**'],
    dts: 'src/types/route-map.d.ts',
  }),
  Layouts({ defaultLayout: 'default' }),
  vue(),
  // ...其余不变（tailwindcss, vueDevTools, AutoImport, Component）
]
```

- `VueRouter` 的 `exclude`：pages 下的 `components/` `layouts/` `data/` `validators/` `types/` 子目录不当作路由（为阶段6 预留；当前这些子目录尚不存在，exclude 无副作用）。与模板 `RouteGenerateExclude` 完全一致（多加 `validators/**`，为阶段6 预留）。注意：`layouts/**` 在此处是防御性的——`VueRouter` 默认只扫 `src/pages/`，`src/layouts/` 由 vite-plugin-vue-layouts 单独处理；列入 exclude 与模板保持一致。
- `dts: 'src/types/route-map.d.ts'`：生成路由名→路径的类型映射，入库（同 auto-import.d.ts）。
- `Layouts({ defaultLayout: 'default' })`：所有 page 默认套 `layouts/default.vue`，无需每页 `definePage({ layout })`。

**保留**：阶段0+1 的所有配置（lightningcss、AutoImport、Component、port 1420、strictPort、`/api` proxy、vitest test 块、`@` alias、`vitest/config` 导入）。

### 4.2 router/index.ts + guard（新建，替代旧 router.ts）

```
src/router/
├── index.ts          # createRouter，import 文件路由生成的 routes
└── guard/
    ├── index.ts      # setupRouterGuard(router)
    └── common-guard.ts  # nprogress start/done
```

**`src/router/index.ts`**：
```ts
import { setupLayouts } from 'virtual:generated-layouts'
import { createRouter, createWebHashHistory } from 'vue-router'
import { handleHotUpdate, routes } from 'vue-router/auto-routes'

import { setupRouterGuard } from './guard'

const router = createRouter({
  history: createWebHashHistory(),   // 保留 hash：web + Tauri prod 双安全
  routes: setupLayouts(routes),
  scrollBehavior() {
    return { left: 0, top: 0, behavior: 'smooth' }
  },
})

setupRouterGuard(router)

export default router

if (import.meta.hot) {
  handleHotUpdate(router)
}
```

**`src/router/guard/common-guard.ts`**：
```ts
import type { Router } from 'vue-router'

import nprogress from 'nprogress'

export function setupCommonGuard(router: Router) {
  router.beforeEach(() => {
    nprogress.start()
  })
  router.afterEach(() => {
    nprogress.done()
  })
}
```

**`src/router/guard/index.ts`**：
```ts
import type { Router } from 'vue-router'

import { setupCommonGuard } from './common-guard'

export function setupRouterGuard(router: Router) {
  setupCommonGuard(router)
  // auth guard 不做：kanban 无 auth（YAGNI）
}
```

### 4.3 plugins/router/setup.ts 调整

阶段0+1 的 `src/plugins/router/setup.ts` 原 import 自 `@/router`（旧 `src/router.ts` 的 named 导出 `router`）。现在 `@/router` 解析到 `src/router/index.ts` 的 **default** 导出。改为：

```ts
import type { App } from 'vue';
import router from "@/router";

export function setupRouter(app: App) {
  app.use(router);
}
```

（仅 import 方式从 named `{ router }` 改 default `router`。）

### 4.4 目录重组：views/ → pages/<feature>/

每个功能一个文件夹，路由由文件名约定生成。`git mv` 保留历史。

| 现有文件 | 迁移后路径 | 生成路由 |
|---|---|---|
| `views/KanbanView.vue` | `pages/kanban/index.vue` | `/kanban` |
| `views/ProjectsView.vue` | `pages/projects/index.vue` | `/projects` |
| `views/ResourcesView.vue` | `pages/resources/index.vue` | `/resources` |
| `views/TeamsView.vue` | `pages/teams/index.vue` | `/teams` |
| `views/AllocationsView.vue` | `pages/allocations/index.vue` | `/allocations` |
| `views/CalendarView.vue` | `pages/calendar/index.vue` | `/calendar` |
| `views/GanttView.vue` | `pages/gantt/index.vue` | `/gantt` |
| `views/CalendarGridView.vue` | `pages/calendar-grid/index.vue` | `/calendar-grid` |
| `views/CatalogView.vue` | `pages/catalog/index.vue` | `/catalog` |
| `views/AiPanelView.vue` | `pages/ai/index.vue` | `/ai` |
| `views/DashboardView.vue` | `pages/dashboard/index.vue` | `/dashboard` |
| `views/ReportsView.vue` | `pages/reports/index.vue` | `/reports` |
| `views/SettingsView.vue` | `pages/settings/index.vue` | `/settings` |
| —（新建） | `pages/index.vue` | `/` → redirect `/kanban` |

**`pages/index.vue`**（根重定向）：
```vue
<script setup lang="ts">
// 根路径重定向到看板（文件路由用 definePage 声明 redirect）
</script>

<template>
  <div />
</template>

<route lang="yaml">
redirect: /kanban
</route>
```

**关键点**：
- **`calendar-grid` 保持连字符**：`VueRouter` 把文件夹名当作单段路由，`calendar-grid` → `/calendar-grid`（不会拆成 `/calendar/grid`）。已验证符合现路由。
- **全部变 lazy**：文件路由默认 code-split 每个 page，比现状（3 eager + 11 lazy）更彻底。首屏更小，可接受。
- **page 内部代码不改**：`<script setup>` 里的 `import` 全用 `@/` 别名（`@/components`、`@/stores`、`@/api`、`@/utils`），文件移动后这些路径不受影响，**无需改任何 import**。仅文件位置和（可选的）组件 `name` 变化。

### 4.5 布局迁移

```
src/components/AppLayout.vue → src/layouts/default.vue   （git mv，内容保留）
src/layouts/blank.vue                                    （新建，仅 <router-view/>）
```

**`layouts/default.vue`**：内容与现 `AppLayout.vue` **完全一致**，不做任何功能改动。包括：
- shadcn Sidebar（collapsible="icon"）+ SidebarInset；
- sidebar nav 13 项（中文标签 + lucide 图标 + RouterLink + active 态绑定 `route.path`）；
- 顶栏：SidebarTrigger + Separator + project `<Select>`（绑 `projects.current`）+ unit `<Select>`（绑 `unit.unit`）；
- `onMounted` bootstrap 轮询（projects/catalog/unit 加载重试 40 次，ready 前显示 Skeleton）；
- `<main>` 内 `<router-view>`（ready 后）。

阶段5 才增强（theme toggle、command panel、nav 抽 composable、用户菜单）。

**`layouts/blank.vue`**：
```vue
<template>
  <router-view />
</template>
```
为将来 auth/error 页预留（本期无 page 用它，但符合模板结构，成本极低）。

### 4.6 App.vue 调整

阶段0+1 的 App.vue 当前挂 `<AppLayout/>` + `<Toaster/>` + `<VueQueryDevtools v-if="isDev"/>`。布局现由路由插件注入，改为：

```vue
<script setup lang="ts">
import { VueQueryDevtools } from "@tanstack/vue-query-devtools";
import { Toaster } from "@/components/ui/sonner";

const isDev = import.meta.env.DEV;
</script>

<template>
  <router-view />
  <Toaster />
  <VueQueryDevtools v-if="isDev" />
</template>
```

- 移除 `import AppLayout` 与 `<AppLayout/>`（layout 经 `setupLayouts(routes)` 自动套在每个 page 外层）。
- `<router-view/>` 渲染的是 layout，layout 内部再渲染 page 的 `<router-view/>`（vite-plugin-vue-layouts 的两级 router-view 机制）。
- Toaster / VueQueryDevtools 保持在根（全局，不随路由切换）。

### 4.7 清理

- 删除 `src/router.ts`（被 `src/router/index.ts` 取代）。
- `src/views/` 目录清空后删除（14 文件全部 git mv 到 pages/）。
- `src/components/AppLayout.vue` git mv 到 layouts/ 后，原位置不存在（无需手动删）。

### 4.8 依赖

仅需安装 **1 个** devDep：
```
pnpm add -D vite-plugin-vue-layouts
```

**重要修正（spec 初稿曾误列 `unplugin-vue-router`）**：文件路由的 Vite 插件 `VueRouter`（`import VueRouter from 'vue-router/vite'`）、虚拟模块 `vue-router/auto-routes`、以及 `VueRouterAutoImports`（阶段0+1 已用）——这三者**全部内置于 `vue-router@5` 本身**（`dist/unplugin/` 下），**不是**独立的 `unplugin-vue-router` 包。模板的 package.json 也只声明了 `vite-plugin-vue-layouts`，印证了这一点。故本阶段只装 `vite-plugin-vue-layouts`（提供 `virtual:generated-layouts` 虚拟模块）。

## 5. 验证标准（Definition of Done）

1. `pnpm build` 通过（vue-tsc -b + vite build）。
2. `pnpm test` 16/16 通过（单元测试不应受影响——它们测 stores/api，不涉路由）。
3. `pnpm dev` 起 1420（或备用端口），13 路由全部可达：
   - `/` 重定向到 `/kanban`；
   - 侧边栏 13 项可点进；
   - 深链刷新（hash history，如 `/#/projects`）正常加载。
4. 顶栏 project/unit 选择器、bootstrap 轮询（ready 前 Skeleton）、侧边栏 active 态 与改造前一致。
5. 路由切换时 nprogress 进度条出现。
6. `src/types/route-map.d.ts` 生成入库。
7. `src/router.ts` 已删除；`src/views/` 已删除；`src/router/index.ts` + `guard/` 就位；`pages/` 14 个 index.vue 就位；`layouts/default.vue` + `blank.vue` 就位。
8. 自动导入正常（阶段0+1 的 auto-import 不受影响）。

## 6. 风险与对策

| 风险 | 对策 |
|---|---|
| `virtual:generated-layouts` / `vue-router/auto-routes` 类型找不到 | 前者由 vite-plugin-vue-layouts 提供，后者由 vue-router 内置提供；装好 `vite-plugin-vue-layouts` 并 vite 运行一次生成 `route-map.d.ts` 后类型即就位。若 tsconfig 报错，确认 `src/types/route-map.d.ts` 已生成并纳入 include |
| 文件路由把不该当路由的文件（如 components/）当成路由 | `exclude: ['**/components/**', ...]` 已配置；本期 pages 下尚无这些子目录，无副作用 |
| 两级 `<router-view>` 机制导致 page 不渲染 | 这是 vite-plugin-vue-layouts 的标准模式：App.vue 的 `<router-view/>` 渲染 layout，layout 内的 `<router-view/>` 渲染 page。default.vue 现有 `<router-view/>` 直接复用，无需改动 |
| hash history 与文件路由不兼容 | 兼容：history 实例与 routes 来源解耦，文件路由只生成 routes 数组，history 由 createWebHashHistory() 提供 |
| page 移动后 import 路径失效 | 全部用 `@/` 别名，移动不影响；build + typecheck 会立即暴露任何遗漏 |
| nprogress 样式缺失 | 阶段0+1 已 `import "nprogress/nprogress.css"`（nprogress 内置样式），common-guard 调 start/done 即生效 |

## 7. 不在本 spec 解决的问题（显式记录）

- store 是否改用 vue-query —— 阶段3。
- data-table / BasicPage 等可复用组件 —— 阶段4。
- theme store / 暗色切换 / 命令面板 —— 阶段5。
- page 内部重写、i18n 抽取、Ui* 标签 —— 阶段6。
