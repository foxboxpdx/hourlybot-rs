use std::path::{PathBuf, Path};
use std::fs;
use rand::Rng;
use std::fs::File;
use std::io::{BufReader, BufRead};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

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
        let statefilename = format!("/tmp/{}.statefile", self.name);
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

    // A really simple deduper that hashes each file and dumps out any with matching hashes
    // Note: We could reduce the time spent and the number of hashes calculated by grabbing the 
    // sizes of each file and comparing them first.  Any that match could then be hashed and compared
    // to see if they are dupes.
    pub fn simple_dedupe(&self, mark: bool) {
        println!("Checking for duplicates...({} files)", self.filenames.len());
        let mut hash_hash: HashMap<String, String> = HashMap::new();
        let mut dupes = Vec::new();

        // Loop through the list
        for f in &self.filenames {
            // God this is ugly
            let fname = f.file_name().expect("WTF").to_string_lossy().to_string();

            // Create a SHA-256 "hasher"
            let mut hasher = Sha256::new();
            let mut file = fs::File::open(&f).expect("Couldn't read file");

            let _bytes_written = std::io::copy(&mut file, &mut hasher).expect("Couldn't read bytes");
            let hash_bytes = hasher.finalize();

            // The easiest way to see if there's a file with the same hash is to use the hash as the
            // key in the map.
            let finalized = format!("{:x}", hash_bytes);
            if hash_hash.contains_key(&finalized) {
                dupes.push(format!("{} <=> {}", hash_hash.get(&finalized).unwrap(), &fname));
                // If the 'mark' bool is set, tack 'dup' on the end of the filenames
                if mark {
                    let newname = format!("{}.DUP", f.to_string_lossy());
                    fs::rename(f, Path::new(&newname)).expect("Couldn't rename duplicate");
                }
            }

            // Finalize the hash and stick it in the hash hashmap
            hash_hash.insert(finalized, fname);
        }
        // Now that we're done, sort that dupes vec to make things a little easier and print it
        dupes.sort();
        println!("Found {} duplicates.", dupes.len());
        println!("Done!");
    }
}