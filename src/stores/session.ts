import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { getStatus, onStatus } from "@api";
import { emptyConnectStatus, type ConnectStatus } from "../models/tunnel";

export const useSessionStore = defineStore("session", () => {
  const status = ref<ConnectStatus>({ ...emptyConnectStatus });
  let unlisten: (() => void) | null = null;

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
    status.value = await getStatus();
    unlisten?.();
    unlisten = await onStatus((next) => (status.value = next));
  }

  function dispose(): void {
    unlisten?.();
    unlisten = null;
  }

  return { status, busy, connected, state, initialize, dispose };
});
