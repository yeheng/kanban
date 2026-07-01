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
