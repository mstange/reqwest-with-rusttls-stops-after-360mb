// mod decoder;

use async_compat::CompatExt;
use async_compression::futures::bufread::GzipDecoder;
use futures::{io::BufReader, AsyncRead, TryStreamExt};
use reqwest::header::{AsHeaderName, HeaderMap, CONTENT_ENCODING, CONTENT_LENGTH};
use std::{cell::RefCell, pin::Pin, task::Poll};
use tokio::io::AsyncWriteExt;

fn get_header<K: AsHeaderName>(headers: &HeaderMap, name: K) -> Option<String> {
    Some(headers.get(name)?.to_str().ok()?.to_ascii_lowercase())
}

enum TotalSize {
    Compressed(u64),
    Uncompressed(u64),
}

fn get_total_size(headers: &HeaderMap) -> Option<TotalSize> {
    let response_encoding = get_header(headers, CONTENT_ENCODING);
    let content_length =
        get_header(headers, CONTENT_LENGTH).and_then(|value| value.parse::<u64>().ok());

    // If the server sends a Content-Length header, use the size from that header.
    match content_length {
        Some(len) if len > 0 => {
            let total_size = match response_encoding.as_deref() {
                None => TotalSize::Uncompressed(len),
                Some(_) => TotalSize::Compressed(len),
            };
            return Some(total_size);
        }
        _ => {}
    }

    // Add a fallback for Google Cloud servers which use Transfer-Encoding: chunked with
    // HTTP/1.1 and thus do not include a Content-Length header.
    // This is the case for https://chromium-browser-symsrv.commondatastorage.googleapis.com/
    // (the Chrome symbol server) as of February 2023.
    if response_encoding.as_deref() == Some("gzip") {
        if let (Some("gzip"), Some(len)) = (
            get_header(headers, "x-goog-stored-content-encoding").as_deref(),
            get_header(headers, "x-goog-stored-content-length")
                .and_then(|value| value.parse::<u64>().ok()),
        ) {
            return Some(TotalSize::Compressed(len));
        }
    }

    // Add another fallback for AWS servers. I have not seen a case where this is necessary,
    // but it doesn't hurt either.
    if let Some(len) =
        get_header(headers, "x-amz-meta-original_size").and_then(|value| value.parse::<u64>().ok())
    {
        return Some(TotalSize::Uncompressed(len));
    }

    None
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("Unexpected Content-Encoding header: {0}")]
    UnexpectedContentEncoding(String),
}

fn response_to_uncompressed_stream_with_progress<F>(
    response: reqwest::Response,
    progress: F,
) -> Result<Pin<Box<dyn AsyncRead>>, Error>
where
    F: FnMut(u64, Option<u64>) + 'static,
{
    let headers = response.headers();
    let response_encoding = get_header(headers, CONTENT_ENCODING);
    let total_size = get_total_size(headers);

    let stream = response.bytes_stream();
    let async_read = stream
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        .into_async_read();

    match (response_encoding.as_deref(), total_size) {
        (Some("gzip"), Some(TotalSize::Uncompressed(len))) => {
            let async_buf_read = BufReader::new(async_read);
            let decoder = GzipDecoder::new(async_buf_read);
            let decoder_with_progress = decoder.with_progress(progress, Some(len));
            Ok(Box::pin(decoder_with_progress))
        }
        (Some("gzip"), Some(TotalSize::Compressed(len))) => {
            let async_read_with_progress = async_read.with_progress(progress, Some(len));
            let async_buf_read = BufReader::new(async_read_with_progress);
            let decoder = GzipDecoder::new(async_buf_read);
            Ok(Box::pin(decoder))
        }
        (Some("gzip"), None) => {
            let async_read_with_progress = async_read.with_progress(progress, None);
            let async_buf_read = BufReader::new(async_read_with_progress);
            let decoder = GzipDecoder::new(async_buf_read);
            Ok(Box::pin(decoder))
        }
        (Some(other_encoding), _) => {
            Err(Error::UnexpectedContentEncoding(other_encoding.to_string()))
        }
        (None, Some(TotalSize::Uncompressed(len))) => {
            Ok(Box::pin(async_read.with_progress(progress, Some(len))))
        }
        (None, _) => Ok(Box::pin(async_read.with_progress(progress, None))),
    }
}

trait AsyncReadObserver {
    fn did_read(&self, len: u64);
}

struct ProgressNotifierData<F: FnMut(u64, Option<u64>)> {
    progress_fun: F,
    total_size: Option<u64>,
    accumulated_size: u64,
}

struct ProgressNotifier<F: FnMut(u64, Option<u64>)>(RefCell<ProgressNotifierData<F>>);

impl<F: FnMut(u64, Option<u64>)> AsyncReadObserver for ProgressNotifier<F> {
    fn did_read(&self, len: u64) {
        let mut data = self.0.borrow_mut();
        data.accumulated_size += len;
        let accum = data.accumulated_size;
        let total_size = data.total_size;
        match total_size {
            Some(total_size) if accum <= total_size => (data.progress_fun)(accum, Some(total_size)),
            _ => (data.progress_fun)(accum, None),
        }
    }
}

impl<F: FnMut(u64, Option<u64>)> ProgressNotifier<F> {
    pub fn new(progress_fun: F, total_size: Option<u64>) -> Self {
        Self(RefCell::new(ProgressNotifierData {
            progress_fun,
            total_size,
            accumulated_size: 0,
        }))
    }
}

struct AsyncReadWrapper<I: AsyncRead> {
    inner: Pin<Box<I>>,
    observer: Pin<Box<dyn AsyncReadObserver>>,
}

impl<I: AsyncRead> AsyncReadWrapper<I> {
    pub fn new<O: AsyncReadObserver + 'static>(inner: I, observer: O) -> Self {
        Self {
            inner: Box::pin(inner),
            observer: Box::pin(observer),
        }
    }
}

impl<I: AsyncRead> AsyncRead for AsyncReadWrapper<I> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let res = Pin::new(&mut self.inner).poll_read(cx, buf);
        match res {
            Poll::Ready(Ok(len)) => {
                self.observer.did_read(len as u64);
                Poll::Ready(Ok(len))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

trait WithProgress: AsyncRead + Sized {
    fn with_progress<F: FnMut(u64, Option<u64>) + 'static>(
        self,
        progress_fun: F,
        total_size: Option<u64>,
    ) -> AsyncReadWrapper<Self>;
}

impl<AR: AsyncRead + Sized> WithProgress for AR {
    fn with_progress<F: FnMut(u64, Option<u64>) + 'static>(
        self,
        progress_fun: F,
        total_size: Option<u64>,
    ) -> AsyncReadWrapper<Self> {
        AsyncReadWrapper::new(self, ProgressNotifier::new(progress_fun, total_size))
    }
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // let url = "https://symbols.mozilla.org/XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym";
    // let url = "https://msdl.microsoft.com/download/symbols/combase.pdb/071849A7C75FD246A3367704EE1CA85B1/combase.pdb";
    // let url = "http://msdl.microsoft.com/download/symbols/combase.pdb/071849A7C75FD246A3367704EE1CA85B1/combase.pdb";
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
    let response = builder.send().await.unwrap();

    let mut file = tokio::fs::File::create(
        "/Users/mstange/sym/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb",
    )
    .await
    .unwrap();

    let mut previous_reported = 0;

    let mut decompressed_async_read =
        response_to_uncompressed_stream_with_progress(response, move |accum, total| {
            if accum >= previous_reported + 100000 {
                previous_reported = accum;
                match total {
                    Some(total) => eprintln!("Downloaded {accum} of {total} bytes."),
                    None => eprintln!("Downloaded {accum} bytes."),
                }
            }
        })
        .unwrap();

    futures::io::copy(&mut decompressed_async_read, &mut file.compat_mut())
        .await
        .unwrap();
    file.flush().await.unwrap();
    drop(file);
}
