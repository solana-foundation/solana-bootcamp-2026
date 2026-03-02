"use client";

import { type ReactNode, useCallback, useState } from "react";

import { type Address } from "@solana/kit";
import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";

import { getClaimWinningsInstructionAsync } from "../generated/prediction_market";
import { type Market } from "../generated/prediction_market/accounts/market";
import { type UserPosition } from "../generated/prediction_market/accounts/userPosition";

const LAMPORTS_PER_SOL = 1_000_000_000n;
const STATUS_CLEAR_DELAY_MS = 3000;

interface PositionCardProps {
  position: UserPosition;
  positionAddress: Address;
  market: Market | null;
  marketAddress: Address;
  onUpdate?: () => void;
  animationDelay?: number;
}

type PositionStatus = "active" | "won" | "lost" | "claimed";

function getPositionStatus(position: UserPosition, market: Market | null): PositionStatus {
  if (!market || !market.resolved) return "active";
  if (position.claimed) return "claimed";

  const outcome = market.outcome;
  if (outcome === null || outcome === undefined) return "active";

  const userWinningBet = outcome ? position.yesAmount : position.noAmount;
  return userWinningBet > 0n ? "won" : "lost";
}

function formatSol(lamports: bigint): string {
  const sol = Number(lamports) / Number(LAMPORTS_PER_SOL);
  if (sol === 0) return "0";
  if (sol < 0.01) return sol.toFixed(4);
  if (sol < 1) return sol.toFixed(3);
  return sol.toFixed(2);
}

function calculateWinnings(
  position: UserPosition,
  market: Market
): { payout: bigint; profit: bigint } | null {
  if (!market.resolved) return null;

  const outcome = market.outcome;
  if (outcome === null || outcome === undefined) return null;

  const userWinningBet = outcome ? position.yesAmount : position.noAmount;
  if (userWinningBet === 0n) return null;

  const winningPool = outcome ? market.yesPool : market.noPool;
  const losingPool = outcome ? market.noPool : market.yesPool;

  const profit = winningPool > 0n ? (userWinningBet * losingPool) / winningPool : 0n;
  const payout = userWinningBet + profit;

  return { payout, profit };
}

function getTimeInfo(market: Market | null): string | null {
  if (!market || market.resolved) return null;

  const now = Date.now() / 1000;
  const resolutionTime = Number(market.resolutionTime);
  const diff = resolutionTime - now;

  if (diff <= 0) return "Pending resolution";
  if (diff < 60) return `${Math.floor(diff)}s left`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m left`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h left`;
  return `${Math.floor(diff / 86400)}d left`;
}

export function PositionCard({
  position,
  positionAddress,
  market,
  marketAddress,
  onUpdate,
  animationDelay = 0,
}: PositionCardProps): ReactNode {
  const { wallet } = useWalletConnection();
  const { send, isSending } = useSendTransaction();
  const [txStatus, setTxStatus] = useState<string | null>(null);

  const status = getPositionStatus(position, market);
  const winnings = market ? calculateWinnings(position, market) : null;

  const handleClaim = useCallback(async () => {
    if (!wallet) return;

    try {
      setTxStatus("Claiming...");

      const instruction = await getClaimWinningsInstructionAsync({
        user: wallet.account,
        market: marketAddress,
      });

      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Claimed! ${signature?.slice(0, 8)}...`);
      setTimeout(() => setTxStatus(null), STATUS_CLEAR_DELAY_MS);
      onUpdate?.();
    } catch (err) {
      console.error("Claim failed:", err);
      const message = err instanceof Error ? err.message : "Unknown error";
      setTxStatus(`Error: ${message}`);
    }
  }, [wallet, marketAddress, send, onUpdate]);

  const statusConfig: Record<PositionStatus, { label: string; bgClass: string }> = {
    active: { label: "Active", bgClass: "bg-blue-100 text-blue-700" },
    won: { label: "Won", bgClass: "bg-green-100 text-green-700" },
    lost: { label: "Lost", bgClass: "bg-red-100 text-red-700" },
    claimed: { label: "Claimed", bgClass: "bg-gray-100 text-gray-600" },
  };

  const { label, bgClass } = statusConfig[status];
  const timeInfo = getTimeInfo(market);

  return (
    <div
      className="animate-fade-in rounded-xl border border-border-low bg-card overflow-hidden"
      style={{ animationDelay: `${animationDelay}ms` }}
    >
      <div className="p-4">
        {/* Header Row */}
        <div className="flex items-start justify-between gap-3 mb-3">
          <div className="flex-1 min-w-0">
            <h3 className="font-medium leading-snug truncate">
              {market?.question ?? "Unknown Market"}
            </h3>
            {timeInfo && (
              <p className="text-xs text-muted mt-0.5">{timeInfo}</p>
            )}
          </div>
          <span className={`shrink-0 rounded px-2 py-0.5 text-xs font-medium ${bgClass}`}>
            {label}
          </span>
        </div>

        {/* Bet Info */}
        <div className="flex flex-wrap gap-3 mb-3">
          {position.yesAmount > 0n && (
            <div className="flex items-center gap-1.5">
              <span className="inline-flex items-center justify-center w-5 h-5 rounded bg-green-100 text-green-700 text-xs font-bold">
                Y
              </span>
              <span className="font-mono text-sm">
                {formatSol(position.yesAmount)} SOL
              </span>
            </div>
          )}
          {position.noAmount > 0n && (
            <div className="flex items-center gap-1.5">
              <span className="inline-flex items-center justify-center w-5 h-5 rounded bg-red-100 text-red-700 text-xs font-bold">
                N
              </span>
              <span className="font-mono text-sm">
                {formatSol(position.noAmount)} SOL
              </span>
            </div>
          )}
        </div>

        {/* Result Info (for resolved markets) */}
        {market?.resolved && (
          <div className="text-sm">
            <div className="flex items-center gap-2 text-muted">
              <span>
                Outcome:{" "}
                <span
                  className={`font-medium ${
                    market.outcome ? "text-green-600" : "text-red-600"
                  }`}
                >
                  {market.outcome ? "YES" : "NO"}
                </span>
              </span>
              {winnings && (
                <>
                  <span className="text-border-low">|</span>
                  <span>
                    {status === "won" || status === "claimed" ? (
                      <>
                        Payout:{" "}
                        <span className="font-mono font-medium text-green-600">
                          {formatSol(winnings.payout)} SOL
                        </span>
                        <span className="text-green-600/70 ml-1">
                          (+{formatSol(winnings.profit)})
                        </span>
                      </>
                    ) : (
                      <span className="text-red-600">
                        Lost {formatSol(position.yesAmount + position.noAmount)} SOL
                      </span>
                    )}
                  </span>
                </>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Claim Button */}
      {status === "won" && !position.claimed && (
        <div className="border-t border-border-low p-3 bg-green-50">
          <button
            onClick={handleClaim}
            disabled={isSending}
            className="w-full rounded-lg bg-green-600 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-green-700 disabled:opacity-50"
          >
            {isSending ? "Claiming..." : `Claim ${formatSol(winnings?.payout ?? 0n)} SOL`}
          </button>
        </div>
      )}

      {/* Status Message */}
      {txStatus && (
        <div className="border-t border-border-low px-3 py-2 text-xs text-muted bg-cream/50">
          {txStatus}
        </div>
      )}
    </div>
  );
}
