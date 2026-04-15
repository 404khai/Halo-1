import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

export interface CpuStats {
  usagePercent: number;
  available: boolean;
}

export interface MemoryStats {
  usedBytes: number;
  totalBytes: number;
  usagePercent: number;
  available: boolean;
}

export interface BatteryStats {
  percentage: number | null;
  status: string;
  available: boolean;
}

export interface NetworkStats {
  downloadBytesPerSecond: number;
  uploadBytesPerSecond: number;
  connected: boolean;
}

export interface VolumeStats {
  levelPercent: number | null;
  muted: boolean;
  available: boolean;
}

export interface NotificationStats {
  count: number;
}

export interface SystemStatsPayload {
  cpu: CpuStats;
  memory: MemoryStats;
  battery: BatteryStats;
  network: NetworkStats;
  volume: VolumeStats;
  notifications: NotificationStats;
}

const POLL_INTERVAL_MS = 2_000;

const defaultStats = (): SystemStatsPayload => ({
  cpu: { usagePercent: 0, available: false },
  memory: { usedBytes: 0, totalBytes: 0, usagePercent: 0, available: false },
  battery: { percentage: null, status: "Unavailable", available: false },
  network: { downloadBytesPerSecond: 0, uploadBytesPerSecond: 0, connected: false },
  volume: { levelPercent: null, muted: false, available: false },
  notifications: { count: 0 },
});

const isTauriRuntime = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const buildMockStats = (seed: number): SystemStatsPayload => {
  const cpu = 18 + ((seed * 7) % 34);
  const memoryPercent = 37 + ((seed * 5) % 25);
  const memoryTotal = 16 * 1024 ** 3;
  const memoryUsed = Math.round((memoryPercent / 100) * memoryTotal);
  const batteryPercent = 72 - (seed % 4);
  const connected = seed % 5 !== 0;
  const muted = seed % 6 === 0;

  return {
    cpu: { usagePercent: cpu, available: true },
    memory: {
      usedBytes: memoryUsed,
      totalBytes: memoryTotal,
      usagePercent: memoryPercent,
      available: true,
    },
    battery: {
      percentage: batteryPercent,
      status: batteryPercent < 75 ? "Discharging" : "Idle",
      available: true,
    },
    network: {
      downloadBytesPerSecond: connected ? 180_000 + seed * 3_000 : 0,
      uploadBytesPerSecond: connected ? 42_000 + seed * 1_500 : 0,
      connected,
    },
    volume: {
      levelPercent: muted ? 0 : 48 + (seed % 5) * 3,
      muted,
      available: true,
    },
    notifications: {
      count: seed % 3,
    },
  };
};

export const useSystemStatsStore = defineStore("systemStats", () => {
  const stats = ref<SystemStatsPayload>(defaultStats());
  const isLoading = ref(false);
  const errorMessage = ref<string | null>(null);
  const isUsingMockData = ref(false);
  const lastUpdatedAt = ref<number | null>(null);

  let intervalId: number | null = null;
  let mockSeed = 0;

  const hasData = computed(() => lastUpdatedAt.value !== null);

  const refresh = async () => {
    isLoading.value = !hasData.value;

    try {
      if (isTauriRuntime()) {
        stats.value = await invoke<SystemStatsPayload>("get_system_stats");
        isUsingMockData.value = false;
      } else {
        mockSeed += 1;
        stats.value = buildMockStats(mockSeed);
        isUsingMockData.value = true;
      }

      errorMessage.value = null;
      lastUpdatedAt.value = Date.now();
    } catch (error) {
      errorMessage.value =
        error instanceof Error ? error.message : "Unable to refresh system statistics.";
      isUsingMockData.value = true;

      if (!hasData.value) {
        mockSeed += 1;
        stats.value = buildMockStats(mockSeed);
        lastUpdatedAt.value = Date.now();
      }
    } finally {
      isLoading.value = false;
    }
  };

  const startPolling = async () => {
    if (intervalId !== null) {
      return;
    }

    await refresh();
    intervalId = window.setInterval(() => {
      void refresh();
    }, POLL_INTERVAL_MS);
  };

  const stopPolling = () => {
    if (intervalId !== null) {
      window.clearInterval(intervalId);
      intervalId = null;
    }
  };

  const toggleMuted = async () => {
    const nextMuted = !stats.value.volume.muted;

    try {
      if (isTauriRuntime()) {
        await invoke("set_volume_muted", { muted: nextMuted });
      }

      stats.value = {
        ...stats.value,
        volume: {
          ...stats.value.volume,
          muted: nextMuted,
          available: true,
          levelPercent:
            stats.value.volume.levelPercent === null
              ? nextMuted
                ? 0
                : 50
              : stats.value.volume.levelPercent,
        },
      };
    } catch (error) {
      errorMessage.value =
        error instanceof Error ? error.message : "Unable to update mute state.";
    }
  };

  return {
    errorMessage,
    hasData,
    isLoading,
    isUsingMockData,
    lastUpdatedAt,
    refresh,
    startPolling,
    stats,
    stopPolling,
    toggleMuted,
  };
});
