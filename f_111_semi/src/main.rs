/*  simple Rust implementation of the Linux tree() command:
    https://linux.die.net/man/1/tree
    Synopsis
    tree [-adfghilnopqrstuvxACDFNS] [-L level [-R]] [-H baseHREF] [-T title] [-o filename] [--nolinks] [-P pattern] [-I pattern] [--inodes] [--device] [--noreport] [--dirsfirst] [--version] [--help] [--filelimit #] [directory ...]
*/

/*  I will only cover :
    tree [-ad] [-L level] [--nolinks] [directory ...]
    where :
    -a
    All files are printed. By default tree does not print hidden files (those beginning with a dot '.').
    In no event does tree print the file system constructs '.' (current directory) and '..' (previous directory).

    -d
    List directories only.

    -l
    Follows symbolic links if they point to directories, as if they were directories.
    Symbolic links that will result in recursion are avoided when detected.

    -p
    Print the file type and permissions for each file (as per ls -l).

    -L level
    Max display depth of the directory tree.

    //  --filelimit #
    //  Do not descend directories that contain more than # entries.

    //  -o filename
    //  Send output to filename.

*/
/*  the code will be divided in two phases:
    -   get the dir-tree in local-memory
    -   print nicely the dir-tree
*/

//  //  #[macro_use]
//  //  extern crate Parser;

//  extern crate tree;
mod treelibs;
//  use treelibs::*;

use clap::Parser;
use std::path::PathBuf;

use std::process;

/// A tree clone written in Rust

#[derive(Parser, Debug)]
//  #[Parser(name = "rstree")]
pub struct Opt {
    /// Print all files, including hidden
    #[clap(short = 'a', default_value = "false")]
    show_hidden: bool,

    /// Print only directories
    #[clap(short = 'd', default_value = "false")]
    only_dir: bool,

    /// Follow sym-links if they point to directories, as if they were directories
    #[clap(short = 'l', default_value = "false")]
    follow_symlink: bool,

    /// Keep canonical : full prefix
    #[clap(long, default_value = "false", group = "extendPaths")]
    keep_canonical: bool,

    /// Print complete relative path prefix for all 
    #[clap(short = 'f', default_value = "false", group = "extendPaths")]
    full_path: bool,

    /// Force base canonical
    #[clap(long, default_value = "false")]
    base_canonical: bool,

    /// Colorize output
    #[clap(short = 'c', default_value = "false")]
    colorize: bool,

    /// Print file type and permissions, as per "ls -l"
    #[clap(short = 'p', default_value = "false", group = "permissions")]
    perms: bool,

    /// Print numerical file permissions
    #[clap(long, default_value = "false", group = "permissions")]
    num_perms: bool,

    /// Ignore cycle avoidance methods, faster but risks symlink cycles
    #[clap(long, default_value = "false")]
    fast_rsc: bool,

    /// Set the depth of the iteraton, if 0 it goes to depth infinity
    #[clap(short = 'L', default_value = "0")]
    level: usize,

    /// do not descend directories with more than # entries
    #[clap(long, default_value = "0")]
    filelimit: usize,

    /// Directory to start with
    #[clap(name = "DIRECTORY", default_value = ".")]
    directory: PathBuf,
}

fn main() {
    let opt = Opt::parse();
    println!("{:?}", opt);

    if let Err(e) = treelibs::run(&opt) {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
