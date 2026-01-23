"use client";

import { type ReactNode, useCallback, useState } from "react";

import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";

import { getCreateMarketInstructionAsync } from "../generated/prediction_market";

interface CreateMarketFormProps {
  onCreated?: () => void;
}

const PLACEHOLDER_QUESTION = "Will SOL hit $200 this month?";
const SECONDS_PER_MINUTE = 60;
const MILLISECONDS_PER_SECOND = 1000;

export function CreateMarketForm({ onCreated }: CreateMarketFormProps): ReactNode {
  const { wallet, status } = useWalletConnection();
  const { send, isSending } = useSendTransaction();

  const [question, setQuestion] = useState("");
  const [durationMinutes, setDurationMinutes] = useState("5");
  const [txStatus, setTxStatus] = useState<string | null>(null);

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>): void {
    if (e.key === "Tab" && !question) {
      e.preventDefault();
      setQuestion(PLACEHOLDER_QUESTION);
    }
  }

  const walletAddress = wallet?.account.address;

  const handleCreate = useCallback(async () => {
    if (!wallet || !walletAddress || !question.trim()) return;

    try {
      setTxStatus("Creating market...");

      const marketId = BigInt(Date.now());
      const nowInSeconds = Math.floor(Date.now() / MILLISECONDS_PER_SECOND);
      const durationInSeconds = parseInt(durationMinutes) * SECONDS_PER_MINUTE;
      const resolutionTime = BigInt(nowInSeconds + durationInSeconds);

      const instruction = await getCreateMarketInstructionAsync({
        creator: wallet.account,
        marketId,
        question: question.trim(),
        resolutionTime,
      });

      const signature = await send({ instructions: [instruction] });

      setTxStatus(`Created! ${signature?.slice(0, 8)}...`);
      setQuestion("");
      setTimeout(() => {
        setTxStatus(null);
        onCreated?.();
      }, 1500);
    } catch (err) {
      console.error("Create market failed:", err);
      const message = err instanceof Error ? err.message : "Unknown error";
      setTxStatus(`Error: ${message}`);
    }
  }, [wallet, walletAddress, question, durationMinutes, send, onCreated]);

  if (status !== "connected") {
    return (
      <div className="rounded-xl border border-border-low bg-card p-4">
        <p className="text-sm text-muted text-center">Connect your wallet to create a market</p>
      </div>
    );
  }

  return (
    <div className="rounded-xl border border-border-low bg-card p-4 space-y-4">
      <div>
        <label className="block text-xs font-medium text-muted mb-1.5">Question (yes/no)</label>
        <input
          type="text"
          placeholder={PLACEHOLDER_QUESTION}
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={handleKeyDown}
          disabled={isSending}
          maxLength={200}
          className="w-full rounded-md border border-border-low bg-card px-3 py-2 text-sm outline-none placeholder:text-muted/60 focus:border-foreground/30 disabled:opacity-60"
        />
        <p className="text-xs text-muted/50 mt-1">Press Tab to use suggestion</p>
      </div>

      <div className="flex gap-3">
        <div className="flex-1">
          <label className="block text-xs font-medium text-muted mb-1.5">Betting ends in</label>
          <select
            value={durationMinutes}
            onChange={(e) => setDurationMinutes(e.target.value)}
            disabled={isSending}
            className="w-full rounded-md border border-border-low bg-card px-3 py-2 text-sm outline-none focus:border-foreground/30 disabled:opacity-60"
          >
            <option value="2">2 minutes</option>
            <option value="5">5 minutes</option>
            <option value="60">1 hour</option>
            <option value="1440">1 day</option>
            <option value="10080">1 week</option>
          </select>
        </div>
        <div className="flex items-end">
          <button
            onClick={handleCreate}
            disabled={isSending || !question.trim()}
            className="rounded-md bg-foreground px-6 py-2 text-sm font-medium text-background transition hover:opacity-90 disabled:opacity-40"
          >
            {isSending ? "..." : "Create"}
          </button>
        </div>
      </div>

      {txStatus && (
        <p className="text-xs text-muted">{txStatus}</p>
      )}
    </div>
  );
}
