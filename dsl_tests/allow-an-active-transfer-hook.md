## Allow an active transfer hook — PASS

> a mint carrying a configured transfer hook is accepted

### Alice initializes over a mint with an active transfer hook

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Approve ──────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 9012cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token-2022::Approve [2] ✓ 1098cu
Compute Units (this run): 9012
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**InitSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token_2022 as "Token-2022"
    alice ->> subscriptions: InitSubscriptionAuthority (9012cu)
    subscriptions ->> System: CreateAccount
    subscriptions ->> Token_2022: Approve (1098cu)
```

**InitSubscriptionAuthority: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token_2022 as "Token-2022"
    alice ->>+ subscriptions: InitSubscriptionAuthority
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions ->>+ Token_2022: Approve
    Token_2022 -->>- subscriptions: ok (1098cu)
    subscriptions -->>- alice: ok (9012cu)
```

**InitSubscriptionAuthority: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority([SubAuthority]):::signer
    Ens4LR9N_KDG5[("Ens4LR9N…KDG5")]:::writable
    System[System]:::program
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token_2022
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| Ens4LR9N_KDG5
    Token_2022 -->|writes| Ens4LR9N_KDG5
```

**InitSubscriptionAuthority: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    alice[(alice)]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    Token_2022["Token-2022"]:::owner
    Ens4LR9N_KDG5[("Ens4LR9N…KDG5")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| Ens4LR9N_KDG5
```
