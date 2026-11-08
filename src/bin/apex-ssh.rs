//! SSH transport for the shared Apex Ratatui application.
use apex::{ui, App, View};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal, TerminalOptions, Viewport};
use russh::{
    server::{self, Auth, ChannelOpenHandle, Msg, Server as _, Session, VerifiedPublicKey},
    Channel, ChannelId, Pty,
};
use std::{env, io, net::SocketAddr, sync::Arc, time::Duration};
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};

struct Settings {
    user: String,
    public: bool,
    keys: Vec<russh::keys::PublicKey>,
}
struct Server {
    config: Arc<Settings>,
    slots: Arc<Semaphore>,
}
struct Client {
    config: Arc<Settings>,
    slots: Arc<Semaphore>,
    permit: Option<OwnedSemaphorePermit>,
    channel: Option<ChannelId>,
    size: Option<(u16, u16)>,
    tx: Option<mpsc::Sender<Input>>,
}
#[derive(Clone, Default)]
struct Output(Arc<std::sync::Mutex<Vec<u8>>>);
impl io::Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("output lock poisoned"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
enum Input {
    Bytes(Vec<u8>),
    Resize(u16, u16),
}
fn dimensions(w: u32, h: u32) -> (u16, u16) {
    (w.clamp(20, 240) as u16, h.clamp(8, 120) as u16)
}
impl server::Server for Server {
    type Handler = Client;
    fn new_client(&mut self, _: Option<SocketAddr>) -> Client {
        Client {
            config: self.config.clone(),
            slots: self.slots.clone(),
            permit: None,
            channel: None,
            size: None,
            tx: None,
        }
    }
    fn handle_session_error(&mut self, e: russh::Error) {
        eprintln!("SSH session: {e}");
    }
}
impl server::Handler for Client {
    type Error = russh::Error;
    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(if self.config.public {
            Auth::Accept
        } else {
            Auth::reject()
        })
    }
    async fn auth_publickey(
        &mut self,
        user: &str,
        key: VerifiedPublicKey<'_>,
    ) -> Result<Auth, Self::Error> {
        Ok(
            if self.config.public
                || (user == self.config.user
                    && self
                        .config
                        .keys
                        .iter()
                        .any(|k| k.key_data() == key.public_key.key_data()))
            {
                Auth::Accept
            } else {
                Auth::reject()
            },
        )
    }
    async fn channel_open_session(
        &mut self,
        channel: Channel<Msg>,
        reply: ChannelOpenHandle,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel.is_some() {
            drop(reply);
            return Ok(());
        }
        if let Ok(permit) = self.slots.clone().try_acquire_owned() {
            self.permit = Some(permit);
            self.channel = Some(channel.id());
            reply.accept().await;
        } else {
            drop(reply);
        }
        Ok(())
    }
    async fn pty_request(
        &mut self,
        ch: ChannelId,
        _: &str,
        w: u32,
        h: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel != Some(ch) || self.tx.is_some() {
            session.channel_failure(ch)?;
        } else {
            self.size = Some(dimensions(w, h));
            session.channel_success(ch)?;
        }
        Ok(())
    }
    async fn shell_request(
        &mut self,
        ch: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel != Some(ch) || self.tx.is_some() || self.size.is_none() {
            session.channel_failure(ch)?;
            return Ok(());
        }
        let Some(size) = self.size else {
            session.channel_failure(ch)?;
            return Ok(());
        };
        let (tx, rx) = mpsc::channel(32);
        self.tx = Some(tx);
        let handle = session.handle();
        session.channel_success(ch)?;
        tokio::spawn(async move {
            if let Err(e) = run_session(handle, ch, size, rx).await {
                eprintln!("SSH terminal: {e}");
            }
        });
        Ok(())
    }
    async fn exec_request(
        &mut self,
        ch: ChannelId,
        _: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(ch)?;
        Ok(())
    }
    async fn subsystem_request(
        &mut self,
        ch: ChannelId,
        _: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(ch)?;
        Ok(())
    }
    async fn data(
        &mut self,
        ch: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(ch) {
            if let Some(tx) = &self.tx {
                if data.len() > 8192 || tx.try_send(Input::Bytes(data.to_vec())).is_err() {
                    session.close(ch)?;
                    self.tx = None;
                }
            }
        }
        Ok(())
    }
    async fn window_change_request(
        &mut self,
        ch: ChannelId,
        w: u32,
        h: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(ch) {
            let (w, h) = dimensions(w, h);
            self.size = Some((w, h));
            if let Some(tx) = &self.tx {
                let _ = tx.try_send(Input::Resize(w, h));
            }
        }
        Ok(())
    }
    async fn channel_eof(&mut self, ch: ChannelId, _: &mut Session) -> Result<(), Self::Error> {
        if self.channel == Some(ch) {
            self.tx = None;
        }
        Ok(())
    }
    async fn channel_close(&mut self, ch: ChannelId, _: &mut Session) -> Result<(), Self::Error> {
        if self.channel == Some(ch) {
            self.tx = None;
            self.channel = None;
            self.permit = None;
        }
        Ok(())
    }
}

fn apply_input(app: &mut App, pending: &mut Vec<u8>, flush_escape: bool) -> bool {
    let mut n = 0;
    while n < pending.len() {
        let c = pending[n];
        if c == 27 {
            if n + 1 == pending.len() {
                if flush_escape {
                    app.back();
                    n += 1;
                }
                break;
            }
            if pending[n + 1] == b'[' || pending[n + 1] == b'O' {
                let final_byte = pending[n + 2..]
                    .iter()
                    .position(|b| (0x40..=0x7e).contains(b));
                if let Some(offset) = final_byte {
                    let end = n + 2 + offset;
                    // Only plain arrow keys navigate; ignore modified/unknown sequences.
                    if end == n + 2 {
                        match pending[end] {
                            b'A' => app.navigate(-1),
                            b'B' => app.navigate(1),
                            b'C' => app.change_section(1),
                            b'D' => app.change_section(-1),
                            b'Z' => app.focus_link(true),
                            _ => {}
                        }
                    }
                    n = end + 1;
                } else if flush_escape {
                    n = pending.len();
                } else {
                    break;
                }
            } else {
                app.back();
                n += 1;
            }
        } else {
            n += 1;
            match c {
                b'q' | 3 => {
                    pending.clear();
                    return true;
                }
                b'\t' => app.focus_link(false),
                b'\r' | b'\n' => {
                    let _ = app.activate();
                }
                32..=126 => app.key(c as char),
                _ => {}
            }
        }
    }
    pending.drain(..n);
    if pending.len() > 16 {
        pending.clear();
    }
    false
}
async fn run_session(
    handle: server::Handle,
    ch: ChannelId,
    (w, h): (u16, u16),
    mut rx: mpsc::Receiver<Input>,
) -> io::Result<()> {
    let send = |bytes: Vec<u8>| async {
        handle
            .data(ch, bytes)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "SSH client disconnected"))
    };
    let output = Output::default();
    let mut term = Terminal::with_options(
        CrosstermBackend::new(output.clone()),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, w, h)),
        },
    )?;
    send(b"\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H".to_vec()).await?;
    let mut app = App::new();
    let mut size = (w, h);
    let mut pending = Vec::new();
    let mut timer = tokio::time::interval(Duration::from_millis(10));
    let mut dirty = true;
    let mut last_data = std::time::Instant::now();
    let mut last_generation = std::time::Instant::now();
    let result: io::Result<()> = async {
        loop {
            tokio::select! {
                event = rx.recv() => match event {
                    Some(Input::Bytes(data)) => {
                        pending.extend(data);
                        last_data = std::time::Instant::now();
                        if apply_input(&mut app, &mut pending, false) { break; }
                        dirty = true;
                    }
                    Some(Input::Resize(w, h)) => {
                        size = (w, h);
                        term.resize(Rect::new(0, 0, w, h))?;
                        dirty = true;
                    }
                    None => break,
                },
                _ = timer.tick() => {
                    if !pending.is_empty() && last_data.elapsed() >= Duration::from_millis(70) {
                        if apply_input(&mut app, &mut pending, true) { break; }
                        dirty = true;
                    }
                    let elapsed = last_generation.elapsed().as_millis().min(u64::MAX as u128) as u64;
                    last_generation = std::time::Instant::now();
                    if app.advance_lichen(elapsed) > 0 || app.advance_fractal(elapsed) { dirty = true; }
                }
            }
            if dirty {
                app.resize(size.0, size.1);
                if app.view == View::Lichen {
                    app.lichen.resize(size.0 as usize, size.1 as usize * 2);
                }
                term.draw(|f| ui::draw(f, &app))?;
                let out = std::mem::take(
                    &mut *output.0.lock().map_err(|_| io::Error::other("output lock poisoned"))?,
                );
                if !out.is_empty() { send(out).await?; }
                dirty = false;
            }
        }
        Ok(())
    }.await;
    let _ = handle
        .data(ch, b"\x1b[0m\x1b[?25h\x1b[?1049l".to_vec())
        .await;
    let _ = handle.exit_status_request(ch, 0).await;
    let _ = handle.eof(ch).await;
    let _ = handle.close(ch).await;
    result
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host_path = env::var("APEX_SSH_HOST_KEY")
        .map_err(|_| "Set APEX_SSH_HOST_KEY to an existing unencrypted Ed25519 SSH host key")?;
    let host = russh::keys::load_secret_key(&host_path, None)?;
    let fingerprint = host.public_key().fingerprint(russh::keys::HashAlg::Sha256);
    let public = match env::var("APEX_SSH_PUBLIC") {
        Ok(value) if value == "1" => true,
        Ok(value) if value == "0" => false,
        Ok(_) => return Err("APEX_SSH_PUBLIC must be 1 (public) or 0 (key-only)".into()),
        Err(env::VarError::NotPresent) if env::var_os("APEX_SSH_AUTHORIZED_KEYS").is_some() =>
            return Err("An authorized-key file does not restrict public mode. Set APEX_SSH_PUBLIC=0 for key-only access (or 1 for public).".into()),
        Err(env::VarError::NotPresent) => true,
        Err(e) => return Err(e.into()),
    };
    let mut keys = Vec::new();
    if let Ok(path) = env::var("APEX_SSH_AUTHORIZED_KEYS") {
        for (n, line) in std::fs::read_to_string(path)?.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let key = russh::keys::PublicKey::from_openssh(line).map_err(|e| {
                format!(
                    "authorized key line {}: {e}; key options are not supported",
                    n + 1
                )
            })?;
            keys.push(key);
        }
    }
    if !public && keys.is_empty() {
        return Err("Key-only mode requires APEX_SSH_AUTHORIZED_KEYS".into());
    }
    let user = env::var("APEX_SSH_USER").unwrap_or_else(|_| "apex".to_string());
    let address = env::var("APEX_SSH_LISTEN").unwrap_or_else(|_| "0.0.0.0:2222".to_string());
    eprintln!("SSH host key: {fingerprint}");
    let config = server::Config {
        keys: vec![host],
        inactivity_timeout: Some(Duration::from_secs(900)),
        auth_rejection_time: Duration::from_secs(1),
        ..Default::default()
    };
    let mut server = Server {
        config: Arc::new(Settings {
            user: user.clone(),
            public,
            keys,
        }),
        slots: Arc::new(Semaphore::new(32)),
    };
    if public {
        eprintln!("Apex SSH listening on {address} (public access; any username)");
    } else {
        eprintln!("Apex SSH listening on {address} (key-only access; user {user})");
    }
    server.run_on_address(Arc::new(config), address).await?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_escape_and_vim() {
        let mut a = App::new();
        a.resize(80, 24);
        let mut p = vec![27];
        assert!(!apply_input(&mut a, &mut p, false));
        p.extend_from_slice(b"[C");
        apply_input(&mut a, &mut p, false);
        assert_eq!(a.view, View::Portfolio(1));
        p.extend_from_slice(b"jggG");
        apply_input(&mut a, &mut p, false);
        assert_eq!(
            a.scroll_offset,
            apex::ui::portfolio_page(80, 24).max_scroll()
        );
    }
}
