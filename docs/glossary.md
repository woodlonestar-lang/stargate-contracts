# Glossary: Invoice, Settlement, Dispute, and Escrow Terms

This glossary defines the domain terms used across the Stargate contracts (`invoice`, `treasury`, `compliance`). Definitions are derived directly from the Soroban contract types in this repository.

---

## Invoice Terms

**Invoice**
A payment request created by a merchant. Stored on-chain with a unique numeric ID, the merchant's address, a net amount (`amount_usdc`), a gross amount (`gross_usdc`), an expiry timestamp, and an optional metadata hash and payment-link hash.

**Merchant**
The Stellar address that creates an invoice and receives payment. Must authorise the `create_invoice` call.

**Payer**
The Stellar address that pays an invoice. Recorded on the invoice when `mark_paid` is called. The payer may later raise a refund request.

**amount_usdc**
The net settlement amount (in USDC stroops) that the merchant expects to receive.

**gross_usdc**
The gross invoice amount before fees. Must be ≥ `amount_usdc`. Both must be positive.

**expires_at**
The ledger timestamp (Unix seconds) after which the invoice can no longer be paid. Payment at exactly `expires_at` is rejected; the boundary is exclusive.

**metadata_hash**
An optional SHA-256 (or equivalent) hash of off-chain invoice metadata (e.g. line items, PO number). Stored as raw bytes; not interpreted by the contract.

**payment_link_hash**
An optional hash of a payment-link URI, enabling deterministic linking between on-chain state and an off-chain checkout page.

### InvoiceStatus

| Status            | Meaning                                                                 |
|-------------------|-------------------------------------------------------------------------|
| `Pending`         | Created and awaiting payment. Can be paid, cancelled, or expired.       |
| `Paid`            | Marked paid by the admin. Payer and `paid_at` timestamp are recorded.   |
| `Expired`         | The ledger passed `expires_at` before payment. Set by `batch_expire`.   |
| `Cancelled`       | Cancelled by the merchant or admin before payment.                      |
| `RefundRequested` | The payer requested a refund on a paid invoice (initiates escrow dispute). |

### InvoiceError

| Code | Name                  | Trigger                                                          |
|------|-----------------------|------------------------------------------------------------------|
| 1    | `Unauthorized`        | Caller is not the merchant, admin, or payer.                     |
| 2    | `ContractPaused`      | A state-changing call was made while the contract is paused.     |
| 3    | `InvalidAmount`       | `amount_usdc` ≤ 0 or `gross_usdc` < `amount_usdc`.              |
| 4    | `NotPending`          | Operation requires `Pending` status but invoice is in another state. |
| 5    | `Expired`             | Payment attempted after `expires_at`.                            |
| 6    | `NotFound`            | No invoice exists for the given ID.                              |
| 7    | `AlreadyInitialized`  | `initialize` called when the contract is already set up.         |
| 8    | `ZeroDuration`        | `expires_in_seconds` was 0 on invoice creation.                  |
| 9    | `ExpiryOverflow`      | `ledger_timestamp + expires_in_seconds` overflows `u64`.         |
| 10   | `NotPaid`             | `request_refund` called on an invoice that is not `Paid`.        |

---

## Settlement Terms

**Settlement**
A multi-sig treasury disbursement to a merchant. Created by an authorised signer via `propose_settlement`; funds are transferred only after the cumulative approval weight meets the configured threshold.

**Signer**
A Stellar address registered in the treasury with a positive weight. Signers propose, approve, and execute settlements. The admin is automatically registered as a signer with weight 1 during initialisation.

**approval_weight**
The running sum of the weights of all unique signers who have approved a settlement or dispute resolution. A settlement can be executed only when `approval_weight ≥ threshold`.

**Threshold**
The minimum cumulative signer weight required to execute a settlement or apply a dispute resolution. Set at initialisation; updateable by the admin via `update_threshold` (must be > 0).

**Token Allowlist**
An optional list of token contract addresses that the treasury accepts for settlement execution. If the list is non-empty, any `execute_settlement` call that references an unlisted token is rejected with `TokenNotAllowed`.

**Partial Settlement**
A settlement where only a fraction of the original amount is disbursed in a single execution. The status transitions to `PartiallyExecuted`; the full settlement record remains for subsequent operations.

### SettlementStatus

| Status               | Meaning                                                                 |
|----------------------|-------------------------------------------------------------------------|
| `Pending`            | Proposed and awaiting sufficient approvals.                             |
| `Executed`           | Full amount transferred to the merchant.                                |
| `PartiallySettled`   | Legacy partial-execution state (see `partial_settle`).                  |
| `PartiallyExecuted`  | A partial amount was transferred via `partially_execute_settlement`.    |
| `OnHold`             | Blocked from execution (e.g. by compliance review or an open dispute).  |
| `Cancelled`          | Cancelled by an authorised signer before execution.                     |

### SettlementHoldReason

| Variant             | Meaning                                         |
|---------------------|-------------------------------------------------|
| `None`              | Not on hold (default state).                    |
| `ComplianceReview`  | Held pending a compliance review.               |
| `FraudCheck`        | Held for fraud investigation.                   |
| `KycPending`        | Held until KYC verification is complete.        |
| `AdminHold`         | Held by an admin for an unspecified reason.     |

### TreasuryError

| Code | Name                      | Trigger                                                          |
|------|---------------------------|------------------------------------------------------------------|
| 1    | `AlreadyInitialized`      | `initialize` called on an already-initialized treasury.          |
| 2    | `ZeroThreshold`           | Threshold set to 0.                                              |
| 3    | `SettlementNotFound`      | No settlement found for the given ID.                            |
| 4    | `AlreadyExecuted`         | Operation requires a non-executed settlement.                    |
| 5    | `ThresholdNotMet`         | `approval_weight` < `threshold` at execution time.              |
| 6    | `ThresholdNotConfigured`  | Threshold key is missing or zero.                                |
| 7    | `InvalidAmount`           | Amount ≤ 0 or partial amount is out of range.                    |
| 8    | `ContractPaused`          | State-changing call while the treasury is paused.                |
| 9    | `Unauthorized`            | Caller is not the admin.                                         |
| 10   | `UnauthorizedSigner`      | Caller has no weight registered as a signer.                     |
| 11   | `InvalidTokenContract`    | Token address equals the treasury's own contract address.        |
| 12   | `TokenNotAllowed`         | Token is not in the allowlist (when the allowlist is active).    |
| 13   | `RotationNotFound`        | No signer-rotation proposal found for the given ID.              |
| 14   | `RotationAlreadyExecuted` | Rotation proposal has already been executed.                     |

---

## Dispute Terms

**Dispute**
An on-chain record raised by a claimant (typically the payer) against a counterparty (typically the merchant) over a specific settlement amount. Raising a dispute automatically places the referenced settlement `OnHold`.

**Claimant**
The party initiating a dispute. Must authorise the `raise_dispute` call.

**Counterparty**
The opposing party in a dispute (usually the merchant whose settlement is contested).

**resolution_for_claimant**
Boolean flag set on the first vote that determines which direction the dispute resolution will go. All subsequent votes must match this direction; a mismatch panics with `ResolutionDirectionMismatch`.

**resolution_weight**
Cumulative weight of signers who have voted on the dispute resolution. When it reaches the treasury threshold the dispute status transitions to `ResolvedClaimant` or `ResolvedCounterparty`.

### DisputeStatus

| Status                  | Meaning                                                         |
|-------------------------|-----------------------------------------------------------------|
| `Raised`                | Dispute created and awaiting resolution votes.                  |
| `ResolvedClaimant`      | Resolved in favour of the claimant.                             |
| `ResolvedCounterparty`  | Resolved in favour of the counterparty (merchant).              |

---

## Escrow Terms

**Escrow**
In the Stargate model, escrow refers to the treasury's role as a neutral custodian of funds between payment receipt and merchant settlement. The treasury holds deposited tokens until signers collectively approve disbursement.

**Escrow Dispute**
Initiated when a payer calls `request_refund` on a paid invoice, transitioning the invoice to `RefundRequested`. The associated treasury settlement is placed `OnHold` via `raise_dispute`, pausing disbursement until the dispute is resolved by multi-sig vote.

**Deposit**
A transfer of tokens from an external address into the treasury contract. Recorded per depositor address in persistent storage (`DataKey::Balance`).

**Withdraw**
A transfer of tokens from the treasury back to a registered depositor, bounded by the depositor's recorded balance.

**Merchant Payout Address**
An optional override address where a merchant's settlement funds are sent. Set via `update_merchant_payout_address`. When not configured, the merchant's signing address is used.

**Signer Rotation**
A governance process for replacing one authorised signer with another. Proposed via `propose_signer_rotation` and executed automatically when the approval weight meets the threshold. The old signer's weight is transferred to the new signer and the old signer's weight is set to 0.

---

## Cross-Contract Workflow Summary

```
Merchant           InvoiceContract         TreasuryContract
   |                    |                        |
   |-- create_invoice ->|                        |
   |                    |                        |
Payer pays off-chain    |                        |
   |                    |                        |
Admin -- mark_paid ---->|                        |
   |                    |                        |
Signer ----- propose_settlement --------------->|
Signer ----- approve_settlement --------------->|
Signer ----- execute_settlement --------------->|-- token transfer --> Merchant
   |                    |                        |
[If payer disputes]     |                        |
Payer -- request_refund->|                      |
Payer ----- raise_dispute ---------------------->| (settlement -> OnHold)
Signers -- vote_dispute_resolution ------------->| (threshold met -> Resolved)
```
