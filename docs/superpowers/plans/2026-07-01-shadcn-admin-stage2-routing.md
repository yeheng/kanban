# 阶段2：目录与路由迁移 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 kanban 前端的手写扁平路由迁移到 vue-router 内置的文件路由（`src/pages/<route>/index.vue` 自动生成路由）+ vite-plugin-vue-layouts（default layout 自动套用），保留 hash history，接通 nprogress guard —— 同时 14 个 page 内部代码完全不动，应用行为零变化（仅新增路由切换进度条）。

**Architecture:** 文件路由由 vue-router@5 内置的 `VueRouter` vite 插件（`vue-router/vite`）按 `src/pages/` 文件约定生成 routes 数组；`vite-plugin-vue-layouts` 把 `src/layouts/default.vue` 自动套在每个 page 外层（两级 router-view）；`src/router/index.ts` 用 `createWebHashHistory()`（web+Tauri 双安全）+ `setupLayouts(routes)`；common guard 在路由切换时 start/done nprogress。AppLayout.vue → layouts/default.vue（内容不变），App.vue 改为根 `<router-view/>`。

**Tech Stack:** vue-router@5（内置文件路由）+ vite-plugin-vue-layouts@0.11 + nprogress + hash history；在阶段0+1 的 vite/tsconfig/plugins 基建之上。

**Spec:** `docs/superpowers/specs/2026-07-01-shadcn-admin-stage2-routing-design.md`

**重要前置事实（已验证）**：
- 文件路由的 vite 插件 `VueRouter` 来自 `vue-router/vite`（**vue-router@5 自带**，不是独立包）。`vue-router/auto-routes`、`VueRouterAutoImports` 也都内置于 vue-router。
- 唯一需要新装的依赖是 `vite-plugin-vue-layouts`（提供 `virtual:generated-layouts`）。
- `@/router` 目前只被 `src/plugins/router/setup.ts` import（named `{ router }`）。
- `AppLayout` 只被 `src/App.vue` import。
- `views/` 只被 `src/router.ts` import（即将删除）。

---

## File Structure

**Create:**
- `src/router/index.ts` — createRouter（hash history + setupLayouts(routes) + guard）。
- `src/router/guard/index.ts` — `setupRouterGuard(router)` 编排。
- `src/router/guard/common-guard.ts` — nprogress start/done。
- `src/layouts/blank.vue` — 仅 `<router-view/>`（预留 auth/error 页）。
- `src/pages/index.vue` — `/` → redirect `/kanban`。
- `src/pages/<feature>/index.vue` × 13 — 由 `src/views/*.vue` git mv 而来（内部代码不改）。

**Move (git mv):**
- `src/views/KanbanView.vue` → `src/pages/kanban/index.vue`
- `src/views/ProjectsView.vue` → `src/pages/projects/index.vue`
- `src/views/ResourcesView.vue` → `src/pages/resources/index.vue`
- `src/views/TeamsView.vue` → `src/pages/teams/index.vue`
- `src/views/AllocationsView.vue` → `src/pages/allocations/index.vue`
- `src/views/CalendarView.vue` → `src/pages/calendar/index.vue`
- `src/views/GanttView.vue` → `src/pages/gantt/index.vue`
- `src/views/CalendarGridView.vue` → `src/pages/calendar-grid/index.vue`
- `src/views/CatalogView.vue` → `src/pages/catalog/index.vue`
- `src/views/AiPanelView.vue` → `src/pages/ai/index.vue`
- `src/views/DashboardView.vue` → `src/pages/dashboard/index.vue`
- `src/views/ReportsView.vue` → `src/pages/reports/index.vue`
- `src/views/SettingsView.vue` → `src/pages/settings/index.vue`
- `src/components/AppLayout.vue` → `src/layouts/default.vue`（内容不变）

**Modify:**
- `package.json` — 加 `vite-plugin-vue-layouts` devDep。
- `vite.config.ts` — 加 `VueRouter`（from `vue-router/vite`）+ `Layouts` 插件到 plugins 头部。
- `src/plugins/router/setup.ts` — named import 改 default import。
- `src/App.vue` — 移除 AppLayout import + `<AppLayout/>`，改 `<router-view/>`。
- `src/types/route-map.d.ts` — vite 自动生成（入库）。
- `src/types/auto-import-components.d.ts` — 自动重新生成（AppLayout 移走后更新）。

**Delete:**
- `src/router.ts`（被 `src/router/index.ts` 取代）。
- `src/views/`（14 文件全移走后删空目录）。

---

## Task 1: 安装 vite-plugin-vue-layouts

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装**

Run:
```bash
pnpm add -D vite-plugin-vue-layouts
```
Expected: 安装成功，`vite-plugin-vue-layouts` 出现在 devDependencies（版本约 ^0.11）。无 peer dep 致命错误（warning 可接受）。

- [ ] **Step 2: 校验**

Run:
```bash
node -e "console.log(require('./package.json').devDependencies['vite-plugin-vue-layouts'])"
```
Expected: 打印版本号（如 `^0.11.0`）。

- [ ] **Step 3: typecheck 回归（不应受影响）**

Run:
```bash
pnpm exec vue-tsc -b
```
Expected: exit 0（仅装依赖，不动代码）。

- [ ] **Step 4: Commit**

```bash
git add package.json
git commit -m "build: 阶段2 安装 vite-plugin-vue-layouts（default layout 自动套用）"
```
（pnpm-lock.yaml gitignored，不提交。）

---

## Task 2: 改造 vite.config.ts（加 VueRouter + Layouts 插件）

**Files:**
- Modify: `vite.config.ts`

- [ ] **Step 1: 读取当前 vite.config.ts 确认基线**

Run:
```bash
cat vite.config.ts
```
Expected: 看到阶段0+1 的配置（import path/browserslist/tailwindcss/vue/lightningcss/AutoImport/Component/defineConfig from vitest-config/vueDevTools/VueRouterAutoImports；plugins 数组含 vue/tailwindcss/vueDevTools/AutoImport/Component；保留 port 1420/strictPort/`/api` proxy/vitest test 块/lightningcss transformer）。

- [ ] **Step 2: 改写 vite.config.ts**

Modify `vite.config.ts` → 完整替换为：
```ts
import path from "path";
import browserslist from "browserslist";
import tailwindcss from "@tailwindcss/vite";
import vue from "@vitejs/plugin-vue";
import { browserslistToTargets } from "lightningcss";
import AutoImport from "unplugin-auto-import/vite";
import Component from "unplugin-vue-components/vite";
import { defineConfig } from "vitest/config";
import vueDevTools from "vite-plugin-vue-devtools";
import Layouts from "vite-plugin-vue-layouts";
import VueRouter from "vue-router/vite";
import { VueRouterAutoImports } from "vue-router/unplugin";

export default defineConfig({
  plugins: [
    VueRouter({
      exclude: ["**/components/**", "**/layouts/**", "**/data/**", "**/types/**", "**/validators/**"],
      dts: "src/types/route-map.d.ts",
    }),
    Layouts({ defaultLayout: "default" }),
    vue(),
    tailwindcss(),
    vueDevTools(),
    AutoImport({
      imports: ["vue", VueRouterAutoImports],
      dirs: ["src/composables/**/*.ts", "src/constants/**/*.ts", "src/stores/**/*.ts"],
      ignore: ["**/*.test.ts", "**/*.spec.ts"],
      defaultExportByFilename: true,
      dts: "src/types/auto-import.d.ts",
    }),
    Component({
      dirs: ["src/components"],
      collapseSamePrefixes: true,
      directoryAsNamespace: true,
      dts: "src/types/auto-import-components.d.ts",
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  css: {
    transformer: "lightningcss",
    lightningcss: {
      targets: browserslistToTargets(browserslist(["> 1%", "last 2 versions"])),
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    proxy: {
      "/api": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
    },
  },
  test: { environment: "jsdom", globals: true },
});
```

**关键变更**（相对阶段0+1）：
- 新增 import：`Layouts from "vite-plugin-vue-layouts"`、`VueRouter from "vue-router/vite"`。
- plugins 数组**头部**插入 `VueRouter({...})` 和 `Layouts({ defaultLayout: "default" })`——必须在 `vue()` 之前（文件路由要在 vue SFC 编译前介入）。
- `VueRouter` 的 `exclude` 与模板一致（多加 `validators/**` 为阶段6 预留；`layouts/**` 防御性）。
- 其余**完全不变**（lightningcss/AutoImport/Component/port/proxy/vitest 全保留；`defineConfig` 仍 from `vitest/config`；`VueRouterAutoImports` 仍 from `vue-router/unplugin`）。

**注意**：此步会让 `src/types/route-map.d.ts` 开始生成，但 `src/pages/` 目录尚不存在 → 暂时无路由生成。vite build/dev 可能因找不到 pages 目录而行为异常。**这是预期的中间状态**——Task 3-5 才补齐 pages 与 router/index。本 task 仅改 vite.config，**不单独验证 build/dev**（Task 6 统一验证）。

- [ ] **Step 3: typecheck（可能因 pages/ 缺失报错——预期）**

Run:
```bash
pnpm exec vue-tsc -b 2>&1 | tail -10
```
Expected: exit 0 或仅与路由虚拟模块相关的暂态警告。若报 `Cannot find module 'vue-router/auto-routes'` 等错误，记录但**不阻塞**——`src/router/index.ts`（Task 3）建立后这些模块才有消费者。若报与 vite.config.ts 本身配置无关的硬错误，则需排查。

- [ ] **Step 4: Commit**

```bash
git add vite.config.ts
git commit -m "build(vite): 阶段2 加 VueRouter（vue-router/vite 文件路由）+ Layouts 插件

- VueRouter from vue-router/vite（内置，非独立包），exclude 与模板一致
- Layouts defaultLayout=default（vite-plugin-vue-layouts）
- plugins 头部插入（须在 vue() 之前）
- 保留阶段0+1 全部配置（lightningcss/auto-import/proxy/vitest）"
```

---

## Task 3: 建立 router/index.ts + guard（新建，暂不删除旧 router.ts）

**Files:**
- Create: `src/router/index.ts`
- Create: `src/router/guard/index.ts`
- Create: `src/router/guard/common-guard.ts`

> 顺序说明：先建新 router 目录结构（但不接线），Task 4 移 page，Task 5 才改 plugins/router/setup.ts 接线 + 删旧 router.ts。这样每步都可独立 commit，避免中间态崩溃。

- [ ] **Step 1: 创建 common-guard.ts**

Create `src/router/guard/common-guard.ts`:
```ts
import type { Router } from "vue-router";

import nprogress from "nprogress";

export function setupCommonGuard(router: Router) {
  router.beforeEach(() => {
    nprogress.start();
  });
  router.afterEach(() => {
    nprogress.done();
  });
}
```

- [ ] **Step 2: 创建 guard/index.ts**

Create `src/router/guard/index.ts`:
```ts
import type { Router } from "vue-router";

import { setupCommonGuard } from "./common-guard";

export function setupRouterGuard(router: Router) {
  setupCommonGuard(router);
  // auth guard 不做：kanban 无 auth（YAGNI）
}
```

- [ ] **Step 3: 创建 router/index.ts**

Create `src/router/index.ts`:
```ts
import { setupLayouts } from "virtual:generated-layouts";
import { createRouter, createWebHashHistory } from "vue-router";
import { handleHotUpdate, routes } from "vue-router/auto-routes";

import { setupRouterGuard } from "./guard";

const router = createRouter({
  history: createWebHashHistory(),
  routes: setupLayouts(routes),
  scrollBehavior() {
    return { left: 0, top: 0, behavior: "smooth" };
  },
});

setupRouterGuard(router);

export default router;

if (import.meta.hot) {
  handleHotUpdate(router);
}
```

**关键点**：
- `createWebHashHistory()` 保留 hash（web + Tauri prod 双安全）。
- `routes` 来自 `vue-router/auto-routes`（vue-router 内置虚拟模块，由 Task 2 的 VueRouter 插件按 `src/pages/` 生成）。
- `setupLayouts(routes)` 来自 `virtual:generated-layouts`（vite-plugin-vue-layouts），把 layouts/ 套到 routes 上。
- `export default router`（与 Task 5 的 setup.ts default import 对应）。
- **本 task 完成后 `src/router.ts` 仍在**（旧 named export），`plugins/router/setup.ts` 仍 import 旧的——所以 app 仍用旧路由。这是安全的中间态。

- [ ] **Step 4: Commit（新文件，尚未接线，行为不变）**

```bash
git add src/router/
git commit -m "feat(router): 新增 router/index.ts（文件路由+hash history）+ guard（nprogress）

- router/index.ts: createWebHashHistory + setupLayouts(routes) + scrollBehavior
- guard/: common-guard（nprogress start/done）；auth guard 跳过（kanban 无 auth）
- 尚未接线（plugins/router/setup.ts 仍用旧 router.ts），运行行为不变"
```

---

## Task 4: 移动 views/ → pages/ + 建 pages/index.vue 重定向

**Files:**
- Move: 14 个 `src/views/*.vue` → `src/pages/<feature>/index.vue`
- Create: `src/pages/index.vue`
- Delete: `src/views/`（移空后）

> 用 `git mv` 保留历史。page 内部代码**完全不改**（import 全用 `@/` 别名，移动不受影响）。注意：此步后旧 `src/router.ts` 的 `import "./views/XxxView.vue"` 会失效——但 Task 3 已建新 router（Task 5 接线），本 task 之后到 Task 5 接线前，**app 处于短暂不可构建状态**（旧 router.ts 引用的 views 文件已移走）。这是预期的，Task 5 收尾后恢复。为减少这个窗口，本 task 与 Task 5 紧邻执行。

- [ ] **Step 1: 创建 pages 目录结构 + 移动 13 个 view（git mv）**

Run（一次性）:
```bash
mkdir -p src/pages/kanban src/pages/projects src/pages/resources src/pages/teams src/pages/allocations src/pages/calendar src/pages/gantt src/pages/calendar-grid src/pages/catalog src/pages/ai src/pages/dashboard src/pages/reports src/pages/settings

git mv src/views/KanbanView.vue       src/pages/kanban/index.vue
git mv src/views/ProjectsView.vue     src/pages/projects/index.vue
git mv src/views/ResourcesView.vue    src/pages/resources/index.vue
git mv src/views/TeamsView.vue        src/pages/teams/index.vue
git mv src/views/AllocationsView.vue  src/pages/allocations/index.vue
git mv src/views/CalendarView.vue     src/pages/calendar/index.vue
git mv src/views/GanttView.vue        src/pages/gantt/index.vue
git mv src/views/CalendarGridView.vue src/pages/calendar-grid/index.vue
git mv src/views/CatalogView.vue      src/pages/catalog/index.vue
git mv src/views/AiPanelView.vue      src/pages/ai/index.vue
git mv src/views/DashboardView.vue    src/pages/dashboard/index.vue
git mv src/views/ReportsView.vue      src/pages/reports/index.vue
git mv src/views/SettingsView.vue     src/pages/settings/index.vue
```
Expected: 13 个文件移动成功。`git status` 显示 13 个 rename。

- [ ] **Step 2: 根路径重定向（在 router/index.ts 顶层 routes 前置，见 Task 3）**

> **执行修正（已实现，2026-07-01）**：原计划用 `pages/index.vue` + `<route lang="yaml">` 块声明 `redirect: /kanban`，**但 vue-router@5 的 VueRouter 插件并未解析该块**（生成的 routes 无 redirect 字段，运行时 `/` 渲染空 div 而非重定向 —— 行为回归）。`definePage({ redirect })` 对根 `pages/index.vue` 同样无效。
>
> **正确做法**（已落地）：删掉 `pages/index.vue`（不创建），在 `router/index.ts` 的 `createRouter` 里把 redirect 记录作为顶层 routes 数组的第一个元素，且**必须放在 `setupLayouts(...)` 之外**——`setupLayouts` 会给每个 top-level route 套上 layout 组件并降为 child，对无 `component` 的 redirect 记录会破坏其语义：
> ```ts
> routes: [{ path: "/", redirect: "/kanban" }, ...setupLayouts(routes)]
> ```
> 见 Task 3 的 router/index.ts 修正版。新增 `src/router/redirect.test.ts` 覆盖运行时行为（`/` → `/kanban` 且 `redirectedFrom` 正确），防止回归。

- [ ] **Step 3: 删除空的 src/views/ 目录**

Run:
```bash
rmdir src/views 2>/dev/null; ls src/views 2>&1 | head -1
```
Expected: `src/views` 不存在或报 "No such file"（git mv 已移走全部文件，目录空后 rmdir 成功；若 git 跟踪的目录 git 会自动清理）。

- [ ] **Step 4: 确认 page 内部代码未被改动（抽查，git mv 应是纯重命名 R100）**

Run:
```bash
git diff --cached --stat --follow -- src/pages/projects/index.vue src/pages/kanban/index.vue src/pages/settings/index.vue
```
Expected: 显示 `R100`（纯重命名，内容 0 变化）或 `{old} → {new}` 重命名行，无 `+`/`-` 内容差异。若显示内容改动，说明误编辑了文件——检查并还原。

- [ ] **Step 5: 不单独 commit，与 Task 5 合并**

本 task 的移动与 Task 5 的接线（删旧 router.ts、改 setup.ts、改 App.vue、移 AppLayout）合为一个 commit，避免中间不可构建状态入库。直接进入 Task 5。

---

## Task 5: 接线 + 移 AppLayout → layouts/default.vue + 改 App.vue + 删旧 router.ts

**Files:**
- Move: `src/components/AppLayout.vue` → `src/layouts/default.vue`
- Create: `src/layouts/blank.vue`
- Modify: `src/plugins/router/setup.ts`
- Modify: `src/App.vue`
- Delete: `src/router.ts`
- Regenerate: `src/types/route-map.d.ts`, `src/types/auto-import-components.d.ts`

> 与 Task 4 的移动一起 commit，消除中间态。

- [ ] **Step 1: 移动 AppLayout → layouts/default.vue（git mv，内容不变）**

Run:
```bash
mkdir -p src/layouts
git mv src/components/AppLayout.vue src/layouts/default.vue
```
Expected: 文件移动成功。**不修改内容**（sidebar nav / project/unit 选择器 / bootstrap 轮询全部保留）。

- [ ] **Step 2: 创建 layouts/blank.vue**

Create `src/layouts/blank.vue`:
```vue
<template>
  <router-view />
</template>
```

- [ ] **Step 3: 改 plugins/router/setup.ts（named → default import）**

Modify `src/plugins/router/setup.ts` → 完整替换为:
```ts
import type { App } from "vue";
import router from "@/router";

export function setupRouter(app: App) {
  app.use(router);
}
```
（仅 import 从 `{ router }` 改 `router` default；现在 `@/router` 解析到 Task 3 的 `src/router/index.ts` default 导出。）

- [ ] **Step 4: 改 App.vue（移除 AppLayout，改 router-view）**

Modify `src/App.vue` → 完整替换为:
```vue
<script setup lang="ts">
import { VueQueryDevtools } from "@tanstack/vue-query-devtools";
import { Toaster } from "@/components/ui/sonner";

// 模板表达式作用域无法直接用 import.meta，故在 setup 中预计算。
const isDev = import.meta.env.DEV;
</script>

<template>
  <router-view />
  <Toaster />
  <VueQueryDevtools v-if="isDev" />
</template>
```
**关键变更**：移除 `import AppLayout` 与 `<AppLayout/>`；根模板改 `<router-view/>`（由 vite-plugin-vue-layouts 注入 default layout，layout 内再渲染 page 的 router-view）。保留 Toaster + VueQueryDevtools（全局）。

- [ ] **Step 5: 删除旧 src/router.ts**

Run:
```bash
git rm src/router.ts
```
Expected: 删除成功（被 `src/router/index.ts` 取代）。

- [ ] **Step 6: 重新生成 d.ts（vite build 触发 VueRouter + Component 重新扫描）**

Run:
```bash
pnpm exec vite build 2>&1 | tail -20
```
Expected: build 成功（exit 0），dist 发出。这会重新生成：
- `src/types/route-map.d.ts`（含 13 路由 + `/` 的类型映射）
- `src/types/auto-import-components.d.ts`（AppLayout 移走后更新；现在 components/ 下不再有 AppLayout）

若 build 失败，看错误——常见：page 内某 import 因路径变化失效（不应发生，全用 `@/` 别名）；或 `<route>` 块语法。报告 BLOCKED 带完整错误。

- [ ] **Step 7: typecheck**

Run:
```bash
pnpm exec vue-tsc -b 2>&1 | tail -15
```
Expected: exit 0。若报 `Cannot find module 'virtual:generated-layouts'` 或 `'vue-router/auto-routes'`，确认 `src/types/route-map.d.ts` 已生成且 tsconfig.app.json 的 include 含 `src/types/**/*.d.ts`（阶段0+1 已配）。

- [ ] **Step 8: 确认 route-map.d.ts 含 13 路由**

Run:
```bash
grep -cE "/kanban|/projects|/resources|/teams|/allocations|/calendar|/gantt|/calendar-grid|/catalog|/ai|/dashboard|/reports|/settings" src/types/route-map.d.ts
```
Expected: ≥ 13（路由名都进了类型映射）。

- [ ] **Step 9: Commit（Task 4 + Task 5 合并）**

```bash
git add -A
git commit -m "refactor(router): 阶段2 接线文件路由 + views→pages + AppLayout→layouts/default

- views/*.vue → pages/<feature>/index.vue（13 个，git mv，内部代码不改）
- pages/index.vue: / → redirect /kanban
- components/AppLayout.vue → layouts/default.vue（内容不变）
- layouts/blank.vue 新增（预留 auth/error）
- plugins/router/setup.ts: named {router} → default import（指向新 router/index.ts）
- App.vue: 移除 <AppLayout/>，改根 <router-view/>（layout 由插件注入）
- 删除 src/router.ts（被 router/index.ts 取代）
- 重新生成 route-map.d.ts + auto-import-components.d.ts"
```

---

## Task 6: 完整验证 + 回归走查

**Files:** 无修改，纯验证。

- [ ] **Step 1: 完整生产构建**

Run:
```bash
pnpm build 2>&1 | tail -15
```
Expected: `vue-tsc -b && vite build` 成功，dist 发出，exit 0。

- [ ] **Step 2: 单元测试**

Run:
```bash
pnpm test 2>&1 | tail -8
```
Expected: 16/16 通过（测试测 stores/api，不涉路由，应不受影响）。

- [ ] **Step 3: 启动 dev 走查（注意端口可能被占）**

若 1420 被占（主 worktree 的旧 vite），用备用端口：
```bash
pnpm exec vite --port 1422 --strictPort false > /tmp/dev2.log 2>&1 &
sleep 6
```
验证根路径与 SPA：
```bash
curl -s -o /dev/null -w "root HTTP %{http_code}\n" http://localhost:1422/
ENTRY=$(curl -s http://localhost:1422/ | grep -oE '/src/main\.ts[^"]*' | head -1)
curl -s -o /dev/null -w "main.ts HTTP %{http_code}\n" "http://localhost:1422$ENTRY"
```
Expected: 都是 HTTP 200。

验证 page 模块可被请求（文件路由生成的 lazy chunk）：
```bash
for p in pages/kanban/index.vue pages/projects/index.vue pages/settings/index.vue; do
  code=$(curl -s -o /dev/null -w "%{http_code}" "http://localhost:1422/src/$p")
  echo "  /src/$p -> HTTP $code"
done
```
Expected: 都是 HTTP 200（说明 page 文件存在且可编译）。

检查 dev log 无错误：
```bash
grep -iE "error|fail|cannot|throw" /tmp/dev2.log | grep -viE "INVALID_ANNOTATION|@vueuse|devtools" | head -10 || echo "(no errors)"
```
Expected: 无错误。

停止 dev：
```bash
pkill -f "vite --port 1422" 2>/dev/null; echo stopped
```

- [ ] **Step 4: 人工走查清单（浏览器，需后端在 localhost:3000）**

启动 dev（端口自定），浏览器开 app，逐项确认：
- [ ] `/` 自动重定向到 `/kanban`（地址栏 `/#/kanban`）
- [ ] 侧边栏 13 项导航全部可点进，页面渲染正常
- [ ] 侧边栏 active 态（当前项高亮）正确
- [ ] 顶栏 project/unit 选择器正常
- [ ] 首次加载时 Skeleton 占位（bootstrap 轮询）正常
- [ ] 路由切换时顶部出现 nprogress 进度条（蓝色细条）
- [ ] 深链刷新（如直接开 `/#/projects`）正常加载，不白屏
- [ ] 控制台无新增红色错误

Expected: 功能与阶段0+1 一致 + 新增 nprogress 进度条。

- [ ] **Step 5: 若走查发现问题，修复后 commit；否则无新 commit**

```bash
git status --short
```
Expected: clean（Task 5 已提交全部改动）。

---

## 完成判据（Definition of Done）

对应 spec §5：
1. ✅ `pnpm build` 通过（Task 6 Step 1）
2. ✅ `pnpm test` 16/16（Task 6 Step 2）
3. ✅ `pnpm dev` 13 路由可达，`/`→`/kanban` 重定向，深链刷新正常（Task 6 Step 3-4）
4. ✅ 顶栏选择器/bootstrap/active 态一致（Task 6 Step 4）
5. ✅ 路由切换 nprogress 进度条出现（Task 6 Step 4）
6. ✅ `route-map.d.ts` 生成入库（Task 5 Step 8）
7. ✅ `router.ts` 删除、`views/` 删除、`router/index.ts`+`guard/` 就位、`pages/` 14 个就位、`layouts/default.vue`+`blank.vue` 就位（Task 4-5）
8. ✅ auto-import 不受影响（Task 5 Step 6 重新生成）

## 风险速查（详见 spec §6）

- `virtual:generated-layouts` / `vue-router/auto-routes` 类型 → vite-plugin-vue-layouts + vue-router 内置提供，route-map.d.ts 生成后纳入 include
- 文件路由误扫 components/ → exclude 已配
- 两级 router-view → vite-plugin-vue-layouts 标准模式，default.vue 内 `<router-view/>` 直接复用
- hash history + 文件路由 → 兼容（history 实例与 routes 来源解耦）
- page 移动后 import 失效 → 全用 `@/` 别名，build 立即暴露
- 中间不可构建窗口（Task 4 移 views 后到 Task 5 接线前）→ 两 task 合并 commit 消除
