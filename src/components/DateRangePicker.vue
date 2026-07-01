<script setup lang="ts">
import { computed } from "vue";
import { CalendarIcon } from "@lucide/vue";
import { CalendarDate, type DateValue } from "@internationalized/date";
import { Button } from "@/components/ui/button";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { RangeCalendar } from "@/components/ui/range-calendar";
import { cn } from "@/lib/utils";
import { fmtDate, parseDate } from "@/utils/date";

const props = defineProps<{
  modelValue: [number, number];
  class?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: [number, number]): void;
}>();

function toDateValue(ms: number): DateValue {
  const s = fmtDate(ms);
  const [year, month, day] = s.split("-").map(Number);
  return new CalendarDate(year, month, day);
}

function fromDateValue(dv: DateValue): number {
  return parseDate(`${dv.year}-${String(dv.month).padStart(2, "0")}-${String(dv.day).padStart(2, "0")}`) ?? Date.now();
}

const dateRange = computed({
  get: () => {
    const start = toDateValue(props.modelValue[0]);
    const end = toDateValue(props.modelValue[1]);
    return { start, end };
  },
  set: (range) => {
    if (range.start && range.end) {
      emit("update:modelValue", [fromDateValue(range.start), fromDateValue(range.end)]);
    }
  },
});

const displayText = computed(() => {
  const start = fmtDate(props.modelValue[0]);
  const end = fmtDate(props.modelValue[1]);
  return `${start} ~ ${end}`;
});
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <Button
        variant="outline"
        :class="cn('w-auto justify-start text-left font-normal', props.class)"
      >
        <CalendarIcon class="mr-2 h-4 w-4" />
        {{ displayText }}
      </Button>
    </PopoverTrigger>
    <PopoverContent class="w-auto p-0" align="start">
      <RangeCalendar v-model="dateRange" initial-focus />
    </PopoverContent>
  </Popover>
</template>
