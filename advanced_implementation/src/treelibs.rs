//  extern crate Parser;
//  extern crate bytesize;
//  extern crate pretty_bytes;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
//  use std::cmp;
//  use filesize::PathExt;
//  use bytesize::ByteSize;
//  use pretty_bites::converter::convert;

use crate::Opt;

const OTHER_CHILD: &str = "│   "; // prefix: pipe
const OTHER_ENTRY: &str = "├── "; // connector: tee
const FINAL_CHILD: &str = "    "; // prefix: no siblings
const FINAL_ENTRY: &str = "└── "; // connector: elbow
const NO_INDENT: &str = "";

//  const PETA: u64 = 1_125_899_906_842_624;
//  const TERA: u64 = 1_099_511_627_776;
//  const GIGA: u64 = 1_073_741_824;
//  const MEGA: u64 = 1_048_576;
//  const KILO: u64 = 1_024;

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

//  ------------------------- constants for permissions ------------------------- */
// from : https://man7.org/linux/man-pages/man7/inode.7.html
const S_IFMT: u32 = 0o0_170_000; //  general mask
const S_IFLNK: u32 = 0o0_120_000; //  symbolic link
const S_IFREG: u32 = 0o0_100_000; //  regular file
const S_IFDIR: u32 = 0o0_040_000; //  directory
                                  //  const S_IFSOCK :u32 =   0o0140000;  //  socket
                                  //  const S_IFBLK  :u32 =   0o0060000;  //  block device
                                  //  const S_IFCHR  :u32 =   0o0020000;  //  character device
                                  //  const S_IFIFO  :u32 =   0o0010000;  //  FIFO
                                  //  ------------------------- constants for permissions ------------------------- */
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

pub fn convert(num: u64, delimiter: u64) -> String {
    let units = ["B", "K", "M", "G", "T", "P", "E", "Z", "Y"];
    let f_delimiter = delimiter as f64;
    let flnum = num as f64;
    let mut runner: f64 = 1.0;
    let mut counter = 0;
    let mut old_counter = counter;
    let mut ratio: f64 = flnum / runner;
    let mut old_ratio: f64 = ratio;
    while ratio > 1.0 {
        old_ratio = ratio;
        old_counter = counter;
        counter += 1;
        runner *= f_delimiter;
        ratio = flnum / runner;
    }
    if old_counter > 0 {
        format!("{:.1}{}", old_ratio, units[old_counter])
    } else {
        format!("{}", old_ratio)
    }
}

fn stringify_permissions(perms: u32) -> String {
    let mut vec_perms: Vec<char> = "rwxrwxrwx".chars().collect();
    let mut b = 1;
    let mut i = 0;
    while i < 9 {
        if (perms & b) == 0 {
            vec_perms[8 - i] = '-';
        }
        b <<= 1;
        i += 1;
    }
    let str_perms: String = vec_perms.into_iter().collect();
    let pre_string: String = match perms & S_IFMT {
        S_IFLNK => "l".to_string(),
        S_IFDIR => "d".to_string(),
        S_IFREG => "-".to_string(),
        _ => "?".to_string(),
    };
    pre_string + &str_perms
}

fn visit_dirs(
    outfile: &mut dyn std::io::Write,
    dirs_visited: &mut Vec<PathBuf>,
    dir: &Path,
    prefix: &str,
    depth: usize,
    opt: &Opt,
) -> io::Result<()> {
    //  my_write(outfile, "visitDir");
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
        //  println!("testing filelimit");
        if (opt.filelimit != 0) && num_entries > opt.filelimit {
            my_write(
                outfile,
                &format!(
                    "{}{}[{} entries exceeded filelimit, not printing dir]",
                    prefix, FINAL_ENTRY, num_entries
                ),
            );
            return Ok(());
        }
        if opt.only_dir {
            entries.retain(|x| x.path().is_dir());
        }
        // help avoid symlink cycles by pre-listing directories which will certainly be visited
        if !opt.fast_rsc {
            for iter_entry in &entries {
                if iter_entry.path().is_dir() {
                    dirs_visited.push(fs::canonicalize(iter_entry.path()).unwrap());
                }
            }
        }
        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();
            let entry_to_use;
            let child_to_use;
            if opt.no_indent {
                entry_to_use = NO_INDENT;
                child_to_use = NO_INDENT;
            } else {
                entry_to_use = if index == entries.len() - 1 {
                    FINAL_ENTRY
                } else {
                    OTHER_ENTRY
                };
                child_to_use = if index == entries.len() - 1 {
                    FINAL_CHILD
                } else {
                    OTHER_CHILD
                };
            }
            if !opt.only_dir || path.is_dir() {
                // do all OR ( do only dirs AND is dir )
                if opt.perms || opt.num_perms || opt.size || opt.hsize || opt.hsize_ib {
                    let this_path = Path::new(&path);
                    let mtd = fs::symlink_metadata(this_path)?;
                    let u32perms = mtd.permissions().mode();
                    //  let realsize = this_path.size_on_disk_fast(&mtd).unwrap();
                    //  let realsize = this_path.size_on_disk().unwrap();
                    let realsize = mtd.len();
                    // here perms is a number
                    let tot_perm_str = stringify_permissions(u32perms);
                    let internal = format!(
                        "[{}{}] ",
                        if opt.num_perms {
                            format!("{:o }", u32perms)
                        } else if opt.perms {
                            tot_perm_str + " "
                        } else {
                            "".to_string()
                        },
                        if opt.size {
                            format!("{:5}", realsize)
                        } else if opt.hsize || opt.hsize_ib {
                            if opt.hsize_ib {
                                format!("{:>6} iB", convert(realsize, 1000_u64))
                            } else {
                                format!("{:>6}", convert(realsize, 1024_u64))
                            }
                        } else {
                            String::new()
                        }
                    );
                    my_write(
                        outfile,
                        &format!(
                            "{}{}{}{}",
                            prefix,
                            entry_to_use,
                            internal,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_rel_path)
                        ),
                    );
                } else {
                    my_write(
                        outfile,
                        &format!(
                            "{}{}{}",
                            prefix,
                            entry_to_use,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_rel_path)
                        ),
                    );
                }
            }
            if path.is_dir() {
                // enter path and tree() it
                let this_metadata = fs::symlink_metadata(&path)?;
                let this_is_symlink = this_metadata.file_type().is_symlink();
                if this_is_symlink {
                    //  my_write(outfile, format!("ad ora il visited-dir ha : {:#?}", dirs_visited));
                    //  my_write(outfile, format!("ciao, il symlink punta qua : {}", fs::canonicalize(path.clone()).unwrap().display()) );
                    // should we follow symlink
                    if !opt.follow_symlink {
                        continue;
                    }
                    // avoid symlink cycles
                    if !opt.fast_rsc
                        && dirs_visited.contains(&fs::canonicalize(path.clone()).unwrap())
                    {
                        my_write(
                            outfile,
                            &format!(
                                "{}{}[symlink cycle detected, will not expand it]",
                                prefix.to_string() + child_to_use,
                                FINAL_ENTRY
                            ),
                        );
                        continue;
                    }
                }
                let depth_new = depth + 1;
                //  let prefix_new = prefix.clone() + "    ";
                let prefix_new = prefix.to_string() + child_to_use;
                visit_dirs(outfile, dirs_visited, &path, &prefix_new, depth_new, opt)?;
            }
        }
    }
    Ok(())
}

// visit base directory
fn visit_base(
    outfile: &mut dyn std::io::Write,
    base: &Path,          // done
    prefix: &str,         // done
    keep_canonical: bool, // done         //  --full_rel_path
    full_rel_path: bool,      // done         //  -f
    opt: &Opt,
) -> io::Result<()> {
    //  my_write(outfile, "visitBase");
    if opt.perms || opt.num_perms || opt.size || opt.hsize || opt.hsize_ib {
        let this_path = Path::new(&base);
        let mtd = fs::symlink_metadata(this_path)?;
        let u32perms = mtd.permissions().mode();
        let realsize = mtd.len();
        let tot_perm_str = stringify_permissions(u32perms);
        let internal = format!(
            "[{}{}] ",
            if opt.num_perms {
                format!("{:o }", u32perms)
            } else if opt.perms {
                tot_perm_str + " "
            } else {
                "".to_string()
            },
            if opt.size {
                format!("{:5}", realsize)
            } else if opt.hsize || opt.hsize_ib {
                if opt.hsize_ib {
                    format!("{:>6} iB", convert(realsize, 1000_u64))
                } else {
                    format!("{:>6}", convert(realsize, 1024_u64))
                }
            } else {
                String::new()
            }
        );
        my_write(
            outfile,
            &format!(
                "{}{}{}",
                prefix,
                internal,
                color_output(opt.colorize, base, keep_canonical, full_rel_path)
            ),
        );
    } else {
        my_write(
            outfile,
            &format!(
                "{}{}",
                prefix,
                color_output(opt.colorize, base, keep_canonical, full_rel_path)
            ),
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
    full_rel_path: bool,
) -> std::string::String {
    //  println!("path : {} ;", path.display());
    let filename: String;
    let symlink: String;
    let parent = path.parent().unwrap();
    //  println!("path : {} ; parent is : {} ", path.display(), parent.display());
    let mut is_sym_and_target_exists = false;
    //  .to_string_lossy().into_owned() == .to_str().unwrap().to_owned(),
    //  ma funziona anche se il path non è UTF8 valido
    //  println!("{}", path.display());
    if !keep_canonical && !full_rel_path {
        // default path
        //  println!("enter first if : {} ; parent is : {} ", path.display(), parent.display());
        if path == Path::new(".") || path == Path::new("..") {
            filename = path.to_string_lossy().into_owned();
        } else {
            filename = path.file_name().unwrap().to_string_lossy().into_owned();
        }
        symlink = match fs::read_link(path) {
            Ok(v) => v.to_string_lossy().into_owned(),
            Err(_err) => String::new(),
        };
    } else if full_rel_path {
        //  println!("enter second if : {} ; parent is : {} ", path.display(), parent.display());
        filename = path.to_string_lossy().into_owned();
        symlink = match fs::read_link(path) {
            Ok(v) => v.to_string_lossy().into_owned(),
            Err(_err) => String::new(),
        };
    } else {
        // full canonical path
        //  println!("enter last if : {} ; parent is : {} ", path.display(), parent.display());
        let _can_path = fs::canonicalize(path).unwrap();
        let _can_parent = fs::canonicalize(_can_path.parent().unwrap()).unwrap();
        //  println!("enter last if : canpath {} ; canparent is : {} ", _can_path.display(), _can_parent.display());
        //  if parent.exists(){
        //      filename = fs::canonicalize(parent)
        //          .unwrap()
        //          .join(path.file_name().unwrap())
        //          .to_string_lossy()
        //          .into_owned();
        //  }else{
        //      filename = "".to_string();
        //  }
        filename = format!("{}", _can_path.display());
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
            symlink = String::new();
        }
    }
    //  println!("{}", filename);
    if !symlink.is_empty() {
        is_sym_and_target_exists = parent.join(path.read_link().unwrap()).exists();
    }

    if colorize {
        if path.is_dir() {
            if symlink.is_empty() {
                format!(
                    "{}{}{}",
                    ANSIColor::Yellow.as_string(),
                    filename,
                    ANSIColor::Reset.as_string()
                )
            } else if is_sym_and_target_exists {
                format!(
                    "{}{}{} -> {}{}{}",
                    ANSIColor::Cyan.as_string(),
                    filename,
                    ANSIColor::Reset.as_string(),
                    ANSIColor::Yellow.as_string(),
                    symlink,
                    ANSIColor::Reset.as_string()
                )
            } else {
                format!(
                    "{}{}{} -> {}",
                    ANSIColor::Red.as_string(),
                    filename,
                    ANSIColor::Reset.as_string(),
                    symlink,
                )
            }
        } else if is_executable(path) {
            if symlink.is_empty() {
                format!(
                    "{}{}{}",
                    ANSIColor::Green.as_string(),
                    filename,
                    ANSIColor::Reset.as_string()
                )
            } else if is_sym_and_target_exists {
                format!(
                    "{}{}{} -> {}{}{}",
                    ANSIColor::Cyan.as_string(),
                    filename,
                    ANSIColor::Reset.as_string(),
                    ANSIColor::Green.as_string(),
                    symlink,
                    ANSIColor::Reset.as_string()
                )
            } else {
                format!(
                    "{}{}{} -> {}",
                    ANSIColor::Red.as_string(),
                    filename,
                    ANSIColor::Reset.as_string(),
                    symlink,
                )
            }
        } else {
            format!(
                "{}{}{}",
                ANSIColor::Magenta.as_string(),
                filename,
                ANSIColor::Reset.as_string()
            )
        }
    } else {
        filename.to_string()
    }
}

fn my_write(writer: &mut dyn std::io::Write, text: &str) {
    writeln!(writer, "{}", text).unwrap();
}

//  function "run", gets all input flags and target dir, does search-and-print
//  opt.level 0 goes to depth-infinity
//  filelimit 0 means no bound on files in dir
pub fn run(opt: &Opt) -> Result<(), Box<dyn Error>> {
    //  //  solution with heap allocations :
    //  let mut outfile: Box<dyn std::io::Write> = if opt.target_file.is_empty() {
    //      let stdout = std::io::stdout();
    //      let stdout = stdout.lock();
    //      Box::new(stdout)
    //  } else {
    //      let file = File::create("my_file.txt").unwrap();
    //      let buf_file = std::io::BufWriter::new(file);
    //      Box::new(buf_file)
    //  };
    //  //  ---------------------------------------------
    //  //  solution without heap allocations :
    let outfile: &mut dyn Write;
    let mut lockstdout;
    let mut buf_file;

    if opt.target_file.is_empty() {
        let stdout = std::io::stdout();
        lockstdout = stdout.lock();
        outfile = &mut lockstdout;
    } else {
        let file = File::create(&opt.target_file)?;
        buf_file = BufWriter::new(file);
        outfile = &mut buf_file;
    }
    my_write(outfile, format!("{:?}", opt).as_str());

    // force_base_canonical is a flavour implementation of tree of mine.
    //  let force_base_canonical = false;
    let mut resulting_canonical = opt.keep_canonical;
    let mut resulting_full_rel_path = opt.full_rel_path;
    if opt.base_canonical {
        resulting_canonical = true;
        resulting_full_rel_path = false;
    }
    visit_base(
        outfile,
        &opt.directory,
        "", //  &String::from("")
        resulting_canonical,
        resulting_full_rel_path,
        opt,
    )?;
    if opt.directory.is_dir() {
        let mut dirs_visited = Vec::new();
        // add it to visited dirs
        if !opt.fast_rsc {
            dirs_visited.push(fs::canonicalize(PathBuf::from(&opt.directory)).unwrap());
        }
        if opt.ladv {
            let tmp_buf = fs::canonicalize(opt.directory.as_path()).unwrap();
            let mut tmp_dir = tmp_buf.as_path();
            while let Some(x) = tmp_dir.parent() {
                dirs_visited.push(fs::canonicalize(PathBuf::from(&x)).unwrap());
                tmp_dir = x;
            }
        }
        visit_dirs(
            outfile,
            &mut dirs_visited,
            &opt.directory,
            "", // &String::from("")
            0,
            opt,
        )?;
    } else {
        my_write(
            outfile,
            &format!(
                "{}{}[given base is not a directory]",
                FINAL_CHILD, FINAL_ENTRY
            ),
        );
    }
    Ok(())
}
