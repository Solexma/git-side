use crate::error::Result;
use crate::side_repo::SideRepo;

/// Show side repo status.
///
/// # Errors
///
/// Returns an error if the side repo cannot be opened or status command fails.
pub fn run() -> Result<()> {
    let repo = SideRepo::open()?;
    let output = repo.status()?;
    println!("{output}");
    Ok(())
}
