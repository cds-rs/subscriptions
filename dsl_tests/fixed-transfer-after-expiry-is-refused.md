## Fixed transfer after expiry is refused — PASS

> a pull within the window succeeds; after the clock passes expiry, a further pull is refused

### Stage: Alice's authority and a fixed delegation to Bob

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 9242cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 9242
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
    alice ->> subscriptions: InitSubscriptionAuthority (9242cu)
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
    subscriptions -->>- alice: ok (9242cu)
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
    GcfWNiEx_eoZb[("GcfWNiEx…eoZb")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| GcfWNiEx_eoZb
    Token -->|writes| GcfWNiEx_eoZb
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
    GcfWNiEx_eoZb[("GcfWNiEx…eoZb")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| GcfWNiEx_eoZb
```

**CreateFixedDelegation: structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✓ 9538cu  signer=alice
    └── System::CreateAccount [2] ✓ (no cu)
Compute Units (this run): 9538
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
    alice ->> subscriptions: CreateFixedDelegation (9538cu)
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
    subscriptions -->>- alice: ok (9538cu)
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

### Bob pulls 30 tokens within the window

**TransferFixed (within window): structured CPI tree**

```text

── subscriptions::TransferFixed ────────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✓ 5555cu  signer=bob
    ├── Token::TransferChecked [2] ✓ 113cu
    └── subscriptions::EmitEvent [2] ✓ 137cu
Compute Units (this run): 5555
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

**TransferFixed (within window): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant bob
    participant subscriptions
    participant Token
    bob ->> subscriptions: TransferFixed (5555cu)
    subscriptions ->> Token: TransferChecked (113cu)
    subscriptions ->> subscriptions: EmitEvent (137cu)
```

**TransferFixed (within window): sequence diagram, with lifelines**

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
    subscriptions -->>- bob: ok (5555cu)
```

**TransferFixed (within window): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    FixedDelegation[(FixedDelegation)]:::writable
    SubAuthority([SubAuthority]):::signer
    GcfWNiEx_eoZb[("GcfWNiEx…eoZb")]:::writable
    AYwwqdGQ_e7Q9[("AYwwqdGQ…e7Q9")]:::writable
    bob([bob]):::signer
    Token[Token]:::program
    EventAuthority([EventAuthority]):::signer
    bob -->|signs| subscriptions
    SubAuthority -->|signs| Token
    EventAuthority -->|signs| subscriptions
    subscriptions -->|writes| FixedDelegation
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| GcfWNiEx_eoZb
    subscriptions -->|writes| AYwwqdGQ_e7Q9
    Token -->|writes| GcfWNiEx_eoZb
    Token -->|writes| AYwwqdGQ_e7Q9
```

**TransferFixed (within window): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    FixedDelegation[(FixedDelegation)]:::account
    SubAuthority[(SubAuthority)]:::account
    Token[Token]:::owner
    GcfWNiEx_eoZb[("GcfWNiEx…eoZb")]:::account
    AYwwqdGQ_e7Q9[("AYwwqdGQ…e7Q9")]:::account
    System[System]:::owner
    bob[(bob)]:::account
    subscriptions -->|owns| FixedDelegation
    subscriptions -->|owns| SubAuthority
    Token -->|owns| GcfWNiEx_eoZb
    Token -->|owns| AYwwqdGQ_e7Q9
    System -->|owns| bob
```

- [x] Bob received 30 tokens: `30000000`

### The clock advances past expiry; a further pull is refused

**TransferFixed (expired): structured CPI tree**

```text

── subscriptions::TransferFixed ────────────────────────────
Transaction  signers=[bob]
└── subscriptions::TransferFixed [1] ✗ 871cu  signer=bob
    └── Error: DelegationExpired (0x80)
Error: InstructionError(0, Custom(128))
Compute Units (this run): 871
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  bob           = 9S8NPMnzAba71o3tr8dHDzUrfT5NesYFzP56QFpYieMR
```

- [x] Bob's balance is unchanged: `30000000`
- [x] the remaining allowance is 20 tokens: `20000000`
