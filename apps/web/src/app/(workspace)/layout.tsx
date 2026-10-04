import { Workspace } from "@/components/workspace/Workspace";

import { GraphChatProvider } from "@/components/chat/GraphChatProvider";

export default function WorkspaceLayout({ children }: { children: React.ReactNode }) {
  return <GraphChatProvider><Workspace>{children}</Workspace></GraphChatProvider>;
}
