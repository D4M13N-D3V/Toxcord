import { useEffect } from "react";
import { onToxEvent, ToxEvent } from "../api/tox";
import { useAuthStore } from "../stores/authStore";
import { useFriendStore } from "../stores/friendStore";
import { useMessageStore } from "../stores/messageStore";
import { useGuildStore } from "../stores/guildStore";
import { useChannelMessageStore } from "../stores/channelMessageStore";
import { useNavigationStore } from "../stores/navigationStore";

export function useToxEvents() {
  const setConnectionStatus = useAuthStore((s) => s.setConnectionStatus);
  const {
    addIncomingRequest,
    updateFriendName,
    updateFriendStatusMessage,
    updateFriendStatus,
    updateFriendConnectionStatus,
  } = useFriendStore();
  const { addIncomingMessage, setFriendTyping } = useMessageStore();
  const {
    addGuildInvite,
    loadGuilds,
    loadDmGroups,
    addMember,
    removeMember,
    updateMemberName,
    updateMemberStatus,
    refreshChannels,
  } = useGuildStore();
  const addChannelMessage = useChannelMessageStore((s) => s.addIncomingMessage);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    onToxEvent((event: ToxEvent) => {
      switch (event.type) {
        case "ConnectionStatus":
          setConnectionStatus(event.data.connected, event.data.status);
          break;
        case "FriendRequest":
          addIncomingRequest(event.data.public_key, event.data.message);
          break;
        case "FriendMessage":
          addIncomingMessage({
            id: event.data.id,
            friend_number: event.data.friend_number,
            sender: "friend",
            content: event.data.message,
            message_type: event.data.message_type,
            timestamp: event.data.timestamp,
            is_outgoing: false,
            delivered: true,
            read: false,
          });
          break;
        case "FriendName":
          updateFriendName(event.data.friend_number, event.data.name);
          break;
        case "FriendStatusMessage":
          updateFriendStatusMessage(event.data.friend_number, event.data.message);
          break;
        case "FriendStatus":
          updateFriendStatus(event.data.friend_number, event.data.status);
          break;
        case "FriendConnectionStatus":
          updateFriendConnectionStatus(
            event.data.friend_number,
            event.data.connected,
            event.data.status,
          );
          break;
        case "FriendTyping":
          setFriendTyping(event.data.friend_number, event.data.is_typing);
          break;
        // Group events
        case "GroupInvite":
          addGuildInvite({
            friendNumber: event.data.friend_number,
            inviteData: event.data.invite_data,
            groupName: event.data.group_name,
          });
          break;
        case "GroupSelfJoin":
          // Reload guilds and DM groups to pick up the newly joined group
          loadGuilds();
          loadDmGroups();
          break;
        case "GroupPeerJoin":
          addMember(event.data.group_number, {
            peer_id: event.data.peer_id,
            name: event.data.name,
            public_key: event.data.public_key,
            role: "user",
            status: "online",
          });
          break;
        case "GroupPeerExit":
          removeMember(event.data.group_number, event.data.peer_id);
          break;
        case "GroupPeerName":
          updateMemberName(
            event.data.group_number,
            event.data.peer_id,
            event.data.name,
          );
          break;
        case "GroupMessage": {
          const channelId = event.data.channel_id;
          // Get fresh state from store to avoid stale closure
          const currentGuilds = useGuildStore.getState().guilds;
          const currentDmGroups = useGuildStore.getState().dmGroups;
          const currentChannels = useGuildStore.getState().channels;
          const selectedChannelId = useNavigationStore.getState().selectedChannelId;

          console.log("[GroupMessage] Received:", {
            group_number: event.data.group_number,
            channel_id: channelId,
            sender: event.data.sender_name,
            sender_pk: event.data.sender_pk?.substring(0, 16) + "...",
            content_preview: event.data.message.substring(0, 50),
            currently_viewing_channel: selectedChannelId,
            will_be_visible: selectedChannelId === channelId,
            guilds_count: currentGuilds.length,
            dm_groups_count: currentDmGroups.length,
          });

          // Check if channel exists locally, if not refresh channels for the guild/dm group
          const guild = currentGuilds.find((g) => g.group_number === event.data.group_number);
          const dmGroup = currentDmGroups.find((g) => g.group_number === event.data.group_number);
          const group = guild || dmGroup;

          if (group) {
            const groupChannels = currentChannels[group.id] ?? [];
            const channelExists = groupChannels.some((c) => c.id === channelId);
            console.log("[GroupMessage] Group found:", group.name, "Type:", guild ? "server" : "dm_group", "Channels:", groupChannels.map(c => c.id), "Channel exists:", channelExists);
            if (!channelExists) {
              // Channel was auto-created on backend, refresh to get it
              refreshChannels(group.id);
            }
          } else {
            console.warn("[GroupMessage] No guild/dm_group found for group_number:", event.data.group_number);
          }
          addChannelMessage(channelId, {
            id: event.data.id,
            channel_id: channelId,
            sender_public_key: event.data.sender_pk,
            sender_name: event.data.sender_name,
            content: event.data.message,
            message_type: event.data.message_type,
            timestamp: event.data.timestamp,
            is_own: false,
          });
          break;
        }
        case "GroupPeerStatus":
          updateMemberStatus(
            event.data.group_number,
            event.data.peer_id,
            event.data.status,
          );
          break;
        case "GroupTopicChange":
        case "GroupJoinFail":
        case "GroupCustomPacket":
          // Handled elsewhere or not yet needed
          break;
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [
    setConnectionStatus,
    addIncomingRequest,
    addIncomingMessage,
    setFriendTyping,
    updateFriendName,
    updateFriendStatusMessage,
    updateFriendStatus,
    updateFriendConnectionStatus,
    addGuildInvite,
    loadGuilds,
    loadDmGroups,
    addMember,
    removeMember,
    updateMemberName,
    updateMemberStatus,
    addChannelMessage,
    refreshChannels,
  ]);
}
