## Revoke clears the delegate (Token-2022) — PASS

> Alice revokes her authority over a Token-2022 mint; her ATA's delegate is cleared

### Alice initializes her subscription authority

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Approve ──────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 8854cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token-2022::Approve [2] ✓ 1035cu
Compute Units (this run): 8854
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
    alice ->> subscriptions: InitSubscriptionAuthority (8854cu)
    subscriptions ->> System: CreateAccount
    subscriptions ->> Token_2022: Approve (1035cu)
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
    Token_2022 -->>- subscriptions: ok (1035cu)
    subscriptions -->>- alice: ok (8854cu)
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
    CxMgEZ2H_jZ3x[("CxMgEZ2H…jZ3x")]:::writable
    System[System]:::program
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token_2022
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| CxMgEZ2H_jZ3x
    Token_2022 -->|writes| CxMgEZ2H_jZ3x
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
    CxMgEZ2H_jZ3x[("CxMgEZ2H…jZ3x")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| CxMgEZ2H_jZ3x
```

- [x] the ATA is delegated for the full amount before revoke: `18446744073709551615`

### Alice revokes her subscription authority

**RevokeSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Revoke ───────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::RevokeSubscriptionAuthority [1] ✓ 3961cu  signer=alice
    └── Token-2022::Revoke [2] ✓ 790cu
Compute Units (this run): 3961
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**RevokeSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant Token_2022 as "Token-2022"
    alice ->> subscriptions: RevokeSubscriptionAuthority (3961cu)
    subscriptions ->> Token_2022: Revoke (790cu)
```

**RevokeSubscriptionAuthority: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant Token_2022 as "Token-2022"
    alice ->>+ subscriptions: RevokeSubscriptionAuthority
    subscriptions ->>+ Token_2022: Revoke
    Token_2022 -->>- subscriptions: ok (790cu)
    subscriptions -->>- alice: ok (3961cu)
```

**RevokeSubscriptionAuthority: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    CxMgEZ2H_jZ3x[("CxMgEZ2H…jZ3x")]:::writable
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| Token_2022
    subscriptions -->|writes| CxMgEZ2H_jZ3x
    Token_2022 -->|writes| CxMgEZ2H_jZ3x
```

**RevokeSubscriptionAuthority: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    alice[(alice)]:::account
    Token_2022["Token-2022"]:::owner
    CxMgEZ2H_jZ3x[("CxMgEZ2H…jZ3x")]:::account
    System -->|owns| alice
    Token_2022 -->|owns| CxMgEZ2H_jZ3x
```

- [x] the delegate is cleared after revoke: `true`
- [x] the delegated amount is zeroed after revoke: `0`
