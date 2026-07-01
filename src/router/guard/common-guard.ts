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
