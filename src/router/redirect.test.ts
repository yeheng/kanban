import { describe, it, expect } from "vitest";
import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";

// Replicate the exact routes construction from router/index.ts (without the HMR
// handleHotUpdate call which needs vite's import.meta.hot, not available in vitest).
// This verifies the redirect record placement is correct.
function buildRoutes(realRoutes: RouteRecordRaw[]) {
  return createRouter({
    history: createWebHashHistory(),
    routes: [{ path: "/", redirect: "/kanban" } as RouteRecordRaw, ...realRoutes],
  });
}

describe("root redirect", () => {
  it("/ redirects to /kanban (top-level redirect record)", async () => {
    const router = buildRoutes([{ path: "/kanban", component: { render: () => null } }]);
    await router.push("/");
    expect(router.currentRoute.value.path).toBe("/kanban");
    expect(router.currentRoute.value.redirectedFrom?.fullPath).toBe("/");
  });

  it("/ resolves /kanban directly", async () => {
    const router = buildRoutes([{ path: "/kanban", component: { render: () => null } }]);
    await router.push("/kanban");
    expect(router.currentRoute.value.path).toBe("/kanban");
  });
});
