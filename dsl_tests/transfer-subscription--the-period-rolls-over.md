## Transfer subscription: the period rolls over — PASS

> advancing past the period boundary resets the pulled amount

### Stage: Alice's authority, the merchant's plan, Alice subscribed

**InitSubscriptionAuthority: structured CPI tree**

```text

Transaction  signers=[alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 13742cu  signer=alice
    ├── System [2] ✓ (no cu)
    └── Token [2] ✓ 126cu
Compute Units (this run): 13742
Fee: 5000 lamports
Legend (2):
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**InitSubscriptionAuthority: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant alice
    participant subscriptions
    participant System
    participant Token
    alice ->> subscriptions: InitSubscriptionAuthority (13742cu)
    subscriptions ->> System: unnamed
    subscriptions ->> Token: unnamed (126cu)
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
    subscriptions ->>+ System: unnamed
    System -->>- subscriptions: ok
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (126cu)
    subscriptions -->>- alice: ok (13742cu)
```

**InitSubscriptionAuthority: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    alice([alice]):::signer
    SubAuthority[(SubAuthority)]:::writable
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::writable
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| J4DQKYnn_6dHE
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
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| J4DQKYnn_6dHE
```

**CreatePlan: structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::CreatePlan [1] ✓ 3468cu  signer=merchant
    └── System [2] ✓ (no cu)
Compute Units (this run): 3468
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**CreatePlan: sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->> subscriptions: CreatePlan (3468cu)
    subscriptions ->> System: unnamed
```

**CreatePlan: sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant System
    merchant ->>+ subscriptions: CreatePlan
    subscriptions ->>+ System: unnamed
    System -->>- subscriptions: ok
    subscriptions -->>- merchant: ok (3468cu)
```

**CreatePlan: authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    merchant([merchant]):::signer
    Plan[(Plan)]:::writable
    System[System]:::program
    merchant -->|signs| subscriptions
    subscriptions -->|writes| Plan
```

**CreatePlan: ownership graph**

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

### The merchant pulls the full 50-token period

**TransferSubscription (period 1): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::TransferSubscription [1] ✓ 10442cu  signer=merchant
    ├── Token [2] ✓ 113cu
    └── subscriptions [2] ✓ 137cu
Compute Units (this run): 10442
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**TransferSubscription (period 1): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->> subscriptions: TransferSubscription (10442cu)
    subscriptions ->> Token: unnamed (113cu)
    subscriptions ->> subscriptions: unnamed (137cu)
```

**TransferSubscription (period 1): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->>+ subscriptions: TransferSubscription
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: unnamed
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- merchant: ok (10442cu)
```

**TransferSubscription (period 1): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    Subscription[(Subscription)]:::writable
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::writable
    EKj5WczT_V43T[("EKj5WczT…V43T")]:::writable
    merchant([merchant]):::signer
    Token[Token]:::program
    merchant -->|signs| subscriptions
    subscriptions -->|writes| Subscription
    subscriptions -->|writes| J4DQKYnn_6dHE
    subscriptions -->|writes| EKj5WczT_V43T
```

**TransferSubscription (period 1): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    Subscription[(Subscription)]:::account
    Token[Token]:::owner
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::account
    EKj5WczT_V43T[("EKj5WczT…V43T")]:::account
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions -->|owns| Subscription
    Token -->|owns| J4DQKYnn_6dHE
    Token -->|owns| EKj5WczT_V43T
    System -->|owns| merchant
```

- [x] the merchant has the full period: `50000000`

### The clock advances one period

### The merchant pulls 30 tokens in the new period

**TransferSubscription (period 2): structured CPI tree**

```text

Transaction  signers=[merchant]
└── subscriptions::TransferSubscription [1] ✓ 10533cu  signer=merchant
    ├── Token [2] ✓ 113cu
    └── subscriptions [2] ✓ 137cu
Compute Units (this run): 10533
Fee: 5000 lamports
Legend (2):
  merchant      = J4fPsxKiTTKiXSiN9bgZ5JBrVgTM9c6N8tjhLN8gfdTq
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
```

**TransferSubscription (period 2): sequence diagram**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->> subscriptions: TransferSubscription (10533cu)
    subscriptions ->> Token: unnamed (113cu)
    subscriptions ->> subscriptions: unnamed (137cu)
```

**TransferSubscription (period 2): sequence diagram, with lifelines**

```mermaid
sequenceDiagram
    autonumber
    participant merchant
    participant subscriptions
    participant Token
    merchant ->>+ subscriptions: TransferSubscription
    subscriptions ->>+ Token: unnamed
    Token -->>- subscriptions: ok (113cu)
    subscriptions ->>+ subscriptions: unnamed
    subscriptions -->>- subscriptions: ok (137cu)
    subscriptions -->>- merchant: ok (10533cu)
```

**TransferSubscription (period 2): authority graph**

```mermaid
flowchart LR
    classDef signer fill:#d4edda,stroke:#28a745;
    classDef program fill:#cce5ff,stroke:#007bff;
    classDef writable fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::program
    Subscription[(Subscription)]:::writable
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::writable
    EKj5WczT_V43T[("EKj5WczT…V43T")]:::writable
    merchant([merchant]):::signer
    Token[Token]:::program
    merchant -->|signs| subscriptions
    subscriptions -->|writes| Subscription
    subscriptions -->|writes| J4DQKYnn_6dHE
    subscriptions -->|writes| EKj5WczT_V43T
```

**TransferSubscription (period 2): ownership graph**

```mermaid
flowchart LR
    classDef owner fill:#cce5ff,stroke:#007bff;
    classDef account fill:#fff3cd,stroke:#ffc107;
    subscriptions[subscriptions]:::owner
    Subscription[(Subscription)]:::account
    Token[Token]:::owner
    J4DQKYnn_6dHE[("J4DQKYnn…6dHE")]:::account
    EKj5WczT_V43T[("EKj5WczT…V43T")]:::account
    System[System]:::owner
    merchant[(merchant)]:::account
    subscriptions -->|owns| Subscription
    Token -->|owns| J4DQKYnn_6dHE
    Token -->|owns| EKj5WczT_V43T
    System -->|owns| merchant
```

- [x] the merchant total is 80 tokens: `80000000`
- [x] the pulled-in-period reset to 30 tokens: `30000000`
