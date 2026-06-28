import { describe, it, expect, vi, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useWorkloadStore } from "./workload";
import { api } from "../api";

vi.mock("../api", () => ({ api: { getThresholds: vi.fn() } }));
beforeEach(() => { setActivePinia(createPinia()); vi.mocked(api.getThresholds).mockReset(); });

describe("workload store", () => {
  it("loads thresholds and bands utilization", async () => {
    vi.mocked(api.getThresholds).mockResolvedValue({ overload: 1.1, underload: 0.5, green: 0.7, yellow: 1.0 });
    const s = useWorkloadStore();
    await s.loadThresholds();
    expect(s.band(0.69)).toBe("under"); // < green
    expect(s.band(0.75)).toBe("green"); // >= green
    expect(s.band(1.0)).toBe("yellow"); // >= yellow
    expect(s.band(1.2)).toBe("red");    // >= overload
  });
});
