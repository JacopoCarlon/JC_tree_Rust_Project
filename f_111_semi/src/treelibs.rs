//  extern crate Parser;

use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

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

//  ------------------------- constants for permissions ------------------------- */
// from : https://man7.org/linux/man-pages/man7/inode.7.html
const S_IFMT   :u32 =   0o0170000;  // general mask
//  const S_IFSOCK :u32 =   0o0140000;  //   socket
const S_IFLNK  :u32 =   0o0120000;  //   symbolic link
const S_IFREG  :u32 =   0o0100000;  //   regular file
//  const S_IFBLK  :u32 =   0o0060000;  //   block device
const S_IFDIR  :u32 =   0o0040000;  //   directory
//  const S_IFCHR  :u32 =   0o0020000;  //   character device
//  const S_IFIFO  :u32 =   0o0010000;  //   FIFO
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


fn stringify_permissions(perms : u32) -> String {
    //  println!("{:o}", perms);
    let mut vec_perms: Vec<char> = "rwxrwxrwx".chars().collect();
    let mut b = 1;
    let mut i = 0;
    while i<9 {
        if (perms & b) == 0{
            vec_perms[8-i] = '-' ;
        }
        b<<=1; 
        i += 1;
    }
    //  println!("{:#?}", vec_perms);
    let str_perms : String = vec_perms.into_iter().collect();
    //  println!("{}", str_perms);  
    let pre_string :String= match perms & S_IFMT {
        S_IFLNK => "l".to_string(),
        S_IFREG => "-".to_string(),
        S_IFDIR => "d".to_string(),
        _       => "-".to_string(),
    };
    //  println!("{}", pre_string);
    //  let tot_perm_str = pre_string + &str_perms;
    //  println!("{}", tot_perm_str);
    pre_string + &str_perms
}


fn visit_dirs(
    dirs_visited: &mut Vec<PathBuf>,
    dir: & Path,
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
        //  //  add it to visited dirs
        //  //  dirs_visited.push(fs::canonicalize(&PathBuf::from(dir)).unwrap());
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

        //  here it could be possible to split the sort by "is_symlink", which would help avoiding symlink

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

        // help avoid symlink cycles by pre-listing directories which will certainly be visited
        if !opt.fast_rsc{
            for iter_entry in entries.iter(){
                if iter_entry.path().is_dir(){
                    dirs_visited.push(fs::canonicalize(PathBuf::from(iter_entry.path())).unwrap() );
                }
            }
        }

        // cycle through elements of current directory
        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();
            let entry_to_use = if index == entries.len()-1 {FINAL_ENTRY}else{OTHER_ENTRY};
            let child_to_use = if index == entries.len()-1 {FINAL_CHILD}else{OTHER_CHILD};
            
            if !opt.only_dir || path.is_dir() {
                // do all OR ( do only dirs AND is dir )
                if opt.perms || opt.num_perms {
                    let mtd = fs::symlink_metadata(&path)?;
                    let u32perms = mtd.permissions().mode(); 
                    // here perms is a number
                    if opt.num_perms{
                        println!(
                            "{}{}[{:o}] {}",
                            prefix,
                            entry_to_use,
                            u32perms,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    } else {
                        let tot_perm_str = stringify_permissions(u32perms);
                        println!(
                            "{}{}[{}] {}",
                            prefix,
                            entry_to_use,
                            tot_perm_str,
                            color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                        );
                    }
                    
                } else {
                    println!(
                        "{}{}{}",
                        prefix,
                        entry_to_use,
                        color_output(opt.colorize, &path, opt.keep_canonical, opt.full_path)?
                    );
                } 
            }
            if path.is_dir() {
                // enter path and tree() it
                let this_metadata = fs::symlink_metadata(&path)?;
                let this_is_symlink = this_metadata.file_type().is_symlink();
                if this_is_symlink {
                    //  println!("ad ora il visited-dir ha : {:#?}", dirs_visited);
                    //  println!("ciao, il symlink punta qua : {}", fs::canonicalize(path.clone()).unwrap().display());
                    // should we follow symlink 
                    if !opt.follow_symlink{
                        continue;
                    } 
                    // avoid symlink cycles 
                    if !opt.fast_rsc && dirs_visited.contains(&fs::canonicalize(path.clone()).unwrap()){
                        println!(
                            "{}{}[symlink cycle detected, will not expand it]",
                            prefix.clone() + child_to_use,
                            FINAL_ENTRY
                        );
                        continue;
                    }
                }

                let depth_new = depth + 1;
                //  let prefix_new = prefix.clone() + "    ";
                let prefix_new = prefix.clone() + child_to_use;
                visit_dirs(
                    dirs_visited,
                    &path,
                    prefix_new,
                    depth_new,
                    opt,
                )?
            }
        }
    }
    Ok(())
}

// visit base directory
fn visit_base(
    base: &Path,            // done
    prefix: String,         // done
    keep_canonical: bool,   // done         // --full_path
    full_path: bool,        //done          -f
    colorize: bool,         // done         // -c   
    perms: bool,          // -p
    num_perms: bool,         // --num_perms
) -> io::Result<()> {
    
    
    if num_perms || perms{
        let mtd = fs::symlink_metadata(base)?;
        let u32perms = mtd.permissions().mode();
        if num_perms{
            println!(
                "{}[{:o}] {}",
                prefix,
                u32perms,
                color_output(colorize, base, keep_canonical, full_path)?
            );
        } else {
            let tot_perm_str = stringify_permissions(u32perms);
            println!(
                "{}[{}] {}",
                prefix,
                tot_perm_str,
                color_output(colorize, base, keep_canonical, full_path)?
            );
        }
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
    let mut is_sym_and_target_exists = false;
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
    if !symlink.is_empty(){
        is_sym_and_target_exists = parent.join(path.read_link().unwrap()).exists();
    }

    // prepare print_name to print
    //  let print_name = if !symlink.is_empty() {
    //      format!("{} -> {}", filename, symlink)
    //  } else {
    //      filename.to_string()
    //  };
    match colorize {
        true => {
            if path.is_dir() {
                if !symlink.is_empty(){
                    if is_sym_and_target_exists{
                        Ok(format!(
                            "{}{}{} -> {}{}{}",
                            ANSIColor::Cyan.as_string(),
                            filename,
                            ANSIColor::Reset.as_string(),
                            ANSIColor::Yellow.as_string(),
                            symlink,
                            ANSIColor::Reset.as_string()
                        ))
                    }else {
                        Ok(format!(
                            "{}{}{} -> {}",
                            ANSIColor::Red.as_string(),
                            filename,
                            ANSIColor::Reset.as_string(),
                            symlink,
                        ))
                    }
                } else {
                    Ok(format!(
                        "{}{}{}",
                        ANSIColor::Yellow.as_string(),
                        filename,
                        ANSIColor::Reset.as_string()
                    ))
                }

            } else if is_executable(path) {
                if !symlink.is_empty(){
                    if is_sym_and_target_exists{
                        Ok(format!(
                            "{}{}{} -> {}{}{}",
                            ANSIColor::Cyan.as_string(),
                            filename,
                            ANSIColor::Reset.as_string(),
                            ANSIColor::Green.as_string(),
                            symlink,
                            ANSIColor::Reset.as_string()
                        ))
                    }else{
                        Ok(format!(
                            "{}{}{} -> {}",
                            ANSIColor::Red.as_string(),
                            filename,
                            ANSIColor::Reset.as_string(),
                            symlink,
                        ))
                    }
                } else {
                    Ok(format!(
                        "{}{}{}",
                        ANSIColor::Green.as_string(),
                        filename,
                        ANSIColor::Reset.as_string()
                    ))
                }
            } else {
                Ok(format!(
                    "{}{}{}",
                    ANSIColor::Magenta.as_string(),
                    filename,
                    ANSIColor::Reset.as_string()
                ))
            }
        }
        false => Ok(filename.to_string()),
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
    if opt.base_canonical{
        resulting_canonical = true;
        resulting_full_path = false;
    }
    visit_base(
        &opt.directory,
        String::from(""),
        resulting_canonical,
        resulting_full_path,
        opt.colorize,
        opt.perms,
        opt.num_perms,
    )?;
    if opt.directory.is_dir(){
        let mut dirs_visited = Vec::new();
        // add it to visited dirs
        if !opt.fast_rsc{
            dirs_visited.push(fs::canonicalize(&PathBuf::from(&opt.directory)).unwrap());
        }
        visit_dirs(
            &mut dirs_visited,
            &opt.directory,
            String::from(""),
            0,
            opt,
        )?;
    }else{
        println!(
            "{}{}[is not a directory]",
            FINAL_CHILD,
            FINAL_ENTRY
        );
    }
    Ok(())
}
