use clap::{App, Arg, SubCommand};
use tractor::{ContainsError, PostError, ResourceError, TractorStorage};
use std::io;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    ResourceError(ResourceError),
    ImportPathNotFound(PathBuf),
    ContainsFileError(ContainsError),
    ReadImportsError(io::Error),
    CopyFileError(PostError),
}

pub fn main() -> Result<(), Error> {
    let matches = App::new("Tractor command-line interface")
        .version("0.0.1")
        .author("Austin Jones <austinbaysjones@gmail.com>")
        .about("Command-line interface for the Tractor storage toolkit")
        .subcommand(
            SubCommand::with_name("import")
                .about("imports image resources into Tractor storage.  deduplicates imports, so it can be run multiple times")
                .version("0.0.1")
                .arg(
                    Arg::with_name("resource")
                        .long("resource")
                        .short("u")
                        .default_value("import")
                        .help("Specifies the target resource name"),
                )
                .arg(
                    Arg::with_name("paths")
                        .index(1)
                        .multiple(true)
                        .help("Files or directories to scan for input"),
                ),
        )
        .get_matches();

    if let Some(import) = matches.subcommand_matches("import") {
        let resource = import.value_of("resource").unwrap();
        let paths = import.values_of("paths");

        let resource_storage = TractorStorage::new(resource)
            .map_err(|e| Error::ResourceError(e))?;
        for path_str in paths.unwrap() {
            let path = Path::new(path_str);

            println!("Found: {}", path.to_str().unwrap(),);

            if !path.exists() {
                return Err(Error::ImportPathNotFound(path.to_path_buf()));
            }

            if path.is_dir() {
                import_dir(&resource_storage, path)?;
            } else if path.is_file() {
                import_file(&resource_storage, path)?;
            }
        }
    }

    Ok(())
}

fn import_dir(resource_storage: &TractorStorage, path: &Path) -> Result<(), Error> {
    let mut dir_entries = Vec::new();
    for dir_entry in path.read_dir().map_err(|e| Error::ReadImportsError(e))? {
        let entry = dir_entry.map_err(|e| Error::ReadImportsError(e))?;
        dir_entries.push(entry);
    }

    dir_entries.sort_by_cached_key(|path| path.metadata().unwrap().modified().unwrap());

    for entry in dir_entries {
        import_file(resource_storage, entry.path().as_path())?;
    }

    Ok(())
}

fn import_file(resource_storage: &TractorStorage, path: &Path) -> Result<(), Error> {
    if resource_storage
        .contains_file(path)
        .map_err(|e| Error::ContainsFileError(e))?
    {
        println!("Skipping (already imported): {}", path.to_str().unwrap());
        return Ok(());
    }

    if !resource_storage.accepts(path) {
        println!("Skipping (invalid format): {}", path.to_str().unwrap());
        return Ok(());
    }

    let id = Uuid::new_v4();
    let target = resource_storage
        .post_file(id, path)
        .map_err(|e| Error::CopyFileError(e))?;

    println!(
        "Copied: {} to {}",
        path.to_str().unwrap(),
        target.to_str().unwrap()
    );

    Ok(())
}
