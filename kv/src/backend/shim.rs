//! An in-process engine behind a socket, `--shim unix`, so that it pays the round trip PostgreSQL
//! pays and the two can be compared as engines. This is the shim of the YCSB spec's section 4.10
//! and engine-v4 section 13.8, and it is not a server.
//!
//! Each session is a `socketpair` with a thread on the other end that owns a session of the engine.
//! The client writes a request, the operation and its arguments with a length in front, and blocks
//! on the read. The thread reads it, runs it on its session, writes the whole result back and blocks
//! on its next read. That is one round trip through the kernel per statement and one thread per
//! connection, the shape of PostgreSQL's backend per connection.
//!
//! Every number on the wire is little endian. A request is a `u32` length, an operation byte and
//! its arguments. A reply is a `u32` length, a status byte, `OK`, `RETRY` or `ERROR`, and then what
//! the operation returns, or the error's message.

use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

use super::{Backend, Failed, Placeholder, Session, Values};

const PREPARE: u8 = 0;
const EXECUTE: u8 = 1;
const BATCH: u8 = 2;
const EXPLAIN: u8 = 3;
const BEGIN: u8 = 4;
const BEGIN_READ: u8 = 5;
const COMMIT: u8 = 6;
const ROLLBACK: u8 = 7;

const OK: u8 = 0;
const RETRY: u8 = 1;
const ERROR: u8 = 2;

/// An engine that runs in process, reached through a socket.
#[derive(Debug)]
pub(crate) struct Shim<B> {
    inner: Arc<B>,
}

impl<B: Backend + Send + 'static> Shim<B> {
    pub(crate) fn new(inner: B) -> Self {
        Self { inner: Arc::new(inner) }
    }
}

impl<B: Backend + Send + 'static> Backend for Shim<B> {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn version(&self) -> String {
        self.inner.version()
    }

    fn boundary(&self) -> &'static str {
        "unix_shim"
    }

    fn placeholder(&self) -> Placeholder {
        self.inner.placeholder()
    }

    fn create_table(&self) -> String {
        self.inner.create_table()
    }

    fn level(&self) -> Result<(), String> {
        self.inner.level()
    }

    fn level_note(&self) -> Option<&'static str> {
        self.inner.level_note()
    }

    fn numbered(&self, text: &str) -> String {
        self.inner.numbered(text)
    }

    fn keeps_rows(&self) -> bool {
        self.inner.keeps_rows()
    }

    fn connect(&self) -> Result<Box<dyn Session + '_>, String> {
        let (ours, theirs) = UnixStream::pair().map_err(|error| format!("socketpair: {error}"))?;
        let inner = Arc::clone(&self.inner);
        let (ready, connected) = mpsc::channel();
        let server = std::thread::Builder::new()
            .name("shim".to_string())
            .spawn(move || {
                let session = match inner.connect() {
                    Ok(session) => session,
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                let _ = ready.send(Ok(()));
                serve(session, theirs);
            })
            .map_err(|error| format!("starting a shim thread: {error}"))?;
        match connected.recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(error),
            Err(_) => return Err("the shim thread ended before it connected".to_string()),
        }
        Ok(Box::new(ShimSession {
            stream: ours,
            request: Vec::new(),
            reply: Vec::new(),
            server: Some(server),
        }))
    }
}

/// Starts a frame in `frame`, leaving room for its length.
fn start(frame: &mut Vec<u8>, kind: u8) {
    frame.clear();
    frame.extend_from_slice(&[0; 4]);
    frame.push(kind);
}

fn put_u32(frame: &mut Vec<u8>, value: usize) {
    let value = u32::try_from(value).expect("a shim frame under 4 GiB");
    frame.extend_from_slice(&value.to_le_bytes());
}

fn put_bytes(frame: &mut Vec<u8>, bytes: &[u8]) {
    put_u32(frame, bytes.len());
    frame.extend_from_slice(bytes);
}

/// Writes the frame's length into its front and sends it in one write.
fn send(stream: &mut UnixStream, frame: &mut [u8]) -> io::Result<()> {
    let length = u32::try_from(frame.len() - 4).expect("a shim frame under 4 GiB");
    frame[..4].copy_from_slice(&length.to_le_bytes());
    stream.write_all(frame)
}

/// Reads one frame into `frame`, its length included. `false` when the other end closed between
/// frames. One end only sends when the other waits for it, so nothing past the frame is ever there
/// to be read by mistake.
fn receive(stream: &mut UnixStream, frame: &mut Vec<u8>) -> io::Result<bool> {
    frame.resize(frame.capacity().max(4096), 0);
    let mut filled = 0;
    let mut want = None;
    loop {
        if want.is_none() && filled >= 4 {
            let length = u32::from_le_bytes(frame[..4].try_into().expect("four bytes"));
            let whole = 4 + length as usize;
            if frame.len() < whole {
                frame.resize(whole, 0);
            }
            want = Some(whole);
        }
        if want.is_some_and(|whole| filled >= whole) {
            frame.truncate(filled);
            return Ok(true);
        }
        let read = stream.read(&mut frame[filled..])?;
        if read == 0 {
            if filled == 0 {
                return Ok(false);
            }
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "a shim frame cut short"));
        }
        filled += read;
    }
}

/// Reads a frame's fields in order.
struct Fields<'a> {
    bytes: &'a [u8],
}

impl<'a> Fields<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        if self.bytes.len() < length {
            return Err("a shim frame cut short".to_string());
        }
        let (taken, rest) = self.bytes.split_at(length);
        self.bytes = rest;
        Ok(taken)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<usize, String> {
        let bytes = self.take(4)?.try_into().expect("four bytes");
        Ok(u32::from_le_bytes(bytes) as usize)
    }

    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("eight bytes")))
    }

    fn bytes(&mut self) -> Result<&'a [u8], String> {
        let length = self.u32()?;
        self.take(length)
    }

    fn text(&mut self) -> Result<&'a str, String> {
        std::str::from_utf8(self.bytes()?)
            .map_err(|_| "a shim frame with text that is not UTF-8".to_string())
    }

    fn rest(&mut self) -> &'a str {
        let rest = std::mem::take(&mut self.bytes);
        std::str::from_utf8(rest).unwrap_or("an error that is not UTF-8")
    }
}

/// The thread's side: runs every request on `session` until the client goes away.
fn serve(mut session: Box<dyn Session + '_>, mut stream: UnixStream) {
    let mut request = Vec::new();
    let mut reply = Vec::new();
    let mut values = Values::default();
    while let Ok(true) = receive(&mut stream, &mut request) {
        let mut fields = Fields { bytes: &request[4..] };
        if let Err(error) = answer(&mut *session, &mut fields, &mut values, &mut reply) {
            start(&mut reply, ERROR);
            reply.extend_from_slice(error.as_bytes());
        }
        if send(&mut stream, &mut reply).is_err() {
            break;
        }
    }
}

/// Runs one request and writes its reply into `reply`. An error back is a frame that made no sense.
fn answer(
    session: &mut dyn Session,
    fields: &mut Fields<'_>,
    values: &mut Values,
    reply: &mut Vec<u8>,
) -> Result<(), String> {
    let done = |reply: &mut Vec<u8>, outcome: Result<(), Failed>| match outcome {
        Ok(()) => start(reply, OK),
        Err(failed) => refuse(reply, &failed),
    };
    match fields.u8()? {
        PREPARE => match session.prepare(fields.rest()) {
            Ok(statement) => {
                start(reply, OK);
                put_u32(reply, statement);
            }
            Err(error) => refuse(reply, &Failed::Error(error)),
        },
        EXECUTE => {
            let statement = fields.u32()?;
            let count = fields.u32()?;
            let mut parameters = Vec::with_capacity(count);
            for _ in 0..count {
                parameters.push(fields.text()?);
            }
            values.clear();
            match session.execute(statement, &parameters, values) {
                Ok(rows) => {
                    start(reply, OK);
                    reply.extend_from_slice(&rows.to_le_bytes());
                    put_u32(reply, values.len());
                    for at in 0..values.len() {
                        put_bytes(reply, values.get(at));
                    }
                }
                Err(failed) => refuse(reply, &failed),
            }
        }
        BATCH => done(reply, session.batch(fields.rest())),
        EXPLAIN => {
            let plan = session.explain(fields.u32()?);
            start(reply, OK);
            reply.push(u8::from(plan.is_some()));
            reply.extend_from_slice(plan.unwrap_or_default().as_bytes());
        }
        BEGIN => done(reply, session.begin()),
        BEGIN_READ => done(reply, session.begin_read()),
        COMMIT => done(reply, session.commit()),
        ROLLBACK => done(reply, session.rollback()),
        other => return Err(format!("no shim operation {other}")),
    }
    Ok(())
}

fn refuse(reply: &mut Vec<u8>, failed: &Failed) {
    start(reply, if matches!(failed, Failed::Retry(_)) { RETRY } else { ERROR });
    reply.extend_from_slice(failed.message().as_bytes());
}

/// The rows a statement returned or changed, with its values put in `out`.
fn returned(fields: &mut Fields<'_>, out: &mut Values) -> Result<u64, String> {
    let rows = fields.u64()?;
    for _ in 0..fields.u32()? {
        out.push(fields.bytes()?);
    }
    Ok(rows)
}

/// The client's side of one session.
struct ShimSession {
    stream: UnixStream,
    request: Vec<u8>,
    reply: Vec<u8>,
    server: Option<JoinHandle<()>>,
}

impl ShimSession {
    /// Sends the request and waits for the reply, and hands back its fields past the status when
    /// it says the operation ran.
    fn call(&mut self) -> Result<Fields<'_>, Failed> {
        let lost = |error: io::Error| Failed::Error(format!("the shim connection: {error}"));
        send(&mut self.stream, &mut self.request).map_err(lost)?;
        if !receive(&mut self.stream, &mut self.reply).map_err(lost)? {
            return Err(Failed::Error("the shim thread went away".to_string()));
        }
        let mut fields = Fields { bytes: &self.reply[4..] };
        match fields.u8().map_err(Failed::Error)? {
            OK => Ok(fields),
            RETRY => Err(Failed::Retry(fields.rest().to_string())),
            _ => Err(Failed::Error(fields.rest().to_string())),
        }
    }

    fn simple(&mut self, kind: u8, sql: &str) -> Result<(), Failed> {
        start(&mut self.request, kind);
        self.request.extend_from_slice(sql.as_bytes());
        self.call().map(|_| ())
    }
}

impl Session for ShimSession {
    fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        start(&mut self.request, PREPARE);
        self.request.extend_from_slice(sql.as_bytes());
        self.call().map_err(|failed| failed.message().to_string())?.u32()
    }

    fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        start(&mut self.request, EXECUTE);
        put_u32(&mut self.request, statement);
        put_u32(&mut self.request, parameters.len());
        for parameter in parameters {
            put_bytes(&mut self.request, parameter.as_bytes());
        }
        let mut fields = self.call()?;
        returned(&mut fields, out).map_err(Failed::Error)
    }

    fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        self.simple(BATCH, sql)
    }

    /// `explain` takes `&self`, and the stream is only written through `&mut`, so this goes over
    /// a clone of the socket, which is the same connection.
    fn explain(&self, statement: usize) -> Option<String> {
        let mut stream = self.stream.try_clone().ok()?;
        let mut request = Vec::new();
        start(&mut request, EXPLAIN);
        put_u32(&mut request, statement);
        send(&mut stream, &mut request).ok()?;
        let mut reply = Vec::new();
        receive(&mut stream, &mut reply).ok().filter(|&got| got)?;
        let mut fields = Fields { bytes: &reply[4..] };
        if fields.u8().ok()? != OK || fields.u8().ok()? == 0 {
            return None;
        }
        Some(fields.rest().to_string())
    }

    fn begin(&mut self) -> Result<(), Failed> {
        self.simple(BEGIN, "")
    }

    fn begin_read(&mut self) -> Result<(), Failed> {
        self.simple(BEGIN_READ, "")
    }

    fn commit(&mut self) -> Result<(), Failed> {
        self.simple(COMMIT, "")
    }

    fn rollback(&mut self) -> Result<(), Failed> {
        self.simple(ROLLBACK, "")
    }
}

impl Drop for ShimSession {
    /// Closes the socket, which ends the thread's loop, and waits for the thread so the engine's
    /// session is gone before the engine is.
    fn drop(&mut self) {
        let _ = self.stream.shutdown(Shutdown::Both);
        if let Some(server) = self.server.take() {
            let _ = server.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::backend::null::NullBackend;

    /// The same statements straight and through the shim give back the same values.
    #[test]
    fn the_shim_hands_back_what_the_session_does() {
        let direct = NullBackend::new(Duration::ZERO, None);
        let shim = Shim::new(NullBackend::new(Duration::ZERO, None));
        assert_eq!(shim.name(), "null");
        assert_eq!(shim.boundary(), "unix_shim");
        let mut straight = direct.connect().unwrap();
        let mut over = shim.connect().unwrap();
        let sql = "SELECT field0, field1 FROM usertable WHERE ycsb_key = ?";
        let (a, b) = (straight.prepare(sql).unwrap(), over.prepare(sql).unwrap());
        assert_eq!(a, b);
        assert_eq!(straight.explain(a), over.explain(b));
        for key in ["user1", "user42"] {
            let (mut left, mut right) = (Values::default(), Values::default());
            let rows = straight.execute(a, &[key], &mut left).unwrap();
            assert_eq!(over.execute(b, &[key], &mut right).unwrap(), rows);
            assert_eq!(left.len(), right.len());
            for at in 0..left.len() {
                assert_eq!(left.get(at), right.get(at));
            }
        }
        over.begin().unwrap();
        over.batch("UPDATE usertable SET field0 = 'x'").unwrap();
        over.commit().unwrap();
        drop(over);
    }

    /// A frame larger than the first read, and one that is cut short.
    #[test]
    fn frames_arrive_whole_or_not_at_all() {
        let (mut a, mut b) = UnixStream::pair().unwrap();
        let mut frame = Vec::new();
        start(&mut frame, BATCH);
        frame.extend(std::iter::repeat_n(b'x', 100_000));
        let mut sent = frame.clone();
        // From a thread, since the frame is more than the socket holds until it is read.
        let writer = std::thread::spawn(move || {
            send(&mut a, &mut sent).unwrap();
            a
        });
        let mut got = Vec::new();
        assert!(receive(&mut b, &mut got).unwrap());
        assert_eq!(got.len(), frame.len());
        assert!(got[5..].iter().all(|&byte| byte == b'x'));
        let mut a = writer.join().unwrap();
        a.write_all(&[9, 0, 0, 0, 1]).unwrap();
        drop(a);
        assert!(receive(&mut b, &mut got).is_err());
        assert!(!receive(&mut b, &mut got).unwrap());
    }
}
