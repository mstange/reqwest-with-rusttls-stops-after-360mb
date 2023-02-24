#[tokio::main(flavor = "current_thread")]
async fn main() {
    eprintln!("Downloading chrome.dll.pdb...");
    let url = "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb";

    let client_builder = reqwest::Client::builder();
    let client = client_builder
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .build()
        .unwrap();
    let builder = client.get(url);
    let builder = builder.header("Accept-Encoding", "gzip");
    let mut response = builder.send().await.unwrap();

    let mut downloaded_len = 0;
    let mut last_reported_len = 0;
    while let Ok(Some(chunk)) = response.chunk().await {
        downloaded_len += chunk.len();
        if downloaded_len >= last_reported_len + 1_000_000 {
            eprintln!("Downloaded {downloaded_len} bytes.");
            last_reported_len = downloaded_len;
        }
    }

    eprintln!("Done! Downloaded {downloaded_len} bytes in total.");
}
