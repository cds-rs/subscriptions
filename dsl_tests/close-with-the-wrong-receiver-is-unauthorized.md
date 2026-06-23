## Close with the wrong receiver is unauthorized — PASS

> the rent receiver must be the stored payer, not an arbitrary account

**InitSubscriptionAuthority: structured CPI tree**

```text

── subscriptions::InitSubscriptionAuthority ────────────────
Transaction  signers=[sponsor, alice]
└── subscriptions::InitSubscriptionAuthority [1] ✓ 6264cu  signer=[alice, sponsor]
    ├── System::CreateAccount [2] ✓ (no cu)
    └── Token::Approve [2] ✓ 126cu
Compute Units (this run): 6264
Fee: 10000 lamports
Legend (3):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  sponsor       = 47cncVPgU4mK37H7VvxLCCsoDEKYaVhNLHHp4MbnEwvx
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
    alice ->> subscriptions: InitSubscriptionAuthority (6264cu)
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
    subscriptions -->>- alice: ok (6264cu)
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
    DNMc2J2S_Vjyq[("DNMc2J2S…Vjyq")]:::writable
    sponsor([sponsor]):::signer
    System[System]:::program
    Token[Token]:::program
    alice -->|signs| subscriptions
    sponsor -->|signs| subscriptions
    sponsor -->|signs| System
    SubAuthority -->|signs| System
    alice -->|signs| Token
    subscriptions -->|writes| SubAuthority
    subscriptions -->|writes| DNMc2J2S_Vjyq
    Token -->|writes| DNMc2J2S_Vjyq
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
    DNMc2J2S_Vjyq[("DNMc2J2S…Vjyq")]:::account
    sponsor[(sponsor)]:::account
    System -->|owns| alice
    subscriptions -->|owns| SubAuthority
    Token -->|owns| DNMc2J2S_Vjyq
    System -->|owns| sponsor
```

### Alice closes, but points the rent at Mallory

**CloseSubscriptionAuthority (wrong receiver): structured CPI tree**

```text

── subscriptions::CloseSubscriptionAuthority ───────────────
Transaction  signers=[alice]
└── subscriptions::CloseSubscriptionAuthority [1] ✗ 1847cu  signer=alice
    └── Error: Unauthorized (0x82)
Error: InstructionError(0, Custom(130))
Compute Units (this run): 1847
Fee: 5000 lamports
Legend (2):
  subscriptions = De1egAFMkMWZSN5rYXRj9CAdheBamobVNubTsi9avR44
  alice         = FXddRd8CdAC8SWKT3Ataasn69R7rbTfQZcKg8ejyrUbF
```
