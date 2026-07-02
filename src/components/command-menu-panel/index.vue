<script setup lang="ts">
import { CommandDialog, CommandEmpty, CommandInput, CommandList, CommandSeparator } from '@/components/ui/command'
import { Button } from '@/components/ui/button'
import { MenuIcon, SearchIcon } from '@lucide/vue'
import { useEventListener } from '@vueuse/core'
import { useAppNav } from '@/composables/use-app-nav'

import CommandChangeTheme from './command-change-theme.vue'
import CommandToPage from './command-to-page.vue'

const { items: navItems } = useAppNav()
const open = ref(false)

useEventListener('keydown', (event: KeyboardEvent) => {
  if (event.key === 'k' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    handleOpenChange()
  }
})

function handleOpenChange() {
  open.value = !open.value
}

const firstKey = computed(() => navigator?.userAgent.includes('Mac OS') ? '⌘' : 'Ctrl')
</script>

<template>
  <div>
    <div
      class="text-sm items-center justify-between text-muted-foreground border border-border bg-muted/5 px-4 py-2 rounded-md md:min-w-[220px] cursor-pointer hidden md:flex"
      @click="handleOpenChange"
    >
      <div class="flex items-center gap-2">
        <SearchIcon class="size-4" />
        <span class="text-xs font-semibold text-muted-foreground">{{ $t('common.search') }}</span>
      </div>
      <kbd class="pointer-events-none inline-flex h-5 select-none items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100">{{ firstKey }} + k</kbd>
    </div>

    <Button variant="outline" size="icon" class="md:hidden" @click="handleOpenChange">
      <SearchIcon />
    </Button>

    <CommandDialog v-model:open="open">
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>
          <div class="flex flex-col items-center justify-center gap-2 text-muted-foreground py-8">
            <MenuIcon class="size-8 opacity-50" />
            <span class="text-sm">No menu found.</span>
            <span class="text-xs">Try searching for a command or check the spelling.</span>
          </div>
        </CommandEmpty>

        <CommandToPage :items="navItems" @click="handleOpenChange" />
        <CommandSeparator />
        <CommandChangeTheme @click="handleOpenChange" />
      </CommandList>
    </CommandDialog>
  </div>
</template>
