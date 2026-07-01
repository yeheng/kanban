<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useWorkloadStore } from "@/stores/workload";
import { useResourcesStore } from "@/stores/resources";
import { useProjectsStore } from "@/stores/projects";
import { useTeamsStore } from "@/stores/teams";
import { useUnitStore } from "@/stores/unit";
import { useRefreshStore } from "@/stores/refresh";
import { parseDateStrict } from "@/utils/date";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  TriangleAlertIcon,
  RefreshCwIcon,
  BriefcaseIcon,
  TrendingUpIcon,
  PanelRightCloseIcon,
  PanelRightOpenIcon,
} from "@lucide/vue";
import DateRangePicker from "@/components/DateRangePicker.vue";

const wl = useWorkloadStore();
const resources = useResourcesStore();
const projects = useProjectsStore();
const teams = useTeamsStore();
const unit = useUnitStore();
const dateRange = ref<[number, number]>([parseDateStrict("2026-06-29"), parseDateStrict("2026-07-03")]);
const selectedTeam = ref<number | null>(null);
const allTeamsValue = "__all__";
const loading = ref(false);
const rightPanelOpen = ref(true);
const activeTab = ref("overview");

const tabs = [
  { label: "Overview", value: "overview" },
  { label: "Analytics", value: "analytics", disabled: true },
  { label: "Reports", value: "reports", disabled: true },
  { label: "Notifications", value: "notifications", disabled: true },
];

const teamOptions = computed(() => [
  { label: "全部团队", value: allTeamsValue },
  ...teams.items.map((t) => ({ label: t.name, value: String(t.id) })),
]);

const selectedTeamValue = computed(() =>
  selectedTeam.value == null ? allTeamsValue : String(selectedTeam.value),
);

const selectedTeamLabel = computed(
  () => teamOptions.value.find((o) => o.value === selectedTeamValue.value)?.label ?? "全部团队",
);

const resourceNameById = computed(() => {
  const map = new Map<number, string>();
  for (const r of resources.items) {
    map.set(r.id, r.name);
  }
  return map;
});

function resourceLabel(id: number): string {
  return resourceNameById.value.get(id) ?? `资源 #${id}`;
}

const averageUtilization = computed(() => {
  if (!wl.resourceSummaries.length) return 0;
  const total = wl.resourceSummaries.reduce((sum, s) => sum + s.utilization, 0);
  return total / wl.resourceSummaries.length;
});

async function refresh() {
  const start = fmtDate(dateRange.value[0]);
  const end = fmtDate(dateRange.value[1]);
  loading.value = true;
  try {
    await resources.load();
    await wl.loadResourceSummaries(
      resources.items.map((r) => r.id),
      start,
      end,
    );
    await wl.loadOverloads(start, end);
    if (projects.current != null) await wl.loadProjectBurn(projects.current);
    if (selectedTeam.value != null) await wl.loadTeamSummary(selectedTeam.value, start, end);
  } finally {
    loading.value = false;
  }
}

async function updateSelectedTeam(value: unknown) {
  const str = String(value);
  selectedTeam.value = str === allTeamsValue ? null : Number(str);
  if (selectedTeam.value != null) {
    const ov = await teams.getOverride(selectedTeam.value);
    unit.applyTeamOverride(ov?.pm_workdays ?? null);
  } else {
    unit.applyTeamOverride(null);
  }
  await refresh();
}

onMounted(async () => {
  await wl.loadThresholds();
  await teams.load();
  await refresh();
});

const refreshBus = useRefreshStore();
watch(
  () => refreshBus.version.workload,
  () => {
    void refresh();
  },
);

function fmtDate(ms: number): string {
  const d = new Date(ms);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex flex-col gap-2 md:flex-row md:items-start md:justify-between">
      <div>
        <h1 class="text-2xl font-bold tracking-tight">Dashboard</h1>
        <p class="text-muted-foreground">人力概览与利用率监控</p>
      </div>
      <div class="flex items-center gap-2">
        <DateRangePicker v-model="dateRange" />
        <Button :disabled="loading" @click="refresh">
          <RefreshCwIcon class="mr-2 h-4 w-4" :class="{ 'animate-spin': loading }" />
          刷新
        </Button>
      </div>
    </div>

    <Tabs v-model="activeTab" class="w-full">
      <TabsList>
        <TabsTrigger
          v-for="tab in tabs"
          :key="tab.value"
          :value="tab.value"
          :disabled="tab.disabled"
        >
          {{ tab.label }}
        </TabsTrigger>
      </TabsList>

      <TabsContent value="overview" class="space-y-4">
        <div class="grid gap-4 sm:grid-cols-2">
          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">平均利用率</CardTitle>
              <TrendingUpIcon class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div class="text-2xl font-bold">{{ Math.round(averageUtilization * 100) }}%</div>
              <Progress :model-value="averageUtilization * 100" class="mt-2 h-2" />
            </CardContent>
          </Card>

          <Card>
            <CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle class="text-sm font-medium">项目预算消耗</CardTitle>
              <BriefcaseIcon class="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div v-if="wl.projectBurn" class="text-2xl font-bold">
                {{ Math.round(wl.projectBurn.usage * 100) }}%
              </div>
              <Skeleton v-else class="h-8 w-16" />
              <p class="text-xs text-muted-foreground">
                <span v-if="wl.projectBurn">
                  {{ unit.formatPd(wl.projectBurn.allocated_pd) }} /
                  {{ unit.formatPd(wl.projectBurn.budget_pd) }}
                </span>
                <span v-else>未选择项目</span>
              </p>
            </CardContent>
          </Card>
        </div>

        <div class="flex gap-2">
          <Card class="flex-1 min-w-0">
            <CardHeader>
              <CardTitle>资源利用率</CardTitle>
              <CardDescription>各资源在选定窗口内的负载情况</CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>资源</TableHead>
                    <TableHead class="w-[240px]">利用率</TableHead>
                    <TableHead>负载 / 容量</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="s in wl.resourceSummaries" :key="s.resource_id">
                    <TableCell class="font-medium">{{ resourceLabel(s.resource_id) }}</TableCell>
                    <TableCell>
                      <div class="flex items-center gap-2">
                        <Progress :model-value="Math.min(s.utilization * 100, 100)" class="h-2 flex-1" />
                        <span class="text-xs w-10 text-right">{{ Math.round(s.utilization * 100) }}%</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ unit.formatPd(s.workload_pd) }} / {{ unit.formatPd(s.capacity_pd) }}
                    </TableCell>
                  </TableRow>
                  <TableRow v-if="!wl.resourceSummaries.length">
                    <TableCell colspan="3" class="text-center text-muted-foreground py-6">
                      暂无数据，请调整日期范围后刷新
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>

          <div class="flex flex-shrink-0">
            <div
              class="relative flex items-center justify-center w-5 cursor-pointer group"
              @click="rightPanelOpen = !rightPanelOpen"
            >
              <Separator orientation="vertical" class="h-full bg-border/60 group-hover:bg-border" />
              <Button
                variant="ghost"
                size="icon"
                class="absolute h-6 w-6 rounded-full opacity-60 group-hover:opacity-100 transition-opacity"
                :aria-label="rightPanelOpen ? '收起右侧面板' : '展开右侧面板'"
              >
                <PanelRightCloseIcon v-if="rightPanelOpen" class="h-4 w-4" />
                <PanelRightOpenIcon v-else class="h-4 w-4" />
              </Button>
            </div>

            <div
              v-if="rightPanelOpen"
              class="w-72 lg:w-80 space-y-4 pl-2 overflow-auto"
            >
              <Card>
                <CardHeader>
                  <CardTitle>过载预警</CardTitle>
                  <CardDescription>利用率超过阈值的资源</CardDescription>
                </CardHeader>
                <CardContent class="space-y-2">
                  <Alert v-for="o in wl.overloads" :key="o.resource_id" variant="destructive">
                    <TriangleAlertIcon class="h-4 w-4" />
                    <AlertTitle>{{ resourceLabel(o.resource_id) }}</AlertTitle>
                    <AlertDescription>利用率 {{ Math.round(o.utilization * 100) }}%</AlertDescription>
                  </Alert>
                  <p v-if="!wl.overloads.length" class="text-sm text-muted-foreground">无过载资源 🎉</p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>团队利用率</CardTitle>
                  <CardDescription>按团队查看整体负载</CardDescription>
                </CardHeader>
                <CardContent class="space-y-4">
                  <Select :model-value="selectedTeamValue" @update:model-value="updateSelectedTeam">
                    <SelectTrigger>
                      <SelectValue placeholder="选择团队" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="opt in teamOptions"
                        :key="opt.value"
                        :value="opt.value"
                      >
                        {{ opt.label }}
                      </SelectItem>
                    </SelectContent>
                  </Select>

                  <div v-if="wl.teamSummary">
                    <div class="flex items-center justify-between text-sm mb-1">
                      <span>整体利用率</span>
                      <span class="font-medium">{{ Math.round(wl.teamSummary.utilization * 100) }}%</span>
                    </div>
                    <Progress :model-value="wl.teamSummary.utilization * 100" class="h-2" />
                    <Separator class="my-3" />
                    <div class="text-xs text-muted-foreground">
                      过载成员：
                      <Badge
                        v-for="id in wl.teamSummary.overloaded_members"
                        :key="id"
                        variant="destructive"
                        class="mr-1"
                      >
                        #{{ id }}
                      </Badge>
                      <span v-if="!wl.teamSummary.overloaded_members.length">无</span>
                    </div>
                  </div>
                  <p v-else class="text-sm text-muted-foreground">选择团队后查看利用率</p>
                </CardContent>
              </Card>
            </div>
          </div>
        </div>
      </TabsContent>
    </Tabs>
  </div>
</template>
