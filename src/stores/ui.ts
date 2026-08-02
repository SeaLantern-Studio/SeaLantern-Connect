import { defineStore } from "pinia";
import { ref } from "vue";
import type { IncomingInvite } from "../invitations";

export type SectionId = "create" | "join" | "personalize" | "settings";

export const useUiStore = defineStore("ui", () => {
  const activeSection = ref<SectionId>("create");
  const sidebarCollapsed = ref(false);
  const closePromptOpen = ref(false);
  const incomingInvite = ref<IncomingInvite | null>(null);
  let nextInviteId = 0;
  function navigate(section: string): void {
    if (!(["create", "join", "personalize", "settings"] as string[]).includes(section)) return;
    activeSection.value = section as SectionId;
  }

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function openClosePrompt(): void {
    closePromptOpen.value = true;
  }

  function closeClosePrompt(): void {
    closePromptOpen.value = false;
  }

  function importInvite(uri: string): void {
    incomingInvite.value = { id: ++nextInviteId, uri };
    activeSection.value = "join";
  }

  function consumeIncomingInvite(id: number): void {
    if (incomingInvite.value?.id === id) incomingInvite.value = null;
  }

  return {
    activeSection,
    sidebarCollapsed,
    closePromptOpen,
    incomingInvite,
    navigate,
    toggleSidebar,
    openClosePrompt,
    closeClosePrompt,
    importInvite,
    consumeIncomingInvite,
  };
});
