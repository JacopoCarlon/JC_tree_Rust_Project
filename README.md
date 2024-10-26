# rust implementation of Tree (recursive directory listing program) 

Linux implementation (standard of comparison) :
<https://linux.die.net/man/1/tree>

Rust implementation :
tree path_root [options] [-o filename]

including the following options : 
-a : print also hidden files;
-d : print only directories;
-l : follow symlinks if they point to directories, as if they were directories;
--keep_canonical : print filly canincalized path;
-f : print complete relative path;
--base_canonical : print full canonical path of root of tree;
-i : no indentation (useful if -f o --keep_canonical;
-c : colorize output;
-p : print files' type and permissions;
-num_perms : print permission in numerical format
-s : print files' sizes in bytes
--hsize_ib : print file size in bytes, converted in human readable format : K,M,G... (powers of 10^3)
--hsize : print file size in bytes, converted in human readable format : Ki,Mi,Gi... (powers of 2^10)
--fast_rcs : ignore cycle avoidance, faster byt risks symlink cycles
--ladv : cycle avoidance
-L <usize> : set (max) depth of iteration to <usize>
--filelimit <usize> : do not descend into directories with more than <usize> entries
-o <outpath(string)> : save output to <outpath>


This was the mandatory project for the "Foundamentals of Rust Programming" course by Prof. Luca Abeni , at Sant'Anna School of Advanced Studies (Pisa), 2022-2023.

by Jacopo Carlon
