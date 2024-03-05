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




use std::env;
use std::io;
use std::path::PathBuf;

mod treefile;
mod treeprint;

use treefile::*;
use treeprint::*;



fn main() -> io::Result<()> {
    let mut args = env::args();
    let root = args.nth(1).unwrap_or(".".to_string());
    let dir: Directory = dir_walk(&PathBuf::from(root.clone()), is_not_hidden, sort_by_name)?;
    print_tree(&root, &dir);
    Ok(())
}
