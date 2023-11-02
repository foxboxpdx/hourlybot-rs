use tokio_cron_scheduler::{JobScheduler, Job};
use mastodon_async::{Data, Mastodon, Result, StatusBuilder, Visibility};
use mastodon_async::helpers::toml;

//use std::env;
use std::path::{PathBuf, Path};
use std::fs;
use rand::Rng;
use std::fs::File;
use std::io::{BufReader, BufRead};

async fn run() -> Result<()> {
    // Step 1: Build an ImageList
    let mut images = ImageList::from_dir("images/");

    // Step 2: Pick a file to post
    let input = images.select();

    // Step 3: Build a Mastodon client instance
    let _data = Data::default();
    let data = match toml::from_file("mastodon-data.toml") {
        Ok(x) => { x },
        Err(e) => { panic!("Couldn't read toml file: {}", e); }
    };
    let mastodon = Mastodon::from(data);

    let description = None;

    // set 'input' to the image path (String)
    // set 'description' to None or Some(String)

    let media = mastodon.media(input, description).await?;
    let media = mastodon
        .wait_for_processing(media, Default::default())
        .await?;
    println!("media upload available at: {}", media.url);
    let status = StatusBuilder::new()
        .status("Posted by hourlybot-rs")
        .media_ids([media.id])
        .visibility(Visibility::Private)
        .build()?;
    let status = mastodon.new_status(status).await?;
    println!("successfully uploaded status. It has the ID {}.", status.id);
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut sched = JobScheduler::new().await.expect("Couldn't init scheduler");

    let jja = Job::new_async("0 0 * * * *", move |_uuid, _l| {
        Box::pin(async move {
            run().await.unwrap();
        })
    })
    .unwrap();
    sched.add(jja).await.expect("Couldn't add job to scheduler");
  

    #[cfg(feature = "signal")]
    sched.shutdown_on_ctrl_c();

    sched.set_shutdown_handler(Box::new(|| {
      Box::pin(async move {
        println!("Shut down done");
      })
    }));

    sched.start().await.expect("Couldn't start scheduler");
  
    // Wait a while so that the jobs actually run
    tokio::time::sleep(core::time::Duration::from_secs(100)).await;
}

// A Struct for tracking images and whether something's been used lately
// This whole thing is super overkill but I couldn't figure out how to 
// do it all in memory with the JobScheduler thing all movin' data through
// inner closures or whatever.
struct ImageList {
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
            name: "Images".to_string(),
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
            let potential = &self.filenames[choice].to_str().expect("Why").to_string();
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