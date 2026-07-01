import { setupLayouts } from "virtual:generated-layouts";
import { createRouter, createWebHashHistory } from "vue-router";
import { handleHotUpdate, routes } from "vue-router/auto-routes";

import { setupRouterGuard } from "./guard";

// 根路径重定向到看板，与旧 router.ts 的 { path: "/", redirect: "/kanban" } 行为一致。
// 注意：redirect 记录必须放在 setupLayouts 之【外】——setupLayouts 会给每个 top-level
// route 套上 layout 组件并把它降为 child，对无 component 的 redirect 记录会破坏其语义。
// 故先让 setupLayouts 只处理真实 page 路由，再把 redirect 作为顶层 sibling 前置。
const router = createRouter({
  history: createWebHashHistory(),
  routes: [{ path: "/", redirect: "/kanban" }, ...setupLayouts(routes)],
  scrollBehavior() {
    return { left: 0, top: 0, behavior: "smooth" };
  },
});

setupRouterGuard(router);

export default router;

if (import.meta.hot) {
  handleHotUpdate(router);
}
