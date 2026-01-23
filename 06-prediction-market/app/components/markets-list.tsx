"use client";

import { type ReactNode, useCallback, useEffect, useMemo, useState } from "react";

import { type Address } from "@solana/kit";

import {
  getMarketDecoder,
  type Market,
  PREDICTION_MARKET_PROGRAM_ADDRESS,
} from "../generated/prediction_market";
import { MarketCard } from "./market-card";

const MARKET_DISCRIMINATOR_BASE58 = "dkokXHR3DTw";
const DEVNET_RPC_URL = "https://api.devnet.solana.com";
const POLL_INTERVAL_MS = 3000;

interface MarketWithAddress {
  address: Address;
  market: Market;
}

type FilterTab = "active" | "past";

export function MarketsList(): ReactNode {
  const [markets, setMarkets] = useState<MarketWithAddress[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<FilterTab>("active");

  const fetchMarkets = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(DEVNET_RPC_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: 1,
          method: "getProgramAccounts",
          params: [
            PREDICTION_MARKET_PROGRAM_ADDRESS,
            {
              encoding: "base64",
              commitment: "confirmed",
              filters: [
                {
                  memcmp: {
                    offset: 0,
                    bytes: MARKET_DISCRIMINATOR_BASE58,
                  },
                },
              ],
            },
          ],
        }),
      });

      const result = await response.json();

      if (result.error) {
        throw new Error(result.error.message);
      }

      const decoder = getMarketDecoder();
      const fetchedMarkets: MarketWithAddress[] = [];

      for (const account of result.result || []) {
        try {
          const data = Uint8Array.from(atob(account.account.data[0]), c => c.charCodeAt(0));
          const market = decoder.decode(data);
          fetchedMarkets.push({
            address: account.pubkey as Address,
            market,
          });
        } catch (decodeError) {
          console.warn("Failed to decode market account:", account.pubkey, decodeError);
        }
      }

      // Sort by resolution time (newest first)
      fetchedMarkets.sort((a, b) => Number(b.market.resolutionTime - a.market.resolutionTime));

      setMarkets(fetchedMarkets);
    } catch (err) {
      console.error("Failed to fetch markets:", err);
      setError(err instanceof Error ? err.message : "Failed to fetch markets");
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchMarkets();
    const interval = setInterval(fetchMarkets, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [fetchMarkets]);

  const { activeMarkets, pastMarkets } = useMemo(() => {
    const active: MarketWithAddress[] = [];
    const past: MarketWithAddress[] = [];

    for (const item of markets) {
      if (item.market.resolved) {
        past.push(item);
      } else {
        active.push(item);
      }
    }

    active.sort((a, b) => Number(a.market.resolutionTime - b.market.resolutionTime));
    past.sort((a, b) => Number(b.market.resolutionTime - a.market.resolutionTime));

    return { activeMarkets: active, pastMarkets: past };
  }, [markets]);

  const displayedMarkets = activeTab === "active" ? activeMarkets : pastMarkets;

  if (loading && markets.length === 0) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="flex items-center gap-2 text-sm text-muted">
          <svg className="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          Loading markets...
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-xl border border-red-200 bg-red-50 p-4 text-center">
        <p className="text-sm text-red-700 mb-2">{error}</p>
        <button
          onClick={fetchMarkets}
          className="text-sm font-medium text-red-600 hover:underline"
        >
          Try again
        </button>
      </div>
    );
  }

  if (markets.length === 0) {
    return (
      <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
        <div className="mx-auto w-12 h-12 rounded-full bg-cream flex items-center justify-center mb-3">
          <svg className="h-6 w-6 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
        </div>
        <p className="text-sm text-muted mb-1">No markets yet</p>
        <p className="text-xs text-muted/70">Create the first prediction market to get started</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Tabs */}
      <div className="flex items-center justify-between">
        <div className="flex gap-1 rounded-lg bg-cream p-1">
          <button
            onClick={() => setActiveTab("active")}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              activeTab === "active"
                ? "bg-card text-foreground shadow-sm"
                : "text-muted hover:text-foreground"
            }`}
          >
            Active
            {activeMarkets.length > 0 && (
              <span className="ml-1.5 text-xs text-muted">({activeMarkets.length})</span>
            )}
          </button>
          <button
            onClick={() => setActiveTab("past")}
            className={`rounded-md px-3 py-1.5 text-sm font-medium transition ${
              activeTab === "past"
                ? "bg-card text-foreground shadow-sm"
                : "text-muted hover:text-foreground"
            }`}
          >
            Past
            {pastMarkets.length > 0 && (
              <span className="ml-1.5 text-xs text-muted">({pastMarkets.length})</span>
            )}
          </button>
        </div>
        <button
          onClick={fetchMarkets}
          disabled={loading}
          className="text-xs text-muted hover:text-foreground transition disabled:opacity-50"
        >
          {loading ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      {/* Markets Grid */}
      {displayedMarkets.length === 0 ? (
        <div className="rounded-xl border border-dashed border-border-low p-8 text-center">
          <p className="text-sm text-muted">
            {activeTab === "active"
              ? "No active markets. Create one to get started!"
              : "No past markets yet."}
          </p>
        </div>
      ) : (
        <div className="grid gap-3 sm:grid-cols-2">
          {displayedMarkets.map((item) => (
            <MarketCard
              key={item.address}
              market={item.market}
              marketAddress={item.address}
              onUpdate={fetchMarkets}
            />
          ))}
        </div>
      )}
    </div>
  );
}
