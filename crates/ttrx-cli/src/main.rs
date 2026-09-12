#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};

use ttrx_core::{ContainerInfo, SourceKind, SourceSummary, Verification};

const HELP: &str = "\
TTRX replay converter

Usage:
  ttrx encode <input.ttr|input.ttrm> [-o <output.ttrx>] [--force]
  ttrx decode <input.ttrx> [-o <output.ttr|output.ttrm>] [--force]
  ttrx inspect <input.ttr|input.ttrm|input.ttrx>
  ttrx verify <input.ttr|input.ttrm|input.ttrx>
  ttrx help
  ttrx version

Options:
  -o, --output <path>  Select the output path.
  --force              Replace an existing output file.
  -h, --help           Show this help.
  -V, --version        Show the program version.
";

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const MAX_SOURCE_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CONTAINER_INPUT_BYTES: usize = 512 * 1024 * 1024 + 72;

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Encode(ConvertArgs),
    Decode(ConvertArgs),
    Inspect(PathBuf),
    Verify(PathBuf),
    Help,
    Version,
}

#[derive(Debug, PartialEq, Eq)]
struct ConvertArgs {
    input: PathBuf,
    output: Option<PathBuf>,
    force: bool,
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Failure(String),
}

impl CliError {
    fn failure(message: impl Into<String>) -> Self {
        Self::Failure(message.into())
    }
}

fn main() -> ExitCode {
    match parse_args(std::env::args_os().skip(1)).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage(message)) => {
            eprintln!("error: {message}\n\nTry 'ttrx help' for usage.");
            ExitCode::from(2)
        }
        Err(CliError::Failure(message)) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args<I>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(Command::Help);
    };
    let command = command
        .to_str()
        .ok_or_else(|| CliError::Usage("the command name is not valid UTF-8".into()))?;
    let remaining: Vec<OsString> = args.collect();

    match command {
        "encode" => parse_convert_args(&remaining).map(Command::Encode),
        "decode" => parse_convert_args(&remaining).map(Command::Decode),
        "inspect" => parse_input_arg(&remaining, "inspect").map(Command::Inspect),
        "verify" => parse_input_arg(&remaining, "verify").map(Command::Verify),
        "help" | "-h" | "--help" => no_extra_args(&remaining, Command::Help),
        "version" | "-V" | "--version" => no_extra_args(&remaining, Command::Version),
        _ => Err(CliError::Usage(format!("unknown command '{command}'"))),
    }
}

fn no_extra_args(remaining: &[OsString], command: Command) -> Result<Command, CliError> {
    if remaining.is_empty() {
        Ok(command)
    } else {
        Err(CliError::Usage(
            "this command does not accept arguments".into(),
        ))
    }
}

fn parse_convert_args(args: &[OsString]) -> Result<ConvertArgs, CliError> {
    let mut input = None;
    let mut output = None;
    let mut force = false;
    let mut positional_only = false;
    let mut index = 0;

    while index < args.len() {
        let value = &args[index];
        if !positional_only && value == "--" {
            positional_only = true;
        } else if !positional_only && (value == "-o" || value == "--output") {
            if output.is_some() {
                return Err(CliError::Usage(
                    "the output path was provided more than once".into(),
                ));
            }
            index += 1;
            let path = args
                .get(index)
                .ok_or_else(|| CliError::Usage("-o/--output requires a path".into()))?;
            output = Some(PathBuf::from(path));
        } else if !positional_only && value == "--force" {
            if force {
                return Err(CliError::Usage(
                    "--force was provided more than once".into(),
                ));
            }
            force = true;
        } else if !positional_only && is_option(value) {
            return Err(CliError::Usage(format!(
                "unknown option '{}'",
                value.to_string_lossy()
            )));
        } else if input.replace(PathBuf::from(value)).is_some() {
            return Err(CliError::Usage(
                "more than one input path was provided".into(),
            ));
        }
        index += 1;
    }

    let input = input.ok_or_else(|| CliError::Usage("an input path is required".into()))?;
    Ok(ConvertArgs {
        input,
        output,
        force,
    })
}

fn parse_input_arg(args: &[OsString], command: &str) -> Result<PathBuf, CliError> {
    match args {
        [input] if !is_option(input) => Ok(PathBuf::from(input)),
        [separator, input] if separator == "--" => Ok(PathBuf::from(input)),
        [] => Err(CliError::Usage(format!(
            "the {command} command requires an input path"
        ))),
        _ => Err(CliError::Usage(format!(
            "the {command} command accepts exactly one input path"
        ))),
    }
}

fn is_option(value: &OsStr) -> bool {
    value.to_str().is_some_and(|value| value.starts_with('-'))
}

fn run(command: Command) -> Result<(), CliError> {
    match command {
        Command::Encode(args) => encode_file(&args),
        Command::Decode(args) => decode_file(&args),
        Command::Inspect(input) => inspect_file(&input),
        Command::Verify(input) => verify_file(&input),
        Command::Help => {
            print!("{HELP}");
            Ok(())
        }
        Command::Version => {
            println!("ttrx {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}

fn encode_file(args: &ConvertArgs) -> Result<(), CliError> {
    let source = read_file(&args.input, MAX_SOURCE_INPUT_BYTES)?;
    let hint = source_hint(&args.input);
    let encoded = ttrx_core::encode(&source, hint).map_err(|error| {
        CliError::failure(format!("cannot encode '{}': {error}", args.input.display()))
    })?;
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.with_extension("ttrx"));
    write_file(&output, &encoded, args.force)?;
    println!(
        "encoded {} -> {} ({} bytes)",
        args.input.display(),
        output.display(),
        encoded.len()
    );
    Ok(())
}

fn decode_file(args: &ConvertArgs) -> Result<(), CliError> {
    let container = read_file(&args.input, MAX_CONTAINER_INPUT_BYTES)?;
    let (source, info) = ttrx_core::decode(&container).map_err(|error| {
        CliError::failure(format!("cannot decode '{}': {error}", args.input.display()))
    })?;
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| default_decoded_output(&args.input, info.source_kind));
    write_file(&output, &source, args.force)?;
    println!(
        "decoded {} -> {} ({} bytes, {})",
        args.input.display(),
        output.display(),
        source.len(),
        info.source_kind
    );
    Ok(())
}

fn inspect_file(input: &Path) -> Result<(), CliError> {
    let container_input = sniff_ttrx(input)?;
    let limit = if container_input {
        MAX_CONTAINER_INPUT_BYTES
    } else {
        MAX_SOURCE_INPUT_BYTES
    };
    let bytes = read_file(input, limit)?;
    if container_input {
        let (source, info) = ttrx_core::decode(&bytes).map_err(|error| {
            CliError::failure(format!("cannot inspect '{}': {error}", input.display()))
        })?;
        let summary = ttrx_core::inspect_source(&source, info.source_kind).map_err(|error| {
            CliError::failure(format!("cannot inspect decoded replay: {error}"))
        })?;
        println!(
            "{}\n\n{}",
            format_container(&info),
            format_summary(&summary)
        );
    } else {
        let summary = ttrx_core::inspect_source(&bytes, source_hint(input)).map_err(|error| {
            CliError::failure(format!("cannot inspect '{}': {error}", input.display()))
        })?;
        println!("{}", format_summary(&summary));
    }
    Ok(())
}

fn verify_file(input: &Path) -> Result<(), CliError> {
    let container_input = sniff_ttrx(input)?;
    let limit = if container_input {
        MAX_CONTAINER_INPUT_BYTES
    } else {
        MAX_SOURCE_INPUT_BYTES
    };
    let bytes = read_file(input, limit)?;
    if container_input {
        let (source, info) = ttrx_core::decode(&bytes).map_err(|error| {
            CliError::failure(format!("cannot verify '{}': {error}", input.display()))
        })?;
        let verification = ttrx_core::verify(&source, info.source_kind)
            .map_err(|error| CliError::failure(format!("cannot verify decoded replay: {error}")))?;
        println!(
            "{}\n\n{}",
            format_container(&info),
            format_verification(&verification)
        );
    } else {
        let verification = ttrx_core::verify(&bytes, source_hint(input)).map_err(|error| {
            CliError::failure(format!("cannot verify '{}': {error}", input.display()))
        })?;
        println!("{}", format_verification(&verification));
    }
    Ok(())
}

fn read_file(path: &Path, limit: usize) -> Result<Vec<u8>, CliError> {
    let file = File::open(path)
        .map_err(|error| CliError::failure(format!("cannot read '{}': {error}", path.display())))?;
    let declared_len = file
        .metadata()
        .map_err(|error| {
            CliError::failure(format!(
                "cannot inspect input '{}': {error}",
                path.display()
            ))
        })?
        .len();
    let limit_u64 = u64::try_from(limit)
        .map_err(|_| CliError::failure("input byte limit does not fit this platform"))?;
    if declared_len > limit_u64 {
        return Err(CliError::failure(format!(
            "input '{}' exceeds the {limit}-byte limit",
            path.display()
        )));
    }

    let capacity = usize::try_from(declared_len)
        .map_err(|_| CliError::failure("input length does not fit this platform"))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.take(limit_u64.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| CliError::failure(format!("cannot read '{}': {error}", path.display())))?;
    if bytes.len() > limit {
        return Err(CliError::failure(format!(
            "input '{}' grew beyond the {limit}-byte limit while being read",
            path.display()
        )));
    }
    Ok(bytes)
}

fn sniff_ttrx(path: &Path) -> Result<bool, CliError> {
    let mut file = File::open(path)
        .map_err(|error| CliError::failure(format!("cannot read '{}': {error}", path.display())))?;
    let mut prefix = [0_u8; 4];
    let read = file
        .read(&mut prefix)
        .map_err(|error| CliError::failure(format!("cannot read '{}': {error}", path.display())))?;
    Ok(is_ttrx(path, &prefix[..read]))
}

fn source_hint(path: &Path) -> SourceKind {
    match path.extension().and_then(OsStr::to_str) {
        Some(extension) if extension.eq_ignore_ascii_case("ttr") => SourceKind::Ttr,
        Some(extension) if extension.eq_ignore_ascii_case("ttrm") => SourceKind::Ttrm,
        _ => SourceKind::Unknown,
    }
}

fn is_ttrx(path: &Path, bytes: &[u8]) -> bool {
    bytes.starts_with(b"TTRX")
        || path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("ttrx"))
}

fn default_decoded_output(input: &Path, kind: SourceKind) -> PathBuf {
    input.with_extension(kind.extension().unwrap_or("json"))
}

fn write_file(path: &Path, bytes: &[u8], force: bool) -> Result<(), CliError> {
    validate_output_target(path, force)?;

    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| CliError::failure(format!("'{}' is not a file path", path.display())))?;
    let (temporary, mut file) = create_temporary(parent, file_name)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(&temporary);
        return Err(CliError::failure(format!(
            "cannot write temporary output '{}': {error}",
            temporary.display()
        )));
    }
    // Windows does not permit renaming this file while our handle is open.
    drop(file);

    if let Err(error) = commit_temporary(&temporary, path, force) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(())
}

fn create_temporary(parent: &Path, file_name: &OsStr) -> Result<(PathBuf, File), CliError> {
    for _ in 0..128 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = OsString::from(".");
        temporary_name.push(file_name);
        temporary_name.push(format!(".{}.{}.tmp", std::process::id(), sequence));
        let temporary = parent.join(temporary_name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => return Ok((temporary, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => {
                return Err(CliError::failure(format!(
                    "cannot create a temporary file beside '{}': {error}",
                    parent.display()
                )));
            }
        }
    }
    Err(CliError::failure(format!(
        "cannot reserve a unique temporary file beside '{}'",
        parent.display()
    )))
}

fn commit_temporary(temporary: &Path, output: &Path, force: bool) -> Result<(), CliError> {
    if validate_output_target(output, force)? {
        return replace_existing_output(temporary, output);
    }

    commit_new_output(temporary, output)
}

#[cfg(not(windows))]
fn replace_existing_output(temporary: &Path, output: &Path) -> Result<(), CliError> {
    // POSIX rename replaces the existing regular file atomically.
    fs::rename(temporary, output).map_err(|error| {
        CliError::failure(format!(
            "cannot replace output '{}': {error}",
            output.display()
        ))
    })
}

#[cfg(windows)]
fn replace_existing_output(temporary: &Path, output: &Path) -> Result<(), CliError> {
    // `std::fs::rename` cannot replace an existing file on Windows. Move the
    // previous output aside first so an ordinary commit failure can restore it.
    let parent = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = output
        .file_name()
        .ok_or_else(|| CliError::failure(format!("'{}' is not a file path", output.display())))?;
    let backup = unique_sibling_path(parent, file_name, "previous")?;
    fs::rename(output, &backup).map_err(|error| {
        CliError::failure(format!(
            "cannot prepare '{}' for replacement: {error}",
            output.display()
        ))
    })?;
    if let Err(error) = fs::rename(temporary, output) {
        let restore = fs::rename(&backup, output);
        return Err(CliError::failure(match restore {
            Ok(()) => format!(
                "cannot move temporary output into '{}': {error}; the previous file was restored",
                output.display()
            ),
            Err(restore_error) => format!(
                "cannot move temporary output into '{}': {error}; the previous file remains at '{}' because restoration failed: {restore_error}",
                output.display(),
                backup.display()
            ),
        }));
    }
    fs::remove_file(&backup).map_err(|error| {
        CliError::failure(format!(
            "output '{}' was replaced, but the previous file could not be removed from '{}': {error}",
            output.display(),
            backup.display()
        ))
    })
}

fn validate_output_target(output: &Path, force: bool) -> Result<bool, CliError> {
    let metadata = match fs::symlink_metadata(output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(CliError::failure(format!(
                "cannot inspect output '{}': {error}",
                output.display()
            )));
        }
    };

    if !force {
        return Err(CliError::failure(format!(
            "output '{}' already exists; pass --force to replace it",
            output.display()
        )));
    }
    if !metadata.file_type().is_file() {
        return Err(CliError::failure(format!(
            "refusing to replace non-regular output '{}'",
            output.display()
        )));
    }
    Ok(true)
}

#[cfg(windows)]
fn commit_new_output(temporary: &Path, output: &Path) -> Result<(), CliError> {
    // Windows rename fails if another process creates `output` after the
    // symlink-metadata check, preserving the no-overwrite contract.
    fs::rename(temporary, output).map_err(|error| {
        CliError::failure(format!(
            "cannot move temporary output into '{}': {error}",
            output.display()
        ))
    })
}

#[cfg(not(windows))]
fn commit_new_output(temporary: &Path, output: &Path) -> Result<(), CliError> {
    // A hard link is an atomic create-if-absent operation on the same
    // filesystem. The temporary file is always created beside the output.
    fs::hard_link(temporary, output).map_err(|error| {
        CliError::failure(format!(
            "cannot commit new output '{}': {error}",
            output.display()
        ))
    })?;
    fs::remove_file(temporary).map_err(|error| {
        CliError::failure(format!(
            "output '{}' was committed, but its temporary link could not be removed: {error}",
            output.display()
        ))
    })
}

#[cfg(windows)]
fn unique_sibling_path(parent: &Path, file_name: &OsStr, role: &str) -> Result<PathBuf, CliError> {
    for _ in 0..128 {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let mut name = OsString::from(".");
        name.push(file_name);
        name.push(format!(".{}.{}.{}.tmp", std::process::id(), sequence, role));
        let path = parent.join(name);
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(CliError::failure(format!(
        "cannot reserve a unique {role} path beside '{}'",
        parent.display()
    )))
}

fn format_container(info: &ContainerInfo) -> String {
    format!(
        "TTRX format: {}.{}\nProfile: {}\nSource kind: {}\nOriginal JSON bytes: {}\nPayload bytes: {}\nPayload CRC32C: {:08x}\nDictionary entries: {}\nShape entries: {}\nEncoded values: {}\nMaximum depth: {}",
        info.format_major,
        info.format_minor,
        info.profile,
        info.source_kind,
        info.original_json_len,
        info.payload_len,
        info.payload_crc32c,
        info.dictionary_entries,
        info.shape_entries,
        info.value_count,
        info.max_depth
    )
}

fn format_summary(summary: &SourceSummary) -> String {
    let container_version = summary.container_version.as_deref().unwrap_or("<missing>");
    let rules_versions = if summary.rules_versions.is_empty() {
        "<missing>".to_owned()
    } else {
        summary.rules_versions.join(", ")
    };
    let event_types = if summary.event_types.is_empty() {
        "<none>".to_owned()
    } else {
        summary
            .event_types
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let option_keys = if summary.option_keys.is_empty() {
        "<none>".to_owned()
    } else {
        summary.option_keys.join(", ")
    };
    format!(
        "Source kind: {}\nContainer version: {container_version}\nRules versions: {rules_versions}\nReplay streams: {}\nEvents: {}\nKey events: {}\nIGE events: {}\nEvent types: {event_types}\nOption keys ({}): {option_keys}",
        summary.kind,
        summary.stream_count,
        summary.event_count,
        summary.key_event_count(),
        summary.ige_event_count(),
        summary.option_keys.len()
    )
}

fn format_verification(verification: &Verification) -> String {
    format!(
        "Verification: passed\nSource kind: {}\nReplay streams: {}\nEvents: {}\nInput JSON bytes: {}\nTTRX bytes: {}\nCanonical JSON bytes: {}\nExact JSON data model: {}",
        verification.summary.kind,
        verification.summary.stream_count,
        verification.summary.event_count,
        verification.input_bytes,
        verification.encoded_bytes,
        verification.canonical_json_bytes,
        verification.exact_data_model
    )
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::Ordering;

    use super::{
        Command, ConvertArgs, TEMP_SEQUENCE, default_decoded_output, is_ttrx, parse_args,
        read_file, source_hint, write_file,
    };
    use ttrx_core::SourceKind;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_encode_options_in_any_order() {
        assert_eq!(
            parse_args(args(&["encode", "--force", "-o", "out.ttrx", "in.ttr"]))
                .expect("valid arguments"),
            Command::Encode(ConvertArgs {
                input: PathBuf::from("in.ttr"),
                output: Some(PathBuf::from("out.ttrx")),
                force: true,
            })
        );
    }

    #[test]
    fn parses_dash_prefixed_input_after_separator() {
        assert_eq!(
            parse_args(args(&["decode", "--", "-archive.ttrx"])).expect("valid arguments"),
            Command::Decode(ConvertArgs {
                input: PathBuf::from("-archive.ttrx"),
                output: None,
                force: false,
            })
        );
    }

    #[test]
    fn rejects_implicit_overwrite_options_without_values() {
        assert!(parse_args(args(&["encode", "input.ttr", "-o"])).is_err());
        assert!(parse_args(args(&["decode", "input.ttrx", "--force", "--force"])).is_err());
    }

    #[test]
    fn infers_source_kind_case_insensitively() {
        assert_eq!(source_hint(Path::new("one.TTR")), SourceKind::Ttr);
        assert_eq!(source_hint(Path::new("many.tTrM")), SourceKind::Ttrm);
        assert_eq!(source_hint(Path::new("replay.json")), SourceKind::Unknown);
    }

    #[test]
    fn selects_decoded_extension_from_container_kind() {
        assert_eq!(
            default_decoded_output(Path::new("one.ttrx"), SourceKind::Ttr),
            PathBuf::from("one.ttr")
        );
        assert_eq!(
            default_decoded_output(Path::new("many.ttrx"), SourceKind::Ttrm),
            PathBuf::from("many.ttrm")
        );
    }

    #[test]
    fn detects_container_magic_even_when_extension_changed() {
        assert!(is_ttrx(Path::new("archive.bin"), b"TTRX\x01\x00"));
        assert!(is_ttrx(Path::new("archive.ttrx"), b"not a container"));
        assert!(!is_ttrx(Path::new("archive.ttr"), b"{\"version\":1}"));
    }

    #[test]
    fn output_requires_force_and_replaces_via_sibling_temporary_file() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "ttrx-cli-write-test-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&directory).expect("create isolated test directory");
        let output = directory.join("output.ttrx");
        fs::write(&output, b"old").expect("create existing output");

        let refused = write_file(&output, b"new", false);
        let after_refusal = fs::read(&output).expect("read preserved output");
        let replaced = write_file(&output, b"new", true);
        let after_replace = fs::read(&output).expect("read replacement output");
        fs::remove_dir_all(&directory).expect("remove isolated test directory");

        assert!(refused.is_err());
        assert_eq!(after_refusal, b"old");
        assert!(replaced.is_ok());
        assert_eq!(after_replace, b"new");
    }

    #[test]
    fn force_refuses_to_replace_a_directory() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir().join(format!(
            "ttrx-cli-directory-test-{}-{sequence}",
            std::process::id()
        ));
        let output = directory.join("existing-directory");
        fs::create_dir_all(&output).expect("create output directory");

        let refused = write_file(&output, b"new", true);
        let output_is_still_a_directory = output.is_dir();
        fs::remove_dir_all(&directory).expect("remove isolated test directory");

        assert!(refused.is_err());
        assert!(output_is_still_a_directory);
    }

    #[test]
    fn input_read_is_bounded_before_returning_bytes() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ttrx-cli-read-test-{}-{sequence}",
            std::process::id()
        ));
        fs::write(&path, b"four").expect("write bounded input fixture");
        let result = read_file(&path, 3);
        fs::remove_file(&path).expect("remove bounded input fixture");
        assert!(result.is_err());
    }
}
