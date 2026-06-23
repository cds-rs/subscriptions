## Fixed transfer succeeds — PASS

> Bob pulls part of his fixed allowance; the remaining allowance decrements

### Stage: Alice's authority and a fixed delegation to Bob

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 7742cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 7742
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
    participant Token
    alice ->> subscriptions: InitSubscriptionAuthority (7742cu)
    subscriptions ->> System: CreateAccount
    subscriptions ->> Token: Approve (126cu)
```

**InitSubscriptionAuthority: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token
    alice ->>+ subscriptions: InitSubscriptionAuthority
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: Approve
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (7742cu)
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
    6WzVuHNZ_gf7X[("6WzVuHNZ…gf7X")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 6WzVuHNZ_gf7X
    Token -->|writes| 6WzVuHNZ_gf7X
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
    Token[Token]:::owner
    6WzVuHNZ_gf7X[("6WzVuHNZ…gf7X")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 6WzVuHNZ_gf7X
```

**CreateFixedDelegation: structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✓ 5038cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 5038
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**CreateFixedDelegation: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->> subscriptions: CreateFixedDelegation (5038cu)
    subscriptions ->> System: CreateAccount
```

**CreateFixedDelegation: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->>+ subscriptions: CreateFixedDelegation
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions -->>- alice: ok (5038cu)
```

**CreateFixedDelegation: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority[(SubAuthority)]:::writable
    FixedDelegation([FixedDelegation]):::signer
    System[System]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    FixedDelegation -->|signs| System
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| FixedDelegation
```

**CreateFixedDelegation: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    alice[(alice)]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    FixedDelegation[(FixedDelegation)]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    subscriptions -->|owns| FixedDelegation
```

- [x] Bob's ATA starts empty: `0`

### Bob pulls 30 tokens against his fixed delegation

**TransferFixed: structured CPI tree**

```text

── subscriptions::TransferFixed ────────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✓ 7055cu  signer=bob
    ├── Token::TransferChecked [2] ✓ 113cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 7055
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferFixed: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->> subscriptions: TransferFixed (7055cu)
    subscriptions ->> Token: TransferChecked (113cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferFixed: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->>+ subscriptions: TransferFixed
    subscriptions ->>+ Token: TransferChecked
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (7055cu)
```

**TransferFixed: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    FixedDelegation[(FixedDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    6WzVuHNZ_gf7X[("6WzVuHNZ…gf7X")]:::writable
    Cr5codBi_dn9k[("Cr5codBi…dn9k")]:::writable
    bob([bob]):::signer
    Token[Token]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| FixedDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 6WzVuHNZ_gf7X
    subscriptions -->|writes| Cr5codBi_dn9k
    Token -->|writes| 6WzVuHNZ_gf7X
    Token -->|writes| Cr5codBi_dn9k
```

**TransferFixed: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    FixedDelegation[(FixedDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token[Token]:::owner
    6WzVuHNZ_gf7X[("6WzVuHNZ…gf7X")]:::account
    Cr5codBi_dn9k[("Cr5codBi…dn9k")]:::account
    System[System]:::owner
    bob[(bob)]:::account
    subscriptions -->|owns| FixedDelegation
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 6WzVuHNZ_gf7X
    Token -->|owns| Cr5codBi_dn9k
    System -->|owns| bob
```

- [x] Bob received 30 tokens: `30000000`
- [x] the remaining allowance is 20 tokens: `20000000`
- [x] the expiry is unchanged: `1700086400`
