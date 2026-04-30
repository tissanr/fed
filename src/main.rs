use clap::{ArgGroup, Parser};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process;

#[derive(Debug, Parser)]
#[command(
    name = "fed",
    about = "Open files in their default application",
    long_about = "Open one or more files using the OS default application, or list applications that can open a file.\n\nExamples:\n  fed photo.jpg\n  fed report.pdf notes.txt\n  fed .\n  fed --options report.pdf",
    override_usage = "fed [OPTIONS] [FILE]...",
    version,
    group(
        ArgGroup::new("mode")
            .args(["files", "options"])
            .required(true)
    )
)]
struct Cli {
    /// One or more files (or directories) to open
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// List applications that can open FILE
    #[arg(
        short = 'o',
        long = "options",
        value_name = "FILE",
        conflicts_with = "files"
    )]
    options: Option<PathBuf>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct OpenOption {
    name: String,
    id: Option<String>,
    path: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(file) = cli.options {
        if !file.exists() {
            eprintln!("fed: '{}': no such file or directory", file.display());
            process::exit(1);
        }

        match discover_open_options(&file) {
            Ok(options) => {
                for option in sorted_unique_options(options) {
                    println!("{}", option.name);
                }
            }
            Err(e) => {
                eprintln!("fed: {}", e);
                process::exit(1);
            }
        }

        return;
    }

    let mut had_error = false;

    for file in &cli.files {
        if !file.exists() {
            eprintln!("fed: '{}': no such file or directory", file.display());
            had_error = true;
            continue;
        }

        if let Err(e) = open::that(file) {
            eprintln!("fed: could not open '{}': {}", file.display(), e);
            had_error = true;
        }
    }

    if had_error {
        process::exit(1);
    }
}

fn sorted_unique_options(options: Vec<OpenOption>) -> Vec<OpenOption> {
    let mut by_identity = BTreeMap::new();

    for option in options.into_iter().filter(|option| !option.name.is_empty()) {
        let key = (
            option.name.to_lowercase(),
            option.id.clone().unwrap_or_default().to_lowercase(),
            option
                .path
                .as_ref()
                .map(|path| path.to_string_lossy().to_lowercase())
                .unwrap_or_default(),
        );
        by_identity.entry(key).or_insert(option);
    }

    by_identity.into_values().collect()
}

#[cfg(target_os = "macos")]
fn discover_open_options(file: &std::path::Path) -> Result<Vec<OpenOption>, String> {
    macos::discover_open_options(file)
}

#[cfg(target_os = "linux")]
fn discover_open_options(file: &std::path::Path) -> Result<Vec<OpenOption>, String> {
    linux::discover_open_options(file)
}

#[cfg(target_os = "windows")]
fn discover_open_options(file: &std::path::Path) -> Result<Vec<OpenOption>, String> {
    windows::discover_open_options(file)
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn discover_open_options(_file: &std::path::Path) -> Result<Vec<OpenOption>, String> {
    Err("application discovery is not supported on this platform".to_string())
}

#[cfg(target_os = "macos")]
mod macos {
    use super::OpenOption;
    use std::ffi::{c_char, c_long, c_void, CStr, CString};
    use std::path::{Path, PathBuf};
    use std::ptr;

    type Boolean = u8;
    type CFIndex = c_long;
    type CFAllocatorRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFTypeRef = *const c_void;
    type CFURLRef = *const c_void;
    type OSStatus = i32;
    type LSRolesMask = u32;

    const K_CFSTRING_ENCODING_UTF8: u32 = 0x0800_0100;
    const K_CFURL_POSIX_PATH_STYLE: CFIndex = 0;
    const K_LS_ROLES_ALL: LSRolesMask = 0xFFFF_FFFF;

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFArrayGetCount(theArray: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(theArray: CFArrayRef, idx: CFIndex) -> *const c_void;
        fn CFRelease(cf: CFTypeRef);
        fn CFStringGetCString(
            theString: CFStringRef,
            buffer: *mut c_char,
            bufferSize: CFIndex,
            encoding: u32,
        ) -> Boolean;
        fn CFURLCopyFileSystemPath(anURL: CFURLRef, pathStyle: CFIndex) -> CFStringRef;
        fn CFURLCreateFromFileSystemRepresentation(
            allocator: CFAllocatorRef,
            buffer: *const u8,
            bufLen: CFIndex,
            isDirectory: Boolean,
        ) -> CFURLRef;
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn LSCopyApplicationURLsForURL(
            inURL: CFURLRef,
            inRoleMask: LSRolesMask,
            outAppURLs: *mut CFArrayRef,
        ) -> OSStatus;
    }

    pub(super) fn discover_open_options(file: &Path) -> Result<Vec<OpenOption>, String> {
        let query_path = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        let file_url = cf_url_from_path(&query_path, file.is_dir())?;
        let mut app_urls: CFArrayRef = ptr::null();

        let status =
            unsafe { LSCopyApplicationURLsForURL(file_url, K_LS_ROLES_ALL, &mut app_urls) };
        unsafe { CFRelease(file_url as CFTypeRef) };

        if status != 0 {
            return Err(format!(
                "could not query Launch Services for '{}': status {}",
                file.display(),
                status
            ));
        }

        if app_urls.is_null() {
            return Ok(Vec::new());
        }

        let count = unsafe { CFArrayGetCount(app_urls) };
        let mut options = Vec::new();

        for index in 0..count {
            let value = unsafe { CFArrayGetValueAtIndex(app_urls, index) as CFURLRef };
            if value.is_null() {
                continue;
            }

            if let Some(path) = path_from_cf_url(value) {
                let name = display_name_from_app_path(&path);
                options.push(OpenOption {
                    name,
                    id: None,
                    path: Some(path),
                });
            }
        }

        unsafe { CFRelease(app_urls as CFTypeRef) };
        Ok(options)
    }

    fn cf_url_from_path(path: &Path, is_directory: bool) -> Result<CFURLRef, String> {
        let path_string = path.to_string_lossy();
        let c_path = CString::new(path_string.as_bytes())
            .map_err(|_| format!("path contains an interior NUL byte: '{}'", path.display()))?;
        let url = unsafe {
            CFURLCreateFromFileSystemRepresentation(
                ptr::null(),
                c_path.as_ptr() as *const u8,
                c_path.as_bytes().len() as CFIndex,
                is_directory as Boolean,
            )
        };

        if url.is_null() {
            Err(format!(
                "could not create file URL for '{}'",
                path.display()
            ))
        } else {
            Ok(url)
        }
    }

    fn path_from_cf_url(url: CFURLRef) -> Option<PathBuf> {
        let cf_path = unsafe { CFURLCopyFileSystemPath(url, K_CFURL_POSIX_PATH_STYLE) };
        if cf_path.is_null() {
            return None;
        }

        let path = string_from_cf_string(cf_path);
        unsafe { CFRelease(cf_path as CFTypeRef) };
        path.map(PathBuf::from)
    }

    fn string_from_cf_string(value: CFStringRef) -> Option<String> {
        let mut buffer = vec![0; 4096];
        let ok = unsafe {
            CFStringGetCString(
                value,
                buffer.as_mut_ptr(),
                buffer.len() as CFIndex,
                K_CFSTRING_ENCODING_UTF8,
            )
        };

        if ok == 0 {
            return None;
        }

        unsafe { CStr::from_ptr(buffer.as_ptr()) }
            .to_str()
            .ok()
            .map(ToOwned::to_owned)
    }

    fn display_name_from_app_path(path: &Path) -> String {
        if let Some(name) = path
            .file_stem()
            .and_then(|name| name.to_str())
            .or_else(|| path.file_name().and_then(|name| name.to_str()))
        {
            name.to_string()
        } else {
            path.as_os_str().to_string_lossy().into_owned()
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::OpenOption;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    pub(super) fn discover_open_options(file: &Path) -> Result<Vec<OpenOption>, String> {
        let mime_type = xdg_mime(&["query", "filetype"], Some(file))?;
        if mime_type.is_empty() {
            return Ok(Vec::new());
        }

        let default = xdg_mime(&["query", "default", &mime_type], None).ok();
        let mut options = Vec::new();

        for desktop_file in application_desktop_files() {
            let Ok(contents) = fs::read_to_string(&desktop_file) else {
                continue;
            };

            let Some(entry) = parse_desktop_entry(&contents) else {
                continue;
            };

            let is_default = default.as_deref().is_some_and(|default| {
                desktop_file.file_name().is_some_and(|name| name == default)
            });

            if is_default
                || entry
                    .mime_types
                    .iter()
                    .any(|entry_mime| entry_mime == &mime_type)
            {
                options.push(OpenOption {
                    name: entry.name,
                    id: desktop_file
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned()),
                    path: Some(desktop_file),
                });
            }
        }

        Ok(options)
    }

    fn xdg_mime(args: &[&str], path: Option<&Path>) -> Result<String, String> {
        let mut command = Command::new("xdg-mime");
        command.args(args);
        if let Some(path) = path {
            command.arg(path);
        }

        let output = command
            .output()
            .map_err(|e| format!("could not run xdg-mime: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = stderr.trim();
            return if detail.is_empty() {
                Err("xdg-mime failed".to_string())
            } else {
                Err(format!("xdg-mime failed: {}", detail))
            };
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn application_desktop_files() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = env::var_os("XDG_DATA_HOME") {
            dirs.push(PathBuf::from(home).join("applications"));
        } else if let Some(home) = env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join(".local/share/applications"));
        }

        if let Some(data_dirs) = env::var_os("XDG_DATA_DIRS") {
            dirs.extend(env::split_paths(&data_dirs).map(|path| path.join("applications")));
        } else {
            dirs.push(PathBuf::from("/usr/local/share/applications"));
            dirs.push(PathBuf::from("/usr/share/applications"));
        }

        let mut files = Vec::new();
        for dir in dirs {
            collect_desktop_files(&dir, &mut files);
        }
        files
    }

    fn collect_desktop_files(dir: &Path, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_desktop_files(&path, files);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "desktop")
            {
                files.push(path);
            }
        }
    }

    struct DesktopEntry {
        name: String,
        mime_types: Vec<String>,
    }

    fn parse_desktop_entry(contents: &str) -> Option<DesktopEntry> {
        let mut in_desktop_entry = false;
        let mut name = None;
        let mut localized_name = None;
        let mut mime_types = Vec::new();

        for line in contents.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            if key == "NoDisplay" && value.eq_ignore_ascii_case("true") {
                return None;
            }

            if key == "Name" {
                name = Some(value.to_string());
            } else if key.starts_with("Name[") && localized_name.is_none() {
                localized_name = Some(value.to_string());
            } else if key == "MimeType" {
                mime_types.extend(
                    value
                        .split(';')
                        .filter(|mime| !mime.is_empty())
                        .map(ToOwned::to_owned),
                );
            }
        }

        localized_name
            .or(name)
            .map(|name| DesktopEntry { name, mime_types })
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::OpenOption;
    use std::path::Path;
    use std::process::Command;

    pub(super) fn discover_open_options(file: &Path) -> Result<Vec<OpenOption>, String> {
        let Some(extension) = file.extension().and_then(|extension| extension.to_str()) else {
            return Ok(Vec::new());
        };
        let extension = format!(".{}", extension);

        let assoc = cmd_output(&["/C", "assoc", &extension])?;
        let Some((_, prog_id)) = assoc.trim().split_once('=') else {
            return Ok(Vec::new());
        };

        let ftype = cmd_output(&["/C", "ftype", prog_id]).unwrap_or_default();
        let name = ftype
            .split_once('=')
            .map(|(_, command)| command)
            .and_then(executable_name)
            .unwrap_or(prog_id)
            .to_string();

        Ok(vec![OpenOption {
            name,
            id: Some(prog_id.to_string()),
            path: None,
        }])
    }

    fn cmd_output(args: &[&str]) -> Result<String, String> {
        let output = Command::new("cmd")
            .args(args)
            .output()
            .map_err(|e| format!("could not query Windows file associations: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok(String::new())
        }
    }

    fn executable_name(command: &str) -> Option<&str> {
        let trimmed = command.trim();
        if let Some(rest) = trimmed.strip_prefix('"') {
            return rest
                .split_once('"')
                .and_then(|(path, _)| Path::new(path).file_stem())
                .and_then(|name| name.to_str());
        }

        trimmed
            .split_whitespace()
            .next()
            .and_then(|path| Path::new(path).file_stem())
            .and_then(|name| name.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn help_includes_options_flag() {
        let mut help = Vec::new();
        Cli::command().write_long_help(&mut help).unwrap();
        let help = String::from_utf8(help).unwrap();

        assert!(help.contains("-o, --options <FILE>"));
    }

    #[test]
    fn options_and_positionals_conflict() {
        let err = Cli::try_parse_from(["fed", "--options", "report.pdf", "notes.txt"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn no_arguments_is_rejected() {
        let err = Cli::try_parse_from(["fed"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn sorted_unique_options_are_deterministic() {
        let options = vec![
            OpenOption {
                name: "Beta".to_string(),
                id: Some("two".to_string()),
                path: Some(PathBuf::from("/Applications/Beta.app")),
            },
            OpenOption {
                name: "alpha".to_string(),
                id: Some("one".to_string()),
                path: Some(PathBuf::from("/Applications/Alpha.app")),
            },
            OpenOption {
                name: "alpha".to_string(),
                id: Some("one".to_string()),
                path: Some(PathBuf::from("/Applications/Alpha.app")),
            },
        ];

        let names: Vec<_> = sorted_unique_options(options)
            .into_iter()
            .map(|option| option.name)
            .collect();

        assert_eq!(names, ["alpha", "Beta"]);
    }
}
