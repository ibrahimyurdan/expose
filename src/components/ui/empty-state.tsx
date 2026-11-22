import type { ReactNode } from "react";

import { Mark } from "@/components/art/Mark";

// a calm, centered message for a screen with nothing to show yet: the mark
// as a flat grey silhouette, a title, one line of explanation and optional
// actions below.
function EmptyState({
  title,
  children,
  actions,
}: {
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
}) {
  return (
    <div className="empty">
      <Mark size={96} color="#3c3c41" />
      <h1 className="empty-title type-h4">{title}</h1>
      {children ? <p className="empty-text type-body-m">{children}</p> : null}
      {actions ? <div className="empty-actions flex items-center gap-2">{actions}</div> : null}
    </div>
  );
}

export { EmptyState };
