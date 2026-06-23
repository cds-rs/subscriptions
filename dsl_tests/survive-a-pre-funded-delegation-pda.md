## Survive a pre-funded delegation PDA — PASS

> a griefer pre-funds the delegation PDA; Alice can still create it

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 10742cu  signer=alice
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 10742
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
    alice ->> subscriptions: InitSubscriptionAuthority (10742cu)
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
    subscriptions -->>- alice: ok (10742cu)
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
    DvCTEmAW_WjEt[("DvCTEmAW…WjEt")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    alice -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| DvCTEmAW_WjEt
    Token -->|writes| DvCTEmAW_WjEt
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
    DvCTEmAW_WjEt[("DvCTEmAW…WjEt")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| DvCTEmAW_WjEt
```

### Despite the pre-funded PDA, Alice creates the fixed delegation

**CreateFixedDelegation (pre-funded PDA): structured CPI tree**

```text

── subscriptions::CreateFixedDelegation ────────────────────
Transaction  signers=[alice]
└── subscriptions::CreateFixedDelegation [1] ✓ 5808cu  signer=alice
    ├── System::Transfer (alice -> FixedDelegation) 2,191,400 lamports [2] ✓ (no cu)
    ├── System::Allocate [2] ✓ (no cu)
    └── System::Assign [2] ✓ (no cu)
Compute Units (this run): 5808
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```

**CreateFixedDelegation (pre-funded PDA): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->> subscriptions: CreateFixedDelegation (5808cu)
    subscriptions ->> System: Transfer (alice → FixedDelegation) 2,191,400 lamports
    subscriptions ->> System: Allocate
    subscriptions ->> System: Assign
```

**CreateFixedDelegation (pre-funded PDA): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    alice ->>+ subscriptions: CreateFixedDelegation
    subscriptions ->>+ System: Transfer (alice → FixedDelegation) 2,191,400 lamports
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Allocate
    System -->>- subscriptions: ok
    subscriptions ->>+ System: Assign
    System -->>- subscriptions: ok
    subscriptions -->>- alice: ok (5808cu)
```

**CreateFixedDelegation (pre-funded PDA): authority graph**

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
    System -->|writes| FixedDelegation
```

**CreateFixedDelegation (pre-funded PDA): ownership graph**

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

- [x] the delegator is Alice: `FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF`
- [x] the delegatee matches: `1112hLa87sNLYBQAUemfLRCzgGV8UiCqYDLksz7xnqs`
- [x] the account is tagged FixedDelegation: `2`
- [x] the delegated amount matches: `100000000`
- [x] the expiry matches: `1700086400`
