import { AuthProvider } from "@/components/auth/AuthProvider";
import { Workspace } from "@/components/workspace/Workspace";

import { GraphChatProvider } from "@/components/chat/GraphChatProvider";

export default function WorkspaceLayout({ children }: { children: React.ReactNode }) {
  return <AuthProvider><GraphChatProvider><Workspace>{children}</Workspace></GraphChatProvider></AuthProvider>;
}
