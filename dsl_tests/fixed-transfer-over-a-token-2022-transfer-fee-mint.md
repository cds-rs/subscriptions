## Fixed transfer over a Token-2022 transfer-fee mint — PASS

> the transfer fee is withheld from the receiver; the allowance decrements by the gross amount

### Stage: Alice's authority and a fixed delegation to Bob

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
    6EP2bypm_idic[("6EP2bypm…idic")]:::writable
    System[System]:::program
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token_2022
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 6EP2bypm_idic
    Token_2022 -->|writes| 6EP2bypm_idic
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
    6EP2bypm_idic[("6EP2bypm…idic")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| 6EP2bypm_idic
```

**CreateFixedDelegation: structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✓ 8038cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 8038
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
    alice ->> subscriptions: CreateFixedDelegation (8038cu)
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
    subscriptions -->>- alice: ok (8038cu)
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

### Bob pulls 10 tokens; the transfer fee is withheld

**TransferFixed: structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✓ 8523cu  signer=bob
    ├── Token-2022::TransferChecked [2] ✓ 2856cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 8523
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
    participant Token_2022 as "Token-2022"
    bob ->> subscriptions: TransferFixed (8523cu)
    subscriptions ->> Token_2022: TransferChecked (2856cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferFixed: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token_2022 as "Token-2022"
    bob ->>+ subscriptions: TransferFixed
    subscriptions ->>+ Token_2022: TransferChecked
    Token_2022 -->>- subscriptions: ok (2856cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (8523cu)
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
    6EP2bypm_idic[("6EP2bypm…idic")]:::writable
    9ndcvDPo_NhCk[("9ndcvDPo…NhCk")]:::writable
    bob([bob]):::signer
    Token_2022["Token-2022"]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token_2022
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| FixedDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 6EP2bypm_idic
    subscriptions -->|writes| 9ndcvDPo_NhCk
    Token_2022 -->|writes| 6EP2bypm_idic
    Token_2022 -->|writes| 9ndcvDPo_NhCk
```

**TransferFixed: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    FixedDelegation[(FixedDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token_2022["Token-2022"]:::owner
    6EP2bypm_idic[("6EP2bypm…idic")]:::account
    9ndcvDPo_NhCk[("9ndcvDPo…NhCk")]:::account
    System[System]:::owner
    bob[(bob)]:::account
    subscriptions -->|owns| FixedDelegation
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| 6EP2bypm_idic
    Token_2022 -->|owns| 9ndcvDPo_NhCk
    System -->|owns| bob
```

- [x] Alice's ATA debited the gross 10 tokens: `90000000`
- [x] Bob received 10 tokens net of the 1% fee: `9900000`
- [x] the allowance decrements by the gross 10 tokens: `40000000`
