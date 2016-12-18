use std;
use std::fs::File;
use std::sync::mpsc::*;
use std::thread::{ JoinHandle, sleep, spawn };
use std::time::Duration;

use io::*;

pub type FileIOCallbackRet = ();

define_io!(FileIO, 
    File, 
    FileIORequest, 
    FileIOResponse, 
    FileIOCallbackRet);

pub enum FileIORequest {
    Read(usize),
    Close
}

unsafe impl Send for FileIORequest {}

pub enum FileIOResponse {
    Read(ReadBuffer<File>),
    Failed(IOError),
    Closed,
    Timeout
}

unsafe impl Send for FileIOResponse {}

impl FileIO {

    pub fn new(name: String, 
           secs: u64, 
           nanos: u32) -> Self {

        FileIO {
            name: name,
            handle: None,
            tx: None,
            rx: None,
            timer: Timer::new(secs, nanos)
        } 
    }
}

impl IO for FileIO {

    type T = File;
    type R = FileIOCallbackRet;
    type Req = FileIORequest;
    type Res = FileIOResponse;
    
    fn start(&mut self) -> IOResult<()> {

        let name = self.name
            .clone();

        let ((req_tx, req_rx), (res_tx, res_rx)) = (channel::<FileIORequest>(), 
            channel::<FileIOResponse>());

        self.timer.start();
        let timer = self.timer();

        let handle = spawn(move || {
          
            let f = match File::open(name) {
                Ok(f) => f,
                _ => panic!("failed to open file")
            };

            let mut worker = Worker::new(res_tx, req_rx, timer);
            let handler = Box::new(handle_fio_request);

            worker
                .run(handler, f)
                .unwrap()
        });

        self.handle = Some(handle); 
        self.tx = Some(req_tx);
        self.rx = Some(res_rx);
        Ok(())
    }

    fn send(&self, req: FileIORequest) -> Result<(), SendError<FileIORequest>> {
        match self.tx {
            Some(ref tx) => tx.send(req),
            _ => panic!("no sender")
        }  
    }
    
    fn recv(&self) -> Result<FileIOResponse, RecvError> {
        match self.rx {
            Some(ref rx) => rx.recv(),
            _ => panic!("no receiver")
        }
    }

    fn timer(&self) -> Timer {
        self.timer 
    }
    
    fn stop(&mut self) -> IOResult<()> {

        match self.send(FileIORequest::Close) {

            Ok(_) => match self.recv() {
                Ok(FileIOResponse::Closed) => {
                    self.handle.take();
                    Ok(())
                },
                Ok(_) => panic!("unexpected response type"),
                Err(e) => io_error(e) 
            },
            
            Err(e) => io_error(e) 
        }
    }

    fn join(&mut self) -> std::thread::Result<()> {

        let handle = self.handle.take();

        let res = match handle {
            Some(handle) => handle
                .join(), 
            _ => panic!("no thread handle")
        };

        res
    }

    fn sender(&self) -> Option<&Sender<FileIORequest>> {
        inner_ref!(self, tx)
    }

    fn receiver(&self) -> Option<&Receiver<FileIOResponse>> {
        inner_ref!(self, rx)
    }
}

macro_rules! define_worker {
    ($req:ident, $res:ident) => {
        struct Worker {
            tx    : Option<Sender<$res>>,
            rx    : Option<Receiver<$req>>,
            timer : Timer
        }
    } 
}

define_worker!(FileIORequest, 
    FileIOResponse);

/*
trait WorkerDefault {

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

const SEND_ERROR : &'static str = "send error";
const DISCONNECTED : &'static str = "disconnected";
const INTERVAL : u64 = 50;

impl Loop<FileIOResponse, FileIORequest, FileIOCallbackRet> for Worker {

    type In = File;
    type Callback = FnMut<Self::Args, Output=FileIOResponse> + Send;
    type Args = (File, FileIORequest);
    type Out = FileIOCallbackRet;

    fn run(&mut self, mut callback: Box<Self::Callback>, input: File)
     -> IOResult<()> { 

        let timer = self.timer();

        let (tx, rx) = match (self.sender(), self.receiver()) {
            (Some(tx), Some(rx)) => (tx, rx),
            _ => return Err(IOError::new(IO_ERROR,
                "worker has been already used."))
        };

        let interval = Duration::from_millis(INTERVAL);

        while !timer.is_timeout() {
                
            match rx.try_recv() {

                Ok(req) => {

                    let res = match callback(input
                        .try_clone()
                        .unwrap(), req) {
                        FileIOResponse::Closed => break,
                        res => res
                    };

                    match tx.send(res) {
                        Ok(_) => (),
                        _ => panic!(SEND_ERROR)
                    }
                },

                Err( TryRecvError::Empty ) => (),
                  
                 _ => panic!(DISCONNECTED) 
            }

            sleep(interval); 
        }

        match timer.is_timeout() {

            true => {
                tx.send(FileIOResponse::Timeout).unwrap();
                Err(IOError::new(IO_ERROR, "timeout"))
            },

            _ => {
                tx.send(FileIOResponse::Closed).unwrap();
                Ok(())
            }
        }
    }
}

fn handle_fio_request(mut f: File, req: FileIORequest) -> FileIOResponse {
    match req {
        FileIORequest::Read(n) => {
            let mut buf = ReadBuffer::new(n);
            match buf.read(&mut f) {
                Err(e) => FileIOResponse::Failed(e),
                _ => FileIOResponse::Read(buf)
            }
        },
        FileIORequest::Close => FileIOResponse::Closed
    }
}

#[cfg(test)]
mod tests {

    use std::fs::File; 
    use std::str::from_utf8;
    use std::sync::mpsc::{ channel, TryRecvError };
    use std::thread::{ sleep, spawn };
    use std::time::Duration;

    use io::*;
    use super::*;
    use super::{ handle_fio_request, Worker };

    const RIFF            : &'static str = "RIFF";
    const WAVE            : &'static str = "WAVE";
    const WAVE_FILE_PATH  : &'static str = "/usr/share/sounds/k3b_success1.wav";
    const RIFF_FIELD_SIZE : usize = 4;

    const ERROR_MESSAGE_1 : &'static str = "read response is expected";
    const ERROR_MESSAGE_2 : &'static str = "closed response is expected";

    #[test]
    fn handler_test() {

        let f = File::open(WAVE_FILE_PATH)
            .unwrap();
        
        let res = handle_fio_request(f
            .try_clone()
            .unwrap(), 
            FileIORequest::Read(RIFF_FIELD_SIZE)); 
     
        match res {

            FileIOResponse::Read(buf) => {

                let riff = unsafe {
                    from_utf8(buf.load())
                        .unwrap()
                };
                assert_eq!(RIFF, riff);
            },

            _ => panic!(ERROR_MESSAGE_1)
        }

        match handle_fio_request(f, FileIORequest::Close) {
            FileIOResponse::Closed => (),
            _ => panic!(ERROR_MESSAGE_2)
        }
    }

    #[test]
    fn worker_test() {

        let (req_tx, req_rx) = channel::<FileIORequest>();          
        let (res_tx, res_rx) = channel::<FileIOResponse>();

        let mut timer = Timer::new(10, 0);
        timer.start();

        let handle = spawn(move || {

            let mut worker = Worker::new(res_tx, 
                req_rx, 
                timer);

            let f = File::open(WAVE_FILE_PATH)
                .unwrap();

            let handler = Box::new(handle_fio_request);

            worker
                .run(handler, f)
                .unwrap();
        });

        req_tx
            .send(FileIORequest::Read(RIFF_FIELD_SIZE))
            .unwrap();

        match res_rx.recv() {

            Ok(FileIOResponse::Read(buf)) => {

                let riff = unsafe {
                    from_utf8(buf.load())
                        .unwrap()
                };
                assert_eq!(RIFF, riff);
            },

            _ => panic!(ERROR_MESSAGE_1)
        }

        req_tx
            .send(FileIORequest::Close)
            .unwrap();

        match res_rx.recv() {
            Ok(FileIOResponse::Closed) => (),
            _ => panic!(ERROR_MESSAGE_2)
        }

        handle
            .join()
            .unwrap();
    }

    #[test]
    fn file_io_test() {

        let path = WAVE_FILE_PATH
            .to_string();

        let mut file_io = FileIO::new(path, 10, 0); 

        file_io
            .start()
            .unwrap();

        let tx = file_io
            .sender()
            .unwrap();

        let rx = file_io
            .receiver()
            .unwrap();

        tx.send(FileIORequest::Read(RIFF_FIELD_SIZE))
            .unwrap();

        loop {
          
            match rx.try_recv() {

                Ok(FileIOResponse::Read(buf)) => {

                    let riff = unsafe {
                        from_utf8(buf.load())
                            .unwrap()
                    };

                    assert_eq!(RIFF, riff);
                    tx.send(FileIORequest::Close)
                        .unwrap();
                },

                Ok(FileIOResponse::Failed(e)) => panic!(e),
                
                Ok(FileIOResponse::Closed) => break,

                Ok(_) => panic!("timeout"),  

                Err(TryRecvError::Empty) => sleep(Duration
                    ::from_millis(10)),

                _ => panic!("disconnected"),
            }
        } 
    }
}
