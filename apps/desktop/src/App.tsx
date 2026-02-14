import { useAuthStore } from "./stores/authStore";
import { LoginPage } from "./pages/LoginPage";
import { HomePage } from "./pages/HomePage";

function App() {
  const isLoggedIn = useAuthStore((s) => s.isLoggedIn);

  return (
    <div className="h-screen w-screen bg-discord-dark">
      {isLoggedIn ? <HomePage /> : <LoginPage />}
    </div>
  );
}

export default App;
