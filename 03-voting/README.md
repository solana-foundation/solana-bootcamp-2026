# Voting Program

A on-chain polling program built with [Anchor](https://www.anchor-lang.com/) on Solana. Demonstrates how to use PDAs to store structured state, pass typed arguments through instructions, and enforce time-gated access rules.

## Overview

Anyone can create a poll with a name, description, and a voting window. The poll creator then adds candidate options. During the voting window any wallet can cast a vote for a candidate; votes outside the window are rejected on-chain.

## Program ID

```
65KHV8cXwJ8apTKMqnpSdhdHkHhRySatgKMwnxm6C3gG
```

## Prerequisites

- [Rust](https://rustup.rs/)
- [Solana CLI](https://solana.com/developers/guides/getstarted/setup-local-development)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation) v1.0.0-rc.2
- [Node.js](https://nodejs.org/) + [Yarn](https://yarnpkg.com/)

## Building

```bash
cd anchor
anchor build
```

## Testing

Tests are written in TypeScript using [@anchor-lang/core](https://www.npmjs.com/package/@anchor-lang/core) and run with Jest against a local validator.

```bash
cd anchor
yarn install
yarn jest
```

## Instructions

### `initialize_poll`

Creates a new poll account.

| Argument      | Type   | Description                              |
|---------------|--------|------------------------------------------|
| `poll_id`     | u64    | Unique identifier used as a PDA seed     |
| `start_time`  | u64    | Unix timestamp when voting opens         |
| `end_time`    | u64    | Unix timestamp when voting closes        |
| `name`        | String | Poll name (max 32 chars)                 |
| `description` | String | Poll description (max 280 chars)         |

### `initialize_candidate`

Adds a candidate option to an existing poll.

| Argument    | Type   | Description                                    |
|-------------|--------|------------------------------------------------|
| `poll_id`   | u64    | ID of the poll to add the candidate to         |
| `candidate` | String | Candidate name, also used as a PDA seed        |

### `vote`

Casts a vote for a candidate. Reverts if the current time is outside the poll's voting window.

| Argument    | Type   | Description                          |
|-------------|--------|--------------------------------------|
| `poll_id`   | u64    | ID of the poll                       |
| `candidate` | String | Name of the candidate to vote for    |

## Accounts

### `PollAccount` — PDA seeds: `["poll", poll_id (little-endian u64)]`

| Field                | Type   | Description                              |
|----------------------|--------|------------------------------------------|
| `poll_name`          | String | Name of the poll (max 32 chars)          |
| `poll_description`   | String | Description (max 280 chars)              |
| `poll_voting_start`  | u64    | Unix timestamp when voting opens         |
| `poll_voting_end`    | u64    | Unix timestamp when voting closes        |
| `poll_option_index`  | u64    | Number of candidates added so far        |

### `CandidateAccount` — PDA seeds: `[poll_id (little-endian u64), candidate_name]`

| Field              | Type   | Description                    |
|--------------------|--------|--------------------------------|
| `candidate_name`   | String | Candidate name (max 32 chars)  |
| `candidate_votes`  | u64    | Total votes received           |

## Error Codes

| Code              | Message                     |
|-------------------|-----------------------------|
| `VotingNotStarted` | Voting has not started yet |
| `VotingEnded`      | Voting has ended           |
