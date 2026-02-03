use colored::Colorize;

use crate::error::Result;
use crate::side_repo::SideRepo;

/// Manage side repo remotes.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or git commands fail.
pub fn run(args: &[String]) -> Result<()> {
    let repo = SideRepo::open()?;
    repo.ensure_initialized()?;

    if args.is_empty() {
        // List remotes
        let output = repo.git(&["remote", "-v"])?;
        if output.is_empty() {
            println!("{}", "No remotes configured.".yellow());
        } else {
            println!("{output}");
        }
    } else {
        // Pass through to git remote
        let args_refs: Vec<&str> = std::iter::once("remote")
            .chain(args.iter().map(String::as_str))
            .collect();
        let output = repo.git(&args_refs)?;
        if !output.is_empty() {
            println!("{output}");
        }

        // Show success message for add/remove
        if args.first().is_some_and(|a| a == "add") {
            println!("{} Remote added.", "Done.".green().bold());
        } else if args.first().is_some_and(|a| a == "remove" || a == "rm") {
            println!("{} Remote removed.", "Done.".green().bold());
        }
    }

    Ok(())
}
