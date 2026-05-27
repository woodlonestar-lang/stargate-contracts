#![no_std]

mod events;
mod invoice;
mod validation;

pub use invoice::{DataKey, Invoice, InvoiceError, InvoiceStatus};

use soroban_sdk::{contract, contractimpl, Address, Env};
use validation::{require_admin, require_not_paused, require_positive_amount};

#[contract]
pub struct InvoiceContract;

#[contractimpl]
impl InvoiceContract {
    pub fn initialize(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::InvoiceCount, &0u64);
        env.storage().instance().set(&DataKey::Paused, &false);
    }

    pub fn create_invoice(
        env: Env,
        merchant: Address,
        amount_usdc: i128,
        gross_usdc: i128,
        expires_in_seconds: u64,
    ) -> Result<u64, InvoiceError> {
        merchant.require_auth();
        require_not_paused(&env)?;
        require_positive_amount(amount_usdc, gross_usdc)?;

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::InvoiceCount)
            .unwrap_or(0);
        let id = count + 1;
        let expires_at = env.ledger().timestamp() + expires_in_seconds;
        let invoice = Invoice {
            id,
            merchant: merchant.clone(),
            amount_usdc,
            gross_usdc,
            status: InvoiceStatus::Pending,
            expires_at,
            paid_at: None,
            payer: merchant,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Invoice(id), &invoice);
        env.storage().instance().set(&DataKey::InvoiceCount, &id);
        events::invoice_created(&env, id, &invoice);
        Ok(id)
    }

    pub fn mark_paid(
        env: Env,
        admin: Address,
        id: u64,
        payer: Address,
    ) -> Result<(), InvoiceError> {
        require_admin(&env, &admin)?;
        require_not_paused(&env)?;

        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .unwrap();

        if invoice.status != InvoiceStatus::Pending {
            return Err(InvoiceError::NotPending);
        }
        if env.ledger().timestamp() > invoice.expires_at {
            invoice.status = InvoiceStatus::Expired;
            env.storage()
                .persistent()
                .set(&DataKey::Invoice(id), &invoice);
            return Err(InvoiceError::Expired);
        }

        invoice.status = InvoiceStatus::Paid;
        invoice.paid_at = Some(env.ledger().timestamp());
        invoice.payer = payer;
        env.storage()
            .persistent()
            .set(&DataKey::Invoice(id), &invoice);
        events::invoice_paid(&env, id, &invoice);
        Ok(())
    }

    pub fn get_invoice(env: Env, id: u64) -> Invoice {
        env.storage()
            .persistent()
            .get(&DataKey::Invoice(id))
            .unwrap()
    }

    pub fn pause(env: Env, admin: Address) -> Result<(), InvoiceError> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        events::contract_paused(&env, &admin);
        Ok(())
    }

    pub fn unpause(env: Env, admin: Address) -> Result<(), InvoiceError> {
        require_admin(&env, &admin)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        events::contract_unpaused(&env, &admin);
        Ok(())
    }
}

#[cfg(test)]
extern crate std;
