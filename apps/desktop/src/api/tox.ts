import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface ProfileInfo {
  tox_id: string;
  name: string;
  status_message: string;
}

export interface ConnectionInfo {
  connected: boolean;
  status: "none" | "tcp" | "udp";
}

export interface FriendInfo {
  friend_number: number;
  public_key: string;
  name: string;
  status_message: string;
  user_status: "none" | "online" | "away" | "busy";
  connection_status: "none" | "tcp" | "udp";
  last_seen: string | null;
  notes: string;
}

export interface FriendRequest {
  public_key: string;
  message: string;
  received_at: string;
}

export interface DirectMessage {
  id: string;
  friend_number: number;
  sender: string;
  content: string;
  message_type: string;
  timestamp: string;
  is_outgoing: boolean;
  delivered: boolean;
  read: boolean;
}

export interface SendMessageResult {
  id: string;
  timestamp: string;
  delivered: boolean;
  queued: boolean;
  error?: string;
}

// ─── Guild types ──────────────────────────────────────────────────

export interface GuildInfo {
  id: string;
  name: string;
  group_number: number | null;
  owner_public_key: string;
  guild_type: "server" | "dm_group";
  created_at: string;
}

export interface ChannelInfo {
  id: string;
  guild_id: string;
  name: string;
  topic: string;
  channel_type: string;
  position: number;
}

export interface ChannelMessage {
  id: string;
  channel_id: string;
  sender_public_key: string;
  sender_name: string;
  content: string;
  message_type: string;
  timestamp: string;
  is_own: boolean;
}

export interface GuildMember {
  peer_id: number;
  name: string;
  public_key: string;
  role: string;
  status: string;
}

export type ToxEvent =
  | { type: "ConnectionStatus"; data: { connected: boolean; status: string } }
  | { type: "FriendRequest"; data: { public_key: string; message: string } }
  | { type: "FriendMessage"; data: { friend_number: number; message_type: string; message: string; id: string; timestamp: string } }
  | { type: "FriendName"; data: { friend_number: number; name: string } }
  | { type: "FriendStatusMessage"; data: { friend_number: number; message: string } }
  | { type: "FriendStatus"; data: { friend_number: number; status: string } }
  | { type: "FriendConnectionStatus"; data: { friend_number: number; connected: boolean; status: string } }
  | { type: "FriendTyping"; data: { friend_number: number; is_typing: boolean } }
  | { type: "GroupInvite"; data: { friend_number: number; invite_data: number[]; group_name: string } }
  | { type: "GroupSelfJoin"; data: { group_number: number } }
  | { type: "GroupJoinFail"; data: { group_number: number; fail_type: string } }
  | { type: "GroupPeerJoin"; data: { group_number: number; peer_id: number; name: string; public_key: string } }
  | { type: "GroupPeerExit"; data: { group_number: number; peer_id: number; name: string } }
  | { type: "GroupPeerName"; data: { group_number: number; peer_id: number; name: string } }
  | { type: "GroupMessage"; data: { group_number: number; peer_id: number; sender_name: string; sender_pk: string; message: string; message_type: string; id: string; timestamp: string; channel_id: string } }
  | { type: "GroupTopicChange"; data: { group_number: number; topic: string } }
  | { type: "GroupCustomPacket"; data: { group_number: number; peer_id: number; data: number[] } }
  | { type: "GroupPeerStatus"; data: { group_number: number; peer_id: number; status: string } };

// ─── Profile management ─────────────────────────────────────────────

export async function listProfiles(): Promise<string[]> {
  return invoke("list_profiles");
}

export async function createProfile(
  profileName: string,
  password: string,
  displayName: string,
): Promise<ProfileInfo> {
  return invoke("create_profile", { profileName, password, displayName });
}

export async function loadProfile(
  profileName: string,
  password: string,
): Promise<ProfileInfo> {
  return invoke("load_profile", { profileName, password });
}

export async function logout(): Promise<void> {
  return invoke("logout");
}

export async function deleteProfile(profileName: string): Promise<void> {
  return invoke("delete_profile", { profileName });
}

export async function getToxId(): Promise<string> {
  return invoke("get_tox_id");
}

export async function getConnectionStatus(): Promise<ConnectionInfo> {
  return invoke("get_connection_status");
}

export async function getProfileInfo(): Promise<ProfileInfo> {
  return invoke("get_profile_info");
}

export async function setDisplayName(name: string): Promise<void> {
  return invoke("set_display_name", { name });
}

export async function setStatusMessage(message: string): Promise<void> {
  return invoke("set_status_message", { message });
}

// ─── Friends ─────────────────────────────────────────────────────────

export async function addFriend(toxId: string, message: string): Promise<number> {
  return invoke("add_friend", { toxId, message });
}

export async function acceptFriendRequest(publicKey: string): Promise<number> {
  return invoke("accept_friend_request", { publicKey });
}

export async function denyFriendRequest(publicKey: string): Promise<void> {
  return invoke("deny_friend_request", { publicKey });
}

export async function removeFriend(friendNumber: number): Promise<void> {
  return invoke("remove_friend", { friendNumber });
}

export async function getFriends(): Promise<FriendInfo[]> {
  return invoke("get_friends");
}

export async function getFriendRequests(): Promise<FriendRequest[]> {
  return invoke("get_friend_requests");
}

// ─── Direct Messages ────────────────────────────────────────────────

export async function sendDirectMessage(
  friendNumber: number,
  message: string,
): Promise<SendMessageResult> {
  return invoke("send_direct_message", { friendNumber, message });
}

export async function getDirectMessages(
  friendNumber: number,
  limit?: number,
  beforeTimestamp?: string,
): Promise<DirectMessage[]> {
  return invoke("get_direct_messages", { friendNumber, limit, beforeTimestamp });
}

export async function setTyping(
  friendNumber: number,
  isTyping: boolean,
): Promise<void> {
  return invoke("set_typing", { friendNumber, isTyping });
}

export async function markMessagesRead(friendNumber: number): Promise<void> {
  return invoke("mark_messages_read", { friendNumber });
}

// ─── Guilds ─────────────────────────────────────────────────────────

export async function createGuild(name: string): Promise<GuildInfo> {
  return invoke("create_guild", { name });
}

export async function getGuilds(): Promise<GuildInfo[]> {
  return invoke("get_guilds");
}

export async function getGuildChannels(guildId: string): Promise<ChannelInfo[]> {
  return invoke("get_guild_channels", { guildId });
}

export async function createChannel(guildId: string, name: string): Promise<ChannelInfo> {
  return invoke("create_channel", { guildId, name });
}

export async function deleteChannel(guildId: string, channelId: string): Promise<void> {
  return invoke("delete_channel", { guildId, channelId });
}

export async function sendChannelMessage(
  guildId: string,
  channelId: string,
  message: string,
): Promise<ChannelMessage> {
  return invoke("send_channel_message", { guildId, channelId, message });
}

export async function getChannelMessages(
  channelId: string,
  limit?: number,
  beforeTimestamp?: string,
): Promise<ChannelMessage[]> {
  return invoke("get_channel_messages", { channelId, limit, beforeTimestamp });
}

export async function inviteToGuild(guildId: string, friendNumber: number): Promise<void> {
  return invoke("invite_to_guild", { guildId, friendNumber });
}

export async function acceptGuildInvite(
  friendNumber: number,
  inviteData: number[],
  groupName: string,
): Promise<GuildInfo> {
  return invoke("accept_guild_invite", { friendNumber, inviteData, groupName });
}

export async function getGuildMembers(guildId: string): Promise<GuildMember[]> {
  return invoke("get_guild_members", { guildId });
}

export async function setChannelTopic(
  guildId: string,
  channelId: string,
  topic: string,
): Promise<void> {
  return invoke("set_channel_topic", { guildId, channelId, topic });
}

export async function kickMember(guildId: string, peerId: number): Promise<void> {
  return invoke("kick_member", { guildId, peerId });
}

export async function setMemberRole(
  guildId: string,
  peerId: number,
  role: string,
): Promise<void> {
  return invoke("set_member_role", { guildId, peerId, role });
}

export async function renameGuild(guildId: string, name: string): Promise<void> {
  return invoke("rename_guild", { guildId, name });
}

export async function renameChannel(channelId: string, name: string): Promise<void> {
  return invoke("rename_channel", { channelId, name });
}

export async function leaveGuild(guildId: string): Promise<void> {
  return invoke("leave_guild", { guildId });
}

export async function createDmGroup(
  name: string,
  friendNumbers: number[],
): Promise<GuildInfo> {
  return invoke("create_dm_group", { name, friendNumbers });
}

export async function sendDmGroupMessage(
  guildId: string,
  message: string,
): Promise<ChannelMessage> {
  return invoke("send_dm_group_message", { guildId, message });
}

export async function getDmGroups(): Promise<GuildInfo[]> {
  return invoke("get_dm_groups");
}

// ─── Event listening ─────────────────────────────────────────────────

export function onToxEvent(callback: (event: ToxEvent) => void): Promise<UnlistenFn> {
  return listen<ToxEvent>("tox://event", (event) => {
    callback(event.payload);
  });
}
