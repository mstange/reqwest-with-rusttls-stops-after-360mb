# Conclusions from testing

 - If `Content-Encoding` is `gzip`, then `Content-Length` is the *length before decompression*.
 - The Microsoft symbol server seems to serve uncompressed files. It does not use `Content-Encoding` at all. It sets `Content-Length`.
 - The Mozilla symbol server uses `Content-Encoding: gzip` and gives a useful `Content-Length` header (which describes the compressed size, as expected).
 - The Chrome symbol server only specifies `Content-Length` when HTTP/2 is used. With HTTP/1.1, it uses `Transfer-Encoding: chunked` and cannot specify the total size.
 - curl uses HTTP/2 with the Chrome symbol server. HTTP/2 forbids `Transfer-Encoding: chunked` and gets a useful `Content-Length` header.
 - I was not able to make an HTTP/2 request using `reqwest`. I don't know why curl can do it but `reqwest` cannot. This means I was not able to make a connection using `reqwest` to the Chrome symbol server while also using compression and getting a useful `Content-Length` header.
 - When running `curl -I` I do not see the `Content-Length` response header. I only see it with `curl -v`. I don't know why.
 - The Chrome symbol server also sends these headers, specifying the compressed size:
    - `x-goog-stored-content-encoding: gzip`
    - `x-goog-stored-content-length: 639588034`
 - The `x-goog-stored-content` header are sent even with the HTTP/1.1 response, i.e. when the `Content-Length` header is missing, and even when no `Accept-Encoding` is set (i.e. when serving uncompressed files).

## Raw logs

```
 % curl --http2 -v --compressed -L "https://symbols.mozilla.org/XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym"
*   Trying 52.41.161.72:443...
* Connected to symbols.mozilla.org (52.41.161.72) port 443 (#0)
[...]
> GET /XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym HTTP/1.1
[...]
< HTTP/1.1 302 Found
[...]
< Location: https://s3.us-west-2.amazonaws.com/org.mozilla.crash-stats.symbols-public/v1/XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym?AWSAccessKeyId=AKIAIWM6JYWLRUT4R6OQ&Signature=nH%2BQFT7CjkBSbI4OksTh3EDYe1A%3D&Expires=1676782717
* Connection #0 to host symbols.mozilla.org left intact
* Issue another request to this URL: 'https://s3.us-west-2.amazonaws.com/org.mozilla.crash-stats.symbols-public/v1/XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym?AWSAccessKeyId=AKIAIWM6JYWLRUT4R6OQ&Signature=nH%2BQFT7CjkBSbI4OksTh3EDYe1A%3D&Expires=1676782717'
*   Trying 52.92.192.24:443...
* Connected to s3.us-west-2.amazonaws.com (52.92.192.24) port 443 (#1)
* ALPN: offers h2
* ALPN: offers http/1.1
[...]
* ALPN: server accepted http/1.1
[...]
> GET /org.mozilla.crash-stats.symbols-public/v1/XUL/307EB34E8F43314EBF556ECBACCE4AE70/XUL.sym?AWSAccessKeyId=AKIAIWM6JYWLRUT4R6OQ&Signature=nH%2BQFT7CjkBSbI4OksTh3EDYe1A%3D&Expires=1676782717 HTTP/1.1
> Host: s3.us-west-2.amazonaws.com
> User-Agent: curl/7.85.0
> Accept: */*
> Accept-Encoding: deflate, gzip
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< x-amz-id-2: MKLAa3Bse+ktIa99JebIj8oZJdc6BiZ9TQUEvfNz8v3m7QCfzHqCevKLn0sZTUWyQ1fH8e7jhhM=
< x-amz-request-id: RHQ7C893ZN0PJVCQ
< Date: Sun, 19 Feb 2023 03:58:39 GMT
< Last-Modified: Thu, 12 Jan 2023 23:18:48 GMT
< x-amz-expiration: expiry-date="Sun, 12 Jan 2025 00:00:00 GMT", rule-id="first_ia_then_delete"
< ETag: "bf1558547cc2b91bf0c2964424f7ef55"
< x-amz-storage-class: STANDARD_IA
< x-amz-server-side-encryption: AES256
< x-amz-meta-original_md5_hash: 2d300bf4460ca429456b0f943bba4d2a
< x-amz-meta-original_size: 526141499
< Content-Encoding: gzip
< x-amz-version-id: sL.3rR1wXPU84GyPvgq1y3UQmlX4P_4W
< Accept-Ranges: bytes
< Content-Type: text/plain
< Server: AmazonS3
< Content-Length: 90394688
```

90394688 = 90MB

526141499 = 526MB

```
% curl --http1.1 -I "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb"
HTTP/1.1 200 OK
X-GUploader-UploadID: ADPycduJoFqz4tiocOfGuaG2iRPIbKWiwDQfnsB1IFSlI14GEGXZB4aWvHc4jBa_sO53LLu6KPB_Z_Ht8rsfnkXMlANSdwmK54_i
Date: Sun, 19 Feb 2023 04:02:02 GMT
Cache-Control: no-transform
Expires: Mon, 19 Feb 2024 04:02:02 GMT
Last-Modified: Thu, 23 Sep 2021 03:32:47 GMT
x-goog-generation: 1632367967842688
x-goog-metageneration: 1
x-goog-stored-content-encoding: gzip
x-goog-stored-content-length: 639588034
Content-Type: application/octet-stream
x-goog-hash: crc32c=voDAsA==
x-goog-hash: md5=nKiS4Q5c9DtSpTemLqU6iQ==
x-goog-storage-class: STANDARD
Accept-Ranges: none
Server: UploadServer
Transfer-Encoding: chunked
Alt-Svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000,h3-Q050=":443"; ma=2592000,h3-Q046=":443"; ma=2592000,h3-Q043=":443"; ma=2592000,quic=":443"; ma=2592000; v="46,43"
Vary: Accept-Encoding
```

639588034 = 640MB

```
% curl --http2 -I "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb"
HTTP/2 200 
x-guploader-uploadid: ADPycdv9XqOZRXvJB6y3eoEbiPHRYswWlVRV-YTx7I6c76nIbZ9GXz14aE4QUrIUmv2PZUw40hFpfOTSs3U1QlGpguY5JKDtSHfd
date: Sun, 19 Feb 2023 04:02:37 GMT
cache-control: no-transform
expires: Mon, 19 Feb 2024 04:02:37 GMT
last-modified: Thu, 23 Sep 2021 03:32:47 GMT
x-goog-generation: 1632367967842688
x-goog-metageneration: 1
x-goog-stored-content-encoding: gzip
x-goog-stored-content-length: 639588034
content-type: application/octet-stream
x-goog-hash: crc32c=voDAsA==
x-goog-hash: md5=nKiS4Q5c9DtSpTemLqU6iQ==
x-goog-storage-class: STANDARD
accept-ranges: none
server: UploadServer
alt-svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000,h3-Q050=":443"; ma=2592000,h3-Q046=":443"; ma=2592000,h3-Q043=":443"; ma=2592000,quic=":443"; ma=2592000; v="46,43"
vary: Accept-Encoding
```

```
% curl --http2 -I --compressed "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb"
HTTP/2 200 
x-guploader-uploadid: ADPycduttt97U1ATcxorlEIBMci8sJziIPs1CjrWShGersCDwlHVQsygBbAmPaZ6w1uEKv95SS5AgkKV97jJe-IpoCyn0YgWy7_D
date: Sun, 19 Feb 2023 04:05:05 GMT
cache-control: no-transform
expires: Mon, 19 Feb 2024 04:05:05 GMT
last-modified: Thu, 23 Sep 2021 03:32:47 GMT
etag: "9ca892e10e5cf43b52a537a62ea53a89"
x-goog-generation: 1632367967842688
x-goog-metageneration: 1
x-goog-stored-content-encoding: gzip
x-goog-stored-content-length: 639588034
content-type: application/octet-stream
content-encoding: gzip
x-goog-hash: crc32c=voDAsA==
x-goog-hash: md5=nKiS4Q5c9DtSpTemLqU6iQ==
x-goog-storage-class: STANDARD
accept-ranges: bytes
server: UploadServer
alt-svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000,h3-Q050=":443"; ma=2592000,h3-Q046=":443"; ma=2592000,h3-Q043=":443"; ma=2592000,quic=":443"; ma=2592000; v="46,43"
```

```
% curl --http2 -v --compressed "https://chromium-browser-symsrv.commondatastorage.googleapis.com/chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb"
*   Trying 142.251.41.48:443...
* Connected to chromium-browser-symsrv.commondatastorage.googleapis.com (142.251.41.48) port 443 (#0)
* ALPN: offers h2
* ALPN: offers http/1.1
[...]
* Using HTTP2, server supports multiplexing
* Copying HTTP/2 data in stream buffer to connection buffer after upgrade: len=0
* h2h3 [:method: GET]
* h2h3 [:path: /chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb]
* h2h3 [:scheme: https]
* h2h3 [:authority: chromium-browser-symsrv.commondatastorage.googleapis.com]
* h2h3 [user-agent: curl/7.85.0]
* h2h3 [accept: */*]
* h2h3 [accept-encoding: deflate, gzip]
* Using Stream ID: 1 (easy handle 0x148011e00)
> GET /chrome.dll.pdb/93B17FC546DE07D14C4C44205044422E1/chrome.dll.pdb HTTP/2
> Host: chromium-browser-symsrv.commondatastorage.googleapis.com
> user-agent: curl/7.85.0
> accept: */*
> accept-encoding: deflate, gzip
> 
< HTTP/2 200
< x-guploader-uploadid: ADPycduZcMa6ov3sFVpuzjjSyXoNZxw_3IuA1euWRtBESVXJTteqzDjUnbu0hGmrY-f6uasTUMiO22P9xAB-mK6OGei__DRjr8Z4
< date: Sun, 19 Feb 2023 04:06:16 GMT
< cache-control: no-transform
< expires: Mon, 19 Feb 2024 04:06:16 GMT
< last-modified: Thu, 23 Sep 2021 03:32:47 GMT
< etag: "9ca892e10e5cf43b52a537a62ea53a89"
< x-goog-generation: 1632367967842688
< x-goog-metageneration: 1
< x-goog-stored-content-encoding: gzip
< x-goog-stored-content-length: 639588034
< content-type: application/octet-stream
< content-encoding: gzip
< x-goog-hash: crc32c=voDAsA==
< x-goog-hash: md5=nKiS4Q5c9DtSpTemLqU6iQ==
< x-goog-storage-class: STANDARD
< accept-ranges: bytes
< content-length: 639588034
< server: UploadServer
< alt-svc: h3=":443"; ma=2592000,h3-29=":443"; ma=2592000,h3-Q050=":443"; ma=2592000,h3-Q046=":443"; ma=2592000,h3-Q043=":443"; ma=2592000,quic=":443"; ma=2592000; v="46,43"
< 
Warning: Binary output can mess up your terminal. Use "--output -" to tell
Warning: curl to output it to your terminal anyway, or consider "--output 
Warning: <FILE>" to save to a file.
* Failure writing output to destination
* Connection #0 to host chromium-browser-symsrv.commondatastorage.googleapis.com left intact
```

```
% curl -vv -L --compressed "http://msdl.microsoft.com/download/symbols/ntkrnlmp.pdb/4738AC02284CCC451E86A727E513D1131/ntkrnlmp.pdb"
*   Trying 204.79.197.219:80...
* Connected to msdl.microsoft.com (204.79.197.219) port 80 (#0)
> GET /download/symbols/ntkrnlmp.pdb/4738AC02284CCC451E86A727E513D1131/ntkrnlmp.pdb HTTP/1.1
[...]
< HTTP/1.1 302 Found
< Location: https://vsblobprodscussu5shard26.blob.core.windows.net/b-4712e0edc5a240eabf23330d7df68e77/84AC477D263AE74E21A0CF0E6ADF04AFE0E5833741616864F2A3E3E3EBD26EBE00.blob?sv=2019-07-07&sr=b&si=1&sig=tLlXAxydN6D4NfS4nvuieL3vAt4hHYEu2UqL8cCmPHc%3D&spr=https&se=2023-02-20T04%3A31%3A05Z&rscl=x-e2eid-e4e08c6b-b6384600-907dabed-0f27f1f5-session-bc6d7d4a-45804b9d-b857f518-397ae64d
[...]
* Connected to vsblobprodscussu5shard26.blob.core.windows.net (20.150.39.196) port 443 (#1)
* ALPN: offers h2
* ALPN: offers http/1.1
[...]
* ALPN: server did not agree on a protocol. Uses default.
[...]
> GET /b-4712e0edc5a240eabf23330d7df68e77/84AC477D263AE74E21A0CF0E6ADF04AFE0E5833741616864F2A3E3E3EBD26EBE00.blob?sv=2019-07-07&sr=b&si=1&sig=tLlXAxydN6D4NfS4nvuieL3vAt4hHYEu2UqL8cCmPHc%3D&spr=https&se=2023-02-20T04%3A31%3A05Z&rscl=x-e2eid-e4e08c6b-b6384600-907dabed-0f27f1f5-session-bc6d7d4a-45804b9d-b857f518-397ae64d HTTP/1.1
> Host: vsblobprodscussu5shard26.blob.core.windows.net
> User-Agent: curl/7.85.0
> Accept: */*
> Accept-Encoding: deflate, gzip
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< Content-Length: 12496896
< Content-Type: application/octet-stream
< Content-Language: x-e2eid-e4e08c6b-b6384600-907dabed-0f27f1f5-session-bc6d7d4a-45804b9d-b857f518-397ae64d
< Last-Modified: Wed, 11 Jan 2023 01:59:52 GMT
< Accept-Ranges: bytes
< ETag: "0x8DAF37787B7E0C5"
< Server: Windows-Azure-Blob/1.0 Microsoft-HTTPAPI/2.0
< x-ms-request-id: fe61432d-401e-0050-3a17-4474ad000000
< x-ms-version: 2019-07-07
< x-ms-creation-time: Wed, 11 Jan 2023 01:59:52 GMT
< x-ms-lease-status: unlocked
< x-ms-lease-state: available
< x-ms-blob-type: BlockBlob
< x-ms-server-encrypted: true
< Access-Control-Expose-Headers: Content-Length
< Access-Control-Allow-Origin: *
< Date: Sun, 19 Feb 2023 04:09:15 GMT
< 
```
