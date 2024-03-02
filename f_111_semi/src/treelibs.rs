//  extern crate Parser;

use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::Opt;

const OTHER_CHILD: &str = "│   "; // prefix: pipe
const OTHER_ENTRY: &str = "├── "; // connector: tee
const FINAL_CHILD: &str = "    "; // prefix: no siblings
const FINAL_ENTRY: &str = "└── "; // connector: elbow

#[allow(dead_code)]
pub enum ANSIColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Reset,
}

//  Checks for matches where all arms match a reference, 
//  suggesting to remove the reference and deref the matched expression instead. 
//  It also checks for if let &foo = bar blocks.
#[allow(dead_code)]
impl ANSIColor {
    pub fn as_string(&self) -> &str {
        match *self {
            ANSIColor::Black => "\u{001B}[0;30m",
            ANSIColor::Red => "\u{001B}[0;31m",
            ANSIColor::Green => "\u{001B}[0;32m",
            ANSIColor::Yellow => "\u{001B}[0;33m",
            ANSIColor::Blue => "\u{001B}[0;34m",
            ANSIColor::Magenta => "\u{001B}[0;35m",
            ANSIColor::Cyan => "\u{001B}[0;36m",
            ANSIColor::White => "\u{001B}[0;37m",
            ANSIColor::Reset => "\u{001B}[0;0m",
        }
    }
}




fn visit_dirs(
    dir: &Path,
    prefix: String,       
    depth: usize,         
    opt: &Opt,
    ) -> io::Result<()> {
    // opt.level == 0 -> go all the way
    // opt.level != 0 -> go only to depth==opt.level
    if (opt.level != 0) & (depth == opt.level) {
        return Ok(());
    }
    if dir.is_dir() {
        // get elements in this directory
        let entry_set = fs::read_dir(dir)?; // contains DirEntry
        let mut entries = entry_set
            .filter_map(|v| match v.ok() {
                Some(v) => {
                    if opt.show_hidden {
                        Some(v)
                    } else if v.file_name().to_str()?.starts_with('.') {
                        None
                    } else {
                        Some(v)
                    }
                }
                None => None,
            })
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

        let num_entries: usize = entries.len();

        // if current dir has too many entries, print none
        if (opt.filelimit != 0) && num_entries > opt.filelimit {
            println!(
                "{}└── [{} entries exceeded filelimit, not printing dir]",
                prefix, num_entries
            );
            return Ok(());
        }

        if opt.only_dir {
            entries.retain(|x| x.path().is_dir())
        }

        // cycle through elements of current directory
        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();

            if index == entries.len() - 1 {
                // is last element
                if !opt.only_dir || path.is_dir() {
                    if !opt.p_type_perms {
                        println!(
                            "{}{}{}",
                            prefix,
                            FINAL_ENTRY,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    } else {
                        let mtd = fs::symlink_metadata(&path)?;
                        let perms = mtd.permissions();
                        println!(
                            "{}{}[{:o}] {}",
                            prefix,
                            FINAL_ENTRY,
                            perms.mode(),
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    }
                }
                if path.is_dir() {
                    let this_metadata = fs::symlink_metadata(&path)?;
                    let this_is_symlink = this_metadata.file_type().is_symlink();
                    if this_is_symlink && !opt.follow_symlink {
                        continue;
                    }

                    let depth_new = depth + 1;
                    //  let prefix_new = prefix.clone() + "    ";
                    let prefix_new = prefix.clone() + FINAL_CHILD;
                    visit_dirs(
                        &path,
                        prefix_new,
                        depth_new,
                        opt,
                    )?
                }
            } else {
                // is not last element
                if !opt.only_dir || path.is_dir() {
                    if !opt.p_type_perms {
                        println!(
                            "{}{}{}",
                            prefix,
                            OTHER_ENTRY,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    } else {
                        let mtd = fs::symlink_metadata(&path)?;
                        let perms = mtd.permissions();
                        println!(
                            "{}{}[{:o}] {}",
                            prefix,
                            OTHER_ENTRY,
                            perms.mode(),
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    }
                }
                if path.is_dir() {
                    let this_metadata = fs::symlink_metadata(&path)?;
                    let this_is_symlink = this_metadata.file_type().is_symlink();
                    if this_is_symlink && !opt.follow_symlink {
                        continue;
                    }

                    let depth_new = depth + 1;
                    //  let prefix_new = prefix.clone() + "│   ";
                    let prefix_new = prefix.clone() + OTHER_CHILD;
                    visit_dirs(
                        &path,
                        prefix_new,
                        depth_new,
                        opt,
                    )?
                }
            }
        }
    }
    Ok(())
}

// visit base directory
fn visit_base(
    base: &Path,           // done
    prefix: String,        // done
    keep_canonical: bool,  // done         // --full_path
    full_path: bool,       //done          -f
    colorize: bool,        // done         // -c   
    p_type_perms: bool,    // -p
) -> io::Result<()> {
    //  //  println!("debug : {}", base.display());
    //  //  let true_base_dir = PathBuf::from(&base);
    //  //  let _canonicalized_base = fs::canonicalize(&true_base_dir)?;

    let mtd = fs::symlink_metadata(base)?;
    let perms = mtd.permissions();
    let _symlink = match fs::read_link(base) {
        Ok(v) => v,
        Err(_err) => PathBuf::new(),
    };

    if p_type_perms {
        println!(
            "{}[{:o}] {}",
            prefix,
            perms.mode(),
            color_output(colorize, base, keep_canonical, full_path)?
        );
    } else {
        println!(
            "{}{}",
            prefix,
            color_output(colorize, base, keep_canonical, full_path)?
        );
    }

    Ok(())
}

fn is_executable(path: &Path) -> bool {
    let metadata = match fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(_err) => return false,
    };

    metadata.permissions().mode() & 0o111 != 0
}

fn color_output(
    colorize: bool,
    path: &Path,
    keep_canonical: bool,
    full_path: bool,
) -> io::Result<String> {
    let filename: String;
    let symlink: String;
    let parent = path.parent().unwrap();
    //  .to_string_lossy().into_owned() == .to_str().unwrap().to_owned(), 
    //  ma funziona anche se il path non è UTF8 valido
    if !keep_canonical && !full_path {
        if path == Path::new(".") || path == Path::new("..") {
            filename = path.to_string_lossy().into_owned();
        } else {
            filename = path.file_name().unwrap().to_string_lossy().into_owned();
        }
        symlink = match fs::read_link(path) {
            Ok(v) => v.to_string_lossy().into_owned(),
            Err(_err) => "".to_owned(),
        };
    } else if full_path {
        filename = path.to_string_lossy().into_owned();
        symlink = match fs::read_link(path) {
            Ok(v) => v.to_string_lossy().into_owned(),
            Err(_err) => "".to_owned(),
        };
    } else {
        // keep canonical for all paths
        filename = fs::canonicalize(parent)
            .unwrap()
            .join(path.file_name().unwrap())
            .to_string_lossy()
            .into_owned();
        if path.is_symlink() {
            if parent.join(path.read_link().unwrap()).exists() {
                symlink = fs::canonicalize(path)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
            } else {
                symlink = path.read_link().unwrap().to_string_lossy().into_owned();
            }
        } 
        else {
            symlink = "".to_owned();
        }
    }

    // prepare print_name to print
    let print_name = if !symlink.is_empty() {
        format!("{} -> {}", filename, symlink)
    } else {
        filename.to_string()
    };

    match colorize {
        true => {
            if path.is_dir() {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::Yellow.as_string(),
                    print_name,
                    ANSIColor::Reset.as_string()
                ))
            } else if is_executable(path) {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::Green.as_string(),
                    print_name,
                    ANSIColor::Reset.as_string()
                ))
            } else {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::Magenta.as_string(),
                    print_name,
                    ANSIColor::Reset.as_string()
                ))
            }
        }
        false => Ok(print_name.to_string()),
    }
}

//  function "run", gets all input flags and target dir, does search-and-print
//  opt.level 0 goes to depth-infinity
//  filelimit 0 means no bound on files in dir
pub fn run(opt: &Opt) -> Result<(), Box<dyn Error>> {
    // force_base_canonical is a flavour implementation of tree of mine.
    //  let force_base_canonical = false;
    let mut resulting_canonical = opt.keep_canonical;
    let mut resulting_full_path = opt.full_path;
    if opt.fbc{
        resulting_canonical = true;
        resulting_full_path = false;
    }
    visit_base(
        &opt.directory,
        String::from(""),
        resulting_canonical,
        resulting_full_path,
        opt.colorize,
        opt.p_type_perms,
    )?;
    visit_dirs(
        &opt.directory,
        String::from(""),
        0,
        opt,
    )?;
    Ok(())
}
