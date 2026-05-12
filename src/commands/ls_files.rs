use crate::error::Result;
use crate::side_repo::SideRepo;

/// List files in the side repo's index.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or `git ls-files` fails.
pub fn run(args: &[String]) -> Result<()> {
    let repo = SideRepo::open()?;
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = repo.ls_files(&arg_refs)?;
    if !output.is_empty() {
        println!("{output}");
    }
    Ok(())
}
