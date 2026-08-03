import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { getP2pStatus, onP2pStatus } from "@api";
import { emptyP2pStatus, type P2pStatus } from "@models/p2p";

export const useSessionStore = defineStore("session", () => {
  const status = ref<P2pStatus>({ ...emptyP2pStatus });
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
    status.value = await getP2pStatus();
    unlisten?.();
    unlisten = await onP2pStatus((next) => (status.value = next));
  }

  function dispose(): void {
    unlisten?.();
    unlisten = null;
  }

  return { status, busy, connected, state, initialize, dispose };
});
