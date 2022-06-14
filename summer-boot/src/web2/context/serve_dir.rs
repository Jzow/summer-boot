use crate::log;
use crate::{Body, Endpoint, Request, Response, Result, StatusCode};

use async_std::path::PathBuf as AsyncPathBuf;

use std::path::{Path, PathBuf};
use std::{ffi::OsStr, io};

pub(crate) struct ServeDir {
    prefix: String,
    dir: PathBuf,
}

impl ServeDir {
    /// Create a new instance of `ServeDir`.
    pub(crate) fn new(prefix: String, dir: PathBuf) -> Self {
        Self { prefix, dir }
    }
}

#[async_trait::async_trait]
impl<State> Endpoint<State> for ServeDir
where
    State: Clone + Send + Sync + 'static,
{
    async fn call(&self, req: Request<State>) -> Result {
        let path = req.url().path();
        let path = path
            .strip_prefix(&self.prefix.trim_end_matches('*'))
            .unwrap();
        let path = path.trim_start_matches('/');
        let mut file_path = self.dir.clone();
        for p in Path::new(path) {
            if p == OsStr::new(".") {
                continue;
            } else if p == OsStr::new("..") {
                file_path.pop();
            } else {
                file_path.push(&p);
            }
        }

        log::info!("Requested file: {:?}", file_path);

        let file_path = AsyncPathBuf::from(file_path);
        if !file_path.starts_with(&self.dir) {
            log::warn!("Unauthorized attempt to read: {:?}", file_path);
            Ok(Response::new(StatusCode::Forbidden))
        } else {
            match Body::from_file(&file_path).await {
                Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    log::warn!("File not found: {:?}", &file_path);
                    Ok(Response::new(StatusCode::NotFound))
                }
                Err(e) => Err(e.into()),
            }
        }
    }
}