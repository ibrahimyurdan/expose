import { useState } from "react";

import { AutoAcceptBar } from "@/components/AutoAcceptBar";
import { ChampSelectActions } from "@/components/ChampSelectActions";
import { SettingsPanel } from "@/components/SettingsPanel";
import { StatusHeader } from "@/components/StatusHeader";
import { TeammateList } from "@/components/TeammateList";
import { Separator } from "@/components/ui/separator";
import { TooltipProvider } from "@/components/ui/tooltip";
import { NoticeProvider } from "@/hooks/notices";
import { useChampions } from "@/hooks/useChampions";
import { useChampSelectPhase } from "@/hooks/useChampSelectPhase";
import { useStatus } from "@/hooks/useStatus";
import { useTeammates } from "@/hooks/useTeammates";

export default function App() {
  const status = useStatus();
  const teammates = useTeammates();
  const champions = useChampions();
  const clock = useChampSelectPhase();
  const [showSettings, setShowSettings] = useState(false);

  return (
    <NoticeProvider>
      <TooltipProvider delayDuration={300}>
        <div className="relative flex h-full flex-col bg-background">
          <StatusHeader
            status={status}
            clock={clock}
            settingsOpen={showSettings}
            onToggleSettings={() => setShowSettings((open) => !open)}
          />
          <Separator />
          {showSettings ? (
            <div className="min-h-0 flex-1 animate-in overflow-y-auto duration-200 fade-in-0 slide-in-from-right-3">
              <SettingsPanel />
            </div>
          ) : (
            <div className="flex min-h-0 flex-1 animate-in flex-col duration-200 fade-in-0 slide-in-from-left-3">
              {status === "champSelect" ? <ChampSelectActions teammates={teammates} /> : null}
              <div className="min-h-0 flex-1">
                <TeammateList status={status} teammates={teammates} champions={champions} />
              </div>
            </div>
          )}
          <Separator />
          <AutoAcceptBar />
        </div>
      </TooltipProvider>
    </NoticeProvider>
  );
}
