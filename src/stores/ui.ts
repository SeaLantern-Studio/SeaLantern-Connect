import { defineStore } from "pinia";
import { ref } from "vue";
import type { IncomingInvite } from "@domain/invitations";

export type SectionId = "create" | "join" | "openfrp" | "sakurafrp" | "personalize" | "settings";

const ACTIVE_SECTION_STORAGE_KEY = "sealantern.active-section";
const sectionIds: readonly SectionId[] = [
  "create",
  "join",
  "openfrp",
  "sakurafrp",
  "personalize",
  "settings",
];

function isSectionId(value: string): value is SectionId {
  return (sectionIds as readonly string[]).includes(value);
}

function loadActiveSection(): SectionId {
  try {
    const stored = window.localStorage.getItem(ACTIVE_SECTION_STORAGE_KEY);
    return stored && isSectionId(stored) ? stored : "create";
  } catch {
    return "create";
  }
}

export const useUiStore = defineStore("ui", () => {
  const activeSection = ref<SectionId>(loadActiveSection());
  const sidebarCollapsed = ref(false);
  const incomingInvite = ref<IncomingInvite | null>(null);
  let nextInviteId = 0;

  function setActiveSection(section: SectionId): void {
    activeSection.value = section;
    try {
      window.localStorage.setItem(ACTIVE_SECTION_STORAGE_KEY, section);
    } catch {
      // The current session still keeps the selected page when storage is unavailable.
    }
  }

  function navigate(section: string): void {
    if (!isSectionId(section)) return;
    setActiveSection(section);
  }

  function toggleSidebar(): void {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function importInvite(uri: string): void {
    incomingInvite.value = { id: ++nextInviteId, uri };
    setActiveSection("join");
  }

  function consumeIncomingInvite(id: number): void {
    if (incomingInvite.value?.id === id) incomingInvite.value = null;
  }

  return {
    activeSection,
    sidebarCollapsed,
    incomingInvite,
    navigate,
    toggleSidebar,
    importInvite,
    consumeIncomingInvite,
  };
});
