mod cli;
mod client;
mod commands;
mod config;
mod output;
mod types;

use anyhow::Result;
use clap::Parser;

use cli::*;
use client::ZuoraClient;
use config::{ConfigStore, Profile};
use output::OutputFormat;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err:#}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let fmt = OutputFormat::from_str_opt(cli.output.as_deref());

    match cli.command {
        // --- No auth needed ---
        Commands::Login { client_id, client_secret, base_url } => {
            commands::login::run(&cli.profile, client_id.as_deref(), client_secret.as_deref(), base_url.as_deref()).await
        }
        Commands::Whoami => {
            let mut client = build_client(&cli.profile, cli.base_url.as_deref(), false, false).ok();
            commands::whoami::run(&cli.profile, client.as_mut()).await
        }
        Commands::Completions { shell } => { Cli::generate_completions(shell); Ok(()) }
        Commands::Config { action } => match action {
            ConfigAction::Set { key, value } => commands::config_cmd::run_set(&cli.profile, &key, &value),
            ConfigAction::Get { key } => commands::config_cmd::run_get(&cli.profile, &key),
            ConfigAction::List => commands::config_cmd::run_list(&cli.profile),
        },

        // --- Auth needed ---
        cmd => {
            let mut c = build_client(&cli.profile, cli.base_url.as_deref(), cli.verbose, cli.dry_run)?;
            dispatch(&mut c, cmd, fmt).await
        }
    }
}

async fn dispatch(c: &mut ZuoraClient, cmd: Commands, fmt: OutputFormat) -> Result<()> {
    match cmd {
        Commands::Status => commands::status::run(c).await,
        Commands::Query { zoql, limit } => commands::query::run(c, &zoql, fmt, limit).await,
        Commands::Describe { object } => commands::describe::run(c, &object, fmt).await,
        Commands::SignUp { body } => commands::sign_up::run(c, &body, fmt).await,
        Commands::ExchangeRates { currency } => commands::custom_exchange_rates::get(c, &currency, fmt).await,
        Commands::BillingDocuments { account } => commands::billing_documents::list(c, &account, fmt).await,

        // Workflow commands
        Commands::BillingContext { account_key } => commands::workflows::billing_context(c, &account_key, fmt).await,
        Commands::Collections { account_key } => commands::workflows::collections(c, &account_key, fmt).await,
        Commands::CustomerHealth { account_key } => commands::workflows::customer_health(c, &account_key, fmt).await,

        // MCP server
        Commands::Mcp => commands::mcp::serve(c).await,

        // Core billing
        Commands::Accounts { action } => match action {
            AccountsAction::List { status, limit } => commands::accounts::list(c, fmt, status.as_deref(), limit).await,
            AccountsAction::Get { account_key } => commands::accounts::get(c, &account_key, fmt).await,
            AccountsAction::Create { name, currency } => commands::accounts::create(c, &name, &currency, fmt).await,
            AccountsAction::Update { account_key, fields } => commands::accounts::update(c, &account_key, &fields, fmt).await,
        },
        Commands::Subscriptions { action } => match action {
            SubscriptionsAction::List { account, status } => commands::subscriptions::list(c, &account, fmt, status.as_deref()).await,
            SubscriptionsAction::Get { subscription_key } => commands::subscriptions::get(c, &subscription_key, fmt).await,
            SubscriptionsAction::Cancel { subscription_key } => commands::subscriptions::cancel(c, &subscription_key, fmt).await,
        },
        Commands::Invoices { action } => match action {
            InvoicesAction::List { account, status, limit } => commands::invoices::list(c, &account, fmt, status.as_deref(), limit).await,
            InvoicesAction::Get { invoice_id } => commands::invoices::get(c, &invoice_id, fmt).await,
            InvoicesAction::Pdf { invoice_id, output_file } => commands::invoices::pdf(c, &invoice_id, output_file.as_deref()).await,
        },
        Commands::Payments { action } => match action {
            PaymentsAction::List { account } => commands::payments::list(c, &account, fmt).await,
            PaymentsAction::Get { payment_id } => commands::payments::get(c, &payment_id, fmt).await,
            PaymentsAction::Create { account, amount, payment_method } => commands::payments::create(c, &account, amount, &payment_method, fmt).await,
        },
        Commands::PaymentMethods { action } => match action {
            PaymentMethodsAction::List { account } => commands::payment_methods::list(c, &account, fmt).await,
            PaymentMethodsAction::Get { pm_id } => commands::payment_methods::get(c, &pm_id, fmt).await,
            PaymentMethodsAction::Create { account, body } => commands::payment_methods::create(c, &account, &body, fmt).await,
            PaymentMethodsAction::Update { pm_id, fields } => commands::payment_methods::update(c, &pm_id, &fields, fmt).await,
            PaymentMethodsAction::Delete { pm_id } => commands::payment_methods::delete(c, &pm_id, fmt).await,
        },
        Commands::Orders { action } => match action {
            OrdersAction::List { account } => commands::orders::list(c, &account, fmt).await,
            OrdersAction::Get { order_number } => commands::orders::get(c, &order_number, fmt).await,
            OrdersAction::Create { file } => commands::orders::create(c, file.as_deref(), fmt).await,
            OrdersAction::Cancel { order_number } => commands::orders::cancel(c, &order_number, fmt).await,
        },
        Commands::CreditMemos { action } => match action {
            CreditMemosAction::List { account } => commands::credit_memos::list(c, &account, fmt).await,
            CreditMemosAction::Get { memo_id } => commands::credit_memos::get(c, &memo_id, fmt).await,
            CreditMemosAction::Create { account, amount, reason } => commands::credit_memos::create(c, &account, amount, &reason, fmt).await,
        },
        Commands::DebitMemos { action } => match action {
            DebitMemosAction::List { account } => commands::debit_memos::list(c, &account, fmt).await,
            DebitMemosAction::Get { memo_id } => commands::debit_memos::get(c, &memo_id, fmt).await,
            DebitMemosAction::Create { body } => commands::debit_memos::create(c, &body, fmt).await,
            DebitMemosAction::Cancel { memo_id } => commands::debit_memos::cancel(c, &memo_id, fmt).await,
        },
        Commands::Refunds { action } => match action {
            RefundsAction::List { account } => commands::refunds::list(c, &account, fmt).await,
            RefundsAction::Get { refund_id } => commands::refunds::get(c, &refund_id, fmt).await,
            RefundsAction::Create { payment, amount } => commands::refunds::create(c, &payment, amount, fmt).await,
        },
        Commands::Contacts { action } => match action {
            ContactsAction::Get { contact_id } => commands::contacts::get(c, &contact_id, fmt).await,
            ContactsAction::Create { body } => commands::contacts::create(c, &body, fmt).await,
            ContactsAction::Update { contact_id, fields } => commands::contacts::update(c, &contact_id, &fields, fmt).await,
            ContactsAction::Delete { contact_id } => commands::contacts::delete(c, &contact_id, fmt).await,
        },

        // Product catalog
        Commands::Catalog { action } => match action {
            CatalogAction::List => commands::catalog::list(c, fmt).await,
            CatalogAction::Get { product_key } => commands::catalog::get(c, &product_key, fmt).await,
            CatalogAction::RatePlans { product_key } => commands::catalog::rate_plans(c, &product_key, fmt).await,
        },
        Commands::CatalogGroups { action } => match action {
            CatalogGroupsAction::List => commands::catalog_groups::list(c, fmt).await,
            CatalogGroupsAction::Get { key } => commands::catalog_groups::get(c, &key, fmt).await,
            CatalogGroupsAction::Create { body } => commands::catalog_groups::create(c, &body, fmt).await,
            CatalogGroupsAction::Delete { key } => commands::catalog_groups::delete(c, &key, fmt).await,
        },

        // Billing operations
        Commands::BillRuns { action } => match action {
            BillRunsAction::Get { bill_run_id } => commands::bill_runs::get(c, &bill_run_id, fmt).await,
            BillRunsAction::Create { body } => commands::bill_runs::create(c, &body, fmt).await,
            BillRunsAction::Cancel { bill_run_id } => commands::bill_runs::cancel(c, &bill_run_id, fmt).await,
            BillRunsAction::Post { bill_run_id } => commands::bill_runs::post(c, &bill_run_id, fmt).await,
            BillRunsAction::Delete { bill_run_id } => commands::bill_runs::delete(c, &bill_run_id, fmt).await,
        },
        Commands::PaymentRuns { action } => match action {
            PaymentRunsAction::List => commands::payment_runs::list(c, fmt).await,
            PaymentRunsAction::Get { key } => commands::payment_runs::get(c, &key, fmt).await,
            PaymentRunsAction::Create { body } => commands::payment_runs::create(c, &body, fmt).await,
            PaymentRunsAction::Summary { key } => commands::payment_runs::summary(c, &key, fmt).await,
            PaymentRunsAction::Delete { key } => commands::payment_runs::delete(c, &key, fmt).await,
        },
        Commands::PaymentSchedules { action } => match action {
            PaymentSchedulesAction::List => commands::payment_schedules::list(c, fmt).await,
            PaymentSchedulesAction::Get { key } => commands::payment_schedules::get(c, &key, fmt).await,
            PaymentSchedulesAction::Create { body } => commands::payment_schedules::create(c, &body, fmt).await,
            PaymentSchedulesAction::Cancel { key } => commands::payment_schedules::cancel(c, &key, fmt).await,
            PaymentSchedulesAction::Delete { key } => commands::payment_schedules::delete(c, &key, fmt).await,
        },
        Commands::Usage { action } => match action {
            UsageAction::Upload { body } => commands::usage::upload(c, &body, fmt).await,
            UsageAction::Query { account } => commands::usage::query(c, &account, fmt).await,
        },
        Commands::BillingPreview { action } => match action {
            BillingPreviewAction::Create { body } => commands::billing_preview::create(c, &body, fmt).await,
            BillingPreviewAction::Get { id } => commands::billing_preview::get(c, &id, fmt).await,
        },
        Commands::Operations { action } => match action {
            OperationsAction::InvoiceCollect { body } => commands::operations::invoice_collect(c, &body, fmt).await,
            OperationsAction::JobStatus { job_id } => commands::operations::job_status(c, &job_id, fmt).await,
        },
        Commands::Adjustments { action } => match action {
            AdjustmentsAction::List => commands::adjustments::list(c, fmt).await,
            AdjustmentsAction::Get { key } => commands::adjustments::get(c, &key, fmt).await,
            AdjustmentsAction::Create { body } => commands::adjustments::create(c, &body, fmt).await,
            AdjustmentsAction::Cancel { id } => commands::adjustments::cancel(c, &id, fmt).await,
        },
        Commands::InvoiceSchedules { action } => match action {
            InvoiceSchedulesAction::Get { key } => commands::invoice_schedules::get(c, &key, fmt).await,
            InvoiceSchedulesAction::Create { body } => commands::invoice_schedules::create(c, &body, fmt).await,
            InvoiceSchedulesAction::Pause { key } => commands::invoice_schedules::pause(c, &key, fmt).await,
            InvoiceSchedulesAction::Resume { key } => commands::invoice_schedules::resume(c, &key, fmt).await,
            InvoiceSchedulesAction::Delete { key } => commands::invoice_schedules::delete(c, &key, fmt).await,
        },

        // Files & attachments
        Commands::Files { action } => match action {
            FilesAction::Get { file_id } => commands::files::get(c, &file_id, fmt).await,
            FilesAction::Download { file_id, output_file } => commands::files::download(c, &file_id, output_file.as_deref()).await,
        },
        Commands::Attachments { action } => match action {
            AttachmentsAction::List { object_type, object_key } => commands::attachments::list(c, &object_type, &object_key, fmt).await,
            AttachmentsAction::Get { id } => commands::attachments::get(c, &id, fmt).await,
            AttachmentsAction::Delete { id } => commands::attachments::delete(c, &id, fmt).await,
        },

        // Accounting & revenue
        Commands::AccountingCodes { action } => match action {
            AccountingCodesAction::List => commands::accounting_codes::list(c, fmt).await,
            AccountingCodesAction::Get { id } => commands::accounting_codes::get(c, &id, fmt).await,
            AccountingCodesAction::Create { body } => commands::accounting_codes::create(c, &body, fmt).await,
            AccountingCodesAction::Activate { id } => commands::accounting_codes::activate(c, &id, fmt).await,
            AccountingCodesAction::Deactivate { id } => commands::accounting_codes::deactivate(c, &id, fmt).await,
            AccountingCodesAction::Delete { id } => commands::accounting_codes::delete(c, &id, fmt).await,
        },
        Commands::AccountingPeriods { action } => match action {
            AccountingPeriodsAction::List => commands::accounting_periods::list(c, fmt).await,
            AccountingPeriodsAction::Get { id } => commands::accounting_periods::get(c, &id, fmt).await,
            AccountingPeriodsAction::Create { body } => commands::accounting_periods::create(c, &body, fmt).await,
            AccountingPeriodsAction::Close { id } => commands::accounting_periods::close(c, &id, fmt).await,
            AccountingPeriodsAction::Reopen { id } => commands::accounting_periods::reopen(c, &id, fmt).await,
            AccountingPeriodsAction::Delete { id } => commands::accounting_periods::delete(c, &id, fmt).await,
        },
        Commands::JournalEntries { action } => match action {
            JournalEntriesAction::List { journal_run } => commands::journal_entries::list(c, &journal_run, fmt).await,
            JournalEntriesAction::Get { je_number } => commands::journal_entries::get(c, &je_number, fmt).await,
            JournalEntriesAction::Create { body } => commands::journal_entries::create(c, &body, fmt).await,
            JournalEntriesAction::Cancel { je_number } => commands::journal_entries::cancel(c, &je_number, fmt).await,
            JournalEntriesAction::Delete { je_number } => commands::journal_entries::delete(c, &je_number, fmt).await,
        },
        Commands::JournalRuns { action } => match action {
            JournalRunsAction::Get { jr_number } => commands::journal_runs::get(c, &jr_number, fmt).await,
            JournalRunsAction::Create { body } => commands::journal_runs::create(c, &body, fmt).await,
            JournalRunsAction::Cancel { jr_number } => commands::journal_runs::cancel(c, &jr_number, fmt).await,
            JournalRunsAction::Delete { jr_number } => commands::journal_runs::delete(c, &jr_number, fmt).await,
        },
        Commands::TaxationItems { action } => match action {
            TaxationItemsAction::Get { id } => commands::taxation_items::get(c, &id, fmt).await,
            TaxationItemsAction::Update { id, fields } => commands::taxation_items::update(c, &id, &fields, fmt).await,
            TaxationItemsAction::Delete { id } => commands::taxation_items::delete(c, &id, fmt).await,
        },

        // Order details
        Commands::Fulfillments { action } => match action {
            FulfillmentsAction::Get { key } => commands::fulfillments::get(c, &key, fmt).await,
            FulfillmentsAction::Create { body } => commands::fulfillments::create(c, &body, fmt).await,
            FulfillmentsAction::Delete { key } => commands::fulfillments::delete(c, &key, fmt).await,
        },
        Commands::OrderLineItems { action } => match action {
            OrderLineItemsAction::Get { item_id } => commands::order_line_items::get(c, &item_id, fmt).await,
            OrderLineItemsAction::Update { item_id, fields } => commands::order_line_items::update(c, &item_id, &fields, fmt).await,
        },

        // Subscription details
        Commands::SubscriptionChangeLogs { action } => match action {
            SubscriptionChangeLogsAction::BySubscription { subscription_number } => commands::subscription_change_logs::by_subscription(c, &subscription_number, fmt).await,
            SubscriptionChangeLogsAction::ByOrder { order_number } => commands::subscription_change_logs::by_order(c, &order_number, fmt).await,
        },
        Commands::Ramps { action } => match action {
            RampsAction::Get { ramp_number } => commands::ramps::get(c, &ramp_number, fmt).await,
            RampsAction::Metrics { ramp_number } => commands::ramps::metrics(c, &ramp_number, fmt).await,
        },

        // Admin
        Commands::Notifications { action } => match action {
            NotificationsAction::Callouts => commands::notifications::callout_history(c, fmt).await,
            NotificationsAction::Emails => commands::notifications::email_history(c, fmt).await,
        },
        Commands::SequenceSets { action } => match action {
            SequenceSetsAction::List => commands::sequence_sets::list(c, fmt).await,
            SequenceSetsAction::Get { id } => commands::sequence_sets::get(c, &id, fmt).await,
            SequenceSetsAction::Create { body } => commands::sequence_sets::create(c, &body, fmt).await,
            SequenceSetsAction::Delete { id } => commands::sequence_sets::delete(c, &id, fmt).await,
        },
        Commands::Commitments { action } => match action {
            CommitmentsAction::List => commands::commitments::list(c, fmt).await,
            CommitmentsAction::Get { key } => commands::commitments::get(c, &key, fmt).await,
        },

        // Already handled
        Commands::Login { .. } | Commands::Whoami | Commands::Config { .. } | Commands::Completions { .. } => unreachable!(),
    }
}

fn build_client(profile_name: &str, base_url_override: Option<&str>, verbose: bool, dry_run: bool) -> Result<ZuoraClient> {
    let store = ConfigStore::new()?;
    let config_profile = store.get_profile(profile_name)?;
    let resolved = Profile::resolve(
        config_profile.as_ref(),
        std::env::var("ZUORA_CLIENT_ID").ok(),
        std::env::var("ZUORA_CLIENT_SECRET").ok(),
        std::env::var("ZUORA_BASE_URL").ok(),
        base_url_override,
    )?;
    let mut client = ZuoraClient::new(resolved, profile_name.to_string(), store);
    client.verbose = verbose;
    client.dry_run = dry_run;
    Ok(client)
}
