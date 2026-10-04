export type User = { id: string; email: string; username: string; role: "user" | "scientist" | "pharma_scout" };
export type DemoUser = User;
export type Session = { user: User; token: string; expires_at: string };
export type Community = { id: string; slug: string; name: string; description: string };
export type MembershipStatus = { joined: boolean; member_count: number };
export type Channel = { id: string; name: string; slug: string; kind: "announcement" | "discussion" };
export type Post = { id: string; author_id: string; title: string; body: string; created_at: string };
export type Reply = { id: string; author_id: string; body: string; parent_id: string | null; created_at: string };

let session: Session | null = null;
const sessionKey = "herdlink-demo-session";
export const sessionEvent = "herdlink-session-change";
export const demoCredentials = { email: "demo@herdlink.local", password: "HerdlinkDemo123!" };
export class ApiError extends Error { constructor(public status: number, message: string) { super(message); } }
export function storedSession(): Session | null {
  if (!session) { try { session = JSON.parse(sessionStorage.getItem(sessionKey) ?? "null"); } catch { session = null; } }
  return session?.token && Date.parse(session.expires_at) > Date.now() ? session : null;
}
function saveSession(value: Session | null) {
  session = value;
  try { if (value) sessionStorage.setItem(sessionKey, JSON.stringify(value)); else sessionStorage.removeItem(sessionKey); } catch { /* Memory session still works. */ }
  window.dispatchEvent(new Event(sessionEvent));
}
export async function authenticate(input: { email: string; password: string; username?: string }): Promise<Session> {
  const response = await fetch(`/api/auth/${input.username === undefined ? "login" : "register"}`, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(input) });
  const value = await response.json();
  if (!response.ok) throw new ApiError(response.status, response.status === 401 ? "The email or password is incorrect." : value.error ?? "Could not sign in.");
  saveSession(value); return value as Session;
}
export async function logout() {
  try { await communityFetch("/auth/logout", { method: "POST" }); } finally { saveSession(null); }
}
export async function userSession(): Promise<Session> {
  const current = storedSession();
  if (!current) { saveSession(null); throw new ApiError(401, "Please log in to continue."); }
  return current;
}
/** Authenticated fetch also preserves streaming bodies. Never switch identities on expiry. */
export async function communityFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const current = await userSession();
  const headers = new Headers(init.headers);
  headers.set("Authorization", `Bearer ${current.token}`); headers.set("Content-Type", "application/json");
  const response = await fetch(`/api${path}`, { ...init, headers, cache: "no-store" });
  if (response.status === 401 && storedSession()?.token === current.token) saveSession(null);
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new ApiError(response.status, body?.error ?? "Could not reach Herdlink. Check that the backend is running and try again.");
  }
  return response;
}
export async function communityApi<T>(path: string, body?: unknown): Promise<T> {
  const response = await communityFetch(path, { method: body === undefined ? "GET" : "POST", ...(body === undefined ? {} : { body: JSON.stringify(body) }) });
  return response.status === 204 ? undefined as T : response.json() as Promise<T>;
}
export function errorMessage(error: unknown) { return error instanceof Error ? error.message : "Something went wrong. Please try again."; }
