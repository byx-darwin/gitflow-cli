//! Read-only typed Issue/PR searches with preview before execution.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "bounded local response read"
)]

use std::{io::Read, path::Path};

use clap::Args;
use gitflow_core::{
    decision::DecisionResponse,
    issue::{IssueProvider, ListIssueArgs},
    label::LabelProvider,
    pr::{ListPrArgs, PrProvider},
    query_filter::{
        Candidates, Filter, Origin, Plan, Target, apply_response, compile_at, decision_request,
    },
};
use gitflow_gitcode::{GitCodeIssueProvider, GitCodeLabelProvider, GitCodePrProvider};
use gitflow_github::{GitHubIssueProvider, GitHubLabelProvider, GitHubPrProvider};
use gitflow_gitlab::{GitLabIssueProvider, GitLabLabelProvider, GitLabMrProvider};

/// Bounded read-only natural language query.
#[derive(Debug, Clone, Args)]
pub struct SearchArgs {
    /// Query with explicit operators or simple natural language.
    #[arg(long)]
    pub query: String,
    /// Print the compiled plan without contacting the platform.
    #[arg(long)]
    pub explain: bool,
    /// Existing repository label, repeatable.
    #[arg(long = "known-label")]
    pub known_labels: Vec<String>,
    /// Existing user login, repeatable.
    #[arg(long = "known-user")]
    pub known_users: Vec<String>,
    /// Offline provider-neutral decision response for unresolved spans.
    #[arg(long, conflicts_with = "live")]
    pub response: Option<String>,
    /// Explicitly ask Jev only about unresolved spans.
    #[arg(long)]
    pub live: bool,
    /// Execute a high-confidence inferred plan after inspecting it.
    #[arg(long)]
    pub accept_inferred: bool,
    /// Maximum results to return.
    #[arg(long, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..=1000))]
    pub limit: u32,
}

fn read_response(path: &str) -> miette::Result<DecisionResponse> {
    let safe = gitflow_core::SafePath::new_allow_absolute(path)
        .map_err(|_| miette::miette!("invalid response path"))?;
    let mut bytes = Vec::new();
    std::fs::File::open(Path::new(safe.as_path()))
        .map_err(|_| miette::miette!("response cannot be opened"))?
        .take(131_073)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("response cannot be read"))?;
    if bytes.len() > 131_072 {
        return Err(miette::miette!("response too large"));
    }
    serde_json::from_slice(&bytes).map_err(|_| miette::miette!("response JSON invalid"))
}

/// Compile, preview, and optionally execute a read-only search.
///
/// # Errors
/// Returns an error for ambiguity, unsupported filters, truncation, or provider failures.
pub async fn handle(
    target: Target,
    args: SearchArgs,
    platform: &str,
    repo: &str,
    remote_url: &str,
) -> miette::Result<()> {
    let candidates = Candidates {
        labels: args.known_labels,
        users: args.known_users,
    };
    let today = chrono::Local::now().date_naive();
    let mut plan =
        compile_at(&args.query, target, &candidates, today).map_err(|e| miette::miette!("{e}"))?;
    let mut provider_status = "not_needed";
    if !plan.unresolved.is_empty() {
        let response = if let Some(path) = args.response {
            Some(read_response(&path)?)
        } else if args.live {
            live_response(&plan, &candidates).await
        } else {
            None
        };
        if let Some(response) = response {
            match apply_response(&plan, &candidates, &response) {
                Ok(resolved) => {
                    plan = resolved;
                    provider_status = "resolved";
                }
                Err(_) => provider_status = "needs_clarification",
            }
        } else {
            provider_status = "unavailable";
        }
    }
    plan.check_capabilities(platform)
        .map_err(|e| miette::miette!("{e}"))?;
    let mapping = mapping(&plan, platform);
    if args.explain {
        return super::output::print_output(
            &serde_json::json!({"plan": plan, "platform": platform, "mapping": mapping, "providerStatus": provider_status, "willExecute": false}),
            &crate::OutputFormat::Json,
        );
    }
    if !plan.unresolved.is_empty() {
        return Err(miette::miette!(
            "query contains unresolved spans; use --explain and provide verified candidates"
        ));
    }
    if plan.sources.iter().any(|s| s.origin == Origin::Jev) && !args.accept_inferred {
        return Err(miette::miette!(
            "inferred fields require --explain review and --accept-inferred before execution"
        ));
    }
    verify_labels(&plan, platform, repo, remote_url).await?;
    match target {
        Target::Issue => search_issues(&plan, platform, repo, remote_url, args.limit).await,
        Target::Pr => search_prs(&plan, platform, repo, remote_url, args.limit).await,
    }
}

fn mapping(plan: &Plan, platform: &str) -> Vec<serde_json::Value> {
    plan.filters
        .iter()
        .map(|filter| {
            let path = match (plan.target, filter) {
                (Target::Issue, Filter::State { .. } | Filter::Label { negated: false, .. })
                | (Target::Pr, Filter::State { .. }) => "provider_list_argument",
                (Target::Issue, Filter::Text { .. }) => "provider_search_then_local_filter",
                _ => "bounded_local_filter",
            };
            serde_json::json!({"filter": filter, "platform": platform, "mapping": path})
        })
        .collect()
}

async fn search_issues(
    plan: &Plan,
    platform: &str,
    repo: &str,
    remote_url: &str,
    limit: u32,
) -> miette::Result<()> {
    let provider: Box<dyn IssueProvider> = match platform {
        "github" => Box::new(GitHubIssueProvider::new(repo)),
        "gitlab" if !remote_url.is_empty() => {
            Box::new(GitLabIssueProvider::with_remote_url(repo, remote_url))
        }
        "gitlab" => Box::new(GitLabIssueProvider::new(repo)),
        "gitcode" => Box::new(GitCodeIssueProvider::new(repo)),
        _ => return Err(miette::miette!("unsupported platform")),
    };
    let args = ListIssueArgs {
        state: Some(
            plan.filters
                .iter()
                .find_map(|f| {
                    if let Filter::State { value } = f {
                        Some(*value)
                    } else {
                        None
                    }
                })
                .unwrap_or(gitflow_core::types::State::All),
        ),
        labels: plan
            .filters
            .iter()
            .filter_map(|f| {
                if let Filter::Label {
                    value,
                    negated: false,
                } = f
                {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect(),
        search: plan.filters.iter().find_map(|f| {
            if let Filter::Text { value } = f {
                Some(value.clone())
            } else {
                None
            }
        }),
        limit: Some(if plan.has_local_filters() {
            1000
        } else {
            limit
        }),
        milestone: None,
    };
    let paged = provider
        .list(args)
        .await
        .map_err(|e| miette::miette!("issue search failed: {e}"))?;
    if paged.truncated && plan.has_local_filters() {
        return Err(miette::miette!(
            "query result exceeds 1000 rows; narrow the filters"
        ));
    }
    let truncated = paged.truncated;
    let mut items: Vec<_> = paged
        .items
        .into_iter()
        .filter(|item| plan.matches_issue(item))
        .collect();
    let filtered_more = items.len() > limit as usize;
    items.truncate(limit as usize);
    super::output::print_output(
        &serde_json::json!({"plan": plan, "platform": platform, "items": items, "truncated": truncated || filtered_more}),
        &crate::OutputFormat::Json,
    )
}

async fn search_prs(
    plan: &Plan,
    platform: &str,
    repo: &str,
    remote_url: &str,
    limit: u32,
) -> miette::Result<()> {
    let provider: Box<dyn PrProvider> = match platform {
        "github" => Box::new(GitHubPrProvider::new(repo)),
        "gitlab" if !remote_url.is_empty() => {
            Box::new(GitLabMrProvider::with_remote_url(repo, remote_url))
        }
        "gitlab" => Box::new(GitLabMrProvider::new(repo)),
        "gitcode" => Box::new(GitCodePrProvider::new(repo)),
        _ => return Err(miette::miette!("unsupported platform")),
    };
    let args = ListPrArgs {
        state: Some(
            plan.filters
                .iter()
                .find_map(|f| {
                    if let Filter::State { value } = f {
                        Some(*value)
                    } else {
                        None
                    }
                })
                .unwrap_or(gitflow_core::types::State::All),
        ),
        limit: Some(if plan.has_local_filters() {
            1000
        } else {
            limit
        }),
    };
    let paged = provider
        .list(args)
        .await
        .map_err(|e| miette::miette!("PR search failed: {e}"))?;
    if paged.truncated && plan.has_local_filters() {
        return Err(miette::miette!(
            "query result exceeds 1000 rows; narrow the filters"
        ));
    }
    let truncated = paged.truncated;
    let mut items: Vec<_> = paged
        .items
        .into_iter()
        .filter(|item| plan.matches_pr(item))
        .collect();
    let filtered_more = items.len() > limit as usize;
    items.truncate(limit as usize);
    super::output::print_output(
        &serde_json::json!({"plan": plan, "platform": platform, "items": items, "truncated": truncated || filtered_more}),
        &crate::OutputFormat::Json,
    )
}

#[cfg(feature = "gitflow-jev")]
async fn live_response(plan: &Plan, candidates: &Candidates) -> Option<DecisionResponse> {
    use gitflow_core::decision::DecisionEngine;
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return None;
    }
    let request = decision_request(plan, candidates).ok()??;
    let engine = gitflow_jev::JevEngine::from_env().ok()?;
    engine.decide(&request).await.ok()
}
#[cfg(not(feature = "gitflow-jev"))]
async fn live_response(_plan: &Plan, _candidates: &Candidates) -> Option<DecisionResponse> {
    None
}

async fn verify_labels(
    plan: &Plan,
    platform: &str,
    repo: &str,
    remote_url: &str,
) -> miette::Result<()> {
    let requested: Vec<&str> = plan
        .filters
        .iter()
        .filter_map(|filter| {
            if let Filter::Label { value, .. } = filter {
                Some(value.as_str())
            } else {
                None
            }
        })
        .collect();
    if requested.is_empty() {
        return Ok(());
    }
    let provider: Box<dyn LabelProvider> = match platform {
        "github" => Box::new(GitHubLabelProvider::new(repo)),
        "gitlab" if !remote_url.is_empty() => {
            Box::new(GitLabLabelProvider::with_remote_url(repo, remote_url))
        }
        "gitlab" => Box::new(GitLabLabelProvider::new(repo)),
        "gitcode" => Box::new(GitCodeLabelProvider::new(repo)),
        _ => return Err(miette::miette!("unsupported platform")),
    };
    let labels = provider
        .list(Some(1000))
        .await
        .map_err(|e| miette::miette!("label verification failed: {e}"))?;
    if labels.truncated {
        return Err(miette::miette!("label list truncated; cannot verify query"));
    }
    if requested.iter().any(|wanted| {
        !labels
            .items
            .iter()
            .any(|label| label.name.eq_ignore_ascii_case(wanted))
    }) {
        return Err(miette::miette!(
            "query refers to a nonexistent repository label"
        ));
    }
    Ok(())
}
