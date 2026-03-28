# Zuora CLI — AI Agent Guide

## Overview

This is a Rust CLI for the Zuora Billing REST API v1. It covers all API resources with typed commands, ZOQL query support, multi-profile auth, and three output formats.

## Quick Reference

```bash
# Auth
zuora login                                    # Interactive setup
zuora whoami                                   # Check connection
zuora status                                   # API health check

# Queries (most flexible — use for any data retrieval)
zuora query "SELECT Id, Name, Status FROM Account WHERE Status = 'Active' LIMIT 10"
zuora query "SELECT Id, InvoiceNumber, Amount FROM Invoice WHERE AccountId = 'ABC'" --output json

# Object schema (use before writing queries to check valid fields)
zuora describe Account
zuora describe Invoice
zuora describe Subscription

# Accounts
zuora accounts list --limit 20
zuora accounts list --status Active
zuora accounts get <account-id-or-number>
zuora accounts create --name "Acme Corp" --currency USD

# Subscriptions
zuora subscriptions list --account <account-id>
zuora subscriptions get <subscription-key>
zuora subscriptions cancel <subscription-key>

# Invoices
zuora invoices list --account <account-id>
zuora invoices get <invoice-id>
zuora invoices pdf <invoice-id> --output-file invoice.pdf

# Payments
zuora payments list --account <account-id>
zuora payments create --account <id> --amount 100.00 --payment-method <pm-id>

# Payment Methods
zuora payment-methods list --account <account-id>
zuora payment-methods get <pm-id>

# Orders
zuora orders list --account <account-number>
zuora orders get <order-number>
zuora orders create --file order.json

# Credit/Debit Memos
zuora credit-memos list --account <account-id>
zuora credit-memos create --account <id> --amount 50.00 --reason "Billing Error"
zuora debit-memos list --account <account-id>

# Refunds
zuora refunds create --payment <payment-id> --amount 25.00

# Product Catalog
zuora catalog list
zuora catalog get <product-key>
zuora catalog rate-plans <product-key>

# Billing Operations
zuora bill-runs create --body '{"invoiceDate":"2024-01-01","targetDate":"2024-01-01"}'
zuora payment-runs list
zuora operations job-status <job-id>
zuora billing-documents --account <account-id>

# Usage (metered billing)
zuora usage query --account <account-id>

# Accounting
zuora accounting-codes list
zuora accounting-periods list
zuora journal-entries get <je-number>

# Admin
zuora notifications callouts
zuora notifications emails
zuora exchange-rates EUR
zuora sequence-sets list
```

## Key Patterns for AI Agents

### Always use --output json for structured data
```bash
zuora accounts get ACC-123 --output json | jq '.basicInfo.name'
zuora query "SELECT Id FROM Account LIMIT 5" --output raw
```

### Use --dry-run before mutations
```bash
zuora --dry-run accounts create --name "Test Corp"
zuora --dry-run payments create --account ACC-1 --amount 500 --payment-method PM-1
```

### Use ZOQL for flexible queries
ZOQL is the most powerful way to retrieve data. Always check field names first:
```bash
zuora describe Account        # See all fields on Account
zuora describe Invoice        # See all fields on Invoice
```

Common ZOQL patterns:
```sql
-- Find account by name
SELECT Id, Name, AccountNumber FROM Account WHERE Name LIKE '%Acme%'

-- Get invoices with balances
SELECT InvoiceNumber, Amount, Balance, Status FROM Invoice WHERE AccountId = 'ID' AND Balance > 0

-- Find payment failures
SELECT Amount, EffectiveDate, GatewayResponse, Status FROM Payment WHERE AccountId = 'ID' AND Status = 'Error'

-- Get subscription charges with MRR
SELECT Name, ChargeType, Price, MRR FROM RatePlanCharge WHERE SubscriptionId = 'ID'
```

### ZOQL Limitations
- No JOINs — query one object at a time
- ORDER BY only works on certain fields (not CreatedDate on some objects)
- Auto-paginates through all results (follows queryMore automatically)
- Use `zuora describe <Object>` if you get INVALID_FIELD errors

### Workflow Commands (high-level, multi-step)
These are the most useful commands for understanding an account:
```bash
# Full billing picture — account + invoices + payments + payment method + declines
zuora billing-context <account-key>
zuora billing-context <account-key> --output json

# Collections — outstanding invoices aged by 30/60/90+ day buckets
zuora collections <account-key>

# Customer health score — subscriptions, payment status, balance, decline rate
zuora customer-health <account-key>
zuora customer-health <account-key> --output json
```

### MCP Server (for AI clients)
Start as an MCP tool server for Claude Desktop, Cursor, etc.:
```bash
zuora mcp
```

Configure in Claude Desktop (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "zuora": {
      "command": "zuora",
      "args": ["mcp"]
    }
  }
}
```

Available MCP tools: `zuora_query`, `zuora_describe`, `zuora_accounts_list`,
`zuora_accounts_get`, `zuora_subscriptions_list`, `zuora_invoices_list`,
`zuora_payments_list`, `zuora_payment_methods_list`, `zuora_billing_context`,
`zuora_collections`, `zuora_customer_health`, `zuora_catalog_list`.

### Multi-profile for different environments
```bash
zuora --profile prod accounts list
zuora --profile sandbox accounts list
```

### JSON body for complex operations
Commands that take `--body` accept a JSON string:
```bash
zuora contacts create --body '{"firstName":"Jane","lastName":"Doe","accountId":"ACC-1"}'
zuora bill-runs create --body '{"invoiceDate":"2024-01-01","targetDate":"2024-01-01"}'
```

## Error Handling

The CLI parses all 5 Zuora error formats and provides actionable hints:
- `INVALID_VALUE` → Check that the ID exists
- `OBJECT_NOT_FOUND` → Verify the ID or key
- `MALFORMED_QUERY` → Check ZOQL syntax, use `zuora describe`
- `MISSING_REQUIRED_VALUE` → A required field is missing
- `Authentication error` → Run `zuora login`

## Architecture

- **Language:** Rust
- **Config:** `~/.zuora/config.toml` (profiles) + `~/.zuora/tokens.json` (cached tokens)
- **Auth:** OAuth 2.0 client_credentials grant, auto-refresh on expiry
- **Output:** json (structured), table (human-readable), raw (compact, pipe-friendly)
- **Testing:** 200+ tests against real HTTP servers (wiremock), no mocks
- **MCP:** Built-in MCP server for AI client integration
- **Pagination:** Auto-follows queryMore for large ZOQL result sets

## Zuora API Environments

| Environment | Base URL |
|---|---|
| US Production 1 | https://rest.na.zuora.com |
| US Production 2 | https://rest.zuora.com |
| EU Production | https://rest.eu.zuora.com |
| APAC Production | https://rest.ap.zuora.com |
| US Sandbox | https://rest.test.zuora.com |
| EU Sandbox | https://rest.test.eu.zuora.com |
| APAC Sandbox | https://rest.test.ap.zuora.com |
