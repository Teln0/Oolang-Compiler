pub struct StringTree {
    name: String,
    branches: Vec<StringTree>,
}

impl StringTree {
    pub fn new(name: String) -> Self {
        StringTree {
            name,
            branches: vec![],
        }
    }

    pub fn add_tree_branch(&mut self, branch: StringTree) {
        self.branches.push(branch);
    }

    pub fn add_branch(&mut self, name: &str) {
        self.branches.push(StringTree::new(name.to_string()));
    }

    pub fn dump(&self) {
        self.dump_branch("".to_string(), "".to_string())
    }

    fn dump_branch(&self, pd: String, pc: String) {
        println!("{}{}", pd, self.name);
        if self.branches.is_empty() {
            return;
        }
        let n = self.branches.len() - 1;
        for i in 0..=n {
            self.branches[i].dump_branch(
                format!("{}{}", pc, if i == n { "└ " } else { "├ " }),
                format!("{}{}", pc, if i == n { "  " } else { "│ " }),
            );
        }
    }
}
