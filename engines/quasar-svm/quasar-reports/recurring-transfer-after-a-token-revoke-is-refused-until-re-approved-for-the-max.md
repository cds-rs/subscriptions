## Recurring transfer after a token Revoke is refused until re-approved for the max — PASS

> revoking the SPL approval breaks the pull; a partial re-approval still fails; only a max approval restores it

### Stage: Alice's authority and a recurring delegation to Bob

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::Approve ──────────────────────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 9020cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 2904cu
Compute Units (this run): 9020
Fee: 0 lamports
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
    alice ->> subscriptions: InitSubscriptionAuthority (9020cu)
    subscriptions ->> System: CreateAccount
    subscriptions ->> Token: Approve (2904cu)
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
    Token -->>- subscriptions: ok (2904cu)
    subscriptions -->>- alice: ok (9020cu)
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
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 2V1TDexf_QK3x
    Token -->|writes| 2V1TDexf_QK3x
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
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 2V1TDexf_QK3x
```

**CreateRecurringDelegation: structured CPI tree**

```text

── subscriptions::CreateRecurringDelegation ────────────────
Transaction  signers=[alice]
└── subscriptions::CreateRecurringDelegation [1] ✓ 3573cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 3573
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**CreateRecurringDelegation: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->> subscriptions: CreateRecurringDelegation (3573cu)
    subscriptions ->> System: CreateAccount
```

**CreateRecurringDelegation: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->>+ subscriptions: CreateRecurringDelegation
    subscriptions ->>+ System: CreateAccount
    System -->>- subscriptions: ok
    subscriptions -->>- alice: ok (3573cu)
```

**CreateRecurringDelegation: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority[(SubAuthority)]:::writable
    RecurringDelegation([RecurringDelegation]):::signer
    System[System]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    RecurringDelegation -->|signs| System
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| RecurringDelegation
```

**CreateRecurringDelegation: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    System[System]:::owner
    alice[(alice)]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    RecurringDelegation[(RecurringDelegation)]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    subscriptions -->|owns| RecurringDelegation
```

- [x] Bob's ATA starts empty: `0`

### Bob pulls the full 50 tokens in period 0

**TransferRecurring (period 0): structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferRecurring [1] ✓ 11831cu  signer=bob
    ├── Token::TransferChecked [2] ✓ 6280cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
          🔔 RecurringTransfer
               delegatee:        bob,
               amount:           50000000,
               pulled_in_period: 50000000,
               receiver:         bob
Compute Units (this run): 11831
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferRecurring (period 0): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->> subscriptions: TransferRecurring (11831cu)
    subscriptions ->> Token: TransferChecked (6280cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferRecurring (period 0): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->>+ subscriptions: TransferRecurring
    subscriptions ->>+ Token: TransferChecked
    Token -->>- subscriptions: ok (6280cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (11831cu)
```

**TransferRecurring (period 0): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    RecurringDelegation[(RecurringDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    Am5eCnDR_fqmX[("Am5eCnDR…fqmX")]:::writable
    bob([bob]):::signer
    Token[Token]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| RecurringDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 2V1TDexf_QK3x
    subscriptions -->|writes| Am5eCnDR_fqmX
    Token -->|writes| 2V1TDexf_QK3x
    Token -->|writes| Am5eCnDR_fqmX
```

**TransferRecurring (period 0): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    RecurringDelegation[(RecurringDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token[Token]:::owner
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    Am5eCnDR_fqmX[("Am5eCnDR…fqmX")]:::account
    subscriptions -->|owns| RecurringDelegation
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 2V1TDexf_QK3x
    Token -->|owns| Am5eCnDR_fqmX
```

- [x] Bob received 50 tokens: `50000000`

### Alice revokes the SPL token approval over her ATA

**Revoke: structured CPI tree**

```text

── Token::Revoke ───────────────────────────────────────────
Transaction  signers=[alice]
└── Token::Revoke [1] ✓ 2676cu  signer=alice
Compute Units (this run): 2676
Fee: 0 lamports
Legend (1):
  alice = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**Revoke: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->> Token: Revoke (2676cu)
```

**Revoke: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->>+ Token: Revoke
    Token -->>- alice: ok (2676cu)
```

**Revoke: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::program
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    alice([alice]):::signer
    alice -->|signs| Token
    Token -->|writes| 2V1TDexf_QK3x
```

**Revoke: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::owner
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    System[System]:::owner
    alice[(alice)]:::account
    Token -->|owns| 2V1TDexf_QK3x
    System -->|owns| alice
```

### A full period elapses; Bob's pull now fails with an owner mismatch

**TransferRecurring (after revoke): structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferRecurring [1] ✗ 10221cu  signer=bob
    ├── Token::TransferChecked [2] ✗ 5999cu
    │   └── Error: custom program error: 0x4
    └── Error: custom program error: 0x4
Error: custom program error: 0x4
Compute Units (this run): 10221
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

### Alice re-approves, but for too small an amount; the pull still fails

**Approve (partial): structured CPI tree**

```text

── Token::Approve ──────────────────────────────────────────
Transaction  signers=[alice]
└── Token::Approve [1] ✓ 2902cu  signer=alice
Compute Units (this run): 2902
Fee: 0 lamports
Legend (1):
  alice = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**Approve (partial): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->> Token: Approve (2902cu)
```

**Approve (partial): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->>+ Token: Approve
    Token -->>- alice: ok (2902cu)
```

**Approve (partial): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::program
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    SubAuthority[(SubAuthority)]:::writable
    alice([alice]):::signer
    alice -->|signs| Token
    Token -->|writes| 2V1TDexf_QK3x
    Token -->|writes| SubAuthority
```

**Approve (partial): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::owner
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    System[System]:::owner
    alice[(alice)]:::account
    Token -->|owns| 2V1TDexf_QK3x
    subscriptions -->|owns| SubAuthority
    System -->|owns| alice
```

**TransferRecurring (partial approval): structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferRecurring [1] ✗ 10332cu  signer=bob
    ├── Token::TransferChecked [2] ✗ 6110cu
    │   └── Error: custom program error: 0x1
    └── Error: custom program error: 0x1
Error: custom program error: 0x1
Compute Units (this run): 10332
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

### Alice re-approves for the max amount; the pull succeeds again

**Approve (max): structured CPI tree**

```text

── Token::Approve ──────────────────────────────────────────
Transaction  signers=[alice]
└── Token::Approve [1] ✓ 2928cu  signer=alice
Compute Units (this run): 2928
Fee: 0 lamports
Legend (1):
  alice = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**Approve (max): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->> Token: Approve (2928cu)
```

**Approve (max): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant Token
    alice ->>+ Token: Approve
    Token -->>- alice: ok (2928cu)
```

**Approve (max): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::program
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    SubAuthority[(SubAuthority)]:::writable
    alice([alice]):::signer
    alice -->|signs| Token
    Token -->|writes| 2V1TDexf_QK3x
    Token -->|writes| SubAuthority
```

**Approve (max): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    Token[Token]:::owner
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    subscriptions[subscriptions]:::owner
    SubAuthority[(SubAuthority)]:::account
    System[System]:::owner
    alice[(alice)]:::account
    Token -->|owns| 2V1TDexf_QK3x
    subscriptions -->|owns| SubAuthority
    System -->|owns| alice
```

**TransferRecurring (after max approval): structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferRecurring [1] ✓ 11922cu  signer=bob
    ├── Token::TransferChecked [2] ✓ 6280cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
          🔔 RecurringTransfer
               delegatee:        bob,
               amount:           50000000,
               pulled_in_period: 50000000,
               receiver:         bob
Compute Units (this run): 11922
Fee: 0 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferRecurring (after max approval): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->> subscriptions: TransferRecurring (11922cu)
    subscriptions ->> Token: TransferChecked (6280cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferRecurring (after max approval): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->>+ subscriptions: TransferRecurring
    subscriptions ->>+ Token: TransferChecked
    Token -->>- subscriptions: ok (6280cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (11922cu)
```

**TransferRecurring (after max approval): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    RecurringDelegation[(RecurringDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::writable
    Am5eCnDR_fqmX[("Am5eCnDR…fqmX")]:::writable
    bob([bob]):::signer
    Token[Token]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| RecurringDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| 2V1TDexf_QK3x
    subscriptions -->|writes| Am5eCnDR_fqmX
    Token -->|writes| 2V1TDexf_QK3x
    Token -->|writes| Am5eCnDR_fqmX
```

**TransferRecurring (after max approval): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    RecurringDelegation[(RecurringDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token[Token]:::owner
    2V1TDexf_QK3x[("2V1TDexf…QK3x")]:::account
    Am5eCnDR_fqmX[("Am5eCnDR…fqmX")]:::account
    subscriptions -->|owns| RecurringDelegation
    subscriptions -->|owns| SubAuthority
    Token -->|owns| 2V1TDexf_QK3x
    Token -->|owns| Am5eCnDR_fqmX
```
