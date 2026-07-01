import type { Router } from "vue-router";

import { setupCommonGuard } from "./common-guard";

export function setupRouterGuard(router: Router) {
  setupCommonGuard(router);
  // auth guard 不做：kanban 无 auth（YAGNI）
}
