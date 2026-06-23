## Create a plan (happy path) — PASS

> the merchant creates a 30-day plan and every field is recorded

### The merchant creates plan 1

**CreatePlan: structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✓ 3468cu  signer=merchant
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 3468
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```

**CreatePlan: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->> subscriptions: CreatePlan (3468cu)
    subscriptions ->> System: CreateAccount
```

**CreatePlan: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->>+ subscriptions: CreatePlan
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions -->>- merchant: ok (3468cu)
```

**CreatePlan: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    merchant([merchant]):::signer
    Plan([Plan]):::signer
    System[System]:::program
    merchant -->|signs| subscriptions
    merchant -->|signs| System
    Plan -->|signs| System
    subscriptions -->|writes| Plan
```

**CreatePlan: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions[subscriptions]:::owner
    Plan[(Plan)]:::account
    System -->|owns| merchant
    subscriptions -->|owns| Plan
```

- [x] the plan owner is the merchant: `J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq`
- [x] the plan is Active: `1`
- [x] the plan id is 1: `1`
- [x] the plan mint is the USDC mint: `1113XAVvsq5j34NSfwNn2SsxbHrHNkV7CXUsM5GKFyS`
- [x] the amount is 1_000_000: `1000000`
- [x] the period is 720 hours: `720`
- [x] the end timestamp matches: `1702592000`
- [x] the first destination is recorded: `1113ZS586N6P2fz22uf4tGQvE5GyXLQNd8qyMcWD4xJ`
- [x] the first puller is recorded: `1113cr4pCWJsAD6WthPPo7fEKJn24Hv7bmrrRhnw3BX`
