# Spec A — 阶段0+1：构建基建 & 插件化入口

> 应用 shadcn-vue-admin 模板到 kanban 前端的**第一个子项目**（共 7 个阶段的第 1 个）。
> 决策已确认：全面架构对齐；zh/en 双语 i18n；`unplugin-vue-components` 决策点 = **B**（启用但本期不重写现有标签）。

## 1. 背景与定位

kanban 前端已完成 shadcn-vue/Tailwind v4 组件库迁移（14 view 全部 shadcn-vue）。当前差距是**架构组织与可复用基建**：手写扁平路由 + 无 guard、单文件 `api/index.ts`、手写 Table、无暗色切换/无 i18n/无 auto-import。

为避免一次性重构过大、且每个阶段必须可独立验证，分解为 7 阶段：

| 阶段 | 内容 | 依赖 |
|---|---|---|
| **0+1（本 spec）** | 构建基建 + 插件化入口 | — |
| 2 | 目录与路由迁移（views→pages、文件路由、layouts、guard） | 0+1 |
| 3 | 数据层迁移（api 拆分、ofetch、vue-query 取代 refresh 总线） | 0+1 |
| 4 | 可复用组件基建（data-table、global-layout、prop-ui、command-menu） | 0+1 |
| 5 | 外壳与主题（app-sidebar、layouts/default、theme store、暗色切换） | 1, 4 |
| 6 | 页面重写（14 view→BasicPage+data-table+zod；**同步抽取 i18n key**；dashboard 图表） | 3, 4, 5 |

4 可与 3 并行；i18n 已并入 1（骨架）与 6（文案抽取）。

## 2. 目标

阶段结束后，代码库具备：
- 已安装构建/运行时插件依赖；
- `vite.config.ts` 启用 auto-import + components 自动解析 + lightningcss；
- `tsconfig` 拆分为 app/node 三件套；
- `src/plugins/` 模块化入口（pinia+persist / router / vue-query / i18n / dayjs / nprogress / auto-animate）；
- 已注册的 vue-query、vue-i18n 骨架、pinia-persist。

**应用行为零变化**：14 view、13 路由、所有功能照常运行；本期是纯基建，不增加功能、不重写页面。

## 3. 非目标（推迟）

- 文件路由迁移、`views/→pages/` 重命名、layouts 接管 `<router-view>`（阶段2）
- `api/index.ts` 拆分、store 重写、vue-query 接管数据、refresh 总线移除（阶段3）
- data-table、BasicPage、prop-ui 等可复用组件（阶段4）
- app-sidebar、主题 store、暗色切换、命令面板（阶段5）
- 任何 view 的标签重写、文案 i18n 抽取（阶段6）

## 4. 详细设计

### 4.1 依赖安装

仅装本阶段必需的；特性依赖（图表、文件路由、embla/motion-v/vaul）随各自阶段安装。

```
# 构建/开发
unplugin-auto-import unplugin-vue-components lightningcss browserslist vite-plugin-vue-devtools

# 运行时插件
ofetch @tanstack/vue-query @tanstack/vue-query-devtools pinia-plugin-persistedstate \
vue-i18n dayjs nprogress @types/nprogress @formkit/auto-animate
```

推迟明确列出：`unplugin-vue-router`、`vite-plugin-vue-layouts`（阶段2）；`@unovis/ts @unovis/vue`（阶段6 图表）；`embla-carousel-*`、`motion-v`、`vaul-vue`、`@faker-js/faker`、`vue-input-otp`、`@iconify/*`（用到对应组件时再装）。

### 4.2 vite.config.ts 改造

保留：port 1420、strictPort、`/api`→`localhost:3000` proxy、vitest 配置、`@` alias。

新增插件（顺序与模板对齐）：
- `AutoImport`：imports = `['vue', VueRouterAutoImports]`；dirs = `['src/composables/**/*.ts','src/constants/**/*.ts','src/stores/**/*.ts']`；`dts: 'src/types/auto-import.d.ts'`；`defaultExportByFilename: true`。
- `Component`（决策 B）：`dirs: ['src/components']`，`collapseSamePrefixes: true`，`directoryAsNamespace: true`，`dts: 'src/types/auto-import-components.d.ts'`。**现有 14 view 的显式 import 不动**；`Ui*` 自动解析约定留待阶段6 渐进采用。
- `css.transformer: 'lightningcss'` + `lightningcss.targets: browserslistToTargets(browserslist(['> 1%','last 2 versions']))`。
- `vueDevTools()`。

**本期不加** `unplugin-vue-router`、`vite-plugin-vue-layouts`、`VueRouter({})`、`Layouts({})`、`visualizer`（阶段2/用到时再加）。

注意：`unplugin-auto-import` 自动解析 `ref/computed/watch` 等后，**现有显式 `import { ref } from 'vue'` 仍合法**（重复 import 会被 linter 提示但不是错误），本期不强制清理；阶段6 页面重写时顺带去掉。可选地引入 eslint 规则提示，但非本期范围。

### 4.3 tsconfig 拆分（对齐模板）

- `tsconfig.json`：仅 `references` 指向 app + node。
- `tsconfig.app.json`：迁入现有 compilerOptions（target ES2021 / Bundler / strict / TS6 `ignoreDeprecations: "6.0"` / `@/*` paths / `lib`）；`include` 增加 `src/types/*.d.ts`（auto-import 生成文件）；`types: ['vitest/globals','vite/client']` 保留。
- `tsconfig.node.json`：给 vite.config.ts 用。

`src/types/route-map.d.ts` 不在本期生成（阶段2）。生成的 `auto-import.d.ts` / `auto-import-components.d.ts` **入库**（与模板一致；本项目无 `.gitignore` 排除它们，CI 上避免类型缺失）。

### 4.4 src/plugins/ 结构（核心产出）

```
src/plugins/
├── index.ts                       # setupPlugins(app)，按依赖序注册
├── pinia/setup.ts                 # createPinia() + piniaPluginPersistedstate；export const pinia（供 guard 用）
├── router/setup.ts                # app.use(router)；router 仍 import 现有 @/router（src/router.ts，阶段2 再迁）
├── tanstack-vue-query/setup.ts    # 注册 VueQueryPlugin + VueQueryDevTools（仅 dev）
├── i18n/
│   ├── setup.ts                   # createI18n({ legacy:false, locale:'zh', fallbackLocale:'zh', messages })
│   ├── index.ts                   # 导出 i18n 实例 + t
│   ├── zh.json                    # 种子骨架 {app:{title}, common:{...}}
│   └── en.json                    # 同结构英文
├── dayjs/setup.ts                 # dayjs.locale('zh-cn') + 相关插件（按需）
├── nprogress/setup.ts             # nprogress.configure({ showSpinner:false }) + 样式
└── auto-animate/setup.ts          # registerAutoAnimate()
```

**`index.ts` 注册顺序**（顺序很重要，pinia 必须先于用它的插件）：
```
setupDayjs → setupNProgress → setupAutoAnimate(app)
→ setupTanstackVueQuery(app) → setupI18n(app) → setupPinia(app) → setupRouter(app)
```

### 4.5 main.ts 瘦身

改造前：直接 `createApp(App).use(createPinia()).use(router).mount('#app')` + `import './styles.css'`。

改造后：
```ts
import { createApp } from 'vue'
import App from './App.vue'
import { setupPlugins } from './plugins'
import '@/utils/env'           // 新建（对齐模板），读取 VITE_* 到统一对象
import './styles.css'          // 保留位置

function bootstrap() {
  const app = createApp(App)
  setupPlugins(app)
  app.mount('#app')
}
bootstrap()
```

`src/utils/env.ts` 新建：基于 `import.meta.env` 暴露 `VITE_API_BASE`（现有 `src/api/index.ts` 内联读取 import.meta.env.VITE_API_BASE 改为引用此处，避免重复；本期可暂不改 api/index.ts，保持运行，留阶段3）。本期 `utils/env.ts` 只建立，供 `constants/app-config.ts`（阶段3 才建）将来引用。

### 4.6 i18n 骨架（本期不抽文案）

- `zh.json` / `en.json` 仅种极少量 key（如 `app.title`、`common.confirm/cancel/delete/save`），作为 `$t` 全局可用的验证。
- **现有 14 view 的中文硬编码本期不抽取**——阶段6 页面重写时逐页同步抽 key。
- legacy:false（Composition API 模式），`locale: 'zh'`，`fallbackLocale: 'zh'`。
- 语言切换 UI（LanguageChange 组件）留到阶段5；本期仅注册 i18n 引擎。

### 4.7 App.vue / AppLayout.vue / router.ts

**本期均不动**。`App.vue` 继续挂 `<AppLayout/>` + `<Toaster/>`；`AppLayout.vue` 继续 shadcn Sidebar；`router.ts` 继续 hash history 扁平表。这保证功能不回归。

## 5. 验证标准（Definition of Done）

1. `pnpm install` 成功，无 peer dep 冲突报错。
2. `pnpm build`（vue-tsc -b + vite build）通过，无类型错误。
3. `pnpm dev` 起 1420 端口，浏览器加载 app，13 条路由（/kanban, /projects, /resources, /teams, /allocations, /catalog, /dashboard, /calendar, /gantt, /calendar-grid, /ai, /reports, /settings）全部可达，功能与改造前一致。
4. `src/types/auto-import.d.ts`、`src/types/auto-import-components.d.ts` 已生成且入库。
5. Pinia devtools / vue-query devtools（dev 下）可见。
6. 行为零回归：手动走查至少 3 条核心路径（kanban 拖拽建任务、projects 增删、settings 保存）。

## 6. 风险与对策

| 风险 | 对策 |
|---|---|
| auto-import 与现有显式 import 重复导致 lint/类型冲突 | 显式 import 合法，本期不清理；若 eslint 报 no-unused（因 auto-import 已注入）则对受影响文件局部禁用，阶段6 统一清理 |
| lightningcss 改变 CSS 输出，影响 Tailwind v4 产物 | Tailwind v4 原生支持 lightningcss transformer；build 后视觉走查，发现回归可临时回退该插件 |
| TS6 strict + `ignoreDeprecations` 在新 dts 下报新错 | dts 是声明文件，通常不触发 strict；若报错，单独 tsconfig include 控制 |
| pinia-persist 改写现有 store 行为 | 现有 store 未声明 `persist:true`，加插件**不影响**非 persist store；零回归 |
| vue-i18n legacy:false 与现有无 i18n 代码冲突 | 无冲突：i18n 仅新增 `$t`/`t`，未改任何现有组件 |

## 7. 不在本 spec 解决的问题（显式记录，避免越界）

- history 模式（hash vs web）—— 阶段2 决定。
- store 是否改用 vue-query —— 阶段3 决定。
- 阶段6 才需要的 @unovis/embla/motion-v —— 各自阶段安装。
- `Ui*` 标签全量重写 —— 阶段6。
- 现有 list/ 三件套去留 —— 阶段4（与 data-table 比较后定）。
