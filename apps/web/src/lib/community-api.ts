export type DemoUser = { id: string; username: string; role: "user" };
type Session = { user: DemoUser; token: string; expires_at: string };
export type Community = { id: string; slug: string; name: string; description: string };
export type Channel = { id: string; name: string; slug: string; kind: "announcement" | "discussion" };
export type Post = { id: string; author_id: string; title: string; body: string; created_at: string };
export type Reply = { id: string; author_id: string; body: string; parent_id: string | null; created_at: string };

let session: Session | null = null;
let pendingSession: Promise<Session> | null = null;
const sessionKey = "herdlink-demo-session";

async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
  const response = await fetch(`/api${path}`, { ...init, cache: "no-store" });
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new ApiError(response.status, body?.error ?? "Could not reach Herdlink. Check that the backend is running and try again.");
  }
  return response.json() as Promise<T>;
}

class ApiError extends Error {
  constructor(public status: number, message: string) { super(message); }
}

export async function demoSession(): Promise<Session> {
  if (!session) {
    try { session = JSON.parse(sessionStorage.getItem(sessionKey) ?? "null"); }
    catch { session = null; }
  }
  if (session?.token && Date.parse(session.expires_at) > Date.now() + 60_000) return session;
  if (!pendingSession) {
    pendingSession = request<Session>("/auth/demo", { method: "POST" })
      .then((value) => {
        session = value;
        try { sessionStorage.setItem(sessionKey, JSON.stringify(value)); } catch { /* Memory session still works. */ }
        return value;
      })
      .finally(() => { pendingSession = null; });
  }
  return pendingSession;
}

export async function communityApi<T>(path: string, body?: unknown): Promise<T> {
  for (let attempt = 0; attempt < 2; attempt++) {
    const current = await demoSession();
    try {
      return await request<T>(path, {
        method: body === undefined ? "GET" : "POST",
        headers: { Authorization: `Bearer ${current.token}`, "Content-Type": "application/json" },
        ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      });
    } catch (error) {
      if (!(error instanceof ApiError) || error.status !== 401 || attempt > 0) throw error;
      if (session?.token === current.token) {
        session = null;
        try { sessionStorage.removeItem(sessionKey); } catch { /* Storage may be unavailable. */ }
      }
    }
  }
  throw new Error("Could not start the demo session. Please try again.");
}

export function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : "Something went wrong. Please try again.";
}

/** Authenticated fetch preserves streaming bodies; only retry before receiving data. */
export async function communityFetch(path: string, init: RequestInit = {}): Promise<Response> {
  for (let attempt = 0; attempt < 2; attempt++) {
    const current = await demoSession();
    const response = await fetch(`/api${path}`, { ...init, cache: "no-store", headers: { ...init.headers, Authorization: `Bearer ${current.token}`, "Content-Type": "application/json" } });
    if (response.status === 401 && attempt === 0) {
      if (session?.token === current.token) { session = null; try { sessionStorage.removeItem(sessionKey); } catch { /* Memory session still works. */ } }
      continue;
    }
    if (!response.ok) {
      const body = await response.json().catch(() => null);
      throw new Error(body?.error ?? "Could not reach the assistant. Check that the backend is running.");
    }
    return response;
  }
  throw new Error("Could not start the demo session.");
}
