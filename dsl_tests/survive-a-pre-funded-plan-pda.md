## Survive a pre-funded plan PDA — PASS

> a griefer pre-funds the plan PDA; the merchant can still create the plan

### Despite the pre-funded PDA, the merchant creates the plan

**CreatePlan (pre-funded PDA): structured CPI tree**

```text

── subscriptions::CreatePlan ───────────────────────────────
Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✓ 5726cu  signer=merchant
    ├── System::Transfer (merchant -> Plan) 4,307,240 lamports [2] ✓ (no cu)
    ├── System::Allocate [2] ✓ (no cu)
    └── System::Assign [2] ✓ (no cu)
Compute Units (this run): 5726
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
```

**CreatePlan (pre-funded PDA): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->> subscriptions: CreatePlan (5726cu)
    subscriptions ->> System: Transfer (merchant → Plan) 4,307,240 lamports
    subscriptions ->> System: Allocate
    subscriptions ->> System: Assign
```

**CreatePlan (pre-funded PDA): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->>+ subscriptions: CreatePlan
    subscriptions ->>+ System: Transfer (merchant → Plan) 4,307,240 lamports
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Allocate
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Assign
    System -->>- subscriptions: ok
    subscriptions -->>- merchant: ok (5726cu)
```

**CreatePlan (pre-funded PDA): authority graph**

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
    System -->|writes| Plan
```

**CreatePlan (pre-funded PDA): ownership graph**

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
