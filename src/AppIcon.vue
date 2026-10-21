<script setup>
import { computed } from 'vue';

    const ICONS = {
      dashboard: ["M4 20v-7", "M10 20V8", "M16 20v-10", "M2 20h20"],
      calendar: ["M3 4h18v16a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V4z", "M8 2v4", "M16 2v4", "M3 10h18"],
      list: ["M8 6h13", "M8 12h13", "M8 18h13", "M3 6h.01", "M3 12h.01", "M3 18h.01"],
      gear: ["M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z", "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"],
      devoir: ["M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z", "M14 2v6h6", "M16 13H8", "M16 17H8", "M10 9H8"],
      ie: ["M12 20h9", "M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"],
      ds: ["M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z", "M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"],
      exam: ["M12 15a7 7 0 1 0 0-14 7 7 0 0 0 0 14z", "M8.21 13.89 7 23l5-3 5 3-1.21-9.12"],
      'dot-rose': ["M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"],
      'dot-amber': ["M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"],
      'dot-emerald': ["M12 4a8 8 0 1 0 0 16 8 8 0 0 0 0-16z"],
      filter: ["M22 3H2l8 9.46V19l4 2v-8.54L22 3z"],
      search: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z", "M21 21l-4.35-4.35"],
      check: ["M20 6L9 17l-5-5"],
    };

const props = defineProps({ name: { type: String, required: true } });
const estRond = computed(() => props.name.startsWith('dot'));
const chemins = computed(() => ICONS[props.name] || []);
</script>

<template>
  <svg
    :fill="estRond ? 'currentColor' : 'none'"
    :stroke="estRond ? 'none' : 'currentColor'"
    viewBox="0 0 24 24"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true">
    <path v-for="(p, i) in chemins" :key="i" :d="p" />
  </svg>
</template>
