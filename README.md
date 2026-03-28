# zuora

A command-line interface for the Zuora Billing API — built for humans and automation.

```
zuora accounts list --status Active
zuora query "SELECT Id, Name FROM Account LIMIT 5"
zuora invoices pdf INV-00123 --output-file invoice.pdf
```

## Quick Start

```bash
# 1. Install (macOS Apple Silicon — see Installation section for other platforms)
curl -fsSL https://github.com/shrav/zuora-cli/releases/latest/download/zuora-darwin-aarch64.tar.gz | tar xz && sudo mv zuora-darwin-aarch64 /usr/local/bin/zuora

# 2. Authenticate
zuora login

# 3. Try it
zuora whoami
zuora accounts list --limit 5
```

## Installation

### One-liner (macOS / Linux)

```bash
curl -fsSL https://github.com/shrav/zuora-cli/releases/latest/download/zuora-darwin-aarch64.tar.gz | tar xz && sudo mv zuora-darwin-aarch64 /usr/local/bin/zuora
```

<details>
<summary>Other platforms</summary>

```bash
# macOS Intel
curl -fsSL https://github.com/shrav/zuora-cli/releases/latest/download/zuora-darwin-x86_64.tar.gz | tar xz && sudo mv zuora-darwin-x86_64 /usr/local/bin/zuora

# Linux x86_64
curl -fsSL https://github.com/shrav/zuora-cli/releases/latest/download/zuora-linux-x86_64.tar.gz | tar xz && sudo mv zuora-linux-x86_64 /usr/local/bin/zuora

# Linux ARM64
curl -fsSL https://github.com/shrav/zuora-cli/releases/latest/download/zuora-linux-aarch64.tar.gz | tar xz && sudo mv zuora-linux-aarch64 /usr/local/bin/zuora

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/shrav/zuora-cli/releases/latest/download/zuora-windows-x86_64.exe.zip -OutFile zuora.zip; Expand-Archive zuora.zip; Move-Item zuora\zuora-windows-x86_64.exe C:\Windows\zuora.exe
```

</details>

### From source (Rust 1.70+)

```bash
git clone https://github.com/shrav/zuora-cli.git
cd zuora-cli
cargo install --path .
```

## Authentication

### Interactive

```bash
zuora login
```

Prompts to select your Zuora environment (US/EU/APAC, production/sandbox), then asks for your **Client ID** and **Client Secret** from Zuora Settings > Administration > Manage OAuth Clients.

Credentials are saved to `~/.zuora/config.toml`. Tokens are cached in `~/.zuora/tokens.json` and auto-refresh on expiry.

### Non-interactive (CI / scripting)

```bash
# Via flags
zuora login \
  --client-id $ZUORA_CLIENT_ID \
  --client-secret $ZUORA_CLIENT_SECRET \
  --base-url https://rest.na.zuora.com

# Via environment variables (no login needed)
export ZUORA_CLIENT_ID=your-id
export ZUORA_CLIENT_SECRET=your-secret
export ZUORA_BASE_URL=https://rest.na.zuora.com
zuora accounts list
```

### Multi-profile

```bash
# Production (default profile)
zuora login

# Sandbox
zuora --profile sandbox login --base-url https://rest.test.zuora.com

# Switch between them
zuora accounts list                          # uses default
zuora --profile sandbox accounts list        # uses sandbox
```

## Commands

### 44 commands across all Zuora API resources:

**Utility**
```
login, whoami, status, config, query, describe, completions
```

**Core Billing**
```
accounts, subscriptions, invoices, payments, payment-methods,
orders, credit-memos, debit-memos, refunds, contacts
```

**Product Catalog**
```
catalog, catalog-groups
```

**Billing Operations**
```
bill-runs, payment-runs, payment-schedules, usage,
billing-documents, billing-preview, operations, adjustments,
invoice-schedules, sign-up
```

**Files & Attachments**
```
files, attachments
```

**Accounting & Revenue**
```
accounting-codes, accounting-periods, journal-entries,
journal-runs, taxation-items
```

**Order Details**
```
fulfillments, order-line-items
```

**Subscription Details**
```
subscription-change-logs, ramps
```

**Admin**
```
notifications, sequence-sets, exchange-rates, commitments
```

Run `zuora <command> --help` for detailed usage of any command.

### ZOQL Queries

```bash
zuora query "SELECT Id, Name, Status FROM Account WHERE Status = 'Active' LIMIT 10"
zuora query "SELECT InvoiceNumber, Amount FROM Invoice" --output json | jq '.[].Amount'
zuora describe Account    # list all queryable fields
```

### Examples

```bash
# Accounts
zuora accounts list --status Active --limit 20
zuora accounts get A00012345 --output json
zuora accounts create --name "Acme Corp" --currency USD

# Invoices
zuora invoices list --account <id> --status Posted
zuora invoices pdf <invoice-id> --output-file acme-invoice.pdf

# Payments
zuora payments create --account <id> --amount 500.00 --payment-method <pm-id>
zuora payment-methods list --account <id>

# Orders
zuora orders create --file order.json
zuora orders list --account A00012345

# Credit memos
zuora credit-memos create --account <id> --amount 50.00 --reason "Billing Error"

# Catalog
zuora catalog list
zuora catalog rate-plans <product-key>

# Billing operations
zuora bill-runs create --body '{"invoiceDate":"2024-01-01","targetDate":"2024-01-01"}'
zuora operations job-status <job-id>
```

## Output Formats

| Format | Flag | Use case |
|--------|------|----------|
| `table` | `--output table` (default) | Human-readable terminal output |
| `json` | `--output json` | Structured data, scripting |
| `raw` | `--output raw` | Compact JSON for piping to `jq` |

## Global Flags

| Flag | Description |
|------|-------------|
| `--profile <name>` | Named profile (default: `default`) |
| `--output <format>` | `json`, `table`, or `raw` |
| `--base-url <url>` | Override API base URL |
| `--verbose` | Show HTTP request/response details |
| `--dry-run` | Preview mutations without sending |

## Dry Run

```bash
zuora --dry-run accounts create --name "Test Corp"
# DRY RUN — would send:
#   POST https://rest.na.zuora.com/v1/accounts
#   Body: { "name": "Test Corp", "currency": "USD", ... }
```

GETs still execute normally. Only POST/PUT/DELETE are intercepted.

## Environments

`zuora login` prompts to select from all 10 Zuora environments:

| Environment | Base URL |
|---|---|
| US Production 1 | `https://rest.na.zuora.com` |
| US Production 2 | `https://rest.zuora.com` |
| EU Production | `https://rest.eu.zuora.com` |
| APAC Production | `https://rest.ap.zuora.com` |
| US Sandbox | `https://rest.test.zuora.com` |
| US API Sandbox 1 | `https://rest.sandbox.na.zuora.com` |
| US API Sandbox 2 | `https://rest.apisandbox.zuora.com` |
| EU Sandbox | `https://rest.test.eu.zuora.com` |
| EU API Sandbox | `https://rest.sandbox.eu.zuora.com` |
| APAC Sandbox | `https://rest.test.ap.zuora.com` |

## Configuration

```toml
# ~/.zuora/config.toml
[default]
client_id = "your-client-id"
client_secret = "your-client-secret"
base_url = "https://rest.na.zuora.com"

[sandbox]
client_id = "sandbox-client-id"
client_secret = "sandbox-client-secret"
base_url = "https://rest.test.zuora.com"
```

**Resolution order:** CLI flags > environment variables > config file > defaults.

| Variable | Description |
|---|---|
| `ZUORA_CLIENT_ID` | OAuth client ID |
| `ZUORA_CLIENT_SECRET` | OAuth client secret |
| `ZUORA_BASE_URL` | API base URL |

## Shell Completions

```bash
# Bash
zuora completions bash > ~/.local/share/bash-completion/completions/zuora

# Zsh
zuora completions zsh > ~/.zfunc/_zuora

# Fish
zuora completions fish > ~/.config/fish/completions/zuora.fish
```

## Error Handling

The CLI parses all 5 Zuora error response formats and adds actionable hints:

```
Error: GET /v1/accounts/bad-id failed (HTTP 404):
  [OBJECT_NOT_FOUND] Account not found
  Hint: Verify the ID or key.
```

```
Error: POST /v1/action/query failed (HTTP 400):
  [MALFORMED_QUERY] You have an error in your ZOQL syntax
  Hint: Check your ZOQL syntax. Use `zuora describe <Object>` to see available fields.
```

## Development

```bash
cargo build                    # Debug build
cargo test                     # 191 tests (real HTTP servers, no mocks)
cargo clippy -- -W clippy::all # Lint
cargo build --release          # Optimized release build
```

### Architecture

```
src/
├── main.rs              # Entry point, command dispatch
├── cli.rs               # Clap command tree (44 commands)
├── client/
│   ├── api_client.rs    # HTTP client — auth, retry, dry-run, rate limiting
│   ├── auth.rs          # OAuth token fetch + disk cache
│   └── error.rs         # All 5 Zuora error formats + actionable hints
├── config/
│   ├── profile.rs       # Profile resolution (flags > env > config)
│   └── store.rs         # ~/.zuora/ file I/O
├── commands/            # 27 command modules (one per resource)
├── output/
│   └── formatter.rs     # JSON / Table / Raw output
└── types/
    └── responses.rs     # API response structs
```

## License

[MIT](LICENSE)
