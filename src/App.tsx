import { useState } from "react";

import { AutoAcceptBar } from "@/components/AutoAcceptBar";
import { ChampSelectActions } from "@/components/ChampSelectActions";
import { SettingsPanel } from "@/components/SettingsPanel";
import { StatusHeader } from "@/components/StatusHeader";
import { TeammateList } from "@/components/TeammateList";
import { Separator } from "@/components/ui/separator";
import { TooltipProvider } from "@/components/ui/tooltip";
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
    <TooltipProvider>
      <div className="flex h-full flex-col bg-background">
        <StatusHeader
          status={status}
          clock={clock}
          settingsOpen={showSettings}
          onToggleSettings={() => setShowSettings((open) => !open)}
        />
        <Separator />
        {showSettings ? (
          <div className="min-h-0 flex-1 overflow-y-auto">
            <SettingsPanel />
          </div>
        ) : (
          <>
            {status === "champSelect" ? (
              <>
                <ChampSelectActions teammates={teammates} />
                <Separator />
              </>
            ) : null}
            <div className="min-h-0 flex-1">
              <TeammateList status={status} teammates={teammates} champions={champions} />
            </div>
          </>
        )}
        <Separator />
        <AutoAcceptBar />
      </div>
    </TooltipProvider>
  );
}
