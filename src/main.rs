use std::{cmp::min, env, fs::OpenOptions, io::{ErrorKind, Write}};
use indicatif::{ProgressBar, ProgressStyle};
use futures::StreamExt;

mod youtube;

#[tokio::main]
async fn main() {
    // todo:
    // download videos
    // allow user to specify output path & file name

    let args = env::args().collect::<Vec<_>>();
    let url = &args[1];

    // downloading file
    let target_file = youtube::get_download_url(&url).await;

    match target_file {
        Ok(target_file) => {
            let res = reqwest::get(target_file.url).await.unwrap();
            let file_size = res.content_length().unwrap_or(0);
            let mut stream = res.bytes_stream();

            let pb = ProgressBar::new(file_size)
                .with_message(format!("Downloading '{}'", target_file.name))
                .with_style(ProgressStyle::default_bar()
                    .template("{msg}\n[{elapsed_precise}] [{bar:40.green/blue}] {bytes}/{total_bytes}")
                    .progress_chars("#-"));


            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target_file.name);

            match file {
                Ok(mut file) => {
                    let mut downloaded = 0;

                    while let Some(item) = stream.next().await {
                        let item = item.unwrap();

                        file.write(&item).unwrap();
                        downloaded = min(downloaded + (item.len() as u64), file_size);
                        pb.set_position(downloaded);
                    }
                }
                Err(error) => {
                    let message = match error.kind() {
                        ErrorKind::AlreadyExists => "File with same name already exists!".to_owned(),
                        ErrorKind::PermissionDenied => "No permission to create file!".to_owned(),
                        _ => format!("Unhandled error\n{:?}", error)
                    };

                    eprintln!("{message}");
                }
            }
        },
        Err(error) => eprintln!("{}", error.to_string())
    }
}
