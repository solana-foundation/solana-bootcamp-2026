"use client";

import { useState } from "react";
import { useSendTransaction, useWalletConnection } from "@solana/react-hooks";
import labubuMetadata from "../../labubu-metadata.json";
import { getMintRandomInstructionAsync } from "../generated/vault/instructions/mintRandom";
import { get } from "http";

export function LabubuCard() {
  const { wallet, status } = useWalletConnection();
  const [selectedLabubu, setSelectedLabubu] = useState<number | null>(null);

  const { send, isSending } = useSendTransaction();
  const [txSignature, setTxSignature] = useState<string | null>(null);

  const MINT_ADRRESSES = [
    "xU75nFnFbmktGubuXdoy14Eg8YT5D1XkMUC7K97ABMQ",
    "7oD87fuM4wkiP2nrRCpEwG8GHczSuC64Lqzsoq27eHV3",
    "FxuRVdYTfACd9d68zjSQUxN5qqB8FcgEZmf4aNu2umdg",

    "422ewKXmBgoDv9BzEU82TPoWryxvMav9gpVPv9DpeZN8",
    "G2qTj7kNEMgPSMbJ8GS19cLNU8ogKxQojFUE7ihVFrfU",
    "6rGKN6oexWERQJrJFyfU5uTVxN9pswRKAcyry9o9RCYj",

    "BwqNNaibmqEpGz6b8RseQcuXSEgQi2cgsjEhPwmqBjLv",
    "C8AN1xqCTTvSaPjBvRMRQ8jkSczzZTXpTtv5Qx251Zbo",
    "4Y4m9ky35ReZCZhJ1Frq5LWbykEjghYacCUPdCb3eG9p",
    "7Tu4A7BtiGMQ9N1JxGucygxRf47GP3q3Bm7SUKZb3xfF",
    "HLDQAYqyPBK7RKCUoePjd2mSapk5BGPgHGU5b7zeRKWP",
  ];


  // Mock random Labubu NFT minting
  const handleMint = async () => {
    if (!wallet || status !== "connected") {
      alert("Please connect your wallet first!");
      return;
    }

    try {
      // Simulated random selection logic
      const totalWeight = 1206; // 120*10 + 6
      const random = Math.floor(Math.random() * totalWeight);

      let cumulative = 0;
      let selected = 1;

      for (const labubu of labubuMetadata.labubus) {
        cumulative += labubu.supply;
        if (random < cumulative) {
          selected = labubu.id;
          break;
        }
      }

      // TODO: Call on-chain program to mint NFT
      // await mintLabubuNFT(selected.id);
      const mintAddress = MINT_ADRRESSES[selected - 1];

      const userSigner = {
        address: wallet.account.address,
        signTransaction: wallet.signTransaction,
        signMessage: wallet.signMessage,
      };

      const instruction = await getMintRandomInstructionAsync({
        user: userSigner as any,
        mint: mintAddress as any,
        labubuId: selected,
      });

      const signature = await send({
        instructions: [instruction],
      });
      
      setTxSignature(signature);
      setSelectedLabubu(selected);

      alert(`Successfully minted Labubu #${selected}! Transaction: ${signature}`);
    } catch (error: any) {
      alert("Failed to mint Labubu. " + (error.message || "Please try again."));
    }
  };

  const isConnected = status === "connected";
  const currentLabubu = selectedLabubu
    ? labubuMetadata.labubus.find((l) => l.id === selectedLabubu)
    : null;

  return (
    <div className="card">
      <h2>Labubu Mystery Box</h2>
      <p className="subtitle">
        {isConnected
          ? "Click the button to open your mystery box!"
          : "Connect your wallet to get started"}
      </p>

      <div className="labubu-display">
        {!selectedLabubu ? (
          <div className="mystery-box">
            <div className="box-icon">?</div>
            <p>Mystery Box</p>
          </div>
        ) : (
          <div className="labubu-reveal">
            <img
              src={currentLabubu?.image}
              alt={currentLabubu?.name}
              className="labubu-image"
            />
            <h3>{currentLabubu?.name}</h3>
            <p className="description">{currentLabubu?.description}</p>
            <div className="attributes">
              {currentLabubu?.attributes.map((attr, i) => (
                <span key={i} className="badge">
                  {attr.trait_type}: {attr.value}
                </span>
              ))}
            </div>
            {currentLabubu?.rarity === "legendary" && (
              <div className="legendary-badge">🌟 LEGENDARY 🌟</div>
            )}
          </div>
        )}
      </div>

      <button
        className="btn-primary"
        onClick={handleMint}
        disabled={!isConnected}
      >
        Open Mystery Box
      </button>

      <div className="collection-stats">
        <p>Total Labubus: {labubuMetadata.labubus.length} types</p>
        <p>
          Legendary Supply:{" "}
          {labubuMetadata.labubus.find((l) => l.rarity === "legendary")?.supply || 0}
        </p>
      </div>

      <style jsx>{`
        .card {
          background: white;
          border-radius: 12px;
          padding: 2rem;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
          max-width: 500px;
          margin: 0 auto;
        }

        h2 {
          margin: 0 0 0.5rem 0;
          font-size: 1.75rem;
          color: #333;
        }

        .subtitle {
          color: #666;
          margin-bottom: 1.5rem;
        }

        .labubu-display {
          min-height: 400px;
          display: flex;
          align-items: center;
          justify-content: center;
          margin: 1.5rem 0;
          border: 2px dashed #ddd;
          border-radius: 8px;
          padding: 2rem;
        }

        .mystery-box {
          text-align: center;
        }

        .box-icon {
          font-size: 8rem;
          color: #ff6b9d;
          animation: pulse 2s infinite;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        .labubu-reveal {
          text-align: center;
          animation: fadeIn 0.5s ease-in;
        }

        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: scale(0.8);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }

        .labubu-image {
          max-width: 300px;
          width: 100%;
          height: auto;
          border-radius: 8px;
          margin-bottom: 1rem;
        }

        h3 {
          font-size: 1.5rem;
          color: #333;
          margin: 0.5rem 0;
        }

        .description {
          color: #666;
          margin-bottom: 1rem;
        }

        .attributes {
          display: flex;
          flex-wrap: wrap;
          gap: 0.5rem;
          justify-content: center;
          margin-bottom: 1rem;
        }

        .badge {
          background: #f0f0f0;
          padding: 0.25rem 0.75rem;
          border-radius: 12px;
          font-size: 0.875rem;
          color: #555;
        }

        .legendary-badge {
          background: linear-gradient(45deg, #ffd700, #ffed4e);
          color: #333;
          font-weight: bold;
          padding: 0.5rem 1rem;
          border-radius: 8px;
          margin-top: 1rem;
          animation: shimmer 2s infinite;
        }

        @keyframes shimmer {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.8;
          }
        }

        .tx-link {
          display: inline-block;
          margin-top: 1rem;
          padding: 0.5rem 1rem;
          background: #667eea;
          color: white;
          border-radius: 6px;
          text-decoration: none;
          font-size: 0.875rem;
          transition: background 0.2s;
        }

        .tx-link:hover {
          background: #5568d3;
        }

        .btn-primary {
          width: 100%;
          padding: 1rem;
          font-size: 1.125rem;
          font-weight: 600;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          border: none;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary:hover:not(:disabled) {
          transform: translateY(-2px);
          box-shadow: 0 6px 12px rgba(102, 126, 234, 0.3);
        }

        .btn-primary:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .collection-stats {
          margin-top: 1.5rem;
          padding-top: 1.5rem;
          border-top: 1px solid #eee;
          text-align: center;
          color: #666;
          font-size: 0.875rem;
        }

        .collection-stats p {
          margin: 0.25rem 0;
        }
      `}</style>
    </div>
  );
}
