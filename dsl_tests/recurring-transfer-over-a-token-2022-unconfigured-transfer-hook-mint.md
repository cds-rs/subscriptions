## Recurring transfer over a Token-2022 unconfigured transfer-hook mint — PASS

> a mint carrying an unconfigured transfer hook still transfers

### Stage: Alice's authority and a recurring delegation to Bob

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
    CRe15fdJ_jiZy[("CRe15fdJ…jiZy")]:::writable
    System[System]:::program
    Token_2022["Token-2022"]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token_2022
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| CRe15fdJ_jiZy
    Token_2022 -->|writes| CRe15fdJ_jiZy
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
    CRe15fdJ_jiZy[("CRe15fdJ…jiZy")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| CRe15fdJ_jiZy
```

**CreateRecurringDelegation: structured CPI tree**

```text

── subscriptions::CreateRecurringDelegation ────────────────
Transaction  signers=[alice]
└── subscriptions::CreateRecurringDelegation [1] ✓ 3573cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 3573
Fee: 5000 lamports
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

### Bob pulls 10 tokens; the unconfigured hook is a no-op

**TransferRecurring: structured CPI tree**

```text

── subscriptions::TransferChecked ──────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferRecurring [1] ✓ 11175cu  signer=bob
    ├── Token-2022::TransferChecked [2] ✓ 2343cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
          🔔 RecurringTransfer
               delegatee:        bob,
               amount:           10000000,
               pulled_in_period: 10000000,
               receiver:         bob
Compute Units (this run): 11175
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferRecurring: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token_2022 as "Token-2022"
    bob ->> subscriptions: TransferRecurring (11175cu)
    subscriptions ->> Token_2022: TransferChecked (2343cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferRecurring: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token_2022 as "Token-2022"
    bob ->>+ subscriptions: TransferRecurring
    subscriptions ->>+ Token_2022: TransferChecked
    Token_2022 -->>- subscriptions: ok (2343cu)
    subscriptions ->>+ subscriptions: EmitEvent
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- bob: ok (11175cu)
```

**TransferRecurring: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    RecurringDelegation[(RecurringDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    CRe15fdJ_jiZy[("CRe15fdJ…jiZy")]:::writable
    64GoBJyg_C4wK[("64GoBJyg…C4wK")]:::writable
    bob([bob]):::signer
    Token_2022["Token-2022"]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token_2022
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| RecurringDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| CRe15fdJ_jiZy
    subscriptions -->|writes| 64GoBJyg_C4wK
    Token_2022 -->|writes| CRe15fdJ_jiZy
    Token_2022 -->|writes| 64GoBJyg_C4wK
```

**TransferRecurring: ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    RecurringDelegation[(RecurringDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token_2022["Token-2022"]:::owner
    CRe15fdJ_jiZy[("CRe15fdJ…jiZy")]:::account
    64GoBJyg_C4wK[("64GoBJyg…C4wK")]:::account
    System[System]:::owner
    bob[(bob)]:::account
    subscriptions -->|owns| RecurringDelegation
    subscriptions -->|owns| SubAuthority
    Token_2022 -->|owns| CRe15fdJ_jiZy
    Token_2022 -->|owns| 64GoBJyg_C4wK
    System -->|owns| bob
```

- [x] Alice's ATA debited 10 tokens: `90000000`
- [x] Bob received 10 tokens: `10000000`
