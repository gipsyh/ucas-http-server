#![feature(string_remove_matches)]
use std::{
    fs::{self, File},
    mem::forget,
    os::unix::prelude::AsRawFd,
    ptr::NonNull,
};

use libevent::{
    bufferevent, event_base, evhttp_request, Base, BufferEvent, EvBuffer, EvHttp, EvHttpRequest,
    RequestKind,
};
use openssl::{
    ec::EcKey,
    nid,
    ssl::{Ssl, SslConnector, SslContext, SslContextBuilder, SslFiletype, SslMethod},
};

unsafe extern "C" fn handle(request: *mut evhttp_request, _arg: *mut std::os::raw::c_void) {
    let request = EvHttpRequest::from_raw_ptr(request).unwrap();
    let uri = request.get_uri().unwrap();
    let path = uri.path();
    let server_path = "./file".to_string() + &path;
    let mut buffer = EvBuffer::new().unwrap();
    let metadata = match fs::metadata(&server_path) {
        Ok(metedata) => metedata,
        Err(_) => {
            request.reply(404, "Not Found", buffer);
            return;
        }
    };
    if metadata.is_file() {
        let file = File::open(&server_path).unwrap();
        buffer
            .add_file(file.as_raw_fd(), 0, metadata.len() as _)
            .unwrap();
    } else if metadata.is_dir() {
        let paths = fs::read_dir(&server_path).unwrap();
        let base = if path.ends_with('/') {
            path.clone()
        } else {
            path.clone() + "/"
        };
        let mut html = format!(
            "
<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">
        <title>{}</title>
        <base href='{}'>
    </head>
    <body>
        <h1>{}</h1>
        <hr>
            <ul>",
            &path, &base, &path,
        );
        for path in paths {
            let name = path.unwrap().file_name().into_string().unwrap();
            html += &format!("                <li><a href={}>{}</a>", name, name);
        }
        html += "
                </li>
            </ul>
        <hr>
        <form action=\"./\" method=\"post\" enctype=\"multipart/form-data\">
        <p><input type=\"file\" name=\"upload\"></p>
        <p><input type=\"submit\" value=\"submit\"></p>
        </form>
    </body>
</html>";
        buffer.add(html.as_bytes()).unwrap();
        if let RequestKind::Post = request.get_command() {
            let input_buffer = request.get_input_buffer();
            let buffer = vec![0_u8; 4096];
            input_buffer.copyout(&buffer);
            let str = String::from_utf8_lossy(&buffer);
            let filename_start = str.find("filename=").unwrap() + "filename=".len();
            let filename_end = filename_start + str[filename_start..].find("\r\n").unwrap();
            let filename = &str[filename_start..filename_end];
            let content_start = str.find("\r\n\r\n").unwrap() + 4;
            let content_end = content_start + str[content_start..].find("\r\n").unwrap();
            let mut filename = format!("{}{}", server_path, filename);
            filename.remove_matches("\"");
            fs::write(filename, &str[content_start..content_end]).unwrap();
        }
    } else {
        panic!();
    }
    request.reply(200, "OK", buffer);
}

unsafe extern "C" fn ssl_bev_cb(
    base: *mut event_base,
    arg: *mut std::os::raw::c_void,
) -> *mut bufferevent {
    let sslctx = Box::from_raw(arg as *mut SslContext);
    let ssl = Ssl::new(&sslctx).unwrap();
    let base = Base::from_raw(NonNull::new(base).unwrap());
    let ssl_bev = BufferEvent::openssl_socket_new(&base, -1, ssl.ssl_ptr() as _, 2, 1).unwrap();
    let ans = ssl_bev.as_ptr();
    forget(ssl_bev);
    forget(base);
    forget(ssl);
    forget(sslctx);
    ans
}

fn ssl_init() -> SslContext {
    let mut builder = SslContextBuilder::new(SslMethod::tls_server()).unwrap();
    builder
        .set_tmp_ecdh(&EcKey::from_curve_name(nid::Nid::X9_62_PRIME256V1).unwrap())
        .unwrap();
    builder.set_certificate_chain_file("server.cert").unwrap();
    builder
        .set_private_key_file("server.key", SslFiletype::PEM)
        .unwrap();
    builder.check_private_key().unwrap();
    builder.build()
}

fn main() {
    let sslctx = Box::new(ssl_init());
    let base = libevent::Base::new().unwrap();
    let mut http = EvHttp::new(&base).unwrap();
    http.set_bevcb(Some(ssl_bev_cb), sslctx);
    http.bind_socket_with_handle("0.0.0.0:9999").unwrap();
    http.set_gencb(Some(handle));
    base.dispatch();
}
