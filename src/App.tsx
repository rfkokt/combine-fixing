import { Sidebar } from "@/components/layout/sidebar";
import { Topbar } from "@/components/layout/topbar";
import { HomePage } from "@/pages/home";
import { MergePage } from "@/pages/merge";
import { SpellcheckPage } from "@/pages/spellcheck";
import { SettingsPage } from "@/pages/settings";
import { useAppStore } from "@/stores/app-store";

function PageContent() {
  const { currentPage } = useAppStore();

  switch (currentPage) {
    case "home":
      return <HomePage />;
    case "merge":
      return <MergePage />;
    case "spellcheck":
      return <SpellcheckPage />;
    case "settings":
      return <SettingsPage />;
    default:
      return <HomePage />;
  }
}

function App() {
  return (
    <div className="h-screen w-screen flex bg-bg-primary">
      <Sidebar />
      <div className="flex-1 flex flex-col min-w-0">
        <Topbar />
        <main className="flex-1 min-h-0">
          <PageContent />
        </main>
      </div>
    </div>
  );
}

export default App;
