# UNSEEN ⌀

**Privacy-preserving image takedown demo. Your photo never leaves your device, only its fingerprint does.**

> ⚠️ This is a portfolio demo inspired by StopNCII. It is not connected to any real platform and uses test images only.

## What it does

1. You pick an image in the browser.
2. The browser computes a 256-bit perceptual hash (PDQ-style) locally. The image itself is never uploaded.
3. Only the hash is sent to the API, which stores it and searches for near-duplicates using Hamming distance.
4. When a match is registered, a simulated social platform is notified through signed webhooks (HMAC), with automatic retries on failure.

## Tech stack

| Layer | Tech |
|---|---|
| Frontend | React + TypeScript (in-browser hashing, Rust → WebAssembly planned) |
| API | Rust, Axum |
| Database | PostgreSQL (hash storage + Hamming-distance search) |
| Queue / worker | Redis-backed webhook worker with retries |
| Mock platform | Simulated social platform receiving signed webhooks |
| Infra | Docker Compose, GitHub Actions CI/CD |

## Status

🚧 Work in progress.

- [ ] Week 1: Axum API + PostgreSQL in Docker, store hashes, near-match search
- [ ] Week 2: Mock platform + signed webhook worker with retries
- [ ] Week 3: React page with in-browser hashing
- [ ] Week 4: Deploy, tests, CI, docs

Demo only: not connected to real platforms, test images only.

## License

MIT, see [LICENSE](LICENSE).
