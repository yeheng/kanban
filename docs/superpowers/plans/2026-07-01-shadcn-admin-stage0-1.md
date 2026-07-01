# 阶段0+1：构建基建 & 插件化入口 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 安装构建/运行时插件依赖，启用 auto-import + components 自动解析 + lightningcss，拆分 tsconfig，建立 `src/plugins/` 模块化入口，瘦身 main.ts —— 同时保持 14 个 view 与 13 条路由行为零变化。

**Architecture:** 纯增量改造。新增依赖与 `src/plugins/`（8 个 setup 文件）+ `src/utils/env.ts` + `src/types/` 生成目录；改造 `vite.config.ts` / `tsconfig*` / `main.ts`；`App.vue` / `AppLayout.vue` / `router.ts` / `api/index.ts` 本期不动。每个 task 产生可独立验证、可 commit 的变更。

**Tech Stack:** Vue 3.5 + Vite 8 + TS 6 + pnpm 11 + Tailwind v4 + shadcn-vue；新增 unplugin-auto-import / unplugin-vue-components / lightningcss / @tanstack/vue-query / vue-i18n / pinia-plugin-persistedstate / ofetch / dayjs / nprogress / @formkit/auto-animate。

**Spec:** `docs/superpowers/specs/2026-07-01-shadcn-admin-stage0-1-design.md`

---

## File Structure

**Create:**
- `src/utils/env.ts` — 暴露类型安全的 env 对象（读取 `import.meta.env`，对齐 kanban 现有 `VITE_API_BASE`）。非抛错式（与模板的 zod 校验不同，避免本期引入 env 硬依赖）。
- `src/plugins/index.ts` — `setupPlugins(app)` 编排入口。
- `src/plugins/pinia/setup.ts` — createPinia + persistedstate(sessionStorage)；`export default pinia`。
- `src/plugins/router/setup.ts` — `app.use(router)`，复用现有 `@/router`。
- `src/plugins/tanstack-vue-query/setup.ts` — VueQueryPlugin + QueryClient(staleTime 5min)。
- `src/plugins/i18n/index.ts` — Language 类型、SUPPORTED_LOCALES、DEFAULT_LOCALE='zh'、appLocale(useStorage)。
- `src/plugins/i18n/setup.ts` — createI18n(legacy:false)。
- `src/plugins/i18n/zh.json` / `en.json` — 种子 key（app.title + common.*）。
- `src/plugins/dayjs/setup.ts` — dayjs locale zh-cn + relativeTime。
- `src/plugins/nprogress/setup.ts` — nprogress.configure + 样式 import。
- `src/plugins/auto-animate/setup.ts` — autoAnimatePlugin。
- `src/types/` — 目录（auto-import 生成的 dts 落入此处）。

**Modify:**
- `package.json` — 加依赖（dev + deps）。
- `vite.config.ts` — 加 AutoImport / Component / lightningcss / vueDevTools 插件；保留 port/proxy/vitest。
- `tsconfig.json` → 改为 references 索引。
- `tsconfig.app.json`（新建，从原 tsconfig.json 迁入 compilerOptions + 加 dts include）。
- `tsconfig.node.json`（新建，给 vite.config.ts）。
- `src/main.ts` — 改为 setupPlugins 编排。

**Do NOT touch:** `src/App.vue`、`src/components/AppLayout.vue`、`src/router.ts`、`src/api/index.ts`、所有 `src/views/**`、`src/components/**`、`src/stores/**`。

---

## Task 1: 安装依赖

**Files:**
- Modify: `package.json`, `pnpm-lock.yaml`（gitignored）

- [ ] **Step 1: 安装运行时依赖**

Run:
```bash
pnpm add ofetch @tanstack/vue-query @tanstack/vue-query-devtools pinia-plugin-persistedstate vue-i18n dayjs nprogress @formkit/auto-animate
```
Expected: 安装成功，`package.json` dependencies 增加 8 项。无 peer dep 冲突致命错误（warning 可接受）。

- [ ] **Step 2: 安装开发依赖**

Run:
```bash
pnpm add -D unplugin-auto-import unplugin-vue-components lightningcss browserslist vite-plugin-vue-devtools @types/nprogress
```
Expected: 安装成功，devDependencies 增加 6 项。

- [ ] **Step 3: 校验安装结果**

Run:
```bash
node -e "const p=require('./package.json'); console.log('deps:',Object.keys(p.dependencies).filter(k=>['ofetch','@tanstack/vue-query','@tanstack/vue-query-devtools','pinia-plugin-persistedstate','vue-i18n','dayjs','nprogress','@formkit/auto-animate'].includes(k))); console.log('devDeps:',Object.keys(p.devDependencies).filter(k=>['unplugin-auto-import','unplugin-vue-components','lightningcss','browserslist','vite-plugin-vue-devtools','@types/nprogress'].includes(k)))"
```
Expected: 两个数组各列 8 / 6 项，无遗漏。

- [ ] **Step 4: 确认 dev server 仍能起（回归基线）**

Run:
```bash
timeout 25 pnpm dev 2>&1 | head -20
```
Expected: vite 输出 `ready in ... ms`，监听 `localhost:1420`，无启动错误。（依赖安装不应破坏现有构建；此步是后续改造前的基线。）

- [ ] **Step 5: Commit**

```bash
git add package.json
git commit -m "build: 阶段0+1 安装插件依赖（vue-query/i18n/ofetch/auto-import 等）

- deps: ofetch @tanstack/vue-query(-devtools) pinia-plugin-persistedstate vue-i18n dayjs nprogress @formkit/auto-animate
- devDeps: unplugin-auto-import unplugin-vue-components lightningcss browserslist vite-plugin-vue-devtools @types/nprogress"
```
注：`pnpm-lock.yaml` 被项目 .gitignore 排除，不提交。

---

## Task 2: 拆分 tsconfig 为 app/node 三件套

**Files:**
- Modify: `tsconfig.json`
- Create: `tsconfig.app.json`
- Create: `tsconfig.node.json`
- Create: `src/types/`（目录占位，dts 将由 vite 生成落入）

- [ ] **Step 1: 读取当前 tsconfig 作为 app 迁移源**

Run:
```bash
cat tsconfig.json
```
Expected: 看到现有 compilerOptions（target ES2021 / module ESNext / moduleResolution Bundler / strict / ignoreDeprecations "6.0" / paths @/* / types vitest/globals,vite/client / include src/**/*.ts,**.vue）。

- [ ] **Step 2: 创建 tsconfig.app.json**

Create `tsconfig.app.json`:
```json
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "jsx": "preserve",
    "types": ["vitest/globals", "vite/client"],
    "lib": ["ES2021", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "ignoreDeprecations": "6.0",
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    },
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.vue", "src/types/**/*.d.ts"]
}
```
（相比原文件：include 增加 `src/**/*.d.ts` 与 `src/types/**/*.d.ts`，加 `noEmit:true` 以配合 vue-tsc -b 引用模式。）

- [ ] **Step 3: 创建 tsconfig.node.json**

Create `tsconfig.node.json`:
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "skipLibCheck": true,
    "noEmit": true,
    "types": ["node"],
    "lib": ["ES2023"]
  },
  "include": ["vite.config.ts"]
}
```
注：`@types/node` 当前未装；此处的 `"types": ["node"]` 需要 `@types/node`。下一步先装它。

- [ ] **Step 4: 安装 @types/node（tsconfig.node.json 需要）**

Run:
```bash
pnpm add -D @types/node
```
Expected: 成功。

- [ ] **Step 5: 改写 tsconfig.json 为 references 索引**

Modify `tsconfig.json` → 完整替换为:
```json
{
  "files": [],
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

- [ ] **Step 6: 创建 src/types 目录占位**

Run:
```bash
mkdir -p src/types
```
（目录此时为空，Task 4 vite 运行后生成 dts 落入。）

- [ ] **Step 7: 类型检查通过**

Run:
```bash
pnpm exec vue-tsc -b
```
Expected: 退出码 0，无错误。（此时尚未改 vite/main，类型检查应与改造前一致通过。）

- [ ] **Step 8: Commit**

```bash
git add tsconfig.json tsconfig.app.json tsconfig.node.json package.json
git commit -m "build(tsconfig): 拆分为 app/node 三件套 references

- tsconfig.json 改为 references 索引
- 新增 tsconfig.app.json（迁入 compilerOptions，include 加 dts）
- 新增 tsconfig.node.json（vite.config.ts 用）
- 装 @types/node"
```

---

## Task 3: 创建 src/utils/env.ts 与各 plugin setup 文件（无副作用纯文件）

> 这些文件本期创建后**尚不被 main.ts 引用**（main.ts 在 Task 5 才接入）。先建文件再接线，便于分步验证。本 task 全部是新建文件，无类型耦合到现有代码。

**Files:**
- Create: `src/utils/env.ts`
- Create: `src/plugins/dayjs/setup.ts`
- Create: `src/plugins/nprogress/setup.ts`
- Create: `src/plugins/auto-animate/setup.ts`
- Create: `src/plugins/pinia/setup.ts`
- Create: `src/plugins/tanstack-vue-query/setup.ts`
- Create: `src/plugins/i18n/index.ts`
- Create: `src/plugins/i18n/setup.ts`
- Create: `src/plugins/i18n/zh.json`
- Create: `src/plugins/i18n/en.json`
- Create: `src/plugins/router/setup.ts`

- [ ] **Step 1: 创建 src/utils/env.ts**

Create `src/utils/env.ts`:
```ts
/**
 * 类型安全的环境变量访问层。
 * 对齐 kanban 现有约定：使用 VITE_API_BASE（相对路径，dev 下走 vite proxy /api）。
 * 与模板的 zod 严格校验不同：此处采用非抛错策略，缺失时回退默认值，避免本期引入硬依赖。
 */
const env = {
  get VITE_API_BASE() {
    return (import.meta.env.VITE_API_BASE as string | undefined) ?? ""
  },
} as const

export default env
```
（本期仅建立；`src/api/index.ts` 第 5 行现有 `import.meta.env.VITE_API_BASE` 直读**暂不改**，留阶段3 数据层迁移时统一引用此处。）

- [ ] **Step 2: 创建 dayjs 插件**

Create `src/plugins/dayjs/setup.ts`:
```ts
import dayjs from "dayjs"
import relativeTime from "dayjs/plugin/relativeTime"
import "dayjs/locale/zh-cn"

export function setupDayjs() {
  dayjs.locale("zh-cn")
  dayjs.extend(relativeTime)
}
```

- [ ] **Step 3: 创建 nprogress 插件**

Create `src/plugins/nprogress/setup.ts`:
```ts
import nprogress from "nprogress"
import "nprogress/nprogress.css"

export function setupNProgress() {
  nprogress.configure({
    showSpinner: false,
    speed: 500,
    trickleSpeed: 200,
  })
}
```
（不 import 项目自有 nprogress.css —— kanban 无此文件；用 nprogress 内置样式即可。本期仅配置引擎，真正的 start/done 在路由 guard 调用，留阶段2。）

- [ ] **Step 4: 创建 auto-animate 插件**

Create `src/plugins/auto-animate/setup.ts`:
```ts
import type { App } from "vue"
import { autoAnimatePlugin } from "@formkit/auto-animate/vue"

export function setupAutoAnimate(app: App) {
  app.use(autoAnimatePlugin)
}
```

- [ ] **Step 5: 创建 pinia 插件（+persist session）**

Create `src/plugins/pinia/setup.ts`:
```ts
import type { App } from "vue"
import { createPinia } from "pinia"
import { createPersistedState } from "pinia-plugin-persistedstate"

const pinia = createPinia()
pinia.use(createPersistedState({ storage: sessionStorage }))

export function setupPinia(app: App) {
  app.use(pinia)
}

export default pinia
```
（默认 sessionStorage 持久化；现有 13 个 store 均未声明 `persist:true`，加插件**不影响**它们，零回归。阶段5 theme store 才启用 persist。）

- [ ] **Step 6: 创建 vue-query 插件**

Create `src/plugins/tanstack-vue-query/setup.ts`:
```ts
import type { App } from "vue"
import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query"
import { VueQueryDevTools } from "@tanstack/vue-query-devtools"

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60 * 5,
    },
  },
})

export function setupTanstackVueQuery(app: App) {
  app.use(VueQueryPlugin, { queryClient })
  app.use(VueQueryDevTools, { initialIsOpen: false })
}
```

- [ ] **Step 7: 创建 i18n index（类型 + locale state）**

Create `src/plugins/i18n/index.ts`:
```ts
import { watch } from "vue"
import { useStorage } from "@vueuse/core"

export type Language = "zh" | "en"

export const SUPPORTED_LOCALES = new Set<Language>(["zh", "en"])

/** kanban 默认中文（与模板的 'en' 默认相反） */
export const DEFAULT_LOCALE: Language = "zh"

export const appLocale = useStorage<Language>("app-locale", DEFAULT_LOCALE)

watch(
  appLocale,
  (newLocale) => {
    if (!SUPPORTED_LOCALES.has(newLocale)) {
      appLocale.value = DEFAULT_LOCALE
    }
  },
  { immediate: true },
)
```
（显式 `import { watch } from "vue"`，避免依赖 Task 4 的 auto-import 注入，保证本 task 文件独立可类型检查。`watch` 此处在模块加载时注册，需 Pinia/app 之外安全运行——`useStorage` 基于 localStorage，无需 app 上下文。）

- [ ] **Step 8: 创建 i18n setup**

Create `src/plugins/i18n/setup.ts`:
```ts
import type { App } from "vue"
import { createI18n } from "vue-i18n"

import { appLocale, DEFAULT_LOCALE } from "."
import type { Language } from "."
import en from "./en.json"
import zh from "./zh.json"

export function setupI18n(app: App) {
  const i18n = createI18n({
    legacy: false,
    locale: appLocale.value,
    fallbackLocale: DEFAULT_LOCALE,
    messages: {
      zh,
      en,
    } as Record<Language, Record<string, any>>,
  })
  app.use(i18n)
}
```

- [ ] **Step 9: 创建 i18n 种子 zh.json / en.json**

Create `src/plugins/i18n/zh.json`:
```json
{
  "app": {
    "title": "Kanban"
  },
  "common": {
    "confirm": "确认",
    "cancel": "取消",
    "save": "保存",
    "delete": "删除",
    "edit": "编辑",
    "create": "新建",
    "search": "搜索"
  }
}
```

Create `src/plugins/i18n/en.json`:
```json
{
  "app": {
    "title": "Kanban"
  },
  "common": {
    "confirm": "Confirm",
    "cancel": "Cancel",
    "save": "Save",
    "delete": "Delete",
    "edit": "Edit",
    "create": "Create",
    "search": "Search"
  }
}
```
（仅种子；现有 14 view 中文硬编码本期不抽取，阶段6 同步抽 key。）

- [ ] **Step 10: 创建 router 插件（复用现有 @/router）**

Create `src/plugins/router/setup.ts`:
```ts
import type { App } from "vue"
import { router } from "@/router"

export function setupRouter(app: App) {
  app.use(router)
}
```
（`@/router` 是 kanban 现有 `src/router.ts` 导出的 `router`（hash history 扁平表）。本期不改它，阶段2 才迁文件路由。）

- [ ] **Step 11: 创建 plugins/index.ts 编排入口**

Create `src/plugins/index.ts`:
```ts
import type { App } from "vue"
import { setupAutoAnimate } from "./auto-animate/setup"
import { setupDayjs } from "./dayjs/setup"
import { setupI18n } from "./i18n/setup"
import { setupNProgress } from "./nprogress/setup"
import { setupPinia } from "./pinia/setup"
import { setupRouter } from "./router/setup"
import { setupTanstackVueQuery } from "./tanstack-vue-query/setup"

export function setupPlugins(app: App) {
  // 顺序：无依赖的纯配置先行；pinia 必须先于任何用到 store 的插件（router guard 将用 pinia）
  setupDayjs()
  setupNProgress()
  setupAutoAnimate(app)
  setupTanstackVueQuery(app)
  setupI18n(app)
  setupPinia(app)
  setupRouter(app)
}
```

- [ ] **Step 12: Commit（文件已建，尚未接入 main.ts，此时运行行为不变）**

```bash
git add src/utils/env.ts src/plugins/
git commit -m "feat(plugins): 新增模块化插件基建（pinia/vue-query/i18n/dayjs/nprogress/auto-animate/router）

- src/plugins/* 8 个 setup 文件 + i18n 种子 zh/en.json
- src/utils/env.ts 类型安全 env 访问层（非抛错式）
- 尚未接入 main.ts，运行行为不变（Task 5 接线）"
```

---

## Task 4: 改造 vite.config.ts（auto-import + components + lightningcss + devtools）

**Files:**
- Modify: `vite.config.ts`

- [ ] **Step 1: 改写 vite.config.ts**

Modify `vite.config.ts` → 完整替换为:
```ts
import path from "path"
import browserslist from "browserslist"
import tailwindcss from "@tailwindcss/vite"
import vue from "@vitejs/plugin-vue"
import { browserslistToTargets } from "lightningcss"
import AutoImport from "unplugin-auto-import/vite"
import Component from "unplugin-vue-components/vite"
import { defineConfig } from "vite"
import vueDevTools from "vite-plugin-vue-devtools"
import { VueRouterAutoImports } from "vue-router/auto-routes"

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
    vueDevTools(),
    AutoImport({
      imports: ["vue", VueRouterAutoImports],
      dirs: ["src/composables/**/*.ts", "src/constants/**/*.ts", "src/stores/**/*.ts"],
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
})
```
关键保留：port 1420 / strictPort / `/api` proxy / vitest test 块 / `@` alias。
关键新增：AutoImport（vue + VueRouterAutoImports + 3 个 dirs；注意 `VueRouterAutoImports` 来自 `vue-router/auto-routes`，vue-router 5 已装）、Component（决策 B：启用，但现有 14 view 不改标签）、lightningcss transformer、vueDevTools。
**注意**：`VueRouterAutoImports` 即便本期不用文件路由也可安全 import（仅注入 `useRouter/useRoute` 等自动导入条目，与现有显式 import 共存）。

- [ ] **Step 2: 启动 dev 触发 dts 生成**

Run:
```bash
timeout 25 pnpm dev 2>&1 | head -25
```
Expected: vite 启动成功（`ready in ... ms`，监听 1420），并生成 `src/types/auto-import.d.ts` 与 `src/types/auto-import-components.d.ts`。若首次启动有 vue-router/auto-routes 解析告警但非致命即可。

- [ ] **Step 3: 校验 dts 已生成**

Run:
```bash
ls -la src/types/*.d.ts 2>/dev/null && echo "---" && head -5 src/types/auto-import.d.ts
```
Expected: 列出两个 dts 文件，auto-import.d.ts 头部含生成的注释（`// Generated by 'unplugin-auto-import'`）。

- [ ] **Step 4: 类型检查通过（含生成 dts）**

Run:
```bash
pnpm exec vue-tsc -b
```
Expected: 退出码 0。若报与现有显式 import 重复相关的错误，记录但**不在本 task 修**（auto-import 与显式 import 共存合法；阶段6 清理）。若报其他实质错误则需排查。

- [ ] **Step 5: Commit**

```bash
git add vite.config.ts src/types/auto-import.d.ts src/types/auto-import-components.d.ts
git commit -m "build(vite): 启用 auto-import + components 自动解析 + lightningcss

- AutoImport: vue + VueRouterAutoImports + composables/constants/stores dirs
- Component: directoryAsNamespace（决策B：启用，现有 view 标签本期不改）
- lightningcss transformer（browserslist > 1% / last 2 versions）
- vite-plugin-vue-devtools
- 生成 auto-import.d.ts / auto-import-components.d.ts 入库
- 保留 port 1420 / strictPort / /api proxy / vitest"
```

---

## Task 5: 接线 main.ts 到 setupPlugins

**Files:**
- Modify: `src/main.ts`

- [ ] **Step 1: 改写 main.ts**

Modify `src/main.ts` → 完整替换为:
```ts
import { createApp } from "vue"
import App from "./App.vue"
import { setupPlugins } from "./plugins"
import "@/utils/env"
import "./styles.css"

function bootstrap() {
  const app = createApp(App)
  setupPlugins(app)
  app.mount("#app")
}

bootstrap()
```
（移除直接的 `createPinia()` / `router` use —— 现由 setupPlugins 编排。`App.vue` 不动：仍 `<AppLayout/>` + `<Toaster/>`。）

- [ ] **Step 2: 启动 dev 验证 app 加载**

Run:
```bash
timeout 25 pnpm dev 2>&1 | head -25
```
Expected: vite 启动成功，无运行时错误（控制台无 "createPinia not found" / "router not installed" 之类）。

- [ ] **Step 3: 类型检查**

Run:
```bash
pnpm exec vue-tsc -b
```
Expected: 退出码 0。

- [ ] **Step 4: Commit**

```bash
git add src/main.ts
git commit -m "refactor(main): main.ts 改为 setupPlugins 编排入口

- createApp(App) → setupPlugins(app) → mount
- 移除直接的 createPinia/router use（现由 plugins/index.ts 编排）
- 顺序: dayjs→nprogress→auto-animate→vue-query→i18n→pinia→router"
```

---

## Task 6: 完整构建验证 + 回归走查

**Files:** 无修改，纯验证。

- [ ] **Step 1: 完整生产构建**

Run:
```bash
pnpm build 2>&1 | tail -20
```
Expected: `vue-tsc -b && vite build` 成功，产出 `dist/`，无类型错误、无构建错误。

- [ ] **Step 2: 启动 dev 走查核心路由**

Run（后台起 dev）:
```bash
pnpm dev &
sleep 8
```
然后逐路由 curl 头部确认 200（hash 路由下根路径返回 index.html，SPA）:
```bash
curl -s -o /dev/null -w "%{http_code}\n" http://localhost:1420/
```
Expected: `200`。

停止 dev:
```bash
kill %1 2>/dev/null; wait 2>/dev/null
```

- [ ] **Step 3: 人工走查清单（在浏览器执行）**

启动 `pnpm dev`，浏览器开 `http://localhost:1420/`，逐项确认：
- [ ] 侧边栏 13 项导航全部可点进，页面渲染正常
- [ ] `/kanban`：任务卡片、状态列正常
- [ ] `/projects`：项目列表、新建/编辑弹窗、删除可用
- [ ] `/settings`：保存设置触发 toast
- [ ] 顶栏 project/unit 选择器正常
- [ ] 控制台无新增红色错误（vue-query devtools 浮标出现属正常，本期已注册）

Expected: 功能与改造前一致（行为零回归）。

- [ ] **Step 4: 最终 Commit（仅当步骤 1-3 全绿）**

若 Task 1-5 已各自 commit，本步若无新改动则跳过；若走查中发现需微调（如某个 import 因 auto-import 注入而需去重），则:
```bash
git add -A
git commit -m "fix: 阶段0+1 回归走查微调"
```

- [ ] **Step 5: 推送（可选，仅当用户要求）**

本阶段不主动 push，除非用户明确要求。

---

## 完成判据（Definition of Done）

对应 spec §5：
1. ✅ `pnpm install` 成功，无致命 peer 冲突（Task 1）
2. ✅ `pnpm build` 通过（Task 6 Step 1）
3. ✅ `pnpm dev` 起 1420，13 路由可达，功能一致（Task 6 Step 2-3）
4. ✅ `auto-import.d.ts` / `auto-import-components.d.ts` 已生成入库（Task 4 Step 3）
5. ✅ vue-query devtools 在 dev 下可见（Task 6 Step 3）
6. ✅ 核心路径（kanban/projects/settings）零回归（Task 6 Step 3）

## 风险速查（详见 spec §6）

- auto-import 与现有显式 import 重复 → 合法共存，阶段6 统一清理
- lightningcss 改变 CSS 产物 → build 后视觉走查，必要时回退该插件
- TS6 strict + ignoreDeprecations → dts 是声明文件，通常不触发
- pinia-persist → 仅影响声明 persist:true 的 store，现有 store 零影响
