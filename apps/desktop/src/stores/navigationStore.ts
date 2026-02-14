import { create } from "zustand";

type Page = "home" | "friends" | "dm" | "guild" | "dm_group" | "settings";

interface NavigationState {
  currentPage: Page;
  selectedFriendNumber: number | null;
  selectedGuildId: string | null;
  selectedChannelId: string | null;
  selectedDmGroupId: string | null;
  setPage: (page: Page) => void;
  openDM: (friendNumber: number) => void;
  openGuild: (guildId: string) => void;
  openChannel: (guildId: string, channelId: string) => void;
  openDmGroup: (dmGroupId: string) => void;
}

export const useNavigationStore = create<NavigationState>((set) => ({
  currentPage: "friends",
  selectedFriendNumber: null,
  selectedGuildId: null,
  selectedChannelId: null,
  selectedDmGroupId: null,

  setPage: (page) =>
    set({
      currentPage: page,
      selectedFriendNumber: null,
      selectedGuildId: null,
      selectedChannelId: null,
      selectedDmGroupId: null,
    }),

  openDM: (friendNumber) =>
    set({
      currentPage: "dm",
      selectedFriendNumber: friendNumber,
      selectedGuildId: null,
      selectedChannelId: null,
      selectedDmGroupId: null,
    }),

  openGuild: (guildId) =>
    set({
      currentPage: "guild",
      selectedGuildId: guildId,
      selectedFriendNumber: null,
      selectedDmGroupId: null,
    }),

  openChannel: (guildId, channelId) =>
    set({
      currentPage: "guild",
      selectedGuildId: guildId,
      selectedChannelId: channelId,
      selectedFriendNumber: null,
      selectedDmGroupId: null,
    }),

  openDmGroup: (dmGroupId) =>
    set({
      currentPage: "dm_group",
      selectedDmGroupId: dmGroupId,
      selectedFriendNumber: null,
      selectedGuildId: null,
      selectedChannelId: null,
    }),
}));
