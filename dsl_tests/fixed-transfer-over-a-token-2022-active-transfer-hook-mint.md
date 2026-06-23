## Fixed transfer over a Token-2022 active transfer-hook mint — PASS

> a transfer without the hook accounts fails; with them, the hook runs and the transfer succeeds

### Stage: Alice's authority and a fixed delegation to Bob

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Approve ──────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 15012cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token-2022::Approve [2] ✓ 1098cu
Compute Units (this run): 15012
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
    alice ->> subscriptions: InitSubscriptionAuthority (15012cu)
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
    subscriptions -->>- alice: ok (15012cu)
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
    5exAh63c_htJu[("5exAh63c…htJu")]:::writable
    System[System]:::program
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token_2022
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 5exAh63c_htJu
    Token_2022 -->|writes| 5exAh63c_htJu
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
    5exAh63c_htJu[("5exAh63c…htJu")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| 5exAh63c_htJu
```

**CreateFixedDelegation: structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✓ 17038cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 17038
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
    alice ->> subscriptions: CreateFixedDelegation (17038cu)
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
    subscriptions -->>- alice: ok (17038cu)
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

### Bob pulls without supplying the hook accounts; the transfer is refused

**TransferFixed (missing hook accounts): structured CPI tree**

```text

── subscriptions::TransferFixed ────────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✗ 7686cu  signer=bob
    └── Error: TransferHookValidationAccountMissing (0x8a)
Error: InstructionError(0, Custom(138))
Compute Units (this run): 7686
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

- [x] Bob's ATA is still empty: `0`

### Bob retries with the hook accounts attached; the hook runs

**TransferFixed (with hook accounts): structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✓ 20030cu  signer=bob
    ├── Token-2022::TransferChecked [2] ✓ 9403cu
    │   └── 3qbR1eZR…NzYh [3] ✓ 80cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 20030
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferFixed (with hook accounts): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token_2022 as "Token-2022"
    participant 3qbR1eZR_NzYh as "3qbR1eZR…NzYh"
    bob ->> subscriptions: TransferFixed (20030cu)
    subscriptions ->> Token_2022: TransferChecked (9403cu)
    Token_2022 ->> 3qbR1eZR_NzYh: unnamed (80cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferFixed (with hook accounts): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token_2022 as "Token-2022"
    participant 3qbR1eZR_NzYh as "3qbR1eZR…NzYh"
    bob ->>+ subscriptions: TransferFixed
    subscriptions ->>+ Token_2022: TransferChecked
    Token_2022 ->>+ 3qbR1eZR_NzYh: unnamed
    3qbR1eZR_NzYh -->>- Token_2022: ok (80cu)
    Token_2022 -->>- subscriptions: ok (9403cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (20030cu)
```

**TransferFixed (with hook accounts): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    FixedDelegation[(FixedDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    5exAh63c_htJu[("5exAh63c…htJu")]:::writable
    HhCNNxXG_Qr2S[("HhCNNxXG…Qr2S")]:::writable
    bob([bob]):::signer
    111BS654_KGXy[("111BS654…KGXy")]:::writable
    Token_2022["Token-2022"]:::program
    3qbR1eZR_NzYh["3qbR1eZR…NzYh"]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token_2022
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| FixedDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 5exAh63c_htJu
    subscriptions -->|writes| HhCNNxXG_Qr2S
    subscriptions -->|writes| 111BS654_KGXy
    Token_2022 -->|writes| 5exAh63c_htJu
    Token_2022 -->|writes| HhCNNxXG_Qr2S
    Token_2022 -->|writes| 111BS654_KGXy
    3qbR1eZR_NzYh -->|writes| 111BS654_KGXy
```

**TransferFixed (with hook accounts): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    FixedDelegation[(FixedDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token_2022["Token-2022"]:::owner
    5exAh63c_htJu[("5exAh63c…htJu")]:::account
    HhCNNxXG_Qr2S[("HhCNNxXG…Qr2S")]:::account
    System[System]:::owner
    bob[(bob)]:::account
    3qbR1eZR_NzYh["3qbR1eZR…NzYh"]:::owner
    111BS654_KGXy[("111BS654…KGXy")]:::account
    subscriptions -->|owns| FixedDelegation
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| 5exAh63c_htJu
    Token_2022 -->|owns| HhCNNxXG_Qr2S
    System -->|owns| bob
    3qbR1eZR_NzYh -->|owns| 111BS654_KGXy
```

- [x] Alice's ATA debited 10 tokens: `90000000`
- [x] Bob received 10 tokens: `10000000`
- [x] the transfer hook ran once: `1`
