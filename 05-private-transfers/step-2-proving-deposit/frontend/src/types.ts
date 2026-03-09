// Deposit note format - returned from backend after generating deposit
export interface DepositNote {
  nullifier: string;
  secret: string;
  amount: string;
  commitment: string;
  nullifierHash: string;
  merkleRoot: string;
  leafIndex: number;
  timestamp: number;
}

// onchain data format - used for deposit transaction
export interface OnChainData {
  commitment: number[];
  newRoot: number[];
  amount: string;
}

// Withdrawal proof format - returned from backend after generating ZK proof
export interface WithdrawalProof {
  proof: number[];
  publicWitness: number[];
  nullifierHash: string;
  merkleRoot: string;
  recipient: string;
  amount: string;
}

// API response types
export interface DepositApiResponse {
  depositNote: DepositNote;
  onChainData: OnChainData;
}

export interface WithdrawApiResponse {
  withdrawalProof: WithdrawalProof;
}
