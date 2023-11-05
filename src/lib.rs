use std::path::{PathBuf, Path};
use std::fs;
use rand::Rng;
use std::fs::File;
use std::io::{BufReader, BufRead};

// A Struct for tracking images and whether something's been used lately
// This whole thing is super overkill but I couldn't figure out how to 
// do it all in memory with the JobScheduler thing all movin' data through
// inner closures or whatever.
pub struct ImageList {
    name: String,
    filenames: Vec<PathBuf>,
    used: Vec<String>
}

impl ImageList {
    pub fn from_dir(basedir: &str) -> ImageList {
        let fnames: Vec<PathBuf> = match fs::read_dir(basedir) {
            Ok(entries) => entries.filter_map(|e| e.ok())
                .map(|e| e.path()).collect(),
            Err(e) => {
                panic!("Error reading dir: {}!\nError was: {}", basedir, e);
            }
        };
        let mut newlist = ImageList {
            // Get name from basedir
            name: basedir.chars().filter(|c| c.is_alphanumeric()).collect(),
            filenames: fnames,
            used: Vec::new()
        };
        newlist.state_sync(false);
        return newlist;
    }

    pub fn select(&mut self) -> String {
        if self.filenames.len() - self.used.len() < 2 { self.used.clear(); }
        let mut rng = rand::thread_rng();
        loop {
            let choice: usize = rng.gen_range(0..self.filenames.len());
            let potential = &self.filenames[choice].to_str()
                .expect("Couldn't to_str a filename what???").to_string();
            if !self.used.contains(&potential) {
                self.used.push(potential.to_string());
                self.state_sync(true);
                return potential.to_string();
            }
        }
    }

    pub fn state_sync(&mut self, write: bool) {
        let statefilename = format!("{}.statefile", self.name);
        if write {
            fs::write(statefilename, self.used.join("\n")).expect("Error writing statefile");
            return;
        }
        if Path::new(&statefilename).exists() {
            let file = File::open(statefilename).expect("Couldn't open statefile for reading");
            let state: Vec<String> = match BufReader::new(file).lines().collect() {
                Ok(x) => { x },
                Err(e) => { panic!("Couldn't read statefile: {}", e); }
            };
            self.used = state;
        }
    }
}