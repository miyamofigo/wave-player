extern crate alloc;

use self::alloc::heap;
use std::io;
use std::marker::PhantomData;
use std::ptr::copy_nonoverlapping;
use std::slice;
use std::sync::atomic::{ AtomicPtr, Ordering };
use std::sync::mpsc::*;
use std::time::{ Duration, Instant };

trait IO {

    type T;
    type Req;
    type Res;

    fn start(&mut self, opt: IO_OpenOptions<Self::T>) -> io::Result<()>;
    fn sender(&self) -> Option<Sender<Self::Req>>;
    fn receiver(&self) -> Option<Receiver<Self::Res>>;
    fn timer(&self) -> Option<Timer>;
    fn stop(&self) -> io::Result<()>;
}

#[allow(non_camel_case_types)]
trait IO_OpenOptions<T> {
    fn open(&mut self, name: String) -> io::Result<T>;
}

trait CallbackBuilder {

    type IO;
    type Req;
    type Res;

    fn build_callback(&self) -> Box<FnOnce(&mut Self::IO,
        Sender<Self::Req>,
        Receiver<Self::Res>,
        Timer) + Send>;
}

macro_rules! define_buffer {

    ($name:ident, io::$iot:ident) => {

        #[allow(dead_code)]
        struct $name<T: io::$iot> {
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

macro_rules! impl_buffer {

    ($name:ident, io::$iot:ident) => {

        #[allow(dead_code)]
        impl<T: io::$iot> $name<T> {
            
            fn size(&self) -> usize {
                self.size
            }
            
            fn align(&self) -> usize {
                self.align
            }

            unsafe fn load(&mut self) -> &mut [u8] {
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

#[allow(dead_code)]
impl<T: io::Read> ReadBuffer<T> {

    fn new(size: usize, align: usize) -> Self {
        ReadBuffer {
            inner: alloc_atomic_buf!(size, align),
            size: size,
            align: align,
            _type: PhantomData
        } 
    }

    fn read(&mut self, input: &mut T) -> io::Result<usize> {
        let buf = unsafe { self.load() };
        input.read(buf)  
    }
}

bf!(WriteBuffer, io::Write);

#[allow(dead_code)]
impl<T: io::Write> WriteBuffer<T> {

    fn new(buf: &[u8], align: usize) -> Self {
        
        let inner = alloc_atomic_buf!(buf.len(), 
            align);

        unsafe {
            copy_nonoverlapping(buf.as_ptr(),
                inner.load(Ordering::Relaxed),
                buf.len());
        }
        
        WriteBuffer {
            inner: inner,
            size: align,
            align: align,
            _type: PhantomData
        }
    }

    fn write(&mut self, output: &mut T) -> io::Result<usize> {
        let buf = unsafe { self.load() };
        output.write(buf) 
    }
}

#[allow(dead_code)]
struct Timer {
    start   : Option<Instant>,
    timeout : Duration 
}

#[allow(dead_code)]
impl Timer {

    fn new(secs: u64, nanos: u32) -> Self {
        Timer {
            start: None,
            timeout: Duration::new(secs, nanos)
        }
    }

    fn start(&mut self) {
        match self.start {
            Some(_) => panic!("has already started."),
            _ => self.start = Some(Instant::now())
        }
    }

    fn is_timeout(self) -> bool {
        match self.start {
            Some(instant) => instant.elapsed() > self.timeout,
            _ => panic!("has not started yet.") 
        } 
    }
}

trait Loop {

    type Cb: Fn<Self::Args>;
    type Args;
    type Message;

    fn run(&self, callback: Self::Cb, args: Self::Args);
    fn handle_message(&self, msg: Self::Message);
}

