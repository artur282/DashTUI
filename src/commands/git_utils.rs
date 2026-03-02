//! Comando `git` — Utilidades para análisis de repositorios Git integradas al Dashboard TUI.
//!
//! Excluye el uso directo de `println!` para mantener la redibujado
//! inmaculada en Ratatui. Emplea Data Objects.

use crate::error::GitError;
use git2::Repository;
use std::collections::HashMap;

/// Estadísticas aglutinadas devueltas al TUI
pub struct GitStats {
    pub total_commits: u64,
    pub contributors: Vec<(String, u64)>, // (name, commit count)
    pub local_branches: usize,
    pub tracked_files: usize,
}

pub struct ChangelogEntry {
    pub commit_type: String,
    pub messages: Vec<String>,
}

/// Abre el repositorio Git en el directorio actual.
fn open_current_repo() -> Result<Repository, GitError> {
    Repository::open(".").map_err(|_| GitError::NoRepository)
}

/// Obtiene estadísticas del repositorio actual sin pintar.
pub fn get_stats() -> Result<GitStats, GitError> {
    let repo = open_current_repo()?;

    // Contar ramas
    let local_branches = repo.branches(Some(git2::BranchType::Local))?.count();

    // Contar archivos rastreados
    let tracked_files = repo
        .index()
        .map(|idx| {
            let mut count = 0;
            for _ in idx.iter() {
                count += 1;
            }
            count
        })
        .unwrap_or(0);

    // Commits e info de authors
    let mut revwalk = repo.revwalk()?;
    if revwalk.push_head().is_err() {
        // No hay commits aún, retornar stats parciales
        return Ok(GitStats {
            total_commits: 0,
            contributors: Vec::new(),
            local_branches,
            tracked_files,
        });
    }

    let mut total_commits = 0u64;
    let mut hash_contributors: HashMap<String, u64> = HashMap::new();

    for oid in revwalk {
        let Ok(oid) = oid else { continue };
        let Ok(commit) = repo.find_commit(oid) else {
            continue;
        };

        total_commits += 1;
        let author = commit.author();
        let name = author.name().unwrap_or("Unknown").to_string();
        *hash_contributors.entry(name).or_insert(0) += 1;
    }

    let mut contributors: Vec<_> = hash_contributors.into_iter().collect();
    contributors.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(GitStats {
        total_commits,
        contributors,
        local_branches,
        tracked_files,
    })
}

/// Extrae los commits convencionales
pub fn get_changelog(limit: usize) -> Result<Vec<ChangelogEntry>, GitError> {
    let repo = open_current_repo()?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    let mut count = 0;

    for oid in revwalk {
        if count >= limit {
            break;
        }

        let Ok(oid) = oid else { continue };
        let Ok(commit) = repo.find_commit(oid) else {
            continue;
        };
        let Some(message) = commit.message() else {
            continue;
        };

        if let Some((commit_type, description)) = parse_conventional_commit(message) {
            categories.entry(commit_type).or_default().push(description);
        }

        count += 1;
    }

    if categories.is_empty() {
        return Err(GitError::NoConventionalCommits);
    }

    let type_order = [
        "feat", "fix", "refactor", "docs", "test", "chore", "style", "perf",
    ];
    let mut results = Vec::new();

    for t in type_order {
        if let Some(messages) = categories.remove(t) {
            results.push(ChangelogEntry {
                commit_type: t.to_string(),
                messages,
            });
        }
    }

    // El resto
    for (t, messages) in categories {
        results.push(ChangelogEntry {
            commit_type: t,
            messages,
        });
    }

    Ok(results)
}

/// Identifica ramas sin usar
pub fn get_merged_branches() -> Result<Vec<String>, GitError> {
    let repo = open_current_repo()?;
    let head = repo.head()?;
    let head_name = head.shorthand().unwrap_or("main");

    let branches = repo.branches(Some(git2::BranchType::Local))?;
    let mut merged = Vec::new();

    for branch_result in branches {
        let Ok((branch, _)) = branch_result else {
            continue;
        };
        let Some(branch_name) = branch.name()? else {
            continue;
        };

        if ["main", "master", "develop"].contains(&branch_name) || branch_name == head_name {
            continue;
        }

        if branch.is_head() {
            continue;
        }

        if let Ok(upstream) = branch.upstream() {
            if upstream.is_head() {
                merged.push(branch_name.to_string());
            }
        } else {
            merged.push(branch_name.to_string());
        }
    }

    Ok(merged)
}

/// Destruye las ramas dadas que ya sabemos están mergeadas
pub fn delete_branches(branches: &[String]) -> Result<usize, GitError> {
    let repo = open_current_repo()?;
    let mut deleted = 0;

    for bn in branches {
        if let Ok(mut branch) = repo.find_branch(bn, git2::BranchType::Local) {
            if branch.delete().is_ok() {
                deleted += 1;
            }
        }
    }

    Ok(deleted)
}

/// Formatear string a un label leible
pub fn format_type_label(commit_type: &str) -> String {
    match commit_type {
        "feat" => "Features".to_string(),
        "fix" => "Bug Fixes".to_string(),
        "refactor" => "Refactoring".to_string(),
        "docs" => "Documentation".to_string(),
        "test" => "Tests".to_string(),
        "chore" => "Chores".to_string(),
        "style" => "Style".to_string(),
        "perf" => "Performance".to_string(),
        other => other.to_string(),
    }
}

/// Obtiene el emoji dependiend del scope
pub fn get_type_emoji(commit_type: &str) -> &'static str {
    match commit_type {
        "feat" => "✨",
        "fix" => "🐛",
        "refactor" => "♻️",
        "docs" => "📚",
        "test" => "🧪",
        "chore" => "🔧",
        "style" => "💄",
        "perf" => "⚡",
        _ => "📌",
    }
}

fn parse_conventional_commit(message: &str) -> Option<(String, String)> {
    let first_line = message.lines().next()?;
    let colon_pos = first_line.find(':')?;
    let type_part = &first_line[..colon_pos];

    let commit_type = if let Some(paren_pos) = type_part.find('(') {
        &type_part[..paren_pos]
    } else {
        type_part
    };

    if commit_type.is_empty() || !commit_type.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }

    let description = first_line[colon_pos + 1..].trim().to_string();
    if description.is_empty() {
        return None;
    }

    Some((commit_type.to_string(), description))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_conventional_commit_should_extract_feat() {
        let result = parse_conventional_commit("feat: add new feature");
        assert_eq!(
            result,
            Some(("feat".to_string(), "add new feature".to_string()))
        );
    }

    #[test]
    fn format_type_label_should_return_readable_names() {
        assert_eq!(format_type_label("feat"), "Features");
        assert_eq!(format_type_label("fix"), "Bug Fixes");
    }
}
