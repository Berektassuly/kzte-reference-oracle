"use client";

import {
  startTransition,
  useEffect,
  useEffectEvent,
  useMemo,
  useState,
} from "react";

import type { OracleFeedSnapshot } from "@/lib/types/site";
import {
  mergeOracleSnapshots,
  parseHermesStreamEvent,
} from "@/lib/oracle/live-feeds";

type LatestResponse = {
  feeds: OracleFeedSnapshot[];
};

export function useLiveOracleFeeds(initialFeeds: OracleFeedSnapshot[]) {
  const [feeds, setFeeds] = useState(initialFeeds);
  const [streamState, setStreamState] = useState<"connecting" | "live" | "fallback">(
    "connecting",
  );

  const applyFeeds = useEffectEvent((incoming: OracleFeedSnapshot[]) => {
    startTransition(() => {
      setFeeds((current) => mergeOracleSnapshots(current, incoming));
    });
  });

  useEffect(() => {
    let mounted = true;
    const controller = new AbortController();

    async function hydrate() {
      try {
        const response = await fetch("/api/oracle/latest", {
          cache: "no-store",
          signal: controller.signal,
        });

        if (!response.ok) {
          return;
        }

        const payload = (await response.json()) as LatestResponse;

        if (mounted) {
          applyFeeds(payload.feeds);
        }
      } catch {
        // noop: EventSource stream below is the primary path.
      }
    }

    hydrate();

    return () => {
      mounted = false;
      controller.abort();
    };
  }, [applyFeeds]);

  useEffect(() => {
    const source = new EventSource("/api/oracle/stream");

    source.onopen = () => {
      setStreamState("live");
    };

    source.onmessage = (event) => {
      try {
        applyFeeds(parseHermesStreamEvent(event.data));
        setStreamState("live");
      } catch {
        setStreamState("fallback");
      }
    };

    source.onerror = () => {
      setStreamState("fallback");
    };

    return () => {
      source.close();
    };
  }, [applyFeeds]);

  const primaryFeed = useMemo(() => feeds[0] ?? initialFeeds[0], [feeds, initialFeeds]);

  return {
    feeds,
    primaryFeed,
    streamState,
  };
}
