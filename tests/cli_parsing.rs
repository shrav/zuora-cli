use assert_cmd::Command;
use predicates::prelude::*;

fn zuora() -> Command {
    Command::cargo_bin("zuora").unwrap()
}

#[test]
fn help_shows_all_commands() {
    zuora()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("whoami"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("accounts"))
        .stdout(predicate::str::contains("subscriptions"))
        .stdout(predicate::str::contains("invoices"))
        .stdout(predicate::str::contains("payments"))
        .stdout(predicate::str::contains("payment-methods"))
        .stdout(predicate::str::contains("orders"))
        .stdout(predicate::str::contains("credit-memos"))
        .stdout(predicate::str::contains("refunds"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn version_flag() {
    zuora()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("zuora"));
}

#[test]
fn accounts_help_shows_subcommands() {
    zuora()
        .args(["accounts", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn subscriptions_help_shows_subcommands() {
    zuora()
        .args(["subscriptions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("cancel"));
}

#[test]
fn invoices_help_shows_subcommands() {
    zuora()
        .args(["invoices", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("pdf"));
}

#[test]
fn payments_help_shows_subcommands() {
    zuora()
        .args(["payments", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn payment_methods_help_shows_subcommands() {
    zuora()
        .args(["payment-methods", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn orders_help_shows_subcommands() {
    zuora()
        .args(["orders", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("cancel"));
}

#[test]
fn credit_memos_help_shows_subcommands() {
    zuora()
        .args(["credit-memos", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn refunds_help_shows_subcommands() {
    zuora()
        .args(["refunds", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"));
}

#[test]
fn completions_help_shows_shells() {
    zuora()
        .args(["completions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("zsh"))
        .stdout(predicate::str::contains("fish"));
}

#[test]
fn completions_generates_bash() {
    zuora()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("zuora"));
}

#[test]
fn completions_generates_zsh() {
    zuora()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("zuora"));
}

#[test]
fn global_flags_accepted() {
    zuora()
        .args(["--profile", "staging", "--output", "json", "--verbose", "accounts", "--help"])
        .assert()
        .success();
}

#[test]
fn config_list_works_without_credentials() {
    zuora()
        .args(["config", "list"])
        .assert()
        .success();
}

#[test]
fn no_args_shows_help() {
    zuora()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn unknown_command_errors() {
    zuora()
        .arg("bogus")
        .assert()
        .failure();
}

#[test]
fn query_without_arg_errors() {
    // query requires a positional arg
    zuora()
        .arg("query")
        .assert()
        .failure();
}

#[test]
fn accounts_get_without_key_errors() {
    zuora()
        .args(["accounts", "get"])
        .assert()
        .failure();
}

#[test]
fn payments_create_requires_all_flags() {
    zuora()
        .args(["payments", "create", "--account", "acc1"])
        .assert()
        .failure(); // missing --amount and --payment-method
}

#[test]
fn whoami_works_without_credentials() {
    zuora()
        .arg("whoami")
        .assert()
        .success();
}

#[test]
fn login_shows_client_id_flag() {
    zuora()
        .args(["login", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--client-id"))
        .stdout(predicate::str::contains("--client-secret"))
        .stdout(predicate::str::contains("--base-url"));
}

#[test]
fn dry_run_flag_accepted() {
    zuora()
        .args(["--dry-run", "accounts", "--help"])
        .assert()
        .success();
}
