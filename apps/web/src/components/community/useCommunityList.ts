"use client";

import { useEffect, useRef, useState } from "react";
import { communityApi, errorMessage } from "@/lib/community-api";

const pageSize = 20;

export function useCommunityList<T extends { id: string }>(path: string) {
  const pagePath = `${path}${path.includes("?") ? "&" : "?"}limit=${pageSize}`;
  const [items, setItems] = useState<T[]>([]);
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState("");
  const [hasMore, setHasMore] = useState(false);
  const [version, setVersion] = useState(0);
  const offset = useRef(0);
  const inFlight = useRef(false);

  useEffect(() => {
    let cancelled = false;
    communityApi<T[]>(`${pagePath}&offset=0`)
      .then((data) => {
        if (cancelled) return;
        setItems(data);
        offset.current = data.length;
        setHasMore(data.length === pageSize);
        setError("");
      })
      .catch((error) => { if (!cancelled) setError(errorMessage(error)); })
      .finally(() => { if (!cancelled) setBusy(false); });
    return () => { cancelled = true; };
  }, [pagePath, version]);

  function refresh() {
    if (busy || inFlight.current) return;
    setBusy(true);
    setError("");
    setVersion((value) => value + 1);
  }

  async function loadMore() {
    if (busy || inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError("");
    try {
      const data = await communityApi<T[]>(`${pagePath}&offset=${offset.current}`);
      offset.current += data.length;
      setItems((current) => [...current, ...data.filter((item) => !current.some((existing) => existing.id === item.id))]);
      setHasMore(data.length === pageSize);
    } catch (error) { setError(errorMessage(error)); }
    finally { inFlight.current = false; setBusy(false); }
  }

  function add(item: T, position: "start" | "end") {
    if (position === "start") offset.current += 1;
    setItems((current) => position === "start" ? [item, ...current] : [...current, item]);
  }

  return { items, busy, error, hasMore, refresh, loadMore, add };
}
