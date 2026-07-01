<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { CalendarIcon } from "@lucide/vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { useCalendarStore } from "@/stores/calendar";
import { useResourcesStore } from "@/stores/resources";
import { fmtDate, parseDate } from "@/utils/date";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const cal = useCalendarStore();
const resources = useResourcesStore();
const rid = ref<number | null>(null);
const day = ref<number | null>(null);
const frac = ref(1);
const reason = ref("");

const resourceOptions = computed(() =>
  resources.items.map((r) => ({ label: r.name, value: r.id })),
);
const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

function resourceName(id: number): string {
  return resources.items.find((r) => r.id === id)?.name ?? `#${id}`;
}

function toDateValue(ms: number): DateValue {
  const s = fmtDate(ms);
  const [year, month, dayOfMonth] = s.split("-").map(Number);
  return new CalendarDate(year, month, dayOfMonth);
}

function fromDateValue(dv: DateValue): number {
  return (
    parseDate(`${dv.year}-${String(dv.month).padStart(2, "0")}-${String(dv.day).padStart(2, "0")}`) ??
    Date.now()
  );
}

const dateValue = computed<DateValue | undefined>({
  get: () => (day.value == null ? undefined : toDateValue(day.value)),
  set: (dv) => {
    day.value = dv ? fromDateValue(dv) : null;
  },
});

const dateDisplay = computed(() =>
  day.value == null ? "选择日期" : fmtDate(day.value),
);

function updateRid(value: unknown) {
  rid.value = typeof value === "number" ? value : null;
}

function updateFrac(value: unknown) {
  frac.value = typeof value === "number" ? value : 1;
}

async function add() {
  if (rid.value == null || day.value == null) return;
  await cal.addTimeOff(rid.value, fmtDate(day.value), frac.value, reason.value || null);
  day.value = null;
  reason.value = "";
}

onMounted(() => cal.loadTimeOff());
</script>

<template>
  <div class="space-y-4">
    <div class="flex flex-wrap items-end gap-4">
      <div class="grid gap-2">
        <Label>资源</Label>
        <Select :model-value="rid ?? undefined" @update:model-value="updateRid">
          <SelectTrigger class="w-48">
            <SelectValue placeholder="选择资源" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="o in resourceOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="grid gap-2">
        <Label>日期</Label>
        <Popover>
          <PopoverTrigger as-child>
            <Button variant="outline" class="w-40 justify-start text-left font-normal">
              <CalendarIcon class="mr-2 h-4 w-4" />
              {{ dateDisplay }}
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-auto p-0">
            <Calendar v-model="dateValue" />
          </PopoverContent>
        </Popover>
      </div>

      <div class="grid gap-2">
        <Label>类型</Label>
        <Select :model-value="frac" @update:model-value="updateFrac">
          <SelectTrigger class="w-[100px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="o in fracOptions" :key="o.value" :value="o.value">
              {{ o.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div class="grid gap-2">
        <Label>原因</Label>
        <Input v-model="reason" placeholder="请假原因" class="w-48" />
      </div>

      <Button @click="add">添加请假</Button>
    </div>

    <div class="space-y-2">
      <div
        v-for="t in cal.timeOff"
        :key="t.id"
        class="flex items-center justify-between gap-4 rounded-lg border p-3"
      >
        <div>
          <div class="font-medium">{{ resourceName(t.resource_id) }} · {{ t.day }}</div>
          <div class="mt-1 flex items-center gap-2">
            <Badge variant="secondary">{{ t.fraction === 1 ? "全天" : "半天" }}</Badge>
            <span v-if="t.reason" class="text-xs text-muted-foreground">{{ t.reason }}</span>
          </div>
        </div>

        <Dialog>
          <DialogTrigger as-child>
            <Button variant="destructive" size="sm">删除</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>确认删除</DialogTitle>
              <DialogDescription>确定删除此请假记录吗？</DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <DialogClose as-child>
                <Button variant="outline">取消</Button>
              </DialogClose>
              <DialogClose as-child>
                <Button variant="destructive" @click="cal.removeTimeOff(t.id)">确定</Button>
              </DialogClose>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>

    <p v-if="!cal.timeOff.length" class="text-sm text-muted-foreground">暂无请假记录。</p>
  </div>
</template>
