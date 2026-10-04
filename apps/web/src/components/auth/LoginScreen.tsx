"use client";

import Image from "next/image";
import { useState } from "react";
import { authenticate, demoCredentials, errorMessage } from "@/lib/community-api";
import { buttonClass, ErrorNotice, fieldClass, secondaryClass } from "@/components/community/CommunityUi";

export function LoginScreen() {
  const [register, setRegister] = useState(false);
  const [email, setEmail] = useState(demoCredentials.email);
  const [password, setPassword] = useState(demoCredentials.password);
  const [username, setUsername] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  async function submit() {
    setBusy(true); setError("");
    try { await authenticate({ email, password, ...(register ? { username } : {}) }); }
    catch (error) { setError(errorMessage(error)); }
    finally { setBusy(false); }
  }
  function changeMode() { setRegister(!register); setEmail(""); setPassword(""); setError(""); }
  return <main className="herdlink-site flex min-h-dvh items-center justify-center bg-[var(--site-surface)] px-5 py-12" data-theme="light">
    <section className="w-full max-w-md rounded-3xl border border-[var(--site-border)] bg-[var(--site-panel)] p-7 shadow-sm">
      <Image src="/logo.png" alt="Herdlink" width={1449} height={1086} sizes="128px" preload className="mx-auto mb-5 h-20 w-32 object-cover mix-blend-multiply" />
      <h1 className="text-2xl font-medium">{register ? "Create your account" : "Welcome to Herdlink"}</h1>
      <p className="mb-6 mt-2 text-sm leading-6 text-[var(--site-secondary)]">Explore connected diseases, join communities, and take part in surveys.</p>
      <form onSubmit={(event) => { event.preventDefault(); void submit(); }} className="space-y-4">
        {register && <label className="block text-sm">Username<input className={`${fieldClass} mt-2`} value={username} onChange={(event) => setUsername(event.target.value)} autoComplete="username" required minLength={3} maxLength={32} pattern="[a-z0-9_]+" title="3–32 lowercase letters, digits, or underscores" disabled={busy} /></label>}
        <label className="block text-sm">Email<input className={`${fieldClass} mt-2`} type="email" value={email} onChange={(event) => setEmail(event.target.value)} autoComplete="email" required maxLength={254} disabled={busy} /></label>
        <label className="block text-sm">Password<input className={`${fieldClass} mt-2`} type="password" value={password} onChange={(event) => setPassword(event.target.value)} autoComplete={register ? "new-password" : "current-password"} required minLength={register ? 12 : undefined} maxLength={1024} disabled={busy} /></label>
        {register && <p className="text-xs text-[var(--site-secondary)]">Use at least 12 characters.</p>}
        {error && <ErrorNotice message={error} />}
        <button className={`${buttonClass} w-full py-3`} disabled={busy} type="submit">{busy ? "Signing in…" : register ? "Create account" : "Log in"}</button>
      </form>
      {!register && <div className="mt-5 rounded-xl bg-[var(--site-soft)] p-4 text-xs leading-6"><p className="font-medium">Try the demo account</p><p>{demoCredentials.email}</p><p>Password: {demoCredentials.password}</p><button type="button" disabled={busy} onClick={() => { setEmail(demoCredentials.email); setPassword(demoCredentials.password); setError(""); }} className="mt-1 underline">Fill demo credentials</button></div>}
      <button type="button" disabled={busy} onClick={changeMode} className={`${secondaryClass} mt-5 w-full`}>{register ? "Already have an account? Log in" : "Create a new account"}</button>
    </section>
  </main>;
}
