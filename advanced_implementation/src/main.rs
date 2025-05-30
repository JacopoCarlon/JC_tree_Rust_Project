/*  simple Rust implementation of the Linux tree() command:
    https://linux.die.net/man/1/tree
    Synopsis
    tree [-adfghilnopqrstuvxACDFNS] [-L level [-R]] [-H baseHREF] [-T title] [-o filename] [--nolinks] [-P pattern] [-I pattern] [--inodes] [--device] [--noreport] [--dirsfirst] [--version] [--help] [--filelimit #] [directory ...]
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

    /// Keep canonical : full canonicalized path
    #[clap(long, default_value = "false", group = "extendPaths")]
    keep_canonical: bool,

    /// Print complete relative path prefix for all
    #[clap(short = 'f', default_value = "false", group = "extendPaths")]
    full_rel_path: bool,

    /// Force base canonical
    #[clap(long, default_value = "false")]
    base_canonical: bool,

    ///Don't indent, useful if -f or --keep_canonical are used
    #[clap(short = 'i', default_value = "false")]
    no_indent: bool,

    /// Colorize output
    #[clap(short = 'c', default_value = "false", group = "printy_style")]
    colorize: bool,

    /// Print file type and permissions, as per "ls -l"
    #[clap(short = 'p', default_value = "false", group = "permissions")]
    perms: bool,

    /// Print numerical file permissions
    #[clap(long, default_value = "false", group = "permissions")]
    num_perms: bool,

    /// Print file size in bytes
    #[clap(short = 's', default_value = "false", group = "filesize")]
    size: bool,

    /// Print file size in bytes converted in human readable format : K, M, G...
    #[clap(long, default_value = "false", group = "filesize")]
    hsize_ib: bool,

    /// Print file size in bytes converted in human readable format : Ki, Mi, Gi...
    #[clap(long, default_value = "false", group = "filesize")]
    hsize: bool,

    /// Ignore cycle avoidance methods, faster but risks symlink cycles
    #[clap(long, default_value = "false", group = "sym_cycle_mode")]
    fast_rsc: bool,

    /// Advanced cycle detection and avoidance by pre-computing parents of target file
    #[clap(long, default_value = "false", group = "sym_cycle_mode")]
    ladv: bool,

    /// Set the depth of the iteraton, if 0 it goes to depth infinity
    #[clap(short = 'L', default_value = "0")]
    level: usize,

    /// do not descend directories with more than # entries
    #[clap(long, default_value = "0")]
    filelimit: usize,

    /// Directory to start with
    #[clap(name = "DIRECTORY", default_value = ".")]
    directory: PathBuf,

    /// Save to target file
    #[clap(short = 'o', default_value = "", group = "printy_style")]
    target_file: String,
}

fn main() {
    let opt = Opt::parse();
    //  println!("{:?}", opt);

    if let Err(_run_error) = treelibs::run(&opt) {
        //  eprintln!("Application error: {}", _run_error);
        process::exit(1);
    }
}
