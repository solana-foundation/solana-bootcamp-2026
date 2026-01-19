"use client";

import { type ReactNode, useCallback, useEffect, useState } from "react";

import {
  type Address,
  getAddressEncoder,
  getBytesEncoder,
  getProgramDerivedAddress,
} from "@solana/kit";
import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";

import {
  getClaimWinningsInstructionAsync,
  getPlaceBetInstructionAsync,
  getResolveMarketInstruction,
  type Market,
  PREDICTION_MARKET_PROGRAM_ADDRESS,
} from "../generated/prediction_market";
import {
  getUserPositionDecoder,
  type UserPosition,
} from "../generated/prediction_market/accounts/userPosition";

const LAMPORTS_PER_SOL = 1_000_000_000n;
const DEVNET_RPC_URL = "https://api.devnet.solana.com";
const POSITION_SEED = new Uint8Array([112, 111, 115, 105, 116, 105, 111, 110]); // "position"
const POLL_INTERVAL_MS = 3000;
const STATUS_CLEAR_DELAY_MS = 3000;

interface MarketCardProps {
  market: Market;
  marketAddress: Address;
  onUpdate?: () => void;
}

function formatSol(lamports: bigint): string {
  const sol = Number(lamports) / Number(LAMPORTS_PER_SOL);
  if (sol < 0.01) return sol.toFixed(4);
  return sol.toFixed(2);
}

function getTimeRemaining(resolutionTime: number): string {
  const now = Date.now() / 1000;
  const diff = resolutionTime - now;
  if (diff <= 0) return "Ended";
  if (diff < 60) return `${Math.floor(diff)}s left`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m left`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h left`;
  return `${Math.floor(diff / 86400)}d left`;
}

function getStatusBadgeClass(isResolved: boolean, outcome: boolean | null | undefined, canBet: boolean): string {
  if (isResolved) {
    return outcome ? "bg-green-100 text-green-700" : "bg-red-100 text-red-700";
  }
  if (canBet) {
    return "bg-emerald-100 text-emerald-700";
  }
  return "bg-amber-100 text-amber-700";
}

function getStatusBadgeText(
  isResolved: boolean,
  outcome: boolean | null | undefined,
  canBet: boolean,
  resolutionTime: number
): string {
  if (isResolved) {
    return outcome ? "YES" : "NO";
  }
  if (canBet) {
    return getTimeRemaining(resolutionTime);
  }
  return "Pending";
}

async function fetchUserPositionFromRpc(
  marketAddress: Address,
  walletAddress: Address
): Promise<UserPosition | null> {
  const positionAddress = await getProgramDerivedAddress({
    programAddress: PREDICTION_MARKET_PROGRAM_ADDRESS,
    seeds: [
      getBytesEncoder().encode(POSITION_SEED),
      getAddressEncoder().encode(marketAddress),
      getAddressEncoder().encode(walletAddress),
    ],
  });

  const response = await fetch(DEVNET_RPC_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "getAccountInfo",
      params: [positionAddress[0], { encoding: "base64", commitment: "confirmed" }],
    }),
  });

  const result = await response.json();

  if (!result.result?.value) {
    return null;
  }

  const data = Uint8Array.from(atob(result.result.value.data[0]), (c) => c.charCodeAt(0));
  return getUserPositionDecoder().decode(data);
}

interface ClaimSectionProps {
  status: string;
  isResolved: boolean;
  userPosition: UserPosition | null;
  market: Market;
  isSending: boolean;
  onClaim: () => void;
}

function ClaimSection({
  status,
  isResolved,
  userPosition,
  market,
  isSending,
  onClaim,
}: ClaimSectionProps): ReactNode {
  if (status !== "connected" || !isResolved || !userPosition || userPosition.claimed) {
    return null;
  }

  const outcome = market.outcome;
  if (outcome === null || outcome === undefined) {
    return null;
  }

  const userWinningBet = outcome ? userPosition.yesAmount : userPosition.noAmount;
  if (userWinningBet === 0n) {
    return null;
  }

  const winningPool = outcome ? market.yesPool : market.noPool;
  const losingPool = outcome ? market.noPool : market.yesPool;
  const winnings = winningPool > 0n ? (userWinningBet * losingPool) / winningPool : 0n;
  const totalPayout = userWinningBet + winnings;

  return (
    <div className="border-t border-border-low p-3 bg-green-50">
      <button
        onClick={onClaim}
        disabled={isSending}
        className="w-full rounded-md bg-green-600 px-3 py-2 text-sm font-medium text-white transition hover:bg-green-700 disabled:opacity-40"
      >
        {isSending ? "Claiming..." : `Claim ${formatSol(totalPayout)} SOL`}
      </button>
    </div>
  );
}

export function MarketCard({ market, marketAddress, onUpdate }: MarketCardProps): ReactNode {
  const { wallet, status } = useWalletConnection();
  const { send, isSending } = useSendTransaction();

  const [betAmount, setBetAmount] = useState("");
  const [txStatus, setTxStatus] = useState<string | null>(null);
  const [userPosition, setUserPosition] = useState<UserPosition | null>(null);

  const walletAddress = wallet?.account.address;

  useEffect(() => {
    async function fetchPosition(): Promise<void> {
      if (!walletAddress) {
        setUserPosition(null);
        return;
      }

      try {
        const position = await fetchUserPositionFromRpc(marketAddress, walletAddress);
        setUserPosition(position);
      } catch (err) {
        console.warn("Failed to fetch user position:", err);
        setUserPosition(null);
      }
    }

    fetchPosition();
    const interval = setInterval(fetchPosition, POLL_INTERVAL_MS);
    return () => clearInterval(interval);
  }, [walletAddress, marketAddress]);
  const isCreator = walletAddress?.toString() === market.creator.toString();
  const isResolved = market.resolved;
  const now = Date.now() / 1000;
  const resolutionTime = Number(market.resolutionTime);
  const canBet = !isResolved && now < resolutionTime;
  const canResolve = !isResolved && now >= resolutionTime && isCreator;

  const totalPool = market.yesPool + market.noPool;
  const yesPercent = totalPool > 0n ? Number((market.yesPool * 100n) / totalPool) : 50;

  const handlePlaceBet = useCallback(async (betYes: boolean) => {
    if (!wallet || !walletAddress || !betAmount) return;

    try {
      setTxStatus("Building transaction...");
      const amount = BigInt(Math.floor(parseFloat(betAmount) * Number(LAMPORTS_PER_SOL)));

      const instruction = await getPlaceBetInstructionAsync({
        user: wallet.account,
        market: marketAddress,
        amount,
        betYes,
      });

      setTxStatus("Awaiting signature...");
      const signature = await send({ instructions: [instruction] });

      setTxStatus(`Done! ${signature?.slice(0, 8)}...`);
      setBetAmount("");
      setTimeout(() => setTxStatus(null), STATUS_CLEAR_DELAY_MS);
      onUpdate?.();
    } catch (err) {
      console.error("Place bet failed:", err);
      const message = err instanceof Error ? err.message : "Unknown error";
      setTxStatus(`Error: ${message}`);
    }
  }, [wallet, walletAddress, marketAddress, betAmount, send, onUpdate]);

  const handleResolve = useCallback(async (outcome: boolean) => {
    if (!wallet || !walletAddress) return;

    try {
      setTxStatus("Resolving...");

      const instruction = getResolveMarketInstruction({
        creator: wallet.account,
        market: marketAddress,
        outcome,
      });

      const signature = await send({ instructions: [instruction] });
      setTxStatus(`Resolved! ${signature?.slice(0, 8)}...`);
      setTimeout(() => setTxStatus(null), STATUS_CLEAR_DELAY_MS);
      onUpdate?.();
    } catch (err) {
      console.error("Resolve failed:", err);
      const message = err instanceof Error ? err.message : "Unknown error";
      setTxStatus(`Error: ${message}`);
    }
  }, [wallet, walletAddress, marketAddress, send, onUpdate]);

  const handleClaim = useCallback(async () => {
    if (!wallet || !walletAddress) return;

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
  }, [wallet, walletAddress, marketAddress, send, onUpdate]);

  const badgeClass = getStatusBadgeClass(isResolved, market.outcome, canBet);
  const badgeText = getStatusBadgeText(isResolved, market.outcome, canBet, resolutionTime);

  return (
    <div className="rounded-xl border border-border-low bg-card overflow-hidden">
      <div className="p-4 pb-3">
        <div className="flex items-start justify-between gap-3 mb-3">
          <h3 className="font-medium leading-snug">{market.question}</h3>
          <span className={`shrink-0 rounded px-2 py-0.5 text-xs font-medium ${badgeClass}`}>
            {badgeText}
          </span>
        </div>

        {/* Probability Bar */}
        <div className="mb-2">
          <div className="h-2 rounded-full overflow-hidden bg-red-200 flex">
            <div
              className="bg-green-500 transition-all duration-300"
              style={{ width: `${yesPercent}%` }}
            />
          </div>
          <div className="flex justify-between mt-1.5 text-xs">
            <span className="text-green-600 font-medium">{yesPercent}% Yes</span>
            <span className="text-red-600 font-medium">{100 - yesPercent}% No</span>
          </div>
        </div>

        {/* Pool Info */}
        <div className="flex items-center gap-4 text-xs text-muted">
          <span>{formatSol(totalPool)} SOL pool</span>
          {isCreator && <span className="text-blue-600">You created this</span>}
        </div>
      </div>

      {/* Actions */}
      {status === "connected" && canBet && (
        <div className="border-t border-border-low p-3 bg-cream/30">
          <div className="flex gap-2">
            <input
              type="number"
              min="0"
              step="0.01"
              placeholder="SOL"
              value={betAmount}
              onChange={(e) => setBetAmount(e.target.value)}
              disabled={isSending}
              className="w-20 rounded-md border border-border-low bg-card px-2 py-1.5 text-sm outline-none placeholder:text-muted focus:border-foreground/30 disabled:opacity-60"
            />
            <button
              onClick={() => handlePlaceBet(true)}
              disabled={isSending || !betAmount || parseFloat(betAmount) <= 0}
              className="flex-1 rounded-md bg-green-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-green-700 disabled:opacity-40"
            >
              Bet Yes
            </button>
            <button
              onClick={() => handlePlaceBet(false)}
              disabled={isSending || !betAmount || parseFloat(betAmount) <= 0}
              className="flex-1 rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-red-700 disabled:opacity-40"
            >
              Bet No
            </button>
          </div>
        </div>
      )}

      {status === "connected" && canResolve && (
        <div className="border-t border-border-low p-3 bg-amber-50">
          <p className="text-xs text-amber-700 mb-2">You can now resolve this market</p>
          <div className="flex gap-2">
            <button
              onClick={() => handleResolve(true)}
              disabled={isSending}
              className="flex-1 rounded-md bg-green-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-green-700 disabled:opacity-40"
            >
              Yes Won
            </button>
            <button
              onClick={() => handleResolve(false)}
              disabled={isSending}
              className="flex-1 rounded-md bg-red-600 px-3 py-1.5 text-sm font-medium text-white transition hover:bg-red-700 disabled:opacity-40"
            >
              No Won
            </button>
          </div>
        </div>
      )}

      <ClaimSection
        status={status}
        isResolved={isResolved}
        userPosition={userPosition}
        market={market}
        isSending={isSending}
        onClaim={handleClaim}
      />

      {/* Status Message */}
      {txStatus && (
        <div className="border-t border-border-low px-3 py-2 text-xs text-muted bg-cream/50">
          {txStatus}
        </div>
      )}
    </div>
  );
}
