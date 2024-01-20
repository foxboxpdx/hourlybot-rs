use tokio_cron_scheduler::{JobScheduler, Job};
use mastodon_async::{Mastodon, Result, StatusBuilder, Visibility};
use mastodon_async::helpers::toml;
use hourlybot_rs::ImageList;
use clap::Parser;

// Clap struct for command line arguments
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Base directory to load images from
    #[clap(short, long, required = true)]
    basedir: String,

    /// How often should the bot post?
    #[clap(value_enum, required_unless_present("dedupe"))]
    freq: Frequency,

    /// Mastodon configuration file
    #[clap(short, long, required_unless_present("dedupe"))]
    config: String,

    /// Run the deduper and exit
    #[clap(short, long)]
    dedupe: bool,

    /// When running the deduper, add '.DUP' suffix to suspected duplicates
    #[clap(short, long)]
    markdupes: bool,
}

// Clap enum for allowed frequency values
#[derive(clap::ValueEnum, Clone, Debug)]
enum Frequency {
   TopOfHour,
   BottomOfHour,
   OnceDaily,
   TwiceDaily,
   FourTimesDaily,
   SixTimesDaily,
}

// Perform steps necessary to attach and upload an image, then
// post a toot pointing to that image.  Try to fail gracefully
// whenever possible by printing an error and returning.
async fn run(basedir: String, config: String) -> Result<()> {
    // Step 1: Build an ImageList.  This should really only
    // have to happen once per execution but tokio-scheduler is a 
    // punk bitch.
    let mut images = ImageList::from_dir(&basedir);

    // Step 2: Pick a file to post
    let input = images.select();

    // Step 3: Build a Mastodon client instance
    // This needs to panic on error; if we can't read the
    // config file, we can't do squat.
    let data = match toml::from_file(&config) {
        Ok(x) => { x },
        Err(e) => { panic!("Couldn't read toml config file: {}", e); }
    };
    let mastodon = Mastodon::from(data);

    // If you want there to be a description accompanying the image,
    // set this to Some(String); otherwise set to None.
    let description = None;

    // Step 4: Try to load the selected image file
    let attach = match mastodon.media(&input, description).await {
        Ok(x) => x,
        Err(e) => { 
            println!("Error attaching media: {:?}", e);
            return Ok(());
        }
    };

    // Step 5: Wait for the image file to be uploaded and processed
    let media = match mastodon.wait_for_processing(attach, Default::default()).await {
        Ok(x) => {
            println!("Attachment uploaded to: {}", x.url);
            x
        },
        Err(e) => {
            println!("Error processing attachment: {:?}", e);
            return Ok(());
        }
    };

    // Step 6: Build a NewStatus to post
    let status = match StatusBuilder::new().status("").media_ids([media.id]).visibility(Visibility::Public).build() {
        Ok(x) => x,
        Err(e) => {
            println!("Error building NewStatus: {:?}", e);
            return Ok(());
        }
    };

    // Step 7: Toot!
    match mastodon.new_status(status).await {
        Ok(x) => {
            println!("Toot successfully posted with id: {}", x.id);
        },
        Err(e) => {
            println!("Error posting toot: {:?}", e);
        }
    };

    // Return
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Short-circuit if dedupe was specified
    if args.dedupe {
        let images = ImageList::from_dir(&args.basedir);
        images.simple_dedupe(args.markdupes);
        return;
    }

    // Set up the cron string based on the specified frequency
    let freq = match args.freq {
        Frequency::TopOfHour => "0 0 * * * *",
        Frequency::BottomOfHour => "0 30 * * * *",
        Frequency::OnceDaily => "0 0 0 * * *",
        Frequency::TwiceDaily => "0 0 0,12 * * *",
        Frequency::FourTimesDaily => "0 0 0,6,12,18 * * *",
        Frequency::SixTimesDaily => "0 0 0,4,8,12,16,20 * * *" 
    };
    let base = args.basedir.clone();
    let conf = args.config.clone();

    // Startup status
    println!("hourlybot-rs starting up...");
    println!("basedir: {}\nfrequency: {:?}\nconfig file: {}", &base, args.freq, &conf);

    let mut sched = JobScheduler::new().await.expect("Couldn't init scheduler");

    let poster = Job::new_async(freq, move |_uuid, _l| {
        let foo = base.clone();
        let bar = conf.clone();
        Box::pin(async move {
            run(foo, bar).await.unwrap();
        })
    })
    .unwrap();
    sched.add(poster).await.expect("Couldn't add job to scheduler");
  
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
