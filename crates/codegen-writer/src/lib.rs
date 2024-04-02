#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::BufWriter;
use std::mem;

pub struct Codegen {
    writer: Box<dyn io::Write>,
}

impl Codegen {
    pub fn new(writer: impl io::Write + 'static) -> Self {
        Self {
            writer: Box::new(writer),
        }
    }

    pub fn create_file(path: &str) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self::new(BufWriter::with_capacity(1024 * 1024, file)))
    }
}

impl io::Write for Codegen {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Codegen>> = const { RefCell::new(None) };
}

pub fn scoped(g: Codegen, f: impl FnOnce()) -> Codegen {
    let prev = CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        cur.replace(g)
    });

    f();

    CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        mem::replace(&mut *cur, prev).unwrap()
    })
}

pub fn with<T>(f: impl FnOnce(&mut Codegen) -> T) -> T {
    CURRENT.with(|current| {
        let mut cur = current.borrow_mut();
        let g = cur.as_mut().expect("codegen is not in scope");
        f(g)
    })
}

#[macro_export]
macro_rules! g {
    () => {{
        use ::std::io::Write;
        $crate::with(|g| writeln!(g)).unwrap();
    }};
    ($fmt: literal) => {{
        use ::std::io::Write;
        $crate::with(|g| writeln!(g, $fmt)).unwrap();
    }};
    ($fmt: literal, $($arg: tt)*) => {{
        use ::std::io::Write;
        $crate::with(|g| writeln!(g, $fmt, $($arg)*)).unwrap();
    }};
}

#[macro_export]
macro_rules! glines {
    [$($line:literal)+] => {{
        use ::std::io::Write;
        $crate::with(|g| {
            $(
                let line: &str = $line;
                writeln!(g, "{line}").unwrap();
            )+
        });
    }};
}
