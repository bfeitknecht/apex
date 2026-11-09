//! Public SSH frontend: every session gets its own portfolio instance.
//!
//! The server only serves the portfolio. There is no system shell, no `exec`,
//! no SFTP and no forwarding, and a PTY is required to start a session.
//!
//! Configuration is read from the environment:
//!
//! - `APEX_SSH_HOST_KEY`: path to an existing unencrypted Ed25519 host key.
//! - `APEX_SSH_LISTEN`: bind address, `0.0.0.0:2222` by default.
//! - `APEX_SSH_PUBLIC`: `1` accepts any client, `0` requires an allowlist.
//! - `APEX_SSH_AUTHORIZED_KEYS`: one OpenSSH public key per line, key-only mode.
//! - `APEX_SSH_USER`: username key-only mode accepts, `apex` by default.

use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use russh::server::{self, Auth, ChannelOpenHandle, Msg, Server as _, Session, VerifiedPublicKey};
use russh::{Channel, ChannelId, Pty};
use tokio::sync::{mpsc, OwnedSemaphorePermit, Semaphore};

use crate::{ui, App, View};

/// Concurrent sessions per server process.
const MAX_SESSIONS: usize = 32;

/// Terminal size limits: a hostile client can request anything.
const MIN_WIDTH: u32 = 20;
const MAX_WIDTH: u32 = 240;
const MIN_HEIGHT: u32 = 8;
const MAX_HEIGHT: u32 = 120;

/// Input allowed in one SSH packet before the channel is closed.
const MAX_INPUT: usize = 8192;

/// How long an escape byte may wait for the rest of its sequence.
const ESCAPE_TIMEOUT: Duration = Duration::from_millis(70);

const FRAME_INTERVAL: Duration = Duration::from_millis(10);

/// Authentication and binding configuration.
struct Settings {
    user: String,
    public: bool,
    keys: Vec<russh::keys::PublicKey>,
    listen: String,
}

impl Settings {
    fn from_env() -> Result<Self, Box<dyn Error>> {
        let public = match env::var("APEX_SSH_PUBLIC") {
            Ok(value) if value == "1" => true,
            Ok(value) if value == "0" => false,
            Ok(_) => return Err("APEX_SSH_PUBLIC must be 1 (public) or 0 (key-only)".into()),
            Err(env::VarError::NotPresent) if env::var_os("APEX_SSH_AUTHORIZED_KEYS").is_some() => {
                return Err(
                    "An authorized-key file does not restrict public mode. Set APEX_SSH_PUBLIC=0 for key-only access (or 1 for public)."
                        .into(),
                )
            }
            Err(env::VarError::NotPresent) => true,
            Err(error) => return Err(error.into()),
        };

        let mut keys = Vec::new();
        if let Ok(path) = env::var("APEX_SSH_AUTHORIZED_KEYS") {
            for (line_number, line) in std::fs::read_to_string(path)?.lines().enumerate() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let key = russh::keys::PublicKey::from_openssh(line).map_err(|error| {
                    format!(
                        "authorized key line {}: {error}; key options are not supported",
                        line_number + 1
                    )
                })?;
                keys.push(key);
            }
        }
        if !public && keys.is_empty() {
            return Err("Key-only mode requires APEX_SSH_AUTHORIZED_KEYS".into());
        }

        Ok(Self {
            user: env::var("APEX_SSH_USER").unwrap_or_else(|_| "apex".to_string()),
            public,
            keys,
            listen: env::var("APEX_SSH_LISTEN").unwrap_or_else(|_| "0.0.0.0:2222".to_string()),
        })
    }
}

struct Server {
    settings: Arc<Settings>,
    slots: Arc<Semaphore>,
}

struct Client {
    settings: Arc<Settings>,
    slots: Arc<Semaphore>,
    permit: Option<OwnedSemaphorePermit>,
    channel: Option<ChannelId>,
    size: Option<(u16, u16)>,
    input: Option<mpsc::Sender<Input>>,
}

/// Terminal bytes Ratatui renders into before they are sent to the client.
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

impl server::Server for Server {
    type Handler = Client;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Client {
        Client {
            settings: self.settings.clone(),
            slots: self.slots.clone(),
            permit: None,
            channel: None,
            size: None,
            input: None,
        }
    }

    fn handle_session_error(&mut self, error: russh::Error) {
        eprintln!("SSH session: {error}");
    }
}

impl server::Handler for Client {
    type Error = russh::Error;

    async fn auth_none(&mut self, _user: &str) -> Result<Auth, Self::Error> {
        Ok(if self.settings.public {
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
        let allowed = self.settings.public
            || (user == self.settings.user
                && self
                    .settings
                    .keys
                    .iter()
                    .any(|candidate| candidate.key_data() == key.public_key.key_data()));
        Ok(if allowed {
            Auth::Accept
        } else {
            Auth::reject()
        })
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
        channel: ChannelId,
        _: &str,
        width: u32,
        height: u32,
        _: u32,
        _: u32,
        _: &[(Pty, u32)],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel != Some(channel) || self.input.is_some() {
            session.channel_failure(channel)?;
        } else {
            self.size = Some(dimensions(width, height));
            session.channel_success(channel)?;
        }
        Ok(())
    }

    async fn shell_request(
        &mut self,
        channel: ChannelId,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let Some(size) = self.size.filter(|_| self.input.is_none()) else {
            session.channel_failure(channel)?;
            return Ok(());
        };
        if self.channel != Some(channel) {
            session.channel_failure(channel)?;
            return Ok(());
        }
        let (sender, receiver) = mpsc::channel(32);
        self.input = Some(sender);
        let handle = session.handle();
        session.channel_success(channel)?;
        tokio::spawn(async move {
            if let Err(error) = run_session(handle, channel, size, receiver).await {
                eprintln!("SSH terminal: {error}");
            }
        });
        Ok(())
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        _: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn subsystem_request(
        &mut self,
        channel: ChannelId,
        _: &str,
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        session.channel_failure(channel)?;
        Ok(())
    }

    async fn data(
        &mut self,
        channel: ChannelId,
        data: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(channel) {
            if let Some(sender) = &self.input {
                if data.len() > MAX_INPUT || sender.try_send(Input::Bytes(data.to_vec())).is_err() {
                    session.close(channel)?;
                    self.input = None;
                }
            }
        }
        Ok(())
    }

    async fn window_change_request(
        &mut self,
        channel: ChannelId,
        width: u32,
        height: u32,
        _: u32,
        _: u32,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(channel) {
            let (width, height) = dimensions(width, height);
            self.size = Some((width, height));
            if let Some(sender) = &self.input {
                let _ = sender.try_send(Input::Resize(width, height));
            }
        }
        Ok(())
    }

    async fn channel_eof(
        &mut self,
        channel: ChannelId,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(channel) {
            self.input = None;
        }
        Ok(())
    }

    async fn channel_close(
        &mut self,
        channel: ChannelId,
        _: &mut Session,
    ) -> Result<(), Self::Error> {
        if self.channel == Some(channel) {
            self.input = None;
            self.channel = None;
            self.permit = None;
        }
        Ok(())
    }
}

/// Clamp a client-requested terminal size into the range the UI supports.
fn dimensions(width: u32, height: u32) -> (u16, u16) {
    (
        width.clamp(MIN_WIDTH, MAX_WIDTH) as u16,
        height.clamp(MIN_HEIGHT, MAX_HEIGHT) as u16,
    )
}

/// Decode UTF-8 terminal input into application actions.
///
/// `flush_escape` resolves a lone trailing escape byte, which is otherwise held
/// back to see whether it starts a cursor-key sequence.
fn apply_input(app: &mut App, pending: &mut Vec<u8>, flush_escape: bool) -> bool {
    let mut cursor = 0;
    while cursor < pending.len() {
        let byte = pending[cursor];
        if byte == 27 {
            if cursor + 1 == pending.len() {
                if flush_escape {
                    app.back();
                    cursor += 1;
                }
                break;
            }
            if pending[cursor + 1] == b'[' || pending[cursor + 1] == b'O' {
                let final_byte = pending[cursor + 2..]
                    .iter()
                    .position(|byte| (0x40..=0x7e).contains(byte));
                if let Some(offset) = final_byte {
                    let end = cursor + 2 + offset;
                    // Only plain arrow keys navigate; ignore modified sequences.
                    if end == cursor + 2 {
                        match pending[end] {
                            b'A' => app.navigate(-1),
                            b'B' => app.navigate(1),
                            b'C' => app.change_section(1),
                            b'D' => app.change_section(-1),
                            b'Z' => app.focus_link(true),
                            _ => {}
                        }
                    }
                    cursor = end + 1;
                } else if flush_escape {
                    cursor = pending.len();
                } else {
                    break;
                }
            } else {
                app.back();
                cursor += 1;
            }
        } else {
            cursor += 1;
            match byte {
                b'q' | 3 => {
                    pending.clear();
                    return true;
                }
                b'\t' => app.focus_link(false),
                b'\r' | b'\n' => {
                    let _ = app.activate();
                }
                32..=126 => app.key(byte as char),
                _ => {}
            }
        }
    }
    pending.drain(..cursor);
    if pending.len() > 16 {
        pending.clear();
    }
    false
}

/// Drive one visitor's terminal until they quit or disconnect.
async fn run_session(
    handle: server::Handle,
    channel: ChannelId,
    (width, height): (u16, u16),
    mut input: mpsc::Receiver<Input>,
) -> io::Result<()> {
    let send = |bytes: Vec<u8>| async {
        handle
            .data(channel, bytes)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "SSH client disconnected"))
    };
    let output = Output::default();
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(output.clone()),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, width, height)),
        },
    )?;
    // Alternate screen, hidden cursor, cleared.
    send(b"\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H".to_vec()).await?;

    let mut app = App::new();
    let mut size = (width, height);
    let mut pending = Vec::new();
    let mut timer = tokio::time::interval(FRAME_INTERVAL);
    let mut dirty = true;
    let mut last_input = std::time::Instant::now();
    let mut last_generation = std::time::Instant::now();

    let result: io::Result<()> = async {
        loop {
            tokio::select! {
                event = input.recv() => match event {
                    Some(Input::Bytes(bytes)) => {
                        pending.extend(bytes);
                        last_input = std::time::Instant::now();
                        if apply_input(&mut app, &mut pending, false) {
                            break;
                        }
                        dirty = true;
                    }
                    Some(Input::Resize(width, height)) => {
                        size = (width, height);
                        terminal.resize(Rect::new(0, 0, width, height))?;
                        dirty = true;
                    }
                    None => break,
                },
                _ = timer.tick() => {
                    // A lone escape byte becomes "back" once no more input follows.
                    if !pending.is_empty() && last_input.elapsed() >= ESCAPE_TIMEOUT {
                        if apply_input(&mut app, &mut pending, true) {
                            break;
                        }
                        dirty = true;
                    }
                    let elapsed = last_generation.elapsed().as_millis().min(u64::MAX as u128) as u64;
                    last_generation = std::time::Instant::now();
                    if app.tick(elapsed) {
                        dirty = true;
                    }
                }
            }
            if dirty {
                app.resize(size.0, size.1);
                if app.view == View::Lichen {
                    app.lichen
                        .resize(size.0 as usize, size.1 as usize * 2);
                }
                terminal.draw(|frame| ui::draw(frame, &app))?;
                let rendered = std::mem::take(
                    &mut *output
                        .0
                        .lock()
                        .map_err(|_| io::Error::other("output lock poisoned"))?,
                );
                if !rendered.is_empty() {
                    send(rendered).await?;
                }
                dirty = false;
            }
        }
        Ok(())
    }
    .await;

    let _ = handle
        .data(channel, b"\x1b[0m\x1b[?25h\x1b[?1049l".to_vec())
        .await;
    let _ = handle.exit_status_request(channel, 0).await;
    let _ = handle.eof(channel).await;
    let _ = handle.close(channel).await;
    result
}

/// Bind and serve until the process is stopped.
async fn serve() -> Result<(), Box<dyn Error>> {
    let path = env::var("APEX_SSH_HOST_KEY")
        .map_err(|_| "Set APEX_SSH_HOST_KEY to an existing unencrypted Ed25519 SSH host key")?;
    let host_key = russh::keys::load_secret_key(&path, None)?;
    let fingerprint = host_key
        .public_key()
        .fingerprint(russh::keys::HashAlg::Sha256);
    let settings = Settings::from_env()?;

    eprintln!("SSH host key: {fingerprint}");
    if settings.public {
        eprintln!(
            "Apex SSH listening on {} (public access; any username)",
            settings.listen
        );
    } else {
        eprintln!(
            "Apex SSH listening on {} (key-only access; user {})",
            settings.listen, settings.user
        );
    }

    let listen = settings.listen.clone();
    let config = server::Config {
        keys: vec![host_key],
        inactivity_timeout: Some(Duration::from_secs(900)),
        auth_rejection_time: Duration::from_secs(1),
        ..Default::default()
    };
    let mut server = Server {
        settings: Arc::new(settings),
        slots: Arc::new(Semaphore::new(MAX_SESSIONS)),
    };
    server.run_on_address(Arc::new(config), listen).await?;
    Ok(())
}

/// Run the SSH frontend on a fresh Tokio runtime.
pub fn run() -> Result<(), Box<dyn Error>> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(serve())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_escape_and_vim() {
        let mut app = App::new();
        app.resize(80, 24);
        let mut pending = vec![27];
        assert!(!apply_input(&mut app, &mut pending, false));
        pending.extend_from_slice(b"[C");
        apply_input(&mut app, &mut pending, false);
        assert_eq!(app.view, View::Portfolio(1));
        pending.extend_from_slice(b"jggG");
        apply_input(&mut app, &mut pending, false);
        assert_eq!(app.scroll_offset, ui::portfolio_page(80, 24).max_scroll());
    }

    #[test]
    fn quit_keys_end_the_session() {
        for quit in [b"q".as_slice(), b"\x03".as_slice()] {
            let mut app = App::new();
            let mut pending = quit.to_vec();
            assert!(apply_input(&mut app, &mut pending, false));
            assert!(pending.is_empty());
        }
    }

    #[test]
    fn lone_escape_is_held_until_it_times_out() {
        let mut app = App::new();
        app.resize(80, 24);
        app.view = View::Visuals;
        let mut pending = vec![27];
        assert!(!apply_input(&mut app, &mut pending, false));
        assert_eq!(pending, vec![27]);
        assert!(!apply_input(&mut app, &mut pending, true));
        assert_eq!(app.view, View::Portfolio(0));
        assert!(pending.is_empty());
    }

    #[test]
    fn terminal_dimensions_are_clamped() {
        assert_eq!(dimensions(0, 0), (MIN_WIDTH as u16, MIN_HEIGHT as u16));
        assert_eq!(
            dimensions(10_000, 10_000),
            (MAX_WIDTH as u16, MAX_HEIGHT as u16)
        );
        assert_eq!(dimensions(80, 24), (80, 24));
    }
}
