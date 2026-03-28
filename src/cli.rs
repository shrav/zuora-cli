use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zuora",
    version,
    about = "CLI for the Zuora Billing API",
    long_about = "A command-line interface for the Zuora Billing REST API.\n\n\
        Supports all Zuora v1 API resources, ZOQL queries, multi-profile\n\
        management, and multiple output formats (json, table, raw).\n\n\
        Get started:\n  \
        zuora login          # Authenticate\n  \
        zuora whoami          # Verify connection\n  \
        zuora accounts list   # List accounts\n  \
        zuora query \"SELECT Id, Name FROM Account LIMIT 5\"",
    after_help = "ENVIRONMENT VARIABLES:\n  \
        ZUORA_CLIENT_ID       OAuth client ID\n  \
        ZUORA_CLIENT_SECRET   OAuth client secret\n  \
        ZUORA_BASE_URL        API base URL (e.g. https://rest.na.zuora.com)\n\n\
        These override config file values but are overridden by CLI flags."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Named profile from ~/.zuora/config.toml [default: default]
    #[arg(long, global = true, default_value = "default")]
    pub profile: String,

    /// Output format: json, table, raw
    #[arg(long, global = true, value_parser = ["json", "table", "raw"])]
    pub output: Option<String>,

    /// Override the Zuora API base URL
    #[arg(long, global = true)]
    pub base_url: Option<String>,

    /// Show HTTP request/response details for debugging
    #[arg(long, global = true, default_value_t = false)]
    pub verbose: bool,

    /// Preview mutation requests (POST/PUT/DELETE) without sending them
    #[arg(long, global = true, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    // === Utility ===

    /// Authenticate and save credentials to ~/.zuora/config.toml
    Login {
        /// OAuth client ID (or set ZUORA_CLIENT_ID env var)
        #[arg(long)]
        client_id: Option<String>,
        /// OAuth client secret (or set ZUORA_CLIENT_SECRET env var)
        #[arg(long)]
        client_secret: Option<String>,
        /// Zuora API base URL — will prompt to select if not provided
        #[arg(long)]
        base_url: Option<String>,
    },

    /// Show current profile, environment, auth status, and tenant info
    Whoami,

    /// Check Zuora API reachability and response latency
    Status,

    /// Manage CLI configuration profiles
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Execute a ZOQL query (Zuora Object Query Language)
    #[command(after_help = "EXAMPLES:\n  \
        zuora query \"SELECT Id, Name, Status FROM Account LIMIT 10\"\n  \
        zuora query \"SELECT InvoiceNumber, Amount FROM Invoice WHERE AccountId = 'abc'\" --output json\n  \
        zuora query \"SELECT Id FROM Subscription\" --limit 5")]
    Query {
        /// ZOQL query string (e.g. "SELECT Id, Name FROM Account")
        zoql: String,
        /// Maximum number of rows to return
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Describe an object's schema — lists all fields, types, and metadata
    #[command(after_help = "EXAMPLES:\n  \
        zuora describe Account\n  \
        zuora describe Invoice --output json\n  \
        zuora describe Subscription")]
    Describe {
        /// Object name (e.g. Account, Invoice, Subscription, PaymentMethod)
        object: String,
    },

    /// Generate shell completions for bash, zsh, or fish
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    // === Core Billing ===

    /// Manage billing accounts
    Accounts { #[command(subcommand)] action: AccountsAction },
    /// Manage subscriptions
    Subscriptions { #[command(subcommand)] action: SubscriptionsAction },
    /// Manage invoices and download PDFs
    Invoices { #[command(subcommand)] action: InvoicesAction },
    /// Manage payments
    Payments { #[command(subcommand)] action: PaymentsAction },
    /// Manage payment methods (credit cards, ACH, etc.)
    PaymentMethods { #[command(subcommand)] action: PaymentMethodsAction },
    /// Manage orders and order actions
    Orders { #[command(subcommand)] action: OrdersAction },
    /// Manage credit memos
    CreditMemos { #[command(subcommand)] action: CreditMemosAction },
    /// Manage debit memos
    DebitMemos { #[command(subcommand)] action: DebitMemosAction },
    /// Manage refunds
    Refunds { #[command(subcommand)] action: RefundsAction },
    /// Manage account contacts
    Contacts { #[command(subcommand)] action: ContactsAction },

    // === Product Catalog ===

    /// Browse and manage the product catalog
    Catalog { #[command(subcommand)] action: CatalogAction },
    /// Manage catalog groups (product bundles)
    CatalogGroups { #[command(subcommand)] action: CatalogGroupsAction },

    // === Billing Operations ===

    /// Manage bill runs (batch invoice generation)
    BillRuns { #[command(subcommand)] action: BillRunsAction },
    /// Manage payment runs (batch payment processing)
    PaymentRuns { #[command(subcommand)] action: PaymentRunsAction },
    /// Manage payment schedules
    PaymentSchedules { #[command(subcommand)] action: PaymentSchedulesAction },
    /// Upload and query usage records for metered billing
    Usage { #[command(subcommand)] action: UsageAction },
    /// List all billing documents (invoices, memos) for an account
    BillingDocuments {
        /// Account ID to list billing documents for
        #[arg(long)]
        account: String,
    },
    /// Preview upcoming invoices before they are generated
    BillingPreview { #[command(subcommand)] action: BillingPreviewAction },
    /// Bulk operations: invoice-collect, async job status
    Operations { #[command(subcommand)] action: OperationsAction },
    /// Manage invoice adjustments (credits/charges applied to invoices)
    Adjustments { #[command(subcommand)] action: AdjustmentsAction },
    /// Manage invoice schedules (recurring invoice generation rules)
    InvoiceSchedules { #[command(subcommand)] action: InvoiceSchedulesAction },
    /// Create a new account with a subscription in one call
    SignUp {
        /// JSON body for the sign-up request (account + subscription details)
        #[arg(long)]
        body: String,
    },

    // === Files & Attachments ===

    /// Download files and check file status
    Files { #[command(subcommand)] action: FilesAction },
    /// Manage file attachments on Zuora objects
    Attachments { #[command(subcommand)] action: AttachmentsAction },

    // === Accounting & Revenue ===

    /// Manage GL accounting codes
    AccountingCodes { #[command(subcommand)] action: AccountingCodesAction },
    /// Manage accounting periods (open, close, reopen)
    AccountingPeriods { #[command(subcommand)] action: AccountingPeriodsAction },
    /// Manage journal entries for revenue recognition
    JournalEntries { #[command(subcommand)] action: JournalEntriesAction },
    /// Manage journal runs (batch journal entry processing)
    JournalRuns { #[command(subcommand)] action: JournalRunsAction },
    /// Manage tax line items on invoices and memos
    TaxationItems { #[command(subcommand)] action: TaxationItemsAction },

    // === Order Details ===

    /// Manage order fulfillments
    Fulfillments { #[command(subcommand)] action: FulfillmentsAction },
    /// Manage order line items
    OrderLineItems { #[command(subcommand)] action: OrderLineItemsAction },

    // === Subscription Details ===

    /// View subscription change logs (audit trail)
    SubscriptionChangeLogs { #[command(subcommand)] action: SubscriptionChangeLogsAction },
    /// View ramp deal details and metrics
    Ramps { #[command(subcommand)] action: RampsAction },

    // === Admin ===

    /// View notification history (callouts and emails)
    Notifications { #[command(subcommand)] action: NotificationsAction },
    /// Manage document sequence number sets
    SequenceSets { #[command(subcommand)] action: SequenceSetsAction },
    /// Get custom exchange rates for a currency
    ExchangeRates {
        /// Currency code (e.g. EUR, GBP, JPY)
        currency: String,
    },
    /// View minimum commitment details
    Commitments { #[command(subcommand)] action: CommitmentsAction },

    // === Workflow Commands ===

    /// Full billing context: account + invoices + payments + payment method + declines
    #[command(after_help = "EXAMPLE:\n  zuora billing-context A00012345\n  zuora billing-context 8a29... --output json")]
    BillingContext {
        /// Account ID or account number
        account_key: String,
    },
    /// Collections report: outstanding invoices aged by 30/60/90+ day buckets
    Collections {
        /// Account ID or account number
        account_key: String,
    },
    /// Customer health score: subscriptions, payment method, balance, decline rate
    CustomerHealth {
        /// Account ID or account number
        account_key: String,
    },

    // === MCP Server ===

    /// Start MCP server over stdio (for Claude Desktop, Cursor, and other AI clients)
    #[command(after_help = "Configure in Claude Desktop:\n  {\n    \"mcpServers\": {\n      \"zuora\": {\n        \"command\": \"zuora\",\n        \"args\": [\"mcp\"]\n      }\n    }\n  }")]
    Mcp,
}

impl Cli {
    pub fn generate_completions(shell: clap_complete::Shell) {
        let mut cmd = Self::command();
        clap_complete::generate(shell, &mut cmd, "zuora", &mut std::io::stdout());
    }
}

// === Subcommand Enums (with full help text) ===

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value (keys: client_id, client_secret, base_url)
    Set { key: String, value: String },
    /// Get a config value
    Get { key: String },
    /// List all config values for the active profile
    List,
}

#[derive(Subcommand)]
pub enum AccountsAction {
    /// List accounts (uses ZOQL query internally)
    List {
        /// Filter by status: Active, Draft, Canceled
        #[arg(long)]
        status: Option<String>,
        /// Maximum number of accounts to return [default: 20]
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Get detailed account information
    Get {
        /// Account ID (e.g. 8a29...) or account number (e.g. A00001234)
        account_key: String,
    },
    /// Create a new account
    Create {
        /// Account name
        #[arg(long)]
        name: String,
        /// Currency code [default: USD]
        #[arg(long, default_value = "USD")]
        currency: String,
    },
    /// Update account fields
    Update {
        /// Account ID or account number
        account_key: String,
        /// JSON object of fields to update (e.g. '{"name":"New Name"}')
        #[arg(long)]
        fields: String,
    },
}

#[derive(Subcommand)]
pub enum SubscriptionsAction {
    /// List subscriptions for an account
    List {
        /// Account ID to list subscriptions for
        #[arg(long)]
        account: String,
        /// Filter by status: Active, Cancelled, Expired, Suspended
        #[arg(long)]
        status: Option<String>,
    },
    /// Get subscription details
    Get {
        /// Subscription ID or subscription number
        subscription_key: String,
    },
    /// Cancel a subscription at end of current term
    Cancel {
        /// Subscription ID or subscription number to cancel
        subscription_key: String,
    },
}

#[derive(Subcommand)]
pub enum InvoicesAction {
    /// List invoices for an account
    List {
        /// Account ID to list invoices for
        #[arg(long)]
        account: String,
        /// Filter by status: Draft, Posted, Canceled, Error
        #[arg(long)]
        status: Option<String>,
        /// Maximum invoices to return [default: 20]
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Get invoice details
    Get {
        /// Invoice ID
        invoice_id: String,
    },
    /// Download invoice as PDF
    Pdf {
        /// Invoice ID to download
        invoice_id: String,
        /// Output file path [default: invoice-{id}.pdf]
        #[arg(long, short)]
        output_file: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PaymentsAction {
    /// List payments for an account
    List {
        /// Account ID
        #[arg(long)]
        account: String,
    },
    /// Get payment details
    Get { payment_id: String },
    /// Create a new payment
    Create {
        /// Account ID to charge
        #[arg(long)]
        account: String,
        /// Payment amount
        #[arg(long)]
        amount: f64,
        /// Payment method ID to use
        #[arg(long)]
        payment_method: String,
    },
}

#[derive(Subcommand)]
pub enum PaymentMethodsAction {
    /// List payment methods for an account
    List {
        /// Account ID
        #[arg(long)]
        account: String,
    },
    /// Get payment method details
    Get { pm_id: String },
    /// Create a payment method
    Create {
        /// Account ID to associate with
        #[arg(long)]
        account: String,
        /// JSON body for the payment method (e.g. '{"type":"CreditCard",...}')
        #[arg(long)]
        body: String,
    },
    /// Update payment method fields
    Update {
        pm_id: String,
        /// JSON object of fields to update
        #[arg(long)]
        fields: String,
    },
    /// Delete a payment method
    Delete { pm_id: String },
}

#[derive(Subcommand)]
pub enum OrdersAction {
    /// List orders for an account
    List {
        /// Account number (e.g. A00001234)
        #[arg(long)]
        account: String,
    },
    /// Get order details
    Get { order_number: String },
    /// Create an order from a JSON file or stdin
    Create {
        /// Path to JSON file with order body (reads from stdin if omitted)
        #[arg(long)]
        file: Option<String>,
    },
    /// Cancel an order
    Cancel { order_number: String },
}

#[derive(Subcommand)]
pub enum CreditMemosAction {
    /// List credit memos for an account
    List {
        #[arg(long)]
        account: String,
    },
    /// Get credit memo details
    Get { memo_id: String },
    /// Create a credit memo
    Create {
        /// Account ID
        #[arg(long)]
        account: String,
        /// Credit amount
        #[arg(long)]
        amount: f64,
        /// Reason code [default: Customer Goodwill]
        #[arg(long, default_value = "Customer Goodwill")]
        reason: String,
    },
}

#[derive(Subcommand)]
pub enum DebitMemosAction {
    /// List debit memos for an account
    List {
        #[arg(long)]
        account: String,
    },
    /// Get debit memo details
    Get { memo_id: String },
    /// Create a debit memo from JSON body
    Create {
        /// JSON body (e.g. '{"accountId":"...","amount":100}')
        #[arg(long)]
        body: String,
    },
    /// Cancel a debit memo
    Cancel { memo_id: String },
}

#[derive(Subcommand)]
pub enum RefundsAction {
    /// List refunds for an account
    List {
        #[arg(long)]
        account: String,
    },
    /// Get refund details
    Get { refund_id: String },
    /// Create a refund against a payment
    Create {
        /// Payment ID to refund
        #[arg(long)]
        payment: String,
        /// Refund amount
        #[arg(long)]
        amount: f64,
    },
}

#[derive(Subcommand)]
pub enum ContactsAction {
    /// Get contact details
    Get { contact_id: String },
    /// Create a contact from JSON body
    Create {
        /// JSON body with contact fields
        #[arg(long)]
        body: String,
    },
    /// Update a contact
    Update {
        contact_id: String,
        /// JSON object of fields to update
        #[arg(long)]
        fields: String,
    },
    /// Delete a contact
    Delete { contact_id: String },
}

#[derive(Subcommand)]
pub enum CatalogAction {
    /// List all products in the catalog
    List,
    /// Get product details by key
    Get { product_key: String },
    /// List rate plans for a product
    RatePlans { product_key: String },
}

#[derive(Subcommand)]
pub enum CatalogGroupsAction {
    /// List all catalog groups
    List,
    /// Get catalog group details
    Get { key: String },
    /// Create a catalog group from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Delete a catalog group
    Delete { key: String },
}

#[derive(Subcommand)]
pub enum BillRunsAction {
    /// Get bill run status and details
    Get { bill_run_id: String },
    /// Create a bill run from JSON body
    Create {
        /// JSON body (e.g. '{"invoiceDate":"2024-01-01","targetDate":"2024-01-01"}')
        #[arg(long)]
        body: String,
    },
    /// Cancel a pending bill run
    Cancel { bill_run_id: String },
    /// Post a completed bill run (make invoices live)
    Post { bill_run_id: String },
    /// Delete a bill run
    Delete { bill_run_id: String },
}

#[derive(Subcommand)]
pub enum PaymentRunsAction {
    /// List all payment runs
    List,
    /// Get payment run details
    Get { key: String },
    /// Create a payment run from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Get payment run summary (totals, errors, etc.)
    Summary { key: String },
    /// Delete a payment run
    Delete { key: String },
}

#[derive(Subcommand)]
pub enum PaymentSchedulesAction {
    /// List all payment schedules
    List,
    /// Get payment schedule details
    Get { key: String },
    /// Create a payment schedule from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Cancel a payment schedule
    Cancel { key: String },
    /// Delete a payment schedule
    Delete { key: String },
}

#[derive(Subcommand)]
pub enum UsageAction {
    /// Upload usage records (JSON body)
    Upload {
        /// JSON body with usage data
        #[arg(long)]
        body: String,
    },
    /// Query usage records for an account
    Query {
        /// Account ID
        #[arg(long)]
        account: String,
    },
}

#[derive(Subcommand)]
pub enum BillingPreviewAction {
    /// Create a billing preview run
    Create {
        /// JSON body (e.g. '{"accountId":"...","targetDate":"2024-12-31"}')
        #[arg(long)]
        body: String,
    },
    /// Get billing preview run status and results
    Get { id: String },
}

#[derive(Subcommand)]
pub enum OperationsAction {
    /// Run invoice generation and payment collection for an account
    InvoiceCollect {
        /// JSON body (e.g. '{"accountId":"..."}')
        #[arg(long)]
        body: String,
    },
    /// Check async job status by job ID
    JobStatus { job_id: String },
}

#[derive(Subcommand)]
pub enum AdjustmentsAction {
    /// List all adjustments
    List,
    /// Get adjustment details
    Get { key: String },
    /// Create an adjustment from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Cancel an adjustment
    Cancel { id: String },
}

#[derive(Subcommand)]
pub enum InvoiceSchedulesAction {
    /// Get invoice schedule details
    Get { key: String },
    /// Create an invoice schedule from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Pause an active invoice schedule
    Pause { key: String },
    /// Resume a paused invoice schedule
    Resume { key: String },
    /// Delete an invoice schedule
    Delete { key: String },
}

#[derive(Subcommand)]
pub enum FilesAction {
    /// Get file status and metadata
    Get { file_id: String },
    /// Download a file to disk
    Download {
        file_id: String,
        /// Output file path
        #[arg(long, short)]
        output_file: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum AttachmentsAction {
    /// List attachments on an object (e.g. Account, Invoice)
    List {
        /// Object type (e.g. Account, Invoice, Subscription)
        object_type: String,
        /// Object ID or key
        object_key: String,
    },
    /// Get attachment details
    Get { id: String },
    /// Delete an attachment
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum AccountingCodesAction {
    /// List all accounting codes
    List,
    /// Get accounting code details
    Get { id: String },
    /// Create an accounting code from JSON body
    Create {
        /// JSON body (e.g. '{"name":"Deferred Revenue","type":"AccountsReceivable"}')
        #[arg(long)]
        body: String,
    },
    /// Activate an accounting code
    Activate { id: String },
    /// Deactivate an accounting code
    Deactivate { id: String },
    /// Delete an accounting code
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum AccountingPeriodsAction {
    /// List all accounting periods
    List,
    /// Get accounting period details
    Get { id: String },
    /// Create an accounting period from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Close an accounting period
    Close { id: String },
    /// Reopen a closed accounting period
    Reopen { id: String },
    /// Delete an accounting period
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum JournalEntriesAction {
    /// List journal entries by journal run number
    List {
        /// Journal run number (e.g. JR-00000001)
        #[arg(long)]
        journal_run: String,
    },
    /// Get journal entry details
    Get { je_number: String },
    /// Create a journal entry from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Cancel a journal entry
    Cancel { je_number: String },
    /// Delete a journal entry
    Delete { je_number: String },
}

#[derive(Subcommand)]
pub enum JournalRunsAction {
    /// Get journal run details
    Get { jr_number: String },
    /// Create a journal run from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Cancel a journal run
    Cancel { jr_number: String },
    /// Delete a journal run
    Delete { jr_number: String },
}

#[derive(Subcommand)]
pub enum TaxationItemsAction {
    /// Get taxation item details
    Get { id: String },
    /// Update a taxation item
    Update {
        id: String,
        /// JSON object of fields to update
        #[arg(long)]
        fields: String,
    },
    /// Delete a taxation item
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum FulfillmentsAction {
    /// Get fulfillment details
    Get { key: String },
    /// Create a fulfillment from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Delete a fulfillment
    Delete { key: String },
}

#[derive(Subcommand)]
pub enum OrderLineItemsAction {
    /// Get order line item details
    Get { item_id: String },
    /// Update an order line item
    Update {
        item_id: String,
        /// JSON object of fields to update
        #[arg(long)]
        fields: String,
    },
}

#[derive(Subcommand)]
pub enum SubscriptionChangeLogsAction {
    /// View change history for a subscription
    BySubscription { subscription_number: String },
    /// View change history for an order
    ByOrder { order_number: String },
}

#[derive(Subcommand)]
pub enum RampsAction {
    /// Get ramp deal details
    Get { ramp_number: String },
    /// Get ramp metrics (MRR, TCV, etc.)
    Metrics { ramp_number: String },
}

#[derive(Subcommand)]
pub enum NotificationsAction {
    /// View callout notification history
    Callouts,
    /// View email notification history
    Emails,
}

#[derive(Subcommand)]
pub enum SequenceSetsAction {
    /// List all sequence number sets
    List,
    /// Get sequence set details
    Get { id: String },
    /// Create a sequence set from JSON body
    Create {
        #[arg(long)]
        body: String,
    },
    /// Delete a sequence set
    Delete { id: String },
}

#[derive(Subcommand)]
pub enum CommitmentsAction {
    /// List all minimum commitments
    List,
    /// Get commitment details
    Get { key: String },
}
