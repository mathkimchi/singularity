use std::os::unix::net::{UnixListener, UnixStream};

const PATH_PREFIX_ENV_KEY: &str = "XDG_RUNTIME_DIR";
const PATH_SUFFIX_ENV_KEY: &str = "SINGULARITY_SERVER";

/// Server side
pub struct ServerHost {
    path: String,
    pub listener: UnixListener,
}
impl ServerHost {
    pub fn bind_new() -> Option<Self> {
        let path_prefix = std::env::var(PATH_PREFIX_ENV_KEY).ok()?;

        // TODO
        let path_suffix = "singularity-0";

        // FIXME, I think this only applies to children processes
        std::env::set_var(PATH_SUFFIX_ENV_KEY, path_suffix);

        let path = format!("{}/{}", path_prefix, path_suffix);
        Some(Self {
            listener: UnixListener::bind(&path).ok()?,
            path,
        })
    }
}
impl Drop for ServerHost {
    fn drop(&mut self) {
        // smh, rust should have some temp_set_env_var function which returns an empty object so it auto removes on drop
        // std::env::remove_var(PATH_SUFFIX_ENV_KEY);
        // ^ actually, processes might make this unnecessary

        // unix listener doesn't remove the file on drop
        if let Err(e) = std::fs::remove_file(&self.path) {
            dbg!(e);
        }
    }
}

/// represents a single client on the server side
pub struct ClientHandler {
    pub stream: UnixStream,
}

/// Client-side
pub struct ServerHandle {
    pub stream: UnixStream,
}
impl ServerHandle {
    /// Connect to unix socket at `$XDG_RUNTIME_DIR/$SINGULARITY_SERVER`
    pub fn connect_from_env() -> Option<Self> {
        let socket_path = format!(
            "{}/{}",
            std::env::var(PATH_PREFIX_ENV_KEY).ok()?,
            std::env::var(PATH_SUFFIX_ENV_KEY).unwrap_or("singularity-0".to_string())
        );

        Some(Self {
            stream: UnixStream::connect(socket_path).ok()?,
        })
    }
}
