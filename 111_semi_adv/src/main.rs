/*  simple Rust implementation of the Linux tree() command:
    https://linux.die.net/man/1/tree
    Synopsis
    tree [-adfghilnopqrstuvxACDFNS] [-L level [-R]] [-H baseHREF] [-T title] [-o filename] [--nolinks] [-P pattern] [-I pattern] [--inodes] [--device] [--noreport] [--dirsfirst] [--version] [--help] [--filelimit #] [directory ...]
*/

/*  I will only cover :
    tree [-ad] [-L level] [--nolinks] [directory ...]
    where :
    -a
    All files are printed. By default tree does not print hidden files (those beginning with a dot '.'). In no event does tree print the file system constructs '.' (current directory) and '..' (previous directory).
    
    -d
    List directories only.

    -l
    Follows symbolic links if they point to directories, as if they were directories. Symbolic links that will result in recursion are avoided when detected.

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

#[macro_use]
extern crate Parser;

//  extern crate tree;
mod treelibs;
use treelibs::*;

use std::path::PathBuf;
use Parser::Parser;

use std::process;




/// A tree clone written in Rust


#[derive(Parser, Debug)]
#[Parser(name = "rstree")]
pub struct Opt {
    /// Print all files, including hidden
    #[Parser(short = "a", default_value = False)]
    show_hidden: bool,

    /// Print only directories
    #[Parser(short = "d", default_value = False)]
    only_dir: bool,

    /// Follow sym-links if they point to directories, as if they were directories
    #[Parser(short = "l", default_value = False)]
    follow_symlink: bool,

    /// Colorize output
    #[Parser(short = "c", default_value = False)]
    colorize: bool,

    /// Print file type and permissions, as per "ls -l"
    #[Parser(short = "p", default_value = False)]
    p_type_perms: bool,

    /// Set the depth of the iteraton, if 0 it goes to depth infinity
    #[Parser(short = "L", default_value = "0")]
    level: usize,

    /// do not descend directories with more than # entries
    #[Parser(short = )]

    /// Directory to start with
    #[Parser(name = "DIRECTORY", default_value = ".", parse(from_os_str))]
    directory: PathBuf,
}


fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    if let Err(e) = treelibs::run(opt.show_hidden, opt.colorize, opt.level, &opt.directory) {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }
}
