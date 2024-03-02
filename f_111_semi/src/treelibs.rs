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
    BLACK,
    RED,
    GREEN,
    YELLOW,
    BLUE,
    MAGENTA,
    CYAN,
    WHITE,
    RESET,
}

#[allow(dead_code)]
impl ANSIColor {
    pub fn as_string(&self) -> &str {
        match self {
            &ANSIColor::BLACK => "\u{001B}[0;30m",
            &ANSIColor::RED => "\u{001B}[0;31m",
            &ANSIColor::GREEN => "\u{001B}[0;32m",
            &ANSIColor::YELLOW => "\u{001B}[0;33m",
            &ANSIColor::BLUE => "\u{001B}[0;34m",
            &ANSIColor::MAGENTA => "\u{001B}[0;35m",
            &ANSIColor::CYAN => "\u{001B}[0;36m",
            &ANSIColor::WHITE => "\u{001B}[0;37m",
            &ANSIColor::RESET => "\u{001B}[0;0m",
        }
    }
}

fn visit_dirs(
    dir: &Path,           // done
    depth: usize,         // done
    level: usize,         // done         // -L 3
    prefix: String,       // done
    keep_canonical: bool, // todo         // -f
    full_path: bool,
    colorize: bool, // done         // -c   //  TODO : need revision with new functions !!!
    show_hidden: bool, // done         // -a
    only_dir: bool, // done new !   // -d
    follow_symlink: bool, // done new !   // -l
    p_type_perms: bool, // done         // -p
    filelimit: usize, // done new !   // --filelimit 10
) -> io::Result<()> {
    // level == 0 -> go all the way
    // level != 0 -> go only to depth==level
    if (level != 0) & (depth == level) {
        return Ok(());
    }
    if dir.is_dir() {
        // get elements in this directory
        let entry_set = fs::read_dir(dir)?; // contains DirEntry
        let mut entries = entry_set
            .filter_map(|v| match v.ok() {
                Some(v) => {
                    if show_hidden {
                        return Some(v);
                    } else {
                        if v.file_name().to_str()?.starts_with(".") {
                            return None;
                        } else {
                            Some(v)
                        }
                    }
                }
                None => None,
            })
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| a.path().file_name().cmp(&b.path().file_name()));

        let num_entries: usize = entries.len();

        // *filelimit* : if current dir has too many entries, print none
        if (filelimit != 0) && num_entries > filelimit {
            println!(
                "{}└── [{} entries exceeded filelimit, not opening dir]",
                prefix, num_entries
            );
            return Ok(());
        }

        // *only_dir* condition
        if only_dir {
            entries.retain(|x| x.path().is_dir())
        }

        // cycle through elements of current directory
        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();

            if index == entries.len() - 1 {
                // is last element
                // if !only_dir || ( only_dir && path.is_dir() )
                if !only_dir || path.is_dir() {
                    if !p_type_perms {
                        println!(
                            "{}{}{}",
                            prefix,
                            FINAL_ENTRY,
                            color_output(colorize, &path, keep_canonical, full_path)?
                        );
                    } else {
                        let mtd = fs::symlink_metadata(&path)?;
                        let perms = mtd.permissions();
                        println!(
                            "{}{}[{:o}] {}",
                            prefix,
                            FINAL_ENTRY,
                            perms.mode(),
                            color_output(colorize, &path, keep_canonical, full_path)?
                        );
                    }
                }
                if path.is_dir() {
                    let this_metadata = fs::symlink_metadata(&path)?;
                    let this_is_symlink = this_metadata.file_type().is_symlink();
                    if this_is_symlink && !follow_symlink {
                        continue;
                    }

                    let depth = depth + 1;
                    //  let prefix_new = prefix.clone() + "    ";
                    let prefix_new = prefix.clone() + FINAL_CHILD;
                    visit_dirs(
                        &path,
                        depth,
                        level,
                        prefix_new,
                        keep_canonical,
                        full_path,
                        colorize,
                        show_hidden,
                        only_dir,
                        follow_symlink,
                        p_type_perms,
                        filelimit,
                    )?
                }
            } else {
                // is not last element
                if !only_dir || path.is_dir() {
                    if !p_type_perms {
                        println!(
                            "{}{}{}",
                            prefix,
                            OTHER_ENTRY,
                            color_output(colorize, &path, keep_canonical, full_path)?
                        );
                    } else {
                        let mtd = fs::symlink_metadata(&path)?;
                        let perms = mtd.permissions();
                        println!(
                            "{}{}[{:o}] {}",
                            prefix,
                            OTHER_ENTRY,
                            perms.mode(),
                            color_output(colorize, &path, keep_canonical, full_path)?
                        );
                    }
                }
                if path.is_dir() {
                    let this_metadata = fs::symlink_metadata(&path)?;
                    let this_is_symlink = this_metadata.file_type().is_symlink();
                    if this_is_symlink && !follow_symlink {
                        continue;
                    }

                    let depth = depth + 1;
                    //  let prefix_new = prefix.clone() + "│   ";
                    let prefix_new = prefix.clone() + OTHER_CHILD;
                    visit_dirs(
                        &path,
                        depth,
                        level,
                        prefix_new,
                        keep_canonical,
                        full_path,
                        colorize,
                        show_hidden,
                        only_dir,
                        follow_symlink,
                        p_type_perms,
                        filelimit,
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
    _depth: usize,         // done
    _level: usize,         // done         // -L 3
    prefix: String,        // done
    keep_canonical: bool,  // done         // --full_path
    full_path: bool,       //done          -f
    colorize: bool,        // done         // -c   //  TODO : need revision with new functions !!!
    _show_hidden: bool,    // done         // -a
    _only_dir: bool,       // done new !   // -d
    _follow_symlink: bool, // done new !   // -l
    p_type_perms: bool,    // -p
    _filelimit: usize,     // done new !   // --filelimit 10
) -> io::Result<()> {
    //  //  println!("debug : {}", base.display());
    //  //  let true_base_dir = PathBuf::from(&base);
    //  //  let _canonicalized_base = fs::canonicalize(&true_base_dir)?;

    let mtd = fs::symlink_metadata(&base)?;
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
            color_output(colorize, &base, keep_canonical, full_path)?
        );
    } else {
        println!(
            "{}{}",
            prefix,
            color_output(colorize, &base, keep_canonical, full_path)?
        );
    }

    Ok(())
}

fn is_executable(path: &Path) -> bool {
    let metadata = match fs::symlink_metadata(&path) {
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
    // .to_string_lossy().into_owned() == .to_str().unwrap().to_owned(), ma funziona anche se il path non è UTF8 valido
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
        } else {
            symlink = "".to_owned();
        }
    }

    // prepare print_name to print
    let print_name;

    if !symlink.is_empty() {
        print_name = format!("{} -> {}", filename, symlink);
    } else {
        print_name = format!("{}", filename);
    }

    match colorize {
        true => {
            if path.is_dir() {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::YELLOW.as_string(),
                    print_name,
                    ANSIColor::RESET.as_string()
                ))
            } else if is_executable(&path) {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::GREEN.as_string(),
                    print_name,
                    ANSIColor::RESET.as_string()
                ))
            } else {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::MAGENTA.as_string(),
                    print_name,
                    ANSIColor::RESET.as_string()
                ))
            }
        }
        false => Ok(format!("{}", print_name)),
    }
}

//  function "run", gets all input flags and target dir, does search-and-print
//  level 0 goes to depth-infinity
//  filelimit 0 means no bound on files in dir
pub fn run(opt: &Opt) -> Result<(), Box<dyn Error>> {
    let mut force_base_canonical = false;
    force_base_canonical |= opt.keep_canonical;
    visit_base(
        &opt.directory,
        0,
        opt.level,
        String::from(""),
        force_base_canonical,
        opt.full_path,
        opt.colorize,
        opt.show_hidden,
        opt.only_dir,
        opt.follow_symlink,
        opt.p_type_perms,
        opt.filelimit,
    )?;
    // visit_dirs(dir, depth, level, prefix, colorize, show_hidden, only_dir, follow_symlink, p_type_perms, filelimit)
    visit_dirs(
        &opt.directory,
        0,
        opt.level,
        String::from(""),
        opt.keep_canonical,
        opt.full_path,
        opt.colorize,
        opt.show_hidden,
        opt.only_dir,
        opt.follow_symlink,
        opt.p_type_perms,
        opt.filelimit,
    )?;
    Ok(())
}
