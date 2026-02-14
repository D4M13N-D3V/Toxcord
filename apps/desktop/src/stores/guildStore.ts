import { create } from "zustand";
import * as api from "../api/tox";
import type { GuildInfo, ChannelInfo, GuildMember } from "../api/tox";

interface GuildInvite {
  friendNumber: number;
  inviteData: number[];
  groupName: string;
}

interface GuildState {
  guilds: GuildInfo[];
  dmGroups: GuildInfo[];
  channels: Record<string, ChannelInfo[]>;
  members: Record<string, GuildMember[]>;
  selectedGuildId: string | null;
  selectedChannelId: string | null;
  pendingInvites: GuildInvite[];

  loadGuilds: () => Promise<void>;
  loadDmGroups: () => Promise<void>;
  createGuild: (name: string) => Promise<void>;
  createDmGroup: (name: string, friendNumbers: number[]) => Promise<GuildInfo>;
  leaveGuild: (guildId: string) => Promise<void>;
  renameGuild: (guildId: string, name: string) => Promise<void>;
  renameChannel: (guildId: string, channelId: string, name: string) => Promise<void>;
  selectGuild: (guildId: string | null) => void;
  selectChannel: (channelId: string | null) => void;
  loadChannels: (guildId: string) => Promise<void>;
  loadMembers: (guildId: string) => Promise<void>;
  createChannel: (guildId: string, name: string) => Promise<void>;
  addGuildInvite: (invite: GuildInvite) => void;
  acceptInvite: (invite: GuildInvite) => Promise<void>;
  dismissInvite: (friendNumber: number) => void;
  updateGuildFromEvent: (groupNumber: number, update: Partial<GuildInfo>) => void;
  addMember: (groupNumber: number, member: GuildMember) => void;
  removeMember: (groupNumber: number, peerId: number) => void;
  updateMemberName: (groupNumber: number, peerId: number, name: string) => void;
  updateMemberStatus: (groupNumber: number, peerId: number, status: string) => void;
  refreshChannels: (guildId: string) => Promise<void>;
}

export const useGuildStore = create<GuildState>((set, get) => ({
  guilds: [],
  dmGroups: [],
  channels: {},
  members: {},
  selectedGuildId: null,
  selectedChannelId: null,
  pendingInvites: [],

  loadGuilds: async () => {
    try {
      const guilds = await api.getGuilds();
      set({ guilds });
    } catch (e) {
      console.error("Failed to load guilds:", e);
    }
  },

  loadDmGroups: async () => {
    try {
      const dmGroups = await api.getDmGroups();
      set({ dmGroups });
    } catch (e) {
      console.error("Failed to load DM groups:", e);
    }
  },

  createGuild: async (name) => {
    try {
      const guild = await api.createGuild(name);
      set((s) => ({ guilds: [...s.guilds, guild] }));
      // Auto-select the new guild
      get().selectGuild(guild.id);
    } catch (e) {
      console.error("Failed to create guild:", e);
      throw e;
    }
  },

  createDmGroup: async (name, friendNumbers) => {
    try {
      const dmGroup = await api.createDmGroup(name, friendNumbers);
      set((s) => ({ dmGroups: [...s.dmGroups, dmGroup] }));
      return dmGroup;
    } catch (e) {
      console.error("Failed to create DM group:", e);
      throw e;
    }
  },

  leaveGuild: async (guildId) => {
    try {
      await api.leaveGuild(guildId);
      set((s) => {
        // Clean up channels and members for this guild
        const { [guildId]: _channels, ...remainingChannels } = s.channels;
        const { [guildId]: _members, ...remainingMembers } = s.members;
        return {
          guilds: s.guilds.filter((g) => g.id !== guildId),
          dmGroups: s.dmGroups.filter((g) => g.id !== guildId),
          channels: remainingChannels,
          members: remainingMembers,
          selectedGuildId: s.selectedGuildId === guildId ? null : s.selectedGuildId,
          selectedChannelId: s.selectedGuildId === guildId ? null : s.selectedChannelId,
        };
      });
    } catch (e) {
      console.error("Failed to leave guild:", e);
      throw e;
    }
  },

  renameGuild: async (guildId, name) => {
    try {
      await api.renameGuild(guildId, name);
      set((s) => ({
        guilds: s.guilds.map((g) =>
          g.id === guildId ? { ...g, name } : g,
        ),
      }));
    } catch (e) {
      console.error("Failed to rename guild:", e);
      throw e;
    }
  },

  renameChannel: async (guildId, channelId, name) => {
    try {
      await api.renameChannel(channelId, name);
      set((s) => ({
        channels: {
          ...s.channels,
          [guildId]: (s.channels[guildId] ?? []).map((c) =>
            c.id === channelId ? { ...c, name } : c,
          ),
        },
      }));
    } catch (e) {
      console.error("Failed to rename channel:", e);
      throw e;
    }
  },

  selectGuild: (guildId) => {
    set({ selectedGuildId: guildId, selectedChannelId: null });
    if (guildId) {
      get().loadChannels(guildId);
      get().loadMembers(guildId);
    }
  },

  selectChannel: (channelId) => {
    set({ selectedChannelId: channelId });
  },

  loadChannels: async (guildId) => {
    try {
      const channels = await api.getGuildChannels(guildId);
      set((s) => ({
        channels: { ...s.channels, [guildId]: channels },
      }));
    } catch (e) {
      console.error("Failed to load channels:", e);
    }
  },

  loadMembers: async (guildId) => {
    try {
      const members = await api.getGuildMembers(guildId);
      set((s) => ({
        members: { ...s.members, [guildId]: members },
      }));
    } catch (e) {
      console.error("Failed to load members:", e);
    }
  },

  createChannel: async (guildId, name) => {
    try {
      const channel = await api.createChannel(guildId, name);
      set((s) => ({
        channels: {
          ...s.channels,
          [guildId]: [...(s.channels[guildId] ?? []), channel],
        },
      }));
    } catch (e) {
      console.error("Failed to create channel:", e);
      throw e;
    }
  },

  addGuildInvite: (invite) => {
    set((s) => ({
      pendingInvites: [...s.pendingInvites.filter(
        (i) => i.friendNumber !== invite.friendNumber,
      ), invite],
    }));
  },

  acceptInvite: async (invite) => {
    try {
      const guild = await api.acceptGuildInvite(
        invite.friendNumber,
        invite.inviteData,
        invite.groupName,
      );
      // Add to the correct list based on guild_type
      if (guild.guild_type === "dm_group") {
        set((s) => ({
          dmGroups: [...s.dmGroups, guild],
          pendingInvites: s.pendingInvites.filter(
            (i) => i.friendNumber !== invite.friendNumber,
          ),
        }));
      } else {
        set((s) => ({
          guilds: [...s.guilds, guild],
          pendingInvites: s.pendingInvites.filter(
            (i) => i.friendNumber !== invite.friendNumber,
          ),
        }));
      }
    } catch (e) {
      console.error("Failed to accept invite:", e);
      throw e;
    }
  },

  dismissInvite: (friendNumber) => {
    set((s) => ({
      pendingInvites: s.pendingInvites.filter(
        (i) => i.friendNumber !== friendNumber,
      ),
    }));
  },

  updateGuildFromEvent: (_groupNumber, _update) => {
    // Will be used for topic changes etc.
  },

  addMember: (groupNumber, member) => {
    const guild = get().guilds.find((g) => g.group_number === groupNumber);
    const dmGroup = get().dmGroups.find((g) => g.group_number === groupNumber);
    const group = guild || dmGroup;
    if (!group) return;
    set((s) => ({
      members: {
        ...s.members,
        [group.id]: [
          ...(s.members[group.id] ?? []).filter((m) => m.peer_id !== member.peer_id),
          member,
        ],
      },
    }));
  },

  removeMember: (groupNumber, peerId) => {
    const guild = get().guilds.find((g) => g.group_number === groupNumber);
    const dmGroup = get().dmGroups.find((g) => g.group_number === groupNumber);
    const group = guild || dmGroup;
    if (!group) return;
    set((s) => ({
      members: {
        ...s.members,
        [group.id]: (s.members[group.id] ?? []).filter((m) => m.peer_id !== peerId),
      },
    }));
  },

  updateMemberName: (groupNumber, peerId, name) => {
    const guild = get().guilds.find((g) => g.group_number === groupNumber);
    const dmGroup = get().dmGroups.find((g) => g.group_number === groupNumber);
    const group = guild || dmGroup;
    if (!group) return;
    set((s) => ({
      members: {
        ...s.members,
        [group.id]: (s.members[group.id] ?? []).map((m) =>
          m.peer_id === peerId ? { ...m, name } : m,
        ),
      },
    }));
  },

  updateMemberStatus: (groupNumber, peerId, status) => {
    const guild = get().guilds.find((g) => g.group_number === groupNumber);
    const dmGroup = get().dmGroups.find((g) => g.group_number === groupNumber);
    const group = guild || dmGroup;
    if (!group) return;
    set((s) => ({
      members: {
        ...s.members,
        [group.id]: (s.members[group.id] ?? []).map((m) =>
          m.peer_id === peerId ? { ...m, status } : m,
        ),
      },
    }));
  },

  refreshChannels: async (guildId) => {
    try {
      const channels = await api.getGuildChannels(guildId);
      set((s) => ({
        channels: { ...s.channels, [guildId]: channels },
      }));
    } catch (e) {
      console.error("Failed to refresh channels:", e);
    }
  },
}));
