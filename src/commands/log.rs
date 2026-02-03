use crate::error::Result;
use crate::side_repo::SideRepo;

/// Show side repo history.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or log command fails.
pub fn run(args: &[String]) -> Result<()> {
    let repo = SideRepo::open()?;
    let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let output = repo.log(&args_refs)?;
    println!("{output}");
    Ok(())
}
