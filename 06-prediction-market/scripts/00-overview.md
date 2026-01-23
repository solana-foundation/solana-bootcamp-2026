# Video Lecture Script: Full-Stack Solana Prediction Markets

**Total Runtime:** 90 minutes
**Total Target:** ~7,350 words
**Speaking Pace:** 150 wpm (adjusted for visuals)

---

## Parts

| File | Part | Duration | Words |
|------|------|----------|-------|
| `01-why-and-what.md` | Why & What | 15 min | ~1,300 |
| `02-on-chain-program.md` | On-Chain Program | 30 min | ~2,350 |
| `03-codegen-layer.md` | Codegen Layer | 15 min | ~1,150 |
| `04-frontend-architecture.md` | Frontend Architecture | 20 min | ~1,500 |
| `05-security-tradeoffs.md` | Security & Trade-offs | 5 min | ~550 |
| `06-recap.md` | Recap | 5 min | ~500 |

---

## Production Checklist

- [ ] Record Part 2 first (core content)
- [ ] Create diagram assets before recording
- [ ] Test code snippets are readable at 1080p
- [ ] Review pacing after first pass

---

## File References

| Topic | File Path |
|-------|-----------|
| Account structures | `anchor/programs/prediction_market/src/state.rs` |
| Program instructions | `anchor/programs/prediction_market/src/lib.rs` |
| Error definitions | `anchor/programs/prediction_market/src/errors.rs` |
| Generated client | `app/generated/prediction_market/` |
| Markets list component | `app/components/markets-list.tsx` |
| Market card component | `app/components/market-card.tsx` |
| Wallet providers | `app/components/providers.tsx` |
| Codama config | `codama.json` |
