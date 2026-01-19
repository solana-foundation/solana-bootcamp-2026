"use client";

import { type ReactNode, useEffect, useRef, useState } from "react";

import { useWalletConnection } from "@solana/react-hooks";

function truncateAddress(address: string): string {
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

export function WalletButton(): ReactNode {
  const { connectors, connect, disconnect, wallet, status } = useWalletConnection();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const address = wallet?.account.address.toString();

  useEffect(() => {
    function handleClickOutside(event: MouseEvent): void {
      const target = event.target as Node;
      if (dropdownRef.current && !dropdownRef.current.contains(target)) {
        setIsOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  if (status === "connected" && address) {
    return (
      <div className="relative" ref={dropdownRef}>
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 rounded-lg border border-border-low bg-card px-3 py-2 text-sm font-medium transition hover:bg-cream/50"
        >
          <span className="h-2 w-2 rounded-full bg-green-500" />
          <span className="font-mono">{truncateAddress(address)}</span>
          <svg
            className={`h-4 w-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>

        {isOpen && (
          <div className="absolute right-0 top-full mt-2 w-64 rounded-lg border border-border-low bg-card p-3 shadow-lg z-50">
            <div className="mb-3 pb-3 border-b border-border-low">
              <p className="text-xs text-muted mb-1">Connected wallet</p>
              <p className="font-mono text-xs break-all">{address}</p>
            </div>
            <div className="space-y-2">
              <a
                href="https://faucet.solana.com/"
                target="_blank"
                rel="noreferrer"
                className="flex items-center gap-2 w-full rounded-md px-3 py-2 text-sm text-left transition hover:bg-cream/50"
              >
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Get devnet SOL
              </a>
              <button
                onClick={() => {
                  disconnect();
                  setIsOpen(false);
                }}
                className="flex items-center gap-2 w-full rounded-md px-3 py-2 text-sm text-left text-red-600 transition hover:bg-red-50"
              >
                <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                </svg>
                Disconnect
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }

  // Not connected - show connect button with dropdown
  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={status === "connecting"}
        className="flex items-center gap-2 rounded-lg bg-foreground px-4 py-2 text-sm font-medium text-background transition hover:opacity-90 disabled:opacity-60"
      >
        {status === "connecting" ? (
          <>
            <svg className="h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
            Connecting...
          </>
        ) : (
          <>
            Connect Wallet
            <svg
              className={`h-4 w-4 transition-transform ${isOpen ? "rotate-180" : ""}`}
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
            </svg>
          </>
        )}
      </button>

      {isOpen && status !== "connecting" && (
        <div className="absolute right-0 top-full mt-2 w-64 rounded-lg border border-border-low bg-card p-2 shadow-lg z-50">
          <p className="px-3 py-2 text-xs text-muted">Select a wallet</p>
          {connectors.map((connector) => (
            <button
              key={connector.id}
              onClick={() => {
                connect(connector.id);
                setIsOpen(false);
              }}
              className="flex items-center gap-3 w-full rounded-md px-3 py-2.5 text-sm text-left transition hover:bg-cream/50"
            >
              <span className="h-2 w-2 rounded-full bg-border-low" />
              {connector.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
