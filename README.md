# Rust implementation of Tree (recursive directory listing program) 

This is a project developed by me after attending the "Foundamentals of Rust Programming" course by Prof. [Luca Abeni](https://github.com/lucabe72), at Sant'Anna School of Advanced Studies (Pisa), 2022-2023.



# How to Build 
-   clone repository
-   cd advanced_implementation
-   cargo run path_root [options] [-o filename]



# Rust implementation :
tree path_root [options] [-o filename]

including the following options : 
-   -a : print also hidden files;
-   -d : print only directories;
-   -l : follow symlinks if they point to directories, as if they were directories;
-   --keep_canonical : print filly canincalized path;
-   -f : print complete relative path;
-   --base_canonical : print full canonical path of root of tree;
-   -i : no indentation (useful if -f o --keep_canonical;
-   -c : colorize output;
-   -p : print files' type and permissions;
-   -num_perms : print permission in numerical format
-   -s : print files' sizes in bytes
-   --hsize_ib : print file size in bytes, converted in human readable format : K,M,G... (powers of 10^3)
-   --hsize : print file size in bytes, converted in human readable format : Ki,Mi,Gi... (powers of 2^10)
-   --fast_rcs : ignore cycle avoidance, faster byt risks symlink cycles
-   --ladv : cycle avoidance
-   -L <usize> : set (max) depth of iteration to <usize>
-   --filelimit <usize> : do not descend into directories with more than <usize> entries
-   -o <outpath(string)> : save output to <outpath>

<br>

# Linux Reference : 
For reference, Linux implementation (standard of comparison) :
    <https://linux.die.net/man/1/tree>




by Jacopo Carlon






During the development of this leanring project, I was inspired by, and used/modified code from : 
- https://www.georgevreilly.com/blog/2023/01/23/TreeInRust1WalkDirectories.html
- https://www.georgevreilly.com/blog/2023/01/24/TreeInRust2PrintingTrees.html
- https://github.com/alexanderwe/rs-tree

