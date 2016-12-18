extern crate libc;

use std;
use std::io;
use std::ffi::{ CStr, CString };
use std::mem::size_of;
use std::ptr::{ Unique, copy_nonoverlapping };

use io::*;

#[allow(non_camel_case_types)]
type snd_pcm_t = libc::c_void;

#[allow(non_camel_case_types)]
type snd_pcm_stream_t = i32;

const SND_PCM_STREAM_PLAYBACK : snd_pcm_stream_t = 0;
const SND_PCM_STREAM_CAPTURE  : snd_pcm_stream_t = 1;

#[allow(non_camel_case_types)]
type snd_pcm_format_t = i32;

const SND_PCM_FORMAT_UNKNOWN            : snd_pcm_format_t = -1;
const SND_PCM_FORMAT_S8                 : snd_pcm_format_t = 0;
const SND_PCM_FORMAT_U8                 : snd_pcm_format_t = 1; 
const	SND_PCM_FORMAT_S16_LE             : snd_pcm_format_t = 2;
const SND_PCM_FORMAT_S16_BE             : snd_pcm_format_t = 3;
const SND_PCM_FORMAT_U16_LE             : snd_pcm_format_t = 4;
const SND_PCM_FORMAT_U16_BE             : snd_pcm_format_t = 5;
const SND_PCM_FORMAT_S24_LE             : snd_pcm_format_t = 6;
const SND_PCM_FORMAT_S24_BE             : snd_pcm_format_t = 7;
const SND_PCM_FORMAT_U24_LE             : snd_pcm_format_t = 8;
const SND_PCM_FORMAT_U24_BE             : snd_pcm_format_t = 9;
const SND_PCM_FORMAT_S32_LE             : snd_pcm_format_t = 10;
const SND_PCM_FORMAT_S32_BE             : snd_pcm_format_t = 11;
const SND_PCM_FORMAT_U32_LE             : snd_pcm_format_t = 12;
const SND_PCM_FORMAT_U32_BE             : snd_pcm_format_t = 13;
const SND_PCM_FORMAT_FLOAT_LE           : snd_pcm_format_t = 14;
const	SND_PCM_FORMAT_FLOAT_BE           : snd_pcm_format_t = 15;
const SND_PCM_FORMAT_FLOAT64_LE         : snd_pcm_format_t = 16;
const SND_PCM_FORMAT_FLOAT64_BE         : snd_pcm_format_t = 17;
const SND_PCM_FORMAT_IEC958_SUBFRAME_LE : snd_pcm_format_t = 18;
const	SND_PCM_FORMAT_IEC958_SUBFRAME_BE : snd_pcm_format_t = 19;
const	SND_PCM_FORMAT_MU_LAW             : snd_pcm_format_t = 20;
const SND_PCM_FORMAT_A_LAW              : snd_pcm_format_t = 21;
const SND_PCM_FORMAT_IMA_ADPCM          : snd_pcm_format_t = 22;
const SND_PCM_FORMAT_MPEG               : snd_pcm_format_t = 23;
const	SND_PCM_FORMAT_GSM                : snd_pcm_format_t = 24;
const SND_PCM_FORMAT_SPECIAL            : snd_pcm_format_t = 31;
const SND_PCM_FORMAT_S24_3LE            : snd_pcm_format_t = 32;
const SND_PCM_FORMAT_S24_3BE            : snd_pcm_format_t = 33;
const SND_PCM_FORMAT_U24_3LE            : snd_pcm_format_t = 34;
const SND_PCM_FORMAT_U24_3BE            : snd_pcm_format_t = 35;
const SND_PCM_FORMAT_S20_3LE            : snd_pcm_format_t = 36;
const SND_PCM_FORMAT_S20_3BE            : snd_pcm_format_t = 37;
const SND_PCM_FORMAT_U20_3LE            : snd_pcm_format_t = 38;
const SND_PCM_FORMAT_U20_3BE            : snd_pcm_format_t = 39;
const SND_PCM_FORMAT_S18_3LE            : snd_pcm_format_t = 40;
const SND_PCM_FORMAT_S18_3BE            : snd_pcm_format_t = 41;
const SND_PCM_FORMAT_U18_3LE            : snd_pcm_format_t = 42;
const SND_PCM_FORMAT_U18_3BE            : snd_pcm_format_t = 43;
const SND_PCM_FORMAT_G723_24            : snd_pcm_format_t = 44;
const SND_PCM_FORMAT_G723_24_1B         : snd_pcm_format_t = 45;
const SND_PCM_FORMAT_G723_40            : snd_pcm_format_t = 46;
const SND_PCM_FORMAT_G723_40_1B         : snd_pcm_format_t = 47;
const SND_PCM_FORMAT_DSD_U8             : snd_pcm_format_t = 48;
const SND_PCM_FORMAT_DSD_U16_LE         : snd_pcm_format_t = 49;
const SND_PCM_FORMAT_DSD_U32_LE         : snd_pcm_format_t = 50;
const SND_PCM_FORMAT_DSD_U16_BE         : snd_pcm_format_t = 51;
const SND_PCM_FORMAT_DSD_U32_BE         : snd_pcm_format_t = 52;

#[cfg(target_endian = "little")]
const SND_PCM_FORMAT_S16                : snd_pcm_format_t = SND_PCM_FORMAT_S16_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_U16                : snd_pcm_format_t = SND_PCM_FORMAT_U16_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_S24                : snd_pcm_format_t = SND_PCM_FORMAT_S24_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_U24                : snd_pcm_format_t = SND_PCM_FORMAT_U24_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_S32                : snd_pcm_format_t = SND_PCM_FORMAT_S32_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_U32                : snd_pcm_format_t = SND_PCM_FORMAT_U32_LE;
#[cfg(target_endian = "little")]
const	SND_PCM_FORMAT_FLOAT              : snd_pcm_format_t = SND_PCM_FORMAT_FLOAT_LE;
#[cfg(target_endian = "little")]
const SND_PCM_FORMAT_FLOAT64            : snd_pcm_format_t = SND_PCM_FORMAT_FLOAT64_LE;
#[cfg(target_endian = "little")]
const SND_PCM_FORMAT_IEC958_SUBFRAME    : snd_pcm_format_t = SND_PCM_FORMAT_IEC958_SUBFRAME_LE;

#[cfg(target_endian = "big")]
const SND_PCM_FORMAT_S16                : snd_pcm_format_t = SND_PCM_FORMAT_S16_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_U16                : snd_pcm_format_t = SND_PCM_FORMAT_U16_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_S24                : snd_pcm_format_t = SND_PCM_FORMAT_S24_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_U24                : snd_pcm_format_t = SND_PCM_FORMAT_U24_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_S32                : snd_pcm_format_t = SND_PCM_FORMAT_S32_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_U32                : snd_pcm_format_t = SND_PCM_FORMAT_U32_BE;
#[cfg(target_endian = "big")]
const	SND_PCM_FORMAT_FLOAT              : snd_pcm_format_t = SND_PCM_FORMAT_FLOAT_BE;
#[cfg(target_endian = "big")]
const SND_PCM_FORMAT_FLOAT64            : snd_pcm_format_t = SND_PCM_FORMAT_FLOAT64_BE;
#[cfg(target_endian = "big")]
const SND_PCM_FORMAT_IEC958_SUBFRAME    : snd_pcm_format_t = SND_PCM_FORMAT_IEC958_SUBFRAME_BE;

#[allow(non_camel_case_types)]
type snd_pcm_access_t = i32;

const SND_PCM_ACCESS_MMPAP_INTERLEAVED    : snd_pcm_access_t = 0;
const SND_PCM_ACCESS_MMPAP_NONINTERLEAVED : snd_pcm_access_t = 1;
const SND_PCM_ACCESS_MMPAP_COMPLEX        : snd_pcm_access_t = 2;
const SND_PCM_ACCESS_RW_INTERLEAVED       : snd_pcm_access_t = 3;
const SND_PCM_ACCESS_RW_NONINTERLEAVED    : snd_pcm_access_t = 4;

#[allow(non_camel_case_types)]
type snd_pcm_uframes_t = u64;

#[repr(C)]
#[allow(non_camel_case_types)]
struct snd_pcm_chmap_t {
    channels : usize,
    pos      : [usize; 0]
}

#[link(name = "asound")]
extern "C" {

    fn snd_pcm_close(pcm: *mut snd_pcm_t) -> i32;

    fn snd_pcm_drop(pcm: *mut snd_pcm_t) -> i32;

    fn snd_pcm_get_chmap(pcm: *mut snd_pcm_t) -> *mut snd_pcm_chmap_t;

    fn snd_pcm_open(pcm: *mut *mut snd_pcm_t,
        name: *const libc::c_char,
        stream: snd_pcm_stream_t,
        mode: i32) -> i32;

    fn snd_pcm_recover(pcm: *mut snd_pcm_t,
        err: i32,
        silent: i32) -> i32;

    fn snd_pcm_set_params(pcm: *mut snd_pcm_t,
        format: snd_pcm_format_t,
        access: snd_pcm_access_t,
        channels: u32,
        rate: u32,
        soft_resample: i32,
        latency: u32) -> i32; 

    fn snd_pcm_writei(pcm: *mut snd_pcm_t,
        buffer: *const libc::c_void,
        size: usize) -> i32;  

    fn snd_strerror(errnum: i32) -> *const libc::c_char;
}

type SoundPcm = snd_pcm_t; 
type SoundPcmPtr = Option<Unique<SoundPcm>>;

pub struct NonBlockingSoundPcmPlaybackWriter {
    inner : SoundPcmPtr
}

const PLAYBACK_STREAM : snd_pcm_stream_t = SND_PCM_STREAM_PLAYBACK; 

fn snd_pcm_error(errnum: i32) -> Result<&'static str, std::str::Utf8Error> {
    unsafe {
        CStr::from_ptr(snd_strerror(errnum))
            .to_str()
    }
}

const SND_PCM_NONBLOCK : i32 = 1;
const SND_PCM_ASYNC    : i32 = 2;

impl NonBlockingSoundPcmPlaybackWriter {

    // write-only playback stream
    pub fn create(path: &'static str) -> io::Result<Self> {

        let mut raw_ptr : *mut SoundPcm = unsafe {
            std::mem::uninitialized()
        };

        let path_cstr = CString::new(path)
            .unwrap();

        let cpath = path_cstr
            .as_ptr();

        let res = unsafe {
            snd_pcm_open((&mut raw_ptr) as *mut *mut SoundPcm,
                cpath,
                PLAYBACK_STREAM,
                SND_PCM_NONBLOCK)
        }; 

        match res {

            0 => unsafe {
                Ok(NonBlockingSoundPcmPlaybackWriter {
                    inner: Some(Unique::new(raw_ptr)) 
                })
            },

            errnum => Err(io::Error
                ::from_raw_os_error(errnum))
        }    
    }
}

impl Drop for NonBlockingSoundPcmPlaybackWriter {
    fn drop(&mut self) {
        let ptr = *self.inner.take().unwrap();
        unsafe {
            match snd_pcm_close(ptr) {
                0 => (),
                _ => panic!("pcm handle has not been closed properly.") 
            }
        }
    }
}

const NO_SND_PCM_PTR : &'static str = "no pcm pointer";
const EAGAIN : i32 = libc::EAGAIN;

impl io::Write for NonBlockingSoundPcmPlaybackWriter {

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {

        match self.inner {

            Some(ref mut inner) => unsafe {

                let (pcm, len, mut written) = (inner.get_mut(),
                    buf.len(),
                    0);

                while written < len {

                    match snd_pcm_writei(pcm as *mut snd_pcm_t,
                        buf.as_ptr()
                           .offset(written as isize) as 
                               *const libc::c_void,
                        len - written) {

                        res if res >= 0 => {
                            // be supposed to channel length is one.
                            written += res as usize;
                        },

                        errnum if errnum == -EAGAIN => continue,

                        errnum => {
                            match snd_pcm_recover(pcm as *mut snd_pcm_t,
                                errnum as i32,
                                0) {
                                0 => return Err(io::Error
                                    ::from_raw_os_error(errnum as i32)),
                                err => panic!(snd_pcm_error(err))
                            }
                        }
                    }
                }
              
                Ok(written)
            },

            _ => panic!(NO_SND_PCM_PTR)
        }
    }

    fn flush(&mut self) -> io::Result<()> {

        match self.inner {

            Some(ref mut inner) => unsafe {

                let pcm = inner.get_mut();

                match snd_pcm_drop(pcm as *mut snd_pcm_t) {
                    0 => Ok(()),
                    errnum => Err(io::Error
                        ::from_raw_os_error(errnum))
                }
            },

            _ => panic!(NO_SND_PCM_PTR)
        } 
    }
} 

impl NonBlockingSoundPcmPlaybackWriter {

    fn set_params(&mut self, 
        wave_bits: u8,
        wave_rates: u16,
        wave_channels: u8) -> io::Result<()> {
        
        const ALLOW_RESAMPLING    : i32 = 1; 
        const ORDINARY_SAMLE_RATE : u32 = 480000;

        match self.inner {
          
            Some(ref mut inner) => unsafe {
                
                let pcm = inner.get_mut();        

                let format = match wave_bits {
                    8  => SND_PCM_FORMAT_U8, 
                    16 => SND_PCM_FORMAT_S16, 
                    24 => SND_PCM_FORMAT_S24, 
                    32 => SND_PCM_FORMAT_S32, 
                    _  => panic!("unexpected wave bits")
                };

                match snd_pcm_set_params(pcm as *mut snd_pcm_t,
                    format,
                    SND_PCM_ACCESS_RW_INTERLEAVED,
                    wave_channels as u32,
                    wave_rates as u32,
                    ALLOW_RESAMPLING,
                    ORDINARY_SAMLE_RATE) {
                    0 => Ok(()),
                    errnum => Err(io::Error
                        ::from_raw_os_error(errnum))
                }
            },

            _ => panic!(NO_SND_PCM_PTR)
        } 
    }
}

trait FromBuffer {

    fn from_buffer(buf: &[u8]) -> Result<Self, &'static str>
        where Self: Sized + Default {

            let size = size_of::<Self>();

            match size == buf.len() {

                true => unsafe {

                    let mut res = Box
                        ::new(Self::default());

                    copy_nonoverlapping(buf.as_ptr() as *const Self,
                        res.as_mut() as *mut Self,
                        size);

                    Ok(*res)
                },

                _ => Err("buffer size is not matched.")
            }
    }
}

macro_rules! __item {
	  ($i:item) => ($i)
}

macro_rules! s {

	  ( 
        $( 
            pub struct $n:ident { 
                $( $field:tt )* 
            } 
        )* 
    ) 
    
    => { 

  	    $(
		        __item! {
				        #[derive(Default)] 
      	        pub struct $n { 
                    $( $field )* 
                }
			      } 

            impl FromBuffer for $n {} 
		    )*
	  }
}

type Id = [u8; 4];
type DataType = [u8; 4];

s! {
    pub struct RiffHeader {
        riff      : Id,
        size      : isize,
        data_type : DataType 
    }

    pub struct ChunkHeader {
        id           : Id,
        size         : isize
    }

    pub struct Format {
        format       : i16,
        channels     : u16,
        sample_rate  : usize,
        byte_per_sec : usize,
        block_align  : u16,
        bits_width   : u16
    }
}

type PlaybackWriter = NonBlockingSoundPcmPlaybackWriter;

enum SoundPcmIORequest {
    SetParams(Format),
    Write(WriteBuffer<PlaybackWriter>),
    Close
}

unsafe impl Send for SoundPcmIORequest {}

enum SoundPcmIOResponse {
    IsSet,
    Written(usize),
    Failed(IOError), 
    Timeout
}

unsafe impl Send for SoundPcmIOResponse {}

const REQUEST_IS_ODD: &'static str = 
    "this function can't handle Close request";

fn handle_sp_io_request(writer: &mut PlaybackWriter,
    req: SoundPcmIORequest) -> SoundPcmIOResponse {

    match req {

        SoundPcmIORequest
            ::SetParams(format) => match format.format { 

                1 => match writer.set_params(format.bits_width as u8,
                    format.sample_rate as u16,
                    format.channels as u8) { 
                    Ok(_) => SoundPcmIOResponse
                        ::IsSet,
                    Err(err) => SoundPcmIOResponse
                        ::Failed(err)
                },

                _ => SoundPcmIOResponse
                    ::Failed(IOError::new(IO_ERROR,
                        "unknown sp_io request"))
        },

        SoundPcmIORequest
            ::Write(mut buf) => match buf
                .write(writer) {
                Ok(n) => SoundPcmIOResponse
                    ::Written(n),
                Err(err) => SoundPcmIOResponse
                    ::Failed(err)
            },

        _ => panic!(REQUEST_IS_ODD)     
    } 
}

struct Worker {
}
/*
struct Worker {
    tx    : Option<Sender<FileIOResponse>>,
    rx    : Option<Receiver<FileIORequest>>,
    timer : Timer
}

impl Worker {

    fn new(tx: Sender<FileIOResponse>, 
           rx: Receiver<FileIORequest>,
           timer: Timer) -> Self {
        
        if !timer.is_started() {
            panic!("timer has not been started.");
        }
    
        Worker { 
            tx: Some(tx), 
            rx: Some(rx),
            timer: timer 
        }
    }

    fn sender(&mut self) -> Option<Sender<FileIOResponse>> {
        self.tx.take()
    }

    fn receiver(&mut self) -> Option<Receiver<FileIORequest>> {
        self.rx.take()
    }

    fn timer(&self) -> Timer {
        self.timer
    }
}
*/

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
      
    extern crate futures;

    use std::thread;
    use std::fs::File;
    use std::mem::size_of;
    use std::str::from_utf8;

    use self::futures::Future; 

    use super::*; 
    use super::{ handle_sp_io_request, FromBuffer, 
                 SoundPcmIORequest, SoundPcmIOResponse } ;

    use io::*;

    const SOUNDCARD : &'static str = "plughw:0,0";
    const WAVE_FILE_SAMPLE : &'static str = "/usr/share/sounds/k3b_success1.wav";
    const WBUF_ALIGNMENT : usize = 1;

    fn read_file_and_convert<T: FromBuffer + Default>(f: &mut File) 
     -> Result<T, &'static str> {

        let mut buf = ReadBuffer 
            ::<File>
            ::new(size_of::<T>());

        match buf.read(f) {
            Ok(_) => {
                let slice = unsafe {
                    buf.load()
                };
                T::from_buffer(slice)
            },
            _ => panic!("failed to read file")       
        }

    }

    #[test]
    #[allow(unused_mut)]
    #[cfg(feature = "optional")]
    fn playback_stream_test() {
      
        const FREAD_ERROR : &'static str = "failed to read file"; 

        let (tx, rx) = futures
            ::oneshot();

        let handle = thread::spawn(move || {
            tx.complete(
                (
                    File::open(WAVE_FILE_SAMPLE)
                        .unwrap(),
                    NonBlockingSoundPcmPlaybackWriter
                        ::create(SOUNDCARD)
                        .unwrap()
                )
            );
        });  
        

        let fmap = rx

            .map(|(mut file, mut pcm)| 
                match read_file_and_convert
                    ::<RiffHeader>(&mut file) {
                    Ok(header) => {
                        assert_eq!("RIFF", from_utf8(&header.riff)
                            .unwrap()); 
                        assert_eq!("WAVE", from_utf8(&header.data_type)
                            .unwrap());
                        (file, pcm)
                    },
                    _ => panic!(FREAD_ERROR)
                }
            )

            .map(|(mut file, mut pcm)| 
                match read_file_and_convert
                    ::<ChunkHeader>(&mut file) {
                    Ok(header) => {
                        assert_eq!("fmt ", from_utf8(&header.id)
                            .unwrap());
                        (file, pcm)
                    },
                    _ => panic!(FREAD_ERROR)
                }
            )

            .map(|(mut file, mut pcm)|
                match read_file_and_convert
                    ::<Format>(&mut file) {
                    Ok(format) => {
                        pcm.set_params(format.bits_width as u8,
                            format.sample_rate as u16,
                            format.channels as u8)
                            .unwrap();
                        (file, pcm)
                    },
                    _ => panic!(FREAD_ERROR)
                }
            )

            .map(|(mut file, mut pcm)| {
                match read_file_and_convert
                    ::<ChunkHeader>(&mut file) {
                    Ok(header) => {
                        assert_eq!("data", from_utf8(&header.id)
                            .unwrap());

                        let size = header.size;

                        let mut buf = ReadBuffer
                            ::<File>
                            ::new(size as usize); 

                        match buf.read(&mut file) {
                            Ok(_) => {
                                let data = unsafe {
                                    buf.load()
                                };

                                let mut wbuf = WriteBuffer
                                    ::<NonBlockingSoundPcmPlaybackWriter>
                                    ::new(data, 1);
                                
                                wbuf.write(&mut pcm)
                                    .unwrap(); 
                              
                                (file, pcm)
                            },
                            _ => panic!("failed to read data from file")
                        }
                    },
                    _ => panic!(FREAD_ERROR)
                }
            });
        
        fmap.wait()
            .unwrap();

        handle
            .join()
            .unwrap();
    }  

    #[test]
    #[allow(unused_mut)]
    #[cfg(feature = "optional")]
    fn handle_sp_io_request_test() {
        let (tx, rx) = futures
            ::oneshot();

        let handle = thread::spawn(move || {
            tx.complete(
                (
                    File::open(WAVE_FILE_SAMPLE).unwrap(),
                    NonBlockingSoundPcmPlaybackWriter
                        ::create(SOUNDCARD).unwrap()
                )
            );
        }); 

        let fmap = rx
            .map(|(mut file, mut pcm)| {
                read_file_and_convert
                    ::<RiffHeader>(&mut file).unwrap();
                (file, pcm)
            })
            .map(|(mut file, mut pcm)| {
                read_file_and_convert
                    ::<ChunkHeader>(&mut file).unwrap();
                (file, pcm)
            })
            .map(|(mut file, mut pcm)| {
                let format = read_file_and_convert
                    ::<Format>(&mut file)
                    .unwrap();
                
                let req = SoundPcmIORequest
                    ::SetParams(format);
                
                match handle_sp_io_request(&mut pcm,
                    req) {
                    SoundPcmIOResponse
                        ::IsSet => (file, pcm),
                    _ => panic!("failed to handle set_params request properly")
                }
            })
            .map(|(mut file, mut pcm)| {
                let header = read_file_and_convert
                    ::<ChunkHeader>(&mut file)
                    .unwrap();

                let mut rbuf = ReadBuffer
                    ::<File>
                    ::new(header.size as usize);

                rbuf.read(&mut file).unwrap();

                let data = unsafe {
                    rbuf.load()
                };
              
                let mut wbuf = WriteBuffer
                    ::<NonBlockingSoundPcmPlaybackWriter>
                    ::new(data, 1);
              
                let req = SoundPcmIORequest
                    ::Write(wbuf);

                match handle_sp_io_request(&mut pcm,
                    req) {
                    SoundPcmIOResponse
                        ::Written(size) => {
                        assert_eq!(size, header.size as usize);  
                        (file, pcm)
                    },
                    _ => panic!("failed to handle write request properly")
                }
            });
            
        fmap.wait()
            .unwrap();

        handle
            .join()
            .unwrap();
    }
}
