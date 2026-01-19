"use client";

import { type ReactNode, useState } from "react";

import Link from "next/link";

import { CreateMarketForm } from "./components/create-market-form";
import { MarketsList } from "./components/markets-list";
import { WalletButton } from "./components/wallet-button";

function PlusIcon(): ReactNode {
  return (
    <svg className="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
    </svg>
  );
}

const HOW_IT_WORKS_STEPS = [
  { title: "Create", description: "Anyone can create a market with a yes/no question and deadline." },
  { title: "Bet", description: "Stake SOL on YES or NO before the deadline." },
  { title: "Resolve", description: "After the deadline, the creator declares the outcome." },
  { title: "Claim", description: "Winners split the losing pool proportionally." },
];

function HowItWorks(): ReactNode {
  return (
    <details className="mt-12 rounded-lg border border-border-low">
      <summary className="cursor-pointer px-4 py-3 text-sm font-medium hover:bg-cream/30">
        How it works
      </summary>
      <div className="border-t border-border-low px-4 py-4 text-sm text-muted space-y-3">
        {HOW_IT_WORKS_STEPS.map((step, index) => (
          <div key={step.title} className="flex gap-3">
            <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-cream text-xs font-medium">
              {index + 1}
            </span>
            <p>
              <strong className="text-foreground">{step.title}</strong> - {step.description}
            </p>
          </div>
        ))}
      </div>
    </details>
  );
}

export default function Home(): ReactNode {
  const [showCreateForm, setShowCreateForm] = useState(false);

  return (
    <div className="min-h-screen bg-bg1 text-foreground">
      <header className="sticky top-0 z-50 border-b border-border-low bg-bg1/80 backdrop-blur-sm">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-foreground text-background font-bold text-sm">
              PM
            </div>
            <div>
              <h1 className="text-sm font-semibold">Prediction Markets</h1>
              <p className="text-xs text-muted">Solana Devnet</p>
            </div>
          </div>
          <div className="flex items-center gap-4">
            <span className="text-sm font-medium">Markets</span>
            <Link
              href="/activity"
              className="text-sm text-muted hover:text-foreground transition"
            >
              Activity
            </Link>
            <WalletButton />
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-5xl px-4 py-8">
        <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-2xl font-semibold tracking-tight">Markets</h2>
            <p className="text-sm text-muted">
              Bet SOL on yes/no outcomes. Winners take the pool.
            </p>
          </div>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="flex items-center gap-2 rounded-lg bg-foreground px-4 py-2.5 text-sm font-medium text-background transition hover:opacity-90"
          >
            <PlusIcon />
            New Market
          </button>
        </div>

        {showCreateForm && (
          <div className="mb-8">
            <CreateMarketForm onCreated={() => setShowCreateForm(false)} />
          </div>
        )}

        <MarketsList />

        <HowItWorks />
      </main>

      <footer className="border-t border-border-low mt-16">
        <div className="mx-auto max-w-5xl px-4 py-6">
          <div className="flex flex-wrap items-center justify-between gap-4 text-xs text-muted">
            <div className="flex items-center gap-2">
              <span className="rounded bg-yellow-100 px-2 py-0.5 text-yellow-800 font-medium">Devnet</span>
              <span>Built with Anchor + @solana/react-hooks</span>
            </div>
            <div className="flex gap-4">
              <a href="https://www.anchor-lang.com/docs" target="_blank" rel="noreferrer" className="hover:text-foreground transition">
                Anchor Docs
              </a>
              <a href="https://solana.com/docs" target="_blank" rel="noreferrer" className="hover:text-foreground transition">
                Solana Docs
              </a>
              <a href="https://faucet.solana.com/" target="_blank" rel="noreferrer" className="hover:text-foreground transition">
                Faucet
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
