use tokio_cron_scheduler::{JobScheduler, Job};
use mastodon_async::{Data, Mastodon, Result, StatusBuilder, Visibility};
use mastodon_async::helpers::toml;
use hourlybot_rs::ImageList;
use clap::Parser;

// Clap struct for command line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Base directory to load images from
    #[clap(short, long)]
    basedir: String,

    /// Posting Frequency
    #[clap(value_enum)]
    freq: Frequency,
}

// Clap enum for allowed frequency values
#[derive(clap::ValueEnum, Clone)]
enum Frequency {
   TopOfHour,
   BottomOfHour,
   OnceDaily,
   TwiceDaily,
   FourTimesDaily,
   SixTimesDaily,
}

//use std::env;

async fn run(basedir: String) -> Result<()> {
    // Step 1: Build an ImageList
    let mut images = ImageList::from_dir(&basedir);

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
        .status("")
        .media_ids([media.id])
        .visibility(Visibility::Public)
        .build()?;
    let status = mastodon.new_status(status).await?;
    println!("successfully uploaded status. It has the ID {}.", status.id);
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let freq = match args.freq {
        Frequency::TopOfHour => "0 0 * * * *",
        Frequency::BottomOfHour => "0 30 * * * *",
        Frequency::OnceDaily => "0 0 0 * * *",
        Frequency::TwiceDaily => "0 0 0,12 * * *",
        Frequency::FourTimesDaily => "0 0 0,6,12,18 * * *",
        Frequency::SixTimesDaily => "0 0 0,4,8,12,16,20 * * *" 
    };
    let base = args.basedir.clone();

    let mut sched = JobScheduler::new().await.expect("Couldn't init scheduler");

    let jja = Job::new_async(freq, move |_uuid, _l| {
        let foo = base.clone();
        Box::pin(async move {
            run(foo).await.unwrap();
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
  
    // Now we just sleeploop till killed
    loop {
        tokio::time::sleep(core::time::Duration::from_secs(3600)).await;
        println!("Woke up after an hour, back to sleep.")
    }
}
