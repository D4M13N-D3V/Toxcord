import { useEffect } from "react";
import { useAuthStore } from "../stores/authStore";
import { useNavigationStore } from "../stores/navigationStore";
import { useToxEvents } from "../hooks/useToxEvents";
import { useCallEvents } from "../hooks/useCallEvents";
import { getConnectionStatus } from "../api/tox";
import { ServerSidebar } from "../components/layout/ServerSidebar";
import { ChannelSidebar } from "../components/layout/ChannelSidebar";
import { MainContent } from "../components/layout/MainContent";
import { CallOverlay } from "../components/call/CallOverlay";
import { FriendsPage } from "./FriendsPage";
import { DMPage } from "./DMPage";
import { GuildPage } from "./GuildPage";
import { DmGroupPage } from "./DmGroupPage";
import { SettingsPage } from "./SettingsPage";

export function HomePage() {
  useToxEvents();
  useCallEvents();

  const setConnectionStatus = useAuthStore((s) => s.setConnectionStatus);
  const currentPage = useNavigationStore((s) => s.currentPage);

  useEffect(() => {
    const poll = async () => {
      try {
        const status = await getConnectionStatus();
        setConnectionStatus(status.connected, status.status);
      } catch {
        // Not connected yet
      }
    };

    poll();
    const interval = setInterval(poll, 3000);
    return () => clearInterval(interval);
  }, [setConnectionStatus]);

  const renderContent = () => {
    switch (currentPage) {
      case "friends":
        return <FriendsPage />;
      case "dm":
        return <DMPage />;
      case "guild":
        return <GuildPage />;
      case "dm_group":
        return <DmGroupPage />;
      case "settings":
        return <SettingsPage />;
      default:
        return <MainContent />;
    }
  };

  return (
    <>
      <div className="flex h-screen">
        <ServerSidebar />
        <ChannelSidebar />
        {renderContent()}
      </div>
      <CallOverlay />
    </>
  );
}
