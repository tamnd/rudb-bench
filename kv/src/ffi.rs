//! The only module in the driver with `unsafe` in it.
//!
//! It loads SQLite, the DuckDB pin and libpq with `dlopen` at run time, from the paths the harness
//! passes, and wraps the few calls each backend makes in safe types. The backends above it are safe
//! code and never see a raw pointer. Loading at run time rather than linking means the driver builds
//! on a machine that has none of the three, and a run names the exact file it loaded.
//!
//! Every `unsafe` block says what makes it sound. Most of them rest on the same three facts: the
//! function pointer was looked up by its documented name in a library that exports it with that C
//! signature, every pointer passed in is either one the library handed out and has not been freed,
//! or a Rust buffer that outlives the call, and every pointer handed back is read before the next
//! call on the same handle, which is as long as each library promises it stays valid.

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr;
use std::time::Duration;

use crate::backend::{Failed, Values};

const RTLD_NOW: c_int = 2;
const RUSAGE_SELF: c_int = 0;

#[repr(C)]
#[derive(Default)]
struct Timeval {
    seconds: i64,
    #[cfg(target_os = "macos")]
    micros: i32,
    #[cfg(target_os = "macos")]
    _pad: i32,
    #[cfg(not(target_os = "macos"))]
    micros: i64,
}

#[repr(C)]
#[derive(Default)]
struct Rusage {
    user: Timeval,
    system: Timeval,
    rest: [i64; 14],
}

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlerror() -> *mut c_char;
    fn getrusage(who: c_int, usage: *mut Rusage) -> c_int;
}

/// What the process has spent so far, from `getrusage`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Usage {
    pub(crate) user: Duration,
    pub(crate) system: Duration,
    /// The peak resident set, in bytes.
    pub(crate) peak: u64,
}

/// What the process has spent so far.
pub(crate) fn usage() -> Usage {
    let mut raw = Rusage::default();
    // SAFETY: `raw` is a live, writable struct with the layout of `struct rusage` on the two
    // platforms the driver builds for, and getrusage writes only inside it.
    let status = unsafe { getrusage(RUSAGE_SELF, &mut raw) };
    if status != 0 {
        return Usage::default();
    }
    let time = |t: &Timeval| {
        Duration::from_secs(u64::try_from(t.seconds).unwrap_or(0))
            + Duration::from_micros(u64::try_from(t.micros).unwrap_or(0))
    };
    let peak = u64::try_from(raw.rest[0]).unwrap_or(0);
    // Linux says kilobytes and macOS says bytes.
    let peak = if cfg!(target_os = "macos") { peak } else { peak * 1024 };
    Usage { user: time(&raw.user), system: time(&raw.system), peak }
}

/// The text of a C string the library owns, or an empty string for a null pointer.
fn text(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return String::new();
    }
    // SAFETY: every caller passes a pointer the library returned as a NUL-terminated string, and
    // reads it before making another call on the handle it came from.
    unsafe { CStr::from_ptr(pointer) }.to_string_lossy().into_owned()
}

fn c_string(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| format!("{value:?} has a NUL byte in it"))
}

/// A shared library loaded for the life of the process. It is never closed, since the backends
/// that use it live as long as the process does.
#[derive(Debug)]
struct Library {
    handle: *mut c_void,
    path: String,
}

impl Library {
    fn open(path: &str) -> Result<Self, String> {
        let name = c_string(path)?;
        // SAFETY: `name` is NUL-terminated and outlives the call.
        let handle = unsafe { dlopen(name.as_ptr(), RTLD_NOW) };
        if handle.is_null() {
            // SAFETY: dlerror returns null or a string that stays valid until the next dl call on
            // this thread, and `text` copies it at once.
            let why = text(unsafe { dlerror() });
            return Err(format!("could not load {path}: {why}"));
        }
        Ok(Self { handle, path: path.to_string() })
    }

    fn address(&self, name: &str) -> Result<*mut c_void, String> {
        let symbol = c_string(name)?;
        // SAFETY: the handle came from dlopen and was never closed, and `symbol` is NUL-terminated.
        let address = unsafe { dlsym(self.handle, symbol.as_ptr()) };
        if address.is_null() {
            return Err(format!("{} has no {name}", self.path));
        }
        Ok(address)
    }
}

/// A symbol's address as the function pointer type it is declared with.
///
/// # Safety
///
/// `F` has to be the function pointer type of the C function at `address`.
unsafe fn cast<F: Copy>(address: *mut c_void) -> F {
    assert_eq!(size_of::<F>(), size_of::<*mut c_void>(), "a function pointer is a pointer");
    // SAFETY: the sizes match, and the caller says the bits are a function of type `F`.
    unsafe { std::mem::transmute_copy::<*mut c_void, F>(&address) }
}

/// Looks a function up and gives it the type of the field it is assigned to.
macro_rules! bind {
    ($library:expr, $name:literal) => {{
        let address = $library.address($name)?;
        // SAFETY: the symbol is the library's exported C function of that name, and the field this
        // lands in declares the signature the library's header gives it.
        unsafe { cast(address) }
    }};
}

// SQLite.

const SQLITE_OK: c_int = 0;
const SQLITE_BUSY: c_int = 5;
const SQLITE_LOCKED: c_int = 6;
const SQLITE_ROW: c_int = 100;
const SQLITE_DONE: c_int = 101;
/// Read and write, create if missing, and no mutex, since a connection never leaves its thread.
const SQLITE_OPEN: c_int = 0x2 | 0x4 | 0x8000;

type Handle = *mut c_void;

struct SqliteApi {
    libversion: unsafe extern "C" fn() -> *const c_char,
    open_v2: unsafe extern "C" fn(*const c_char, *mut Handle, c_int, *const c_char) -> c_int,
    close_v2: unsafe extern "C" fn(Handle) -> c_int,
    errmsg: unsafe extern "C" fn(Handle) -> *const c_char,
    exec: unsafe extern "C" fn(
        Handle,
        *const c_char,
        *const c_void,
        *mut c_void,
        *mut *mut c_char,
    ) -> c_int,
    free: unsafe extern "C" fn(*mut c_void),
    prepare_v2: unsafe extern "C" fn(
        Handle,
        *const c_char,
        c_int,
        *mut Handle,
        *mut *const c_char,
    ) -> c_int,
    bind_text: unsafe extern "C" fn(Handle, c_int, *const c_char, c_int, isize) -> c_int,
    step: unsafe extern "C" fn(Handle) -> c_int,
    reset: unsafe extern "C" fn(Handle) -> c_int,
    finalize: unsafe extern "C" fn(Handle) -> c_int,
    column_count: unsafe extern "C" fn(Handle) -> c_int,
    column_blob: unsafe extern "C" fn(Handle, c_int) -> *const c_void,
    column_bytes: unsafe extern "C" fn(Handle, c_int) -> c_int,
    changes: unsafe extern "C" fn(Handle) -> c_int,
}

/// libsqlite3, loaded.
pub(crate) struct Sqlite {
    api: SqliteApi,
    path: String,
}

impl std::fmt::Debug for Sqlite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sqlite").field("path", &self.path).finish()
    }
}

impl Sqlite {
    pub(crate) fn load(path: &str) -> Result<Self, String> {
        let library = Library::open(path)?;
        let api = SqliteApi {
            libversion: bind!(library, "sqlite3_libversion"),
            open_v2: bind!(library, "sqlite3_open_v2"),
            close_v2: bind!(library, "sqlite3_close_v2"),
            errmsg: bind!(library, "sqlite3_errmsg"),
            exec: bind!(library, "sqlite3_exec"),
            free: bind!(library, "sqlite3_free"),
            prepare_v2: bind!(library, "sqlite3_prepare_v2"),
            bind_text: bind!(library, "sqlite3_bind_text"),
            step: bind!(library, "sqlite3_step"),
            reset: bind!(library, "sqlite3_reset"),
            finalize: bind!(library, "sqlite3_finalize"),
            column_count: bind!(library, "sqlite3_column_count"),
            column_blob: bind!(library, "sqlite3_column_blob"),
            column_bytes: bind!(library, "sqlite3_column_bytes"),
            changes: bind!(library, "sqlite3_changes"),
        };
        Ok(Self { api, path: path.to_string() })
    }

    pub(crate) fn version(&self) -> String {
        // SAFETY: sqlite3_libversion takes nothing and returns a static string.
        text(unsafe { (self.api.libversion)() })
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    /// One connection to the file at `path`.
    pub(crate) fn open<'a>(&'a self, path: &str) -> Result<SqliteConnection<'a>, String> {
        let name = c_string(path)?;
        let mut db: Handle = ptr::null_mut();
        // SAFETY: `name` outlives the call, `db` is a writable slot, and a null VFS name means the
        // default one.
        let status =
            unsafe { (self.api.open_v2)(name.as_ptr(), &mut db, SQLITE_OPEN, ptr::null()) };
        let connection = SqliteConnection { sqlite: self, db, statements: Vec::new() };
        if status != SQLITE_OK {
            return Err(format!("sqlite could not open {path}: {}", connection.error()));
        }
        Ok(connection)
    }
}

/// One SQLite connection and the statements prepared on it.
pub(crate) struct SqliteConnection<'a> {
    sqlite: &'a Sqlite,
    db: Handle,
    statements: Vec<Handle>,
}

// SAFETY: the connection is opened with SQLITE_OPEN_NOMUTEX, which allows a connection to be used
// from any one thread at a time, and `&mut self` on every call makes it one at a time.
unsafe impl Send for SqliteConnection<'_> {}

impl std::fmt::Debug for SqliteConnection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteConnection").field("statements", &self.statements.len()).finish()
    }
}

impl SqliteConnection<'_> {
    fn error(&self) -> String {
        // SAFETY: the handle is open, or null after a failed open, and sqlite3_errmsg accepts both.
        text(unsafe { (self.sqlite.api.errmsg)(self.db) })
    }

    fn failed(&self, status: c_int) -> Failed {
        let message = self.error();
        if status & 0xff == SQLITE_BUSY || status & 0xff == SQLITE_LOCKED {
            Failed::Retry(message)
        } else {
            Failed::Error(message)
        }
    }

    pub(crate) fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        let sql = c_string(sql).map_err(Failed::Error)?;
        let mut message: *mut c_char = ptr::null_mut();
        // SAFETY: the handle is open, `sql` outlives the call, there is no callback, and `message`
        // is a writable slot that sqlite fills with a string it allocated or leaves null.
        let status = unsafe {
            (self.sqlite.api.exec)(
                self.db,
                sql.as_ptr(),
                ptr::null(),
                ptr::null_mut(),
                &mut message,
            )
        };
        let why = text(message);
        if !message.is_null() {
            // SAFETY: `message` was allocated by sqlite and is freed once, here.
            unsafe { (self.sqlite.api.free)(message.cast()) };
        }
        if status == SQLITE_OK {
            Ok(())
        } else if status & 0xff == SQLITE_BUSY || status & 0xff == SQLITE_LOCKED {
            Err(Failed::Retry(why))
        } else {
            Err(Failed::Error(why))
        }
    }

    pub(crate) fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        let text_sql = c_string(sql)?;
        let mut statement: Handle = ptr::null_mut();
        // SAFETY: the handle is open, `text_sql` is NUL-terminated so -1 is a valid length, and
        // `statement` is a writable slot. The tail pointer is not wanted.
        let status = unsafe {
            (self.sqlite.api.prepare_v2)(
                self.db,
                text_sql.as_ptr(),
                -1,
                &mut statement,
                ptr::null_mut(),
            )
        };
        if status != SQLITE_OK || statement.is_null() {
            return Err(format!("sqlite could not prepare {sql:?}: {}", self.error()));
        }
        self.statements.push(statement);
        Ok(self.statements.len() - 1)
    }

    pub(crate) fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        let api = &self.sqlite.api;
        let handle = self.statements[statement];
        for (at, value) in parameters.iter().enumerate() {
            let length =
                c_int::try_from(value.len()).map_err(|_| Failed::Error("too long".into()))?;
            // SAFETY: the statement is prepared and reset, the index is from one, and the text is
            // bound as SQLITE_STATIC (0), which is sound because `parameters` outlives the step
            // and the reset below, after which sqlite no longer reads it.
            let status = unsafe {
                (api.bind_text)(handle, at as c_int + 1, value.as_ptr().cast(), length, 0)
            };
            if status != SQLITE_OK {
                // SAFETY: as above.
                unsafe { (api.reset)(handle) };
                return Err(self.failed(status));
            }
        }
        let mut rows = 0;
        let result = loop {
            // SAFETY: the statement is prepared and every parameter is bound.
            let status = unsafe { (api.step)(handle) };
            match status {
                SQLITE_ROW => {
                    rows += 1;
                    // SAFETY: the statement is on a row.
                    let columns = unsafe { (api.column_count)(handle) };
                    for column in 0..columns {
                        // SAFETY: the statement is on a row and the column is in range. The
                        // pointer is read before the next call on the statement, and the length
                        // is asked for after the pointer, which is the order sqlite documents.
                        let value = unsafe {
                            let data = (api.column_blob)(handle, column);
                            let length = (api.column_bytes)(handle, column);
                            if data.is_null() || length <= 0 {
                                &[][..]
                            } else {
                                std::slice::from_raw_parts(data.cast::<u8>(), length as usize)
                            }
                        };
                        out.push(value);
                    }
                }
                SQLITE_DONE => {
                    // SAFETY: the statement is prepared, and the connection is open.
                    if unsafe { (api.column_count)(handle) } == 0 {
                        // A statement with no columns wrote, and what it read is what it changed.
                        // SAFETY: as above.
                        rows = u64::try_from(unsafe { (api.changes)(self.db) }).unwrap_or(0);
                    }
                    break Ok(rows);
                }
                status => break Err(self.failed(status)),
            }
        };
        // SAFETY: the statement is prepared. Resetting it releases its read of the database and
        // unbinds nothing, and every parameter is bound again before the next step.
        unsafe { (api.reset)(handle) };
        result
    }
}

impl Drop for SqliteConnection<'_> {
    fn drop(&mut self) {
        for statement in self.statements.drain(..) {
            // SAFETY: each statement was prepared on this connection and is finalized once.
            unsafe { (self.sqlite.api.finalize)(statement) };
        }
        // SAFETY: the handle came from sqlite3_open_v2 and is closed once. close_v2 accepts null.
        unsafe { (self.sqlite.api.close_v2)(self.db) };
    }
}

// DuckDB.

const DUCKDB_SUCCESS: c_int = 0;
const DUCKDB_TYPE_VARCHAR: c_int = 17;

/// `duckdb_result`, whose fields the driver never reads directly.
#[repr(C)]
struct DuckResult {
    column_count: u64,
    row_count: u64,
    rows_changed: u64,
    columns: *mut c_void,
    error: *mut c_char,
    internal: *mut c_void,
}

impl DuckResult {
    const fn empty() -> Self {
        Self {
            column_count: 0,
            row_count: 0,
            rows_changed: 0,
            columns: ptr::null_mut(),
            error: ptr::null_mut(),
            internal: ptr::null_mut(),
        }
    }
}

struct DuckApi {
    library_version: unsafe extern "C" fn() -> *const c_char,
    open_ext: unsafe extern "C" fn(*const c_char, *mut Handle, Handle, *mut *mut c_char) -> c_int,
    close: unsafe extern "C" fn(*mut Handle),
    free: unsafe extern "C" fn(*mut c_void),
    connect: unsafe extern "C" fn(Handle, *mut Handle) -> c_int,
    disconnect: unsafe extern "C" fn(*mut Handle),
    query: unsafe extern "C" fn(Handle, *const c_char, *mut DuckResult) -> c_int,
    prepare: unsafe extern "C" fn(Handle, *const c_char, *mut Handle) -> c_int,
    prepare_error: unsafe extern "C" fn(Handle) -> *const c_char,
    destroy_prepare: unsafe extern "C" fn(*mut Handle),
    bind_varchar_length: unsafe extern "C" fn(Handle, u64, *const c_char, u64) -> c_int,
    execute_prepared: unsafe extern "C" fn(Handle, *mut DuckResult) -> c_int,
    result_error: unsafe extern "C" fn(*mut DuckResult) -> *const c_char,
    destroy_result: unsafe extern "C" fn(*mut DuckResult),
    rows_changed: unsafe extern "C" fn(*mut DuckResult) -> u64,
    column_count: unsafe extern "C" fn(*mut DuckResult) -> u64,
    column_type: unsafe extern "C" fn(*mut DuckResult, u64) -> c_int,
    fetch_chunk: unsafe extern "C" fn(DuckResult) -> Handle,
    chunk_size: unsafe extern "C" fn(Handle) -> u64,
    chunk_vector: unsafe extern "C" fn(Handle, u64) -> Handle,
    vector_data: unsafe extern "C" fn(Handle) -> *mut c_void,
    vector_validity: unsafe extern "C" fn(Handle) -> *mut u64,
    destroy_chunk: unsafe extern "C" fn(*mut Handle),
}

/// libduckdb, loaded, and one database opened with it.
pub(crate) struct Duckdb {
    api: DuckApi,
    database: Handle,
    path: String,
}

// SAFETY: a duckdb_database may be used from any thread, and connections to it are made from each
// client thread, which is how DuckDB's documentation says to use it from more than one thread.
unsafe impl Send for Duckdb {}
// SAFETY: as above. The only call made through `&self` from more than one thread is connect.
unsafe impl Sync for Duckdb {}

impl std::fmt::Debug for Duckdb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Duckdb").field("path", &self.path).finish()
    }
}

/// The version libduckdb at `path` reports, without opening a database.
pub(crate) fn duckdb_version(path: &str) -> Result<String, String> {
    let library = Library::open(path)?;
    let version: unsafe extern "C" fn() -> *const c_char = bind!(library, "duckdb_library_version");
    // SAFETY: duckdb_library_version takes nothing and returns a static string.
    Ok(text(unsafe { version() }))
}

impl Duckdb {
    pub(crate) fn open(library_path: &str, database: &str) -> Result<Self, String> {
        let library = Library::open(library_path)?;
        let api = DuckApi {
            library_version: bind!(library, "duckdb_library_version"),
            open_ext: bind!(library, "duckdb_open_ext"),
            close: bind!(library, "duckdb_close"),
            free: bind!(library, "duckdb_free"),
            connect: bind!(library, "duckdb_connect"),
            disconnect: bind!(library, "duckdb_disconnect"),
            query: bind!(library, "duckdb_query"),
            prepare: bind!(library, "duckdb_prepare"),
            prepare_error: bind!(library, "duckdb_prepare_error"),
            destroy_prepare: bind!(library, "duckdb_destroy_prepare"),
            bind_varchar_length: bind!(library, "duckdb_bind_varchar_length"),
            execute_prepared: bind!(library, "duckdb_execute_prepared"),
            result_error: bind!(library, "duckdb_result_error"),
            destroy_result: bind!(library, "duckdb_destroy_result"),
            rows_changed: bind!(library, "duckdb_rows_changed"),
            column_count: bind!(library, "duckdb_column_count"),
            column_type: bind!(library, "duckdb_column_type"),
            fetch_chunk: bind!(library, "duckdb_fetch_chunk"),
            chunk_size: bind!(library, "duckdb_data_chunk_get_size"),
            chunk_vector: bind!(library, "duckdb_data_chunk_get_vector"),
            vector_data: bind!(library, "duckdb_vector_get_data"),
            vector_validity: bind!(library, "duckdb_vector_get_validity"),
            destroy_chunk: bind!(library, "duckdb_destroy_data_chunk"),
        };
        let name = c_string(database)?;
        let mut handle: Handle = ptr::null_mut();
        let mut message: *mut c_char = ptr::null_mut();
        // SAFETY: `name` outlives the call, a null config means the defaults, and both out slots
        // are writable.
        let status =
            unsafe { (api.open_ext)(name.as_ptr(), &mut handle, ptr::null_mut(), &mut message) };
        let why = text(message);
        if !message.is_null() {
            // SAFETY: the message was allocated by duckdb and is freed once, with its own free.
            unsafe { (api.free)(message.cast()) };
        }
        if status != DUCKDB_SUCCESS {
            return Err(format!("duckdb could not open {database}: {why}"));
        }
        Ok(Self { api, database: handle, path: library_path.to_string() })
    }

    pub(crate) fn version(&self) -> String {
        // SAFETY: duckdb_library_version takes nothing and returns a static string.
        text(unsafe { (self.api.library_version)() })
    }

    pub(crate) fn connect(&self) -> Result<DuckConnection<'_>, String> {
        let mut connection: Handle = ptr::null_mut();
        // SAFETY: the database is open until `self` drops, which outlives the borrow the returned
        // connection holds.
        let status = unsafe { (self.api.connect)(self.database, &mut connection) };
        if status != DUCKDB_SUCCESS {
            return Err("duckdb could not connect".to_string());
        }
        Ok(DuckConnection { duckdb: self, connection, statements: Vec::new() })
    }
}

impl Drop for Duckdb {
    fn drop(&mut self) {
        // SAFETY: the database was opened once and every connection borrowing it has dropped.
        unsafe { (self.api.close)(&mut self.database) };
    }
}

/// One DuckDB connection and the statements prepared on it.
pub(crate) struct DuckConnection<'a> {
    duckdb: &'a Duckdb,
    connection: Handle,
    statements: Vec<Handle>,
}

// SAFETY: a duckdb_connection may be used from any one thread at a time, and `&mut self` on every
// call makes it one at a time.
unsafe impl Send for DuckConnection<'_> {}

impl std::fmt::Debug for DuckConnection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuckConnection").field("statements", &self.statements.len()).finish()
    }
}

/// Whether a DuckDB error is one a client retries, which is a write-write conflict.
fn duck_failed(message: String) -> Failed {
    if message.contains("Conflict") || message.contains("conflict") {
        Failed::Retry(message)
    } else {
        Failed::Error(message)
    }
}

impl DuckConnection<'_> {
    pub(crate) fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        let api = &self.duckdb.api;
        let sql = c_string(sql).map_err(Failed::Error)?;
        let mut result = DuckResult::empty();
        // SAFETY: the connection is open, `sql` outlives the call, and `result` is a writable
        // duckdb_result that is destroyed below whatever the status.
        let status = unsafe { (api.query)(self.connection, sql.as_ptr(), &mut result) };
        let outcome = if status == DUCKDB_SUCCESS {
            Ok(())
        } else {
            // SAFETY: `result` was filled by duckdb_query and not yet destroyed.
            Err(duck_failed(text(unsafe { (api.result_error)(&mut result) })))
        };
        // SAFETY: `result` was filled by duckdb_query and is destroyed once.
        unsafe { (api.destroy_result)(&mut result) };
        outcome
    }

    pub(crate) fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        let api = &self.duckdb.api;
        let text_sql = c_string(sql)?;
        let mut statement: Handle = ptr::null_mut();
        // SAFETY: the connection is open, `text_sql` outlives the call and `statement` is writable.
        let status = unsafe { (api.prepare)(self.connection, text_sql.as_ptr(), &mut statement) };
        if status != DUCKDB_SUCCESS {
            // SAFETY: a failed prepare still hands back a statement that holds the error, and it
            // is destroyed once after the error is copied.
            let why = text(unsafe { (api.prepare_error)(statement) });
            // SAFETY: as above.
            unsafe { (api.destroy_prepare)(&mut statement) };
            return Err(format!("duckdb could not prepare {sql:?}: {why}"));
        }
        self.statements.push(statement);
        Ok(self.statements.len() - 1)
    }

    pub(crate) fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        let api = &self.duckdb.api;
        let handle = self.statements[statement];
        for (at, value) in parameters.iter().enumerate() {
            // SAFETY: the statement is prepared, the index is from one, and duckdb copies the
            // bytes during the call.
            let status = unsafe {
                (api.bind_varchar_length)(
                    handle,
                    at as u64 + 1,
                    value.as_ptr().cast(),
                    value.len() as u64,
                )
            };
            if status != DUCKDB_SUCCESS {
                return Err(Failed::Error(format!("duckdb could not bind parameter {}", at + 1)));
            }
        }
        let mut result = DuckResult::empty();
        // SAFETY: the statement is prepared with every parameter bound, and `result` is writable
        // and destroyed below whatever the status.
        let status = unsafe { (api.execute_prepared)(handle, &mut result) };
        if status != DUCKDB_SUCCESS {
            // SAFETY: `result` was filled and is destroyed once, after the error is copied.
            let why = text(unsafe { (api.result_error)(&mut result) });
            // SAFETY: as above.
            unsafe { (api.destroy_result)(&mut result) };
            return Err(duck_failed(why));
        }
        // SAFETY: `result` holds a successful result that has not been destroyed.
        let columns = unsafe { (api.column_count)(&mut result) };
        let mut rows = 0;
        if columns > 0 {
            for column in 0..columns {
                // SAFETY: as above, and the column is in range.
                let kind = unsafe { (api.column_type)(&mut result, column) };
                if kind != DUCKDB_TYPE_VARCHAR {
                    // SAFETY: destroyed once.
                    unsafe { (api.destroy_result)(&mut result) };
                    return Err(Failed::Error(format!(
                        "duckdb returned column {column} as type {kind}"
                    )));
                }
            }
            loop {
                // SAFETY: duckdb_fetch_chunk takes the result by value but only reads the handle in
                // it, and the struct is copied bit for bit here, which leaves `result` still the
                // owner that is destroyed below.
                let mut chunk = unsafe { (api.fetch_chunk)(ptr::read(&result)) };
                if chunk.is_null() {
                    break;
                }
                // SAFETY: the chunk came from fetch_chunk and is destroyed at the end of this pass.
                let size = unsafe { (api.chunk_size)(chunk) };
                for row in 0..size {
                    for column in 0..columns {
                        // SAFETY: the column is in range, a VARCHAR vector's data is an array of
                        // `size` 16 byte duckdb_string_t, and the validity mask is null or holds
                        // at least `size` bits. A string of up to 12 bytes is inline after its
                        // 4 byte length and a longer one is a pointer at offset 8, which stays
                        // valid while the chunk does.
                        let value = unsafe {
                            let vector = (api.chunk_vector)(chunk, column);
                            let validity = (api.vector_validity)(vector);
                            let valid = validity.is_null()
                                || *validity.add((row / 64) as usize) & (1 << (row % 64)) != 0;
                            if valid {
                                let entry =
                                    (api.vector_data)(vector).cast::<u8>().add(row as usize * 16);
                                let length = ptr::read_unaligned(entry.cast::<u32>()) as usize;
                                let data = if length <= 12 {
                                    entry.add(4)
                                } else {
                                    ptr::read_unaligned(entry.add(8).cast::<*const u8>())
                                };
                                std::slice::from_raw_parts(data, length)
                            } else {
                                &[][..]
                            }
                        };
                        out.push(value);
                    }
                }
                rows += size;
                // SAFETY: destroyed once, after every value read from it was copied into `out`.
                unsafe { (api.destroy_chunk)(&mut chunk) };
            }
        } else {
            // SAFETY: `result` holds a successful result that has not been destroyed.
            rows = unsafe { (api.rows_changed)(&mut result) };
        }
        // SAFETY: destroyed once.
        unsafe { (api.destroy_result)(&mut result) };
        Ok(rows)
    }
}

impl Drop for DuckConnection<'_> {
    fn drop(&mut self) {
        let api = &self.duckdb.api;
        for mut statement in self.statements.drain(..) {
            // SAFETY: each statement was prepared on this connection and is destroyed once.
            unsafe { (api.destroy_prepare)(&mut statement) };
        }
        // SAFETY: the connection was made once and is closed once, before the database.
        unsafe { (api.disconnect)(&mut self.connection) };
    }
}

// libpq.

const CONNECTION_OK: c_int = 0;
const PGRES_COMMAND_OK: c_int = 1;
const PGRES_TUPLES_OK: c_int = 2;
const PG_DIAG_SQLSTATE: c_int = b'C' as c_int;

struct PqApi {
    lib_version: unsafe extern "C" fn() -> c_int,
    connectdb: unsafe extern "C" fn(*const c_char) -> Handle,
    status: unsafe extern "C" fn(Handle) -> c_int,
    error_message: unsafe extern "C" fn(Handle) -> *const c_char,
    server_version: unsafe extern "C" fn(Handle) -> c_int,
    finish: unsafe extern "C" fn(Handle),
    exec: unsafe extern "C" fn(Handle, *const c_char) -> Handle,
    prepare:
        unsafe extern "C" fn(Handle, *const c_char, *const c_char, c_int, *const u32) -> Handle,
    exec_prepared: unsafe extern "C" fn(
        Handle,
        *const c_char,
        c_int,
        *const *const c_char,
        *const c_int,
        *const c_int,
        c_int,
    ) -> Handle,
    result_status: unsafe extern "C" fn(Handle) -> c_int,
    result_error_message: unsafe extern "C" fn(Handle) -> *const c_char,
    result_error_field: unsafe extern "C" fn(Handle, c_int) -> *const c_char,
    ntuples: unsafe extern "C" fn(Handle) -> c_int,
    nfields: unsafe extern "C" fn(Handle) -> c_int,
    getvalue: unsafe extern "C" fn(Handle, c_int, c_int) -> *const c_char,
    getlength: unsafe extern "C" fn(Handle, c_int, c_int) -> c_int,
    cmd_tuples: unsafe extern "C" fn(Handle) -> *const c_char,
    clear: unsafe extern "C" fn(Handle),
}

/// libpq, loaded.
pub(crate) struct Pq {
    api: PqApi,
    path: String,
}

impl std::fmt::Debug for Pq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pq").field("path", &self.path).finish()
    }
}

impl Pq {
    pub(crate) fn load(path: &str) -> Result<Self, String> {
        let library = Library::open(path)?;
        let api = PqApi {
            lib_version: bind!(library, "PQlibVersion"),
            connectdb: bind!(library, "PQconnectdb"),
            status: bind!(library, "PQstatus"),
            error_message: bind!(library, "PQerrorMessage"),
            server_version: bind!(library, "PQserverVersion"),
            finish: bind!(library, "PQfinish"),
            exec: bind!(library, "PQexec"),
            prepare: bind!(library, "PQprepare"),
            exec_prepared: bind!(library, "PQexecPrepared"),
            result_status: bind!(library, "PQresultStatus"),
            result_error_message: bind!(library, "PQresultErrorMessage"),
            result_error_field: bind!(library, "PQresultErrorField"),
            ntuples: bind!(library, "PQntuples"),
            nfields: bind!(library, "PQnfields"),
            getvalue: bind!(library, "PQgetvalue"),
            getlength: bind!(library, "PQgetlength"),
            cmd_tuples: bind!(library, "PQcmdTuples"),
            clear: bind!(library, "PQclear"),
        };
        Ok(Self { api, path: path.to_string() })
    }

    pub(crate) fn library_version(&self) -> i32 {
        // SAFETY: PQlibVersion takes nothing.
        unsafe { (self.api.lib_version)() }
    }

    pub(crate) fn connect(&self, conninfo: &str) -> Result<PqConnection<'_>, String> {
        let info = c_string(conninfo)?;
        // SAFETY: `info` outlives the call. PQconnectdb returns null only when out of memory.
        let connection = unsafe { (self.api.connectdb)(info.as_ptr()) };
        if connection.is_null() {
            return Err("libpq could not allocate a connection".to_string());
        }
        let connection = PqConnection { pq: self, connection, statements: 0, buffer: Vec::new() };
        // SAFETY: the connection is not null.
        if unsafe { (self.api.status)(connection.connection) } != CONNECTION_OK {
            return Err(format!("postgres refused {conninfo:?}: {}", connection.error().trim()));
        }
        Ok(connection)
    }
}

/// One libpq connection, which is one backend process on the server.
pub(crate) struct PqConnection<'a> {
    pq: &'a Pq,
    connection: Handle,
    statements: usize,
    /// The parameters of the statement being run, each with a NUL after it.
    buffer: Vec<u8>,
}

// SAFETY: a PGconn may be used from any one thread at a time, and `&mut self` on every call makes
// it one at a time.
unsafe impl Send for PqConnection<'_> {}

impl std::fmt::Debug for PqConnection<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PqConnection").field("statements", &self.statements).finish()
    }
}

impl PqConnection<'_> {
    fn error(&self) -> String {
        // SAFETY: the connection is open.
        text(unsafe { (self.pq.api.error_message)(self.connection) })
    }

    pub(crate) fn server_version(&self) -> i32 {
        // SAFETY: the connection is open.
        unsafe { (self.pq.api.server_version)(self.connection) }
    }

    /// What a result says, and frees it.
    fn finish(&self, result: Handle, out: Option<&mut Values>) -> Result<u64, Failed> {
        let api = &self.pq.api;
        if result.is_null() {
            return Err(Failed::Error(self.error()));
        }
        // SAFETY: `result` came from libpq and is cleared once at the end of this function.
        let status = unsafe { (api.result_status)(result) };
        let outcome = match status {
            PGRES_TUPLES_OK => {
                // SAFETY: as above.
                let (rows, fields) = unsafe { ((api.ntuples)(result), (api.nfields)(result)) };
                if let Some(out) = out {
                    for row in 0..rows {
                        for field in 0..fields {
                            // SAFETY: row and field are in range, and the value stays valid until
                            // the result is cleared, after it is copied into `out`.
                            let value = unsafe {
                                let data = (api.getvalue)(result, row, field);
                                let length = (api.getlength)(result, row, field);
                                std::slice::from_raw_parts(
                                    data.cast::<u8>(),
                                    length.max(0) as usize,
                                )
                            };
                            out.push(value);
                        }
                    }
                }
                Ok(u64::try_from(rows).unwrap_or(0))
            }
            PGRES_COMMAND_OK => {
                // SAFETY: as above. The count is an empty string for a command with no count.
                let count = text(unsafe { (api.cmd_tuples)(result) });
                Ok(count.parse().unwrap_or(0))
            }
            _ => {
                // SAFETY: as above.
                let state = text(unsafe { (api.result_error_field)(result, PG_DIAG_SQLSTATE) });
                // SAFETY: as above.
                let message =
                    text(unsafe { (api.result_error_message)(result) }).trim().to_string();
                if state == "40001" || state == "40P01" {
                    Err(Failed::Retry(message))
                } else {
                    Err(Failed::Error(message))
                }
            }
        };
        // SAFETY: cleared once.
        unsafe { (api.clear)(result) };
        outcome
    }

    pub(crate) fn batch(&mut self, sql: &str) -> Result<(), Failed> {
        let sql = c_string(sql).map_err(Failed::Error)?;
        // SAFETY: the connection is open and `sql` outlives the call.
        let result = unsafe { (self.pq.api.exec)(self.connection, sql.as_ptr()) };
        self.finish(result, None).map(|_| ())
    }

    pub(crate) fn prepare(&mut self, sql: &str) -> Result<usize, String> {
        let name = c_string(&format!("s{}", self.statements)).map_err(|e| e.to_string())?;
        let text_sql = c_string(sql)?;
        // SAFETY: the connection is open, both strings outlive the call, and a null type array
        // lets the server infer every parameter's type.
        let result = unsafe {
            (self.pq.api.prepare)(self.connection, name.as_ptr(), text_sql.as_ptr(), 0, ptr::null())
        };
        self.finish(result, None).map_err(|failed| {
            format!("postgres could not prepare {sql:?}: {}", failed.message())
        })?;
        self.statements += 1;
        Ok(self.statements - 1)
    }

    pub(crate) fn execute(
        &mut self,
        statement: usize,
        parameters: &[&str],
        out: &mut Values,
    ) -> Result<u64, Failed> {
        let name = format!("s{statement}\0");
        self.buffer.clear();
        let mut starts = Vec::with_capacity(parameters.len());
        for value in parameters {
            starts.push(self.buffer.len());
            self.buffer.extend_from_slice(value.as_bytes());
            self.buffer.push(0);
        }
        let pointers: Vec<*const c_char> =
            starts.iter().map(|&start| self.buffer[start..].as_ptr().cast()).collect();
        // SAFETY: the connection is open, the statement name and every parameter are
        // NUL-terminated and outlive the call, and null length and format arrays mean every
        // parameter is text and the result is text.
        let result = unsafe {
            (self.pq.api.exec_prepared)(
                self.connection,
                name.as_ptr().cast(),
                parameters.len() as c_int,
                pointers.as_ptr(),
                ptr::null(),
                ptr::null(),
                0,
            )
        };
        self.finish(result, Some(out))
    }
}

impl Drop for PqConnection<'_> {
    fn drop(&mut self) {
        // SAFETY: the connection came from PQconnectdb and is finished once.
        unsafe { (self.pq.api.finish)(self.connection) };
    }
}
