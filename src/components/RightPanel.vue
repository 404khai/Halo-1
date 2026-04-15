<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import batteryFullIcon from "../assets/battery-full.svg";
import bellIcon from "../assets/bell.svg";
import cpuIcon from "../assets/cpu.svg";
import ramIcon from "../assets/graphics-card.svg";
import muteIcon from "../assets/mute.svg";
import powerIcon from "../assets/power.svg";
import speakerIcon from "../assets/speaker.svg";
import wifiIcon from "../assets/wifi.svg";
import wifiSlashIcon from "../assets/wifi_slash.svg";
import { useSystemStatsStore } from "../stores/systemStats";
import SystemMetricChip from "./SystemMetricChip.vue";

const store = useSystemStatsStore();

const formatBinarySize = (bytes: number) => {
  if (bytes <= 0) {
    return "0 B";
  }

  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const precision = value >= 10 || unitIndex === 0 ? 0 : 1;
  return `${value.toFixed(precision)} ${units[unitIndex]}`;
};

const formatNetworkSpeed = (bytesPerSecond: number) => `${formatBinarySize(bytesPerSecond)}/s`;

const batteryValue = computed(() => {
  const { battery } = store.stats;

  if (!battery.available || battery.percentage === null) {
    return "N/A";
  }

  return battery.status === "Charging" ? `${battery.percentage}%+` : `${battery.percentage}%`;
});

const volumeValue = computed(() => {
  const { volume } = store.stats;

  if (!volume.available || volume.levelPercent === null) {
    return "N/A";
  }

  return volume.muted ? "Muted" : `${volume.levelPercent}%`;
});

const volumeIcon = computed(() => (store.stats.volume.muted ? muteIcon : speakerIcon));

const networkValue = computed(() => {
  const { network } = store.stats;

  if (!network.connected) {
    return "Offline";
  }

  return `${formatNetworkSpeed(network.downloadBytesPerSecond)} / ${formatNetworkSpeed(
    network.uploadBytesPerSecond,
  )}`;
});

const metrics = computed(() => {
  const { cpu, memory, battery, network, notifications } = store.stats;

  return [
    {
      key: "cpu",
      icon: cpuIcon,
      value: cpu.available ? `${Math.round(cpu.usagePercent)}%` : "N/A",
      tooltip: cpu.available ? `CPU usage: ${cpu.usagePercent.toFixed(1)}%` : "CPU usage unavailable",
      muted: !cpu.available,
      alert: cpu.usagePercent >= 80,
    },
    {
      key: "ram",
      icon: ramIcon,
      value: memory.available ? `${Math.round(memory.usagePercent)}%` : "N/A",
      tooltip: memory.available
        ? `RAM usage: ${formatBinarySize(memory.usedBytes)} of ${formatBinarySize(memory.totalBytes)}`
        : "Memory usage unavailable",
      muted: !memory.available,
      alert: memory.usagePercent >= 85,
    },
    {
      key: "battery",
      icon: batteryFullIcon,
      value: batteryValue.value,
      tooltip: battery.available
        ? `Battery: ${batteryValue.value} (${battery.status})`
        : "Battery information unavailable",
      muted: !battery.available,
      alert: battery.available && battery.percentage !== null && battery.percentage <= 20,
    },
    {
      key: "network",
      icon: network.connected ? wifiIcon : wifiSlashIcon,
      value: networkValue.value,
      tooltip: network.connected
        ? `Download ${formatNetworkSpeed(network.downloadBytesPerSecond)} | Upload ${formatNetworkSpeed(network.uploadBytesPerSecond)}`
        : "Network activity unavailable or offline",
      muted: !network.connected,
    },
    {
      key: "notifications",
      icon: bellIcon,
      value: `${notifications.count}`,
      tooltip: notifications.count === 0 ? "No unread notifications" : `${notifications.count} unread notifications`,
      muted: notifications.count === 0,
    },
  ];
});

onMounted(() => {
  void store.startPolling();
});

onBeforeUnmount(() => {
  store.stopPolling();
});

const handleVolumeToggle = async () => {
  await store.toggleMuted();
};
</script>

<template>
  <section class="panel panel-right" aria-label="System status">
    <SystemMetricChip
      v-for="metric in metrics"
      :key="metric.key"
      :alert="metric.alert"
      :icon="metric.icon"
      :muted="metric.muted"
      :tooltip="metric.tooltip"
      :value="metric.value"
    />

    <button
      class="metric-chip metric-chip-action"
      :class="{ 'metric-chip-muted': store.stats.volume.muted }"
      :data-tooltip="
        store.stats.volume.available
          ? store.stats.volume.muted
            ? 'Unmute system output'
            : 'Mute system output'
          : 'Volume information unavailable'
      "
      type="button"
      @click="handleVolumeToggle"
    >
      <img class="metric-icon metric-icon-large" :src="volumeIcon" alt="Volume toggle icon" />
      <Transition name="metric-fade" mode="out-in">
        <span :key="volumeValue" class="metric-value">{{ volumeValue }}</span>
      </Transition>
    </button>

    <div class="metric-chip metric-chip-action metric-chip-power" data-tooltip="Power status">
      <img class="metric-icon metric-icon-large" :src="powerIcon" alt="Power icon" />
    </div>
  </section>
</template>
