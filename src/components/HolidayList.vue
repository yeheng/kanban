<script setup lang="ts">
import { computed, ref } from "vue";
import { CalendarDate, getLocalTimeZone } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { useCalendarStore } from "@/stores/calendar";
import { fmtDate } from "@/utils/date";
import type { DateValue } from "@internationalized/date";

const cal = useCalendarStore();
const day = ref<number | null>(null);
const frac = ref(1);
const name = ref("");

const fracOptions = [
  { label: "全天", value: 1 },
  { label: "半天", value: 0.5 },
];

const dayDate = computed<DateValue | undefined>({
  get() {
    if (day.value == null) return undefined;
    const [y, m, d] = fmtDate(day.value).split("-").map(Number);
    return new CalendarDate(y, m, d);
  },
  set(date) {
    day.value = date?.toDate(getLocalTimeZone()).getTime() ?? null;
  },
});

function onSelectFrac(value: unknown) {
  frac.value = Number(value);
}

async function add() {
  if (day.value == null) return;
  await cal.addHoliday(fmtDate(day.value), frac.value, name.value || null);
  day.value = null;
  name.value = "";
}

const confirmOpen = ref<Record<number, boolean>>({});
</script>

<template>
  <div class="space-y-4">
    <div class="flex flex-wrap items-end gap-4">
      <div class="grid gap-2">
        <Label>日期</Label>
        <Popover>
          <PopoverTrigger as-child>
            <Button variant="outline" class="w-[140px] justify-start">
              {{ day != null ? fmtDate(day) : "选择日期" }}
            </Button>
          </PopoverTrigger>
          <PopoverContent class="w-auto p-0">
            <Calendar v-model="dayDate" />
          </PopoverContent>
        </Popover>
      </div>
      <div class="grid gap-2">
        <Label>类型</Label>
        <Select :model-value="frac" @update:model-value="onSelectFrac">
          <SelectTrigger class="w-[100px]">
            <SelectValue placeholder="类型" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="opt in fracOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="grid gap-2">
        <Label>名称</Label>
        <Input v-model="name" placeholder="节假日名称" class="w-[200px]" />
      </div>
      <Button @click="add">添加节假日</Button>
    </div>

    <div class="border rounded-lg divide-y">
      <div v-for="h in cal.holidays" :key="h.id" class="flex items-center justify-between p-3">
        <div>
          <div class="font-medium">{{ h.day }}</div>
          <div class="text-sm text-muted-foreground">
            {{ h.fraction === 1 ? "全天" : "半天" }} · {{ h.name ?? "" }}
          </div>
        </div>
        <Dialog v-model:open="confirmOpen[h.id]">
          <DialogTrigger as-child>
            <Button variant="destructive" size="sm">删除</Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>删除节假日</DialogTitle>
              <DialogDescription>确定删除此节假日吗？</DialogDescription>
            </DialogHeader>
            <DialogFooter>
              <Button variant="outline" @click="confirmOpen[h.id] = false">取消</Button>
              <Button variant="destructive" @click="cal.removeHoliday(h.id); confirmOpen[h.id] = false">删除</Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    </div>
  </div>
</template>
