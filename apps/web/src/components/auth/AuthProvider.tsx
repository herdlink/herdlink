"use client";

import { createContext, useContext, useEffect, useState } from "react";
import { ApiError, communityApi, logout, sessionEvent, storedSession, type Session, type User } from "@/lib/community-api";
import { LoginScreen } from "./LoginScreen";

const Context = createContext<Session | null>(null);
export function useAuth() { const value = useContext(Context); if (!value) throw new Error("Sign in to use Herdlink."); return value; }
export { logout };
export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [session, setSession] = useState<Session | null>(null);
  const [ready, setReady] = useState(false);
  useEffect(() => {
    let cancelled = false;
    const update = () => setSession(storedSession());
    window.addEventListener(sessionEvent, update);
    const current = storedSession();
    if (current) {
      communityApi<User>("/me").then((user) => { if (!cancelled) setSession({ ...current, user }); }).catch((error) => { if (!cancelled && !(error instanceof ApiError && error.status === 401)) setSession(current); }).finally(() => { if (!cancelled) setReady(true); });
    } else { queueMicrotask(() => { if (!cancelled) setReady(true); }); }
    return () => { cancelled = true; window.removeEventListener(sessionEvent, update); };
  }, []);
  if (!ready) return <p role="status" className="p-8 text-sm">Loading Herdlink…</p>;
  if (!session) return <LoginScreen />;
  return <Context.Provider value={session}><div key={session.user.id} className="contents">{children}</div></Context.Provider>;
}
