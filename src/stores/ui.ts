import { defineStore } from "pinia";
import { computed, reactive, ref } from "vue";
import type { IncomingInvite } from "../connect";

export type SectionId = "create" | "join" | "personalize" | "settings";
export type SettingsSectionId = Extract<SectionId, "personalize" | "settings">;

interface SaveState {
  enabled: boolean;
  saving: boolean;
}

type SaveAction = () => Promise<void>;

export const useUiStore = defineStore("ui", () => {
  const activeSection = ref<SectionId>("join");
  const sidebarCollapsed = ref(false);
  const closePromptOpen = ref(false);
  const incomingInvite = ref<IncomingInvite | null>(null);
  let nextInviteId = 0;
  const saveStates = reactive<Record<SettingsSectionId, SaveState>>({
    personalize: { enabled: false, saving: false },
    settings: { enabled: false, saving: false },
  });
  const saveActions = new Map<SettingsSectionId, SaveAction>();

  const showsSaveButton = computed(
    () => activeSection.value === "personalize" || activeSection.value === "settings",
  );
  const activeSaveState = computed<SaveState>(() => {
    if (!showsSaveButton.value) return { enabled: false, saving: false };
    return saveStates[activeSection.value as SettingsSectionId];
  });

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

  function registerSaveAction(section: SettingsSectionId, action: SaveAction): void {
    saveActions.set(section, action);
  }

  function unregisterSaveAction(section: SettingsSectionId, action: SaveAction): void {
    if (saveActions.get(section) === action) saveActions.delete(section);
  }

  function setSaveState(section: SettingsSectionId, enabled: boolean, saving: boolean): void {
    saveStates[section].enabled = enabled;
    saveStates[section].saving = saving;
  }

  async function saveActiveSection(): Promise<void> {
    if (!showsSaveButton.value || !activeSaveState.value.enabled) return;
    await saveActions.get(activeSection.value as SettingsSectionId)?.();
  }

  return {
    activeSection,
    sidebarCollapsed,
    closePromptOpen,
    incomingInvite,
    showsSaveButton,
    activeSaveState,
    navigate,
    toggleSidebar,
    openClosePrompt,
    closeClosePrompt,
    importInvite,
    consumeIncomingInvite,
    registerSaveAction,
    unregisterSaveAction,
    setSaveState,
    saveActiveSection,
  };
});
