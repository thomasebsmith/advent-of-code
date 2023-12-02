use std::collections::HashMap;
use std::io;
use std::io::BufRead;

use crate::errors::invalid_input;
use crate::iter::n_elements;
use crate::part::Part;

type Path = Vec<String>;
type PathRef<'a> = &'a [String];

enum FSItem {
    Directory(Directory),
    File(File),
}

impl FSItem {
    fn size(&self) -> usize {
        match self {
            Self::Directory(ref directory) => directory.size_of_children(),
            Self::File(ref file) => file.size(),
        }
    }
}

struct FSItemWalker<'a> {
    iters: Vec<std::collections::hash_map::Iter<'a, String, FSItem>>,
}

impl<'a> Iterator for FSItemWalker<'a> {
    type Item = (&'a str, &'a FSItem);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iters.last_mut() {
                Some(last_iter) => match last_iter.next() {
                    Some((name, item)) => {
                        if let FSItem::Directory(ref dir) = item {
                            self.iters.push(dir.children.iter());
                        }
                        return Some((name, item));
                    }
                    None => {
                        self.iters.pop();
                    }
                },
                None => return None,
            }
        }
    }
}

struct Directory {
    children: HashMap<String, FSItem>,
}

impl Directory {
    fn size_of_children(&self) -> usize {
        self.children.values().map(FSItem::size).sum()
    }

    fn set_children(&mut self, children: HashMap<String, FSItem>) {
        self.children = children;
    }

    fn new() -> Self {
        Directory {
            children: HashMap::new(),
        }
    }

    fn walk(&self) -> FSItemWalker<'_> {
        FSItemWalker {
            iters: vec![self.children.iter()],
        }
    }

    fn navigate_mut_dir(
        &mut self,
        path: PathRef,
    ) -> io::Result<&mut Directory> {
        let Some(child_name) = path.first() else {
            return Ok(self);
        };
        let Some(child) = self.children.get_mut(child_name) else {
            return Err(invalid_input(format!(
                "No child with name {} found",
                child_name
            )));
        };
        match child {
            FSItem::File(_) => Err(invalid_input("Cannot navigate to file")),
            FSItem::Directory(ref mut dir) => dir.navigate_mut_dir(&path[1..]),
        }
    }
}

struct File {
    size: usize,
}

impl File {
    fn size(&self) -> usize {
        self.size
    }
}

fn update_with_ls_output(
    fs: &mut Directory,
    path: &[String],
    ls_output: HashMap<String, FSItem>,
) -> io::Result<()> {
    let cur_dir = fs.navigate_mut_dir(path)?;
    cur_dir.set_children(ls_output);

    Ok(())
}

pub fn run<R: io::Read>(
    part: Part,
    reader: io::BufReader<R>,
) -> io::Result<()> {
    let mut fs = Directory::new();

    let mut cur_path = Path::new();

    let mut in_ls_output = false;
    let mut ls_output = HashMap::<String, FSItem>::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with('$') {
            if in_ls_output {
                update_with_ls_output(&mut fs, cur_path.as_slice(), ls_output)?;
                ls_output = HashMap::new();
                in_ls_output = false;
            }

            let command = &line[2..4];
            match command {
                "cd" => {
                    let arg = &line[5..];
                    match arg {
                        "/" => {
                            cur_path.clear();
                        }
                        ".." => {
                            if cur_path.pop().is_none() {
                                Err(invalid_input("Cannot cd .. at top level"))?
                            }
                        }
                        _ => {
                            cur_path.push(String::from(arg));
                        }
                    }
                }
                "ls" => {
                    in_ls_output = true;
                }
                _ => Err(invalid_input("Unknown command"))?,
            }
        } else if in_ls_output {
            if line.starts_with("dir ") {
                ls_output.insert(
                    String::from(&line[4..]),
                    FSItem::Directory(Directory::new()),
                );
            } else {
                let words =
                    n_elements(2, line.split(' ')).ok_or_else(|| {
                        invalid_input("Expecting exactly 2 words on file lines")
                    })?;
                let size: usize = words[0].parse().map_err(invalid_input)?;
                let name: &str = words[1];
                ls_output
                    .insert(String::from(name), FSItem::File(File { size }));
            }
        } else {
            Err(invalid_input(
                "Line does not start with $ and is not after ls",
            ))?
        }
    }

    if in_ls_output {
        update_with_ls_output(&mut fs, cur_path.as_slice(), ls_output)?;
    }

    match part {
        Part::Part1 => {
            let mut size_sum: usize = 0;
            for (_name, item) in fs.walk() {
                if matches!(item, FSItem::Directory(_)) {
                    let size = item.size();
                    if size <= 100_000 {
                        size_sum += size;
                    }
                }
            }
            println!("{}", size_sum);
        }
        Part::Part2 => {
            const DISK_SIZE: usize = 70_000_000;
            const NEEDED_SPACE: usize = 30_000_000;

            let free_space = DISK_SIZE - fs.size_of_children();

            if free_space >= NEEDED_SPACE {
                println!("0");
            } else {
                let to_free = NEEDED_SPACE - free_space;
                let mut min_possible_size: Option<usize> = None;
                for (_name, item) in fs.walk() {
                    if matches!(item, FSItem::Directory(_)) {
                        let size = item.size();
                        if size >= to_free {
                            match min_possible_size {
                                None => min_possible_size = Some(size),
                                Some(mps) => {
                                    if size < mps {
                                        min_possible_size = Some(size);
                                    }
                                }
                            }
                        }
                    }
                }
                match min_possible_size {
                    Some(mps) => println!("{}", mps),
                    None => {
                        Err(invalid_input("No directories are large enough"))?
                    }
                }
            }
        }
    }

    Ok(())
}
