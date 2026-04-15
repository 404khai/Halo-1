<template>
  <main class="topbar-shell">
    <header class="topbar" data-tauri-drag-region>
      <LeftPanel />
      <CenterPill />
      <RightPanel />
    </header>
  </main>
</template>

<script setup lang="ts">
import CenterPill from "./CenterPill.vue";
import LeftPanel from "./LeftPanel.vue";
import RightPanel from "./RightPanel.vue";
</script>
