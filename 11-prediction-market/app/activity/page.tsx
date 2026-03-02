"use client";

import { type ReactNode } from "react";

import Link from "next/link";

import { type Address } from "@solana/kit";
import { useWalletConnection } from "@solana/react-hooks";

import { PositionsList } from "../components/positions-list";
import { WalletButton } from "../components/wallet-button";

function WalletIcon(): ReactNode {
  return (
    <svg className="h-8 w-8 text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M21 12a2.25 2.25 0 00-2.25-2.25H15a3 3 0 11-6 0H5.25A2.25 2.25 0 003 12m18 0v6a2.25 2.25 0 01-2.25 2.25H5.25A2.25 2.25 0 013 18v-6m18 0V9M3 12V9m18 0a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 9m18 0V6a2.25 2.25 0 00-2.25-2.25H5.25A2.25 2.25 0 003 6v3"
      />
    </svg>
  );
}

function BackArrowIcon(): ReactNode {
  return (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 19l-7-7m0 0l7-7m-7 7h18" />
    </svg>
  );
}

function WalletNotConnected(): ReactNode {
  return (
    <div className="flex flex-col items-center justify-center py-20">
      <div className="mx-auto w-16 h-16 rounded-full bg-cream flex items-center justify-center mb-4">
        <WalletIcon />
      </div>
      <h2 className="text-xl font-semibold mb-2">Connect your wallet</h2>
      <p className="text-sm text-muted mb-6 text-center max-w-sm">
        Connect a Solana wallet to view your betting activity and positions
      </p>
      <WalletButton />
    </div>
  );
}

interface ActivityContentProps {
  walletAddress: Address;
}

function ActivityContent({ walletAddress }: ActivityContentProps): ReactNode {
  return (
    <div>
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-semibold tracking-tight">Your Activity</h2>
          <p className="text-sm text-muted">Track your positions and performance</p>
        </div>
        <Link
          href="/"
          className="flex items-center gap-2 text-sm text-muted hover:text-foreground transition"
        >
          <BackArrowIcon />
          Back to Markets
        </Link>
      </div>
      <PositionsList walletAddress={walletAddress} />
    </div>
  );
}

export default function ActivityPage(): ReactNode {
  const { wallet, status } = useWalletConnection();
  const walletAddress = wallet?.account.address;

  return (
    <div className="min-h-screen bg-bg1 text-foreground">
      <header className="sticky top-0 z-50 border-b border-border-low bg-bg1/80 backdrop-blur-sm">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <div className="flex items-center gap-3">
            <Link href="/" className="flex items-center gap-3 hover:opacity-80 transition">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-foreground text-background font-bold text-sm">
                PM
              </div>
              <div>
                <h1 className="text-sm font-semibold">Prediction Markets</h1>
                <p className="text-xs text-muted">Solana Devnet</p>
              </div>
            </Link>
          </div>
          <div className="flex items-center gap-4">
            <Link href="/" className="text-sm text-muted hover:text-foreground transition">
              Markets
            </Link>
            <span className="text-sm font-medium">Activity</span>
            <WalletButton />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-5xl px-4 py-8">
        {status !== "connected" ? (
          <WalletNotConnected />
        ) : (
          <ActivityContent walletAddress={walletAddress!} />
        )}
      </main>

      <footer className="border-t border-border-low mt-16">
        <div className="mx-auto max-w-5xl px-4 py-6">
          <div className="flex flex-wrap items-center justify-between gap-4 text-xs text-muted">
            <div className="flex items-center gap-2">
              <span className="rounded bg-yellow-100 px-2 py-0.5 text-yellow-800 font-medium">
                Devnet
              </span>
              <span>Built with Anchor + @solana/react-hooks</span>
            </div>
            <div className="flex gap-4">
              <a
                href="https://www.anchor-lang.com/docs"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition"
              >
                Anchor Docs
              </a>
              <a
                href="https://solana.com/docs"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition"
              >
                Solana Docs
              </a>
              <a
                href="https://faucet.solana.com/"
                target="_blank"
                rel="noreferrer"
                className="hover:text-foreground transition"
              >
                Faucet
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
