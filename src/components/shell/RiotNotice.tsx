import { cn } from "@/lib/utils";

// riot's notice for projects built on its games, word for word. it shows in
// settings → about, at the bottom of the welcome screen and in the readme.
export const RIOT_NOTICE =
  "Expose isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games, and all associated properties are trademarks or registered trademarks of Riot Games, Inc.";

// the caller picks the type role: small type on the welcome screen, body
// type in settings.
export function RiotNotice({ className }: { className?: string }) {
  return <p className={cn("riot-notice", className)}>{RIOT_NOTICE}</p>;
}
