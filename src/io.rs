extern crate alloc;

use self::alloc::heap;
use std;
use std::error;
use std::io;
use std::marker::PhantomData;
use std::ptr::copy_nonoverlapping;
use std::slice;
use std::sync::atomic::{ AtomicPtr, Ordering };
use std::sync::mpsc::*;
use std::time::{ Duration, Instant };

pub type IOResult<T> = io::Result<T>; 
pub type IOError = io::Error;
pub type IOErrorKind = io::ErrorKind;
pub const IO_ERROR : io::ErrorKind = io::ErrorKind::Other; 

pub fn io_error<T, E: error::Error>(err: E) -> IOResult<T> {
    Err(IOError::new(IO_ERROR,
        err.description()))
}

const NOT_IMPLEMENTED : &'static str = "not implemented";

pub trait IO {

    type T;
    type R;
    type Req;
    type Res;

    fn start(&mut self) -> io::Result<Self::R>;
    fn send(&self, req: Self::Req) -> Result<(), SendError<Self::Req>>;
    fn recv(&self) -> Result<Self::Res, RecvError>;
    fn timer(&self) -> Timer;
    fn stop(&mut self) -> io::Result<Self::R>;
    fn join(&mut self) -> std::thread::Result<Self::R>;

    // to get a reference for non-blocking IO
    fn sender(&self) -> Option<&Sender<Self::Req>> { panic!(NOT_IMPLEMENTED) }
    fn receiver(&self) -> Option<&Receiver<Self::Res>> { panic!(NOT_IMPLEMENTED) }
}

macro_rules! inner_ref {
    ( $slf:ident, $field:ident ) => {
        match $slf.$field {
            Some(ref inner) => Some(inner),
            _ => None
        }
    } 
}

macro_rules! define_io {
    ($name:ident, $t:ident, $req:ident, $res:ident, $ret:ty) => {
        pub struct $name {
            name     : String,
            handle   : Option<JoinHandle<$ret>>,
            tx       : Option<Sender<$req>>,
            rx       : Option<Receiver<$res>>,
            timer    : Timer
        }
    } 
}

macro_rules! define_buffer {

    ($name:ident, io::$iot:ident) => {

        #[allow(dead_code)]
        pub struct $name<T: io::$iot> {
            inner : AtomicPtr<u8>,
            size  : usize,
            align : usize,
            _type : PhantomData<T>
        }
    }
}

macro_rules! alloc_atomic_buf {
    ($size:expr, $align:expr) => {
        {
            let ptr = unsafe {
                 heap::allocate($size, $align)
            };
            AtomicPtr::new(ptr)
        }
    }
}

macro_rules! dealloc_atomic_buf {
    ($slf:ident) => {
        unsafe {
            heap::deallocate($slf.inner.load(Ordering::Relaxed),
                $slf.size(),
                $slf.align());
        }
    }
}

pub trait BufferLoader {
    unsafe fn load(&self) -> &[u8];
}

pub trait BufferLoaderMut {
    unsafe fn load_mut(&mut self) -> &mut [u8];
}

macro_rules! impl_buffer {

    ($name:ident, io::$iot:ident) => {

        #[allow(dead_code)]
        impl<T: io::$iot> $name<T> {
            
            pub fn size(&self) -> usize {
                self.size
            }
            
            pub fn align(&self) -> usize {
                self.align
            }
        }

        impl<T: io::$iot> BufferLoader for $name<T> {
            unsafe fn load(&self) -> &[u8] {
                let ptr = self.inner.load(Ordering::Relaxed);
                slice::from_raw_parts(ptr, self.size())
            }
        }

        impl<T: io::$iot> BufferLoaderMut for $name<T> {
            unsafe fn load_mut(&mut self) -> &mut [u8] {
                let ptr = self.inner.load(Ordering::Relaxed);
                slice::from_raw_parts_mut(ptr, self.size())
            }
        } 

        impl<T: io::$iot> Drop for $name<T> {
            fn drop(&mut self) {
                dealloc_atomic_buf!(self);
            }
        }
    }  
}

macro_rules! bf {
    ($name:ident, io::$iot:ident) => {
        define_buffer!($name, io::$iot);
        impl_buffer!($name, io::$iot);
    }
}

bf!(ReadBuffer, io::Read);

const READBUF_ALIGN : usize = 1;

impl<T: io::Read> ReadBuffer<T> {

    pub fn new(size: usize) -> Self {
        ReadBuffer {
            inner: alloc_atomic_buf!(size, READBUF_ALIGN),
            size: size,
            align: READBUF_ALIGN,
            _type: PhantomData
        } 
    }

    pub fn read(&mut self, input: &mut T) -> io::Result<usize> {
        let buf = unsafe { self.load_mut() };
        input.read(buf)        
    }
}

bf!(WriteBuffer, io::Write);

impl<T: io::Write> WriteBuffer<T> {

    pub fn new(buf: &[u8], align: usize) -> Self {
        
        let size = buf.len();
        let inner = alloc_atomic_buf!(size, align);

        unsafe {
            copy_nonoverlapping(buf.as_ptr(),
                inner.load(Ordering::Relaxed),
                buf.len());
        }
        
        WriteBuffer {
            inner: inner,
            size: size,
            align: align,
            _type: PhantomData
        }
    }

    pub fn write(&mut self, output: &mut T) -> io::Result<usize> {
        let buf = unsafe { self.load_mut() };
        output.write(buf) 
    }
}

#[derive(Clone, Copy)]
pub struct Timer {
    start   : Option<Instant>,
    timeout : Duration 
}

impl Timer {

    pub fn new(secs: u64, nanos: u32) -> Self {
        Timer {
            start: None,
            timeout: Duration::new(secs, nanos)
        }
    }

    pub fn start(&mut self) {
        match self.start {
            Some(_) => panic!("has already started."),
            _ => self.start = Some(Instant::now())
        }
    }

    pub fn is_started(&self) -> bool {
        self.start.is_some()
    }

    pub fn is_timeout(self) -> bool {
        match self.start {
            Some(instant) => instant.elapsed() > self.timeout,
            _ => panic!("has not started yet.") 
        } 
    }
}

pub trait Loop<SendMsg, RecvMsg, Ret> {

    type In;
    type Callback: FnOnce<Self::Args> + Send + ?Sized;
    type Args;
    type Out; 

    fn run(&mut self, callback: Box<Self::Callback>, input: Self::In) 
     -> io::Result<Ret>;
}

#[cfg(test)]
mod test {

    use super::*;
    use std::fs::File;
    use std::io::Read;
    use std::process::Command;
    use std::str::from_utf8;
    use std::thread;
    use std::time::Duration;

    const RIFF : &'static str = "RIFF";
    const FILEPATH : &'static str = "/usr/share/sounds/k3b_error1.wav";
    const BUFSIZE : usize = 4;
    const BUFALIGN : usize = 1;
    const OUTPUT_FILE : &'static str = "out";

    #[test]
    fn buffer_test() {
      
        let mut f = File::open(FILEPATH).unwrap();
        let mut rbuf = ReadBuffer::<File>::new(BUFSIZE);

        rbuf.read(&mut f).unwrap();
 
        let slice = unsafe {
            rbuf.load()
        };

        let riff = from_utf8(slice)
            .unwrap();

        assert_eq!(RIFF, riff);

        let mut wbuf = WriteBuffer::<File>::new(slice, 
            BUFALIGN);
        assert_eq!(4usize, wbuf.size());

        let mut out = File::create(OUTPUT_FILE).unwrap();
        wbuf.write(&mut out).unwrap();

        let mut ifstream = File::open(OUTPUT_FILE).unwrap();
        let mut st = String::new();
        ifstream.read_to_string(&mut st).unwrap(); 
        assert_eq!(RIFF, st.as_str());

        Command::new("rm").arg(OUTPUT_FILE)
            .output()
            .unwrap();
    }

    #[test]
    fn timer_test() {

        let mut timer = Timer::new(10, 0);
        timer.start();

        while !timer.is_timeout() {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
