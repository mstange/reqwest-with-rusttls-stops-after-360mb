# `reqwest-with-rusttls-stops-after-360mb`

This is a testcase for https://github.com/seanmonstar/reqwest/issues/1761.
It reproduces a bug with reqwest and the Chrome symbol server: The file stops downloading after 360MB. It is 640MB in total.

The bug only occurs when using HTTP/2.

The bug also only occurs when downloading the gzip-compressed version of the file.

## Steps to reproduce:

 1. Clone this repo.
 2. Inside the local clone, run `cargo run --release`
 3. Wait for a 640MB download to complete.

## Expected results:

The program should complete and print `Done! Downloaded 639588034 bytes in total.`

## Actual results:

The program stops downloading after around 360MB. The last printed line is usually something like `Downloaded 361462551 bytes.`
And then the `response.chunk()` future just never completes.

## Workarounds

Any of the following changes make the download complete successfully:

 - Change `https` into `http`.
 - Remove the `Accept-Encoding: gzip` header.
 - Edit `Cargo.toml` to use `"default-tls"` instead of `"rustls-tls"` (causes HTTP/1.1 to be used)
 - Call `.http1_only()` on the client builder.

The file can be downloaded successfully with `curl -o chrome.dll.pdb --compressed "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb"`, which also uses HTTP/2. It decompresses to a 3GB file.

## System configuration

I'm hitting this bug on macOS 13.1, with Rust 1.67.0.
