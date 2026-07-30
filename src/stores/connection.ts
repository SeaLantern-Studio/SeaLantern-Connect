import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { emptyConnectStatus, type ConnectStatus } from "../connect";

export const useConnectionStore = defineStore("connection", () => {
  const status = ref<ConnectStatus>({ ...emptyConnectStatus });
  let unlisten: UnlistenFn | null = null;

  const busy = computed(
    () => status.value.phase === "starting" || status.value.phase === "stopping",
  );
  const connected = computed(() => status.value.phase === "active");
  const state = computed(() => {
    if (connected.value) return "active";
    if (busy.value) return "busy";
    return "idle";
  });

  async function initialize(): Promise<void> {
    status.value = await invoke<ConnectStatus>("get_status");
    unlisten?.();
    unlisten = await listen<ConnectStatus>("connect-status", (event) => {
      status.value = event.payload;
    });
  }

  function dispose(): void {
    unlisten?.();
    unlisten = null;
  }

  return { status, busy, connected, state, initialize, dispose };
});
