use git2::{Error, Oid, Repository};
use std::collections::HashSet;
use std::fmt;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(short = "p")]
    path: String,
    #[structopt(short = "a")]
    branch2: String,
    #[structopt(short = "b")]
    branch1: Option<String>,
    #[structopt(short = "w")]
    ws: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::from_args();
    let branch1 = args.branch1.unwrap_or(String::from("master"));
    if args.ws {
        let path = std::path::Path::new(&args.path);
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                let dir = entry.unwrap();
                if dir.path().is_dir() {
                    match common(dir.path().to_str().unwrap(), &branch1, &args.branch2) {
                        Ok(a) => println!("{:?}: {}", dir.path(), a),
                        Err(e) => eprintln!("ERROR: {:?}: {}", dir.path(), e),
                    }
                }
            }
        }
    } else {
        let a = common(&args.path, &branch1, &args.branch2)?;
        println!("{}", a);
    }
    Ok(())
}

fn common(path: &str, branch1: &str, branch2: &str) -> Result<Common, Error> {
    let repo = Repository::open(path)?;
    let a = find_common(&repo, branch1, branch2)?;
    Ok(a)
}

#[derive(Debug)]
struct Common {
    oid: Oid,
    relation: Relation,
}

impl Common {
    fn new(oid: Oid, relation: Relation) -> Common {
        Common { oid, relation }
    }
}

impl fmt::Display for Common {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.relation {
            Relation::Same => write!(f, "Same: {}", self.oid),
            Relation::Inrow(ref s) => write!(f, "Inrow: {}, branch '{}' is ahead", self.oid, s),
            Relation::Diff => write!(f, "Diff: {}", self.oid),
        }
    }
}

#[derive(Debug)]
enum Relation {
    Same,
    Inrow(String),
    Diff,
}

fn get_commit_set(repo: &Repository, branch: Oid) -> Result<HashSet<Oid>, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push(branch)?;
    let mut res = HashSet::new();
    for i in revwalk {
        res.insert(i.unwrap());
    }
    Ok(res)
}

fn get_oid(repo: &Repository, branch: &str) -> Result<Oid, Error> {
    Ok(repo
        .find_branch(branch, git2::BranchType::Local)?
        .get()
        .target()
        .unwrap())
}

fn find_common(repo: &Repository, branch1: &str, branch2: &str) -> Result<Common, Error> {
    let b1 = get_oid(repo, branch1)?;
    let b2 = get_oid(repo, branch2)?;
    if b1 == b2 {
        return Ok(Common::new(b1, Relation::Same));
    }
    let b1_set = get_commit_set(&repo, b1)?;
    if b1_set.contains(&b2) {
        return Ok(Common::new(b2, Relation::Inrow(branch1.to_owned())));
    }
    let b2_set = get_commit_set(&repo, b2)?;
    if b2_set.contains(&b1) {
        return Ok(Common::new(b1, Relation::Inrow(branch2.to_owned())));
    }
    let mut revwalk = repo.revwalk()?;
    revwalk.push(b1)?;
    for oid in revwalk {
        let oid = oid?;
        if b2_set.contains(&oid) {
            return Ok(Common::new(oid, Relation::Diff));
        }
    }
    Err(Error::from_str("No common commit"))
}
